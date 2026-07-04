use std::f64::consts::PI;

use bevy::prelude::*;
use content_schema::{BallId, BoardDefinition, ObstacleKind, PegKind, ShapeDef};
use physics_core::{
    predict_first_bounce, sample_shot_trajectory, simulate_shot, PhysicsEvent, ShotInput,
    ShotSummary,
};

use crate::plugins::feel_test::TrajectoryPlaybackCursor;

const BOARD_JSON: &str = include_str!("../assets/content/boards/feel_fan_01.json");
const BOARD_SCALE: f32 = 22.0;
const LAUNCH_SPEED: f64 = 24.0;
const SHOT_SEED: u64 = 1;
const TRAJECTORY_SAMPLE_EVERY_TICKS: u64 = 6;
const PLAYBACK_TICKS_PER_SECOND: f64 = 480.0;

pub fn run() {
    let board: BoardDefinition =
        serde_json::from_str(BOARD_JSON).expect("embedded feel-test board JSON is valid");

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.029, 0.04)))
        .insert_resource(FeelTestState::new(board))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Feverfall Physics Feel Test".to_owned(),
                resolution: (960, 860).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (handle_input, advance_shot_playback, refresh_dynamic_overlay).chain(),
        )
        .run();
}

#[derive(Resource)]
struct FeelTestState {
    board: BoardDefinition,
    aim_angle_radians: f64,
    revision: u64,
    drawn_revision: u64,
    last_summary: Option<ShotSummary>,
    hit_peg_ids: Vec<String>,
    first_bounce: Option<PhysicsEvent>,
    trajectory_points: Vec<content_schema::Vec2>,
    shot_ball_position: Option<content_schema::Vec2>,
    playback: Option<TrajectoryPlaybackCursor>,
}

impl FeelTestState {
    fn new(board: BoardDefinition) -> Self {
        Self {
            board,
            aim_angle_radians: PI / 2.0,
            revision: 1,
            drawn_revision: 0,
            last_summary: None,
            hit_peg_ids: Vec::new(),
            first_bounce: None,
            trajectory_points: Vec::new(),
            shot_ball_position: None,
            playback: None,
        }
    }

    fn shot_results_visible(&self) -> bool {
        self.last_summary.is_some() && self.playback.is_none()
    }
}

#[derive(Component)]
struct DynamicOverlay;

fn setup_scene(
    mut commands: Commands,
    state: Res<FeelTestState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    spawn_board(&mut commands, &state.board, &mut meshes, &mut materials);
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut state: ResMut<FeelTestState>,
) {
    if state.playback.is_some() {
        return;
    }

    let previous_angle = state.aim_angle_radians;
    let turn_step = f64::from(time.delta_secs()) * 1.35;

    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        state.aim_angle_radians -= turn_step;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        state.aim_angle_radians += turn_step;
    }

    state.aim_angle_radians = state.aim_angle_radians.clamp(0.18, PI - 0.18);
    if (state.aim_angle_radians - previous_angle).abs() > f64::EPSILON {
        state.revision += 1;
    }

    if keys.just_pressed(KeyCode::Space) {
        let input = current_shot_input(state.aim_angle_radians);
        let result = simulate_shot(SHOT_SEED, &state.board, &input);
        state.first_bounce = predict_first_bounce(&state.board, &input);
        let samples = sample_shot_trajectory(&state.board, &input, TRAJECTORY_SAMPLE_EVERY_TICKS);
        let playback = TrajectoryPlaybackCursor::new(samples, PLAYBACK_TICKS_PER_SECOND);
        state.trajectory_points = playback.trail_points();
        state.shot_ball_position = playback.current_position();
        state.hit_peg_ids = result
            .summary
            .pegs_hit
            .iter()
            .map(|peg| peg.as_str().to_owned())
            .collect();
        state.last_summary = Some(result.summary);
        if playback.is_complete() {
            state.trajectory_points = playback.trail_points();
            state.shot_ball_position = playback.current_position();
            state.playback = None;
        } else {
            state.playback = Some(playback);
        }
        state.revision += 1;
    }
}

fn advance_shot_playback(time: Res<Time>, mut state: ResMut<FeelTestState>) {
    let Some(mut playback) = state.playback.take() else {
        return;
    };

    if !playback.advance(f64::from(time.delta_secs())) {
        state.playback = Some(playback);
        return;
    }

    state.trajectory_points = playback.trail_points();
    state.shot_ball_position = playback.current_position();
    if !playback.is_complete() {
        state.playback = Some(playback);
    }
    state.revision += 1;
}

fn refresh_dynamic_overlay(
    mut commands: Commands,
    mut state: ResMut<FeelTestState>,
    overlays: Query<Entity, With<DynamicOverlay>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if state.drawn_revision == state.revision {
        return;
    }

    for entity in &overlays {
        commands.entity(entity).despawn();
    }

    spawn_aim_line(
        &mut commands,
        &state.board,
        state.aim_angle_radians,
        &mut meshes,
        &mut materials,
    );
    spawn_shot_trajectory(&mut commands, &state, &mut meshes, &mut materials);
    if state.shot_results_visible() {
        spawn_hit_markers(&mut commands, &state, &mut meshes, &mut materials);
        spawn_result_panel(&mut commands, &state, &mut meshes, &mut materials);
    }
    state.drawn_revision = state.revision;
}

fn spawn_board(
    commands: &mut Commands,
    board: &BoardDefinition,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    spawn_rect(
        commands,
        meshes,
        materials,
        RectSpec::new(
            board_to_world(board, board.size.x / 2.0, board.size.y / 2.0),
            Vec2::new(
                (board.size.x as f32) * BOARD_SCALE,
                (board.size.y as f32) * BOARD_SCALE,
            ),
            Color::srgb(0.045, 0.055, 0.075),
            0.0,
            0.0,
        ),
    );

    for peg in &board.pegs {
        let color = match peg.kind {
            PegKind::Orange => Color::srgb(1.0, 0.48, 0.08),
            PegKind::Blue => Color::srgb(0.18, 0.48, 1.0),
            PegKind::Purple => Color::srgb(0.7, 0.32, 1.0),
            PegKind::Green => Color::srgb(0.25, 0.9, 0.35),
            PegKind::Stone => Color::srgb(0.45, 0.47, 0.5),
            PegKind::Ghost => Color::srgba(0.65, 0.85, 1.0, 0.45),
            PegKind::Bomb => Color::srgb(0.95, 0.1, 0.1),
            PegKind::Splitter => Color::srgb(1.0, 0.9, 0.18),
        };
        spawn_shape(commands, meshes, materials, board, &peg.shape, color, 1.0);
    }

    for obstacle in &board.obstacles {
        let color = match obstacle.kind {
            ObstacleKind::Wall => Color::srgb(0.6, 0.63, 0.68),
            ObstacleKind::Stone => Color::srgb(0.35, 0.36, 0.38),
            ObstacleKind::Rubber => Color::srgb(0.9, 0.18, 0.8),
            ObstacleKind::Sensor => Color::srgb(0.15, 0.9, 0.85),
        };
        spawn_shape(
            commands,
            meshes,
            materials,
            board,
            &obstacle.shape,
            color,
            1.5,
        );
    }

    spawn_rect(
        commands,
        meshes,
        materials,
        RectSpec::new(
            board_to_world(board, board.bucket.center.x, board.bucket.center.y),
            Vec2::new(
                (board.bucket.width as f32) * BOARD_SCALE,
                (board.bucket.height as f32) * BOARD_SCALE,
            ),
            Color::srgb(1.0, 0.82, 0.2),
            3.0,
            0.0,
        ),
    );
}

fn spawn_shape(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    board: &BoardDefinition,
    shape: &ShapeDef,
    color: Color,
    z: f32,
) -> Vec<Entity> {
    match shape {
        ShapeDef::Circle { center, radius } => vec![spawn_circle(
            commands,
            meshes,
            materials,
            board_to_world(board, center.x, center.y),
            (*radius as f32) * BOARD_SCALE,
            color,
            z,
        )],
        ShapeDef::Rect {
            center,
            half_extents,
        } => vec![spawn_rect(
            commands,
            meshes,
            materials,
            RectSpec::new(
                board_to_world(board, center.x, center.y),
                Vec2::new(
                    (half_extents.x as f32) * BOARD_SCALE * 2.0,
                    (half_extents.y as f32) * BOARD_SCALE * 2.0,
                ),
                color,
                z,
                0.0,
            ),
        )],
        ShapeDef::Segment { a, b } => vec![spawn_segment(
            commands,
            meshes,
            materials,
            board,
            SegmentSpec::new(a.x, a.y, b.x, b.y, color, z),
        )],
        ShapeDef::Capsule { a, b, radius } => {
            let segment = spawn_segment(
                commands,
                meshes,
                materials,
                board,
                SegmentSpec::new(a.x, a.y, b.x, b.y, color, z),
            );
            let cap_a = spawn_circle(
                commands,
                meshes,
                materials,
                board_to_world(board, a.x, a.y),
                (*radius as f32) * BOARD_SCALE,
                color,
                z + 0.1,
            );
            let cap_b = spawn_circle(
                commands,
                meshes,
                materials,
                board_to_world(board, b.x, b.y),
                (*radius as f32) * BOARD_SCALE,
                color,
                z + 0.1,
            );
            vec![segment, cap_a, cap_b]
        }
    }
}

fn spawn_aim_line(
    commands: &mut Commands,
    board: &BoardDefinition,
    aim_angle_radians: f64,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let start = board.cannon_position;
    let length = 6.0;
    let end_x = start.x + aim_angle_radians.cos() * length;
    let end_y = start.y + aim_angle_radians.sin() * length;
    let aim_line = spawn_segment(
        commands,
        meshes,
        materials,
        board,
        SegmentSpec::new(
            start.x,
            start.y,
            end_x,
            end_y,
            Color::srgb(0.95, 0.95, 0.9),
            5.0,
        ),
    );
    commands.entity(aim_line).insert(DynamicOverlay);
    commands.spawn((
        Mesh2d(meshes.add(Triangle2d::new(
            Vec2::new(0.0, 14.0),
            Vec2::new(-10.0, -10.0),
            Vec2::new(10.0, -10.0),
        ))),
        MeshMaterial2d(materials.add(Color::srgb(0.95, 0.95, 0.9))),
        Transform::from_translation(board_to_world(board, start.x, start.y).extend(6.0))
            .with_rotation(Quat::from_rotation_z((PI / 2.0 - aim_angle_radians) as f32)),
        DynamicOverlay,
    ));
}

fn spawn_hit_markers(
    commands: &mut Commands,
    state: &FeelTestState,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    for peg in &state.board.pegs {
        if state.hit_peg_ids.iter().any(|id| id == peg.id.as_str()) {
            for entity in spawn_shape(
                commands,
                meshes,
                materials,
                &state.board,
                &peg.shape,
                Color::srgb(1.0, 1.0, 1.0),
                4.0,
            ) {
                commands.entity(entity).insert(DynamicOverlay);
            }
        }
    }

    if let Some(event) = &state.first_bounce {
        if let Some(position) = event_position(event) {
            let marker = spawn_circle(
                commands,
                meshes,
                materials,
                board_to_world(&state.board, position.x, position.y),
                9.0,
                Color::srgb(0.2, 1.0, 0.95),
                7.0,
            );
            commands.entity(marker).insert(DynamicOverlay);
        }
    }
}

fn spawn_shot_trajectory(
    commands: &mut Commands,
    state: &FeelTestState,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    for points in state.trajectory_points.windows(2) {
        let [start, end] = points else {
            continue;
        };
        if start == end {
            continue;
        }
        let segment = spawn_segment(
            commands,
            meshes,
            materials,
            &state.board,
            SegmentSpec::new(
                start.x,
                start.y,
                end.x,
                end.y,
                Color::srgba(0.15, 0.9, 1.0, 0.55),
                6.0,
            ),
        );
        commands.entity(segment).insert(DynamicOverlay);
    }

    if let Some(position) = state.shot_ball_position {
        let ball = spawn_circle(
            commands,
            meshes,
            materials,
            board_to_world(&state.board, position.x, position.y),
            7.0,
            Color::srgb(0.98, 1.0, 0.3),
            7.5,
        );
        commands.entity(ball).insert(DynamicOverlay);
    }
}

fn spawn_result_panel(
    commands: &mut Commands,
    state: &FeelTestState,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let Some(summary) = &state.last_summary else {
        return;
    };

    let color = if summary.caught_bucket {
        Color::srgb(0.15, 0.85, 0.22)
    } else {
        Color::srgb(0.82, 0.18, 0.16)
    };
    let x = -((state.board.size.x as f32) * BOARD_SCALE / 2.0) + 32.0;
    let y = ((state.board.size.y as f32) * BOARD_SCALE / 2.0) - 24.0;
    let panel = spawn_rect(
        commands,
        meshes,
        materials,
        RectSpec::new(
            Vec2::new(x, y),
            Vec2::new(42.0 + summary.pegs_hit.len() as f32 * 4.0, 18.0),
            color,
            8.0,
            0.0,
        ),
    );
    commands.entity(panel).insert(DynamicOverlay);
}

fn spawn_circle(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    position: Vec2,
    radius: f32,
    color: Color,
    z: f32,
) -> Entity {
    commands
        .spawn((
            Mesh2d(meshes.add(Circle::new(radius))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_translation(position.extend(z)),
        ))
        .id()
}

fn spawn_rect(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    spec: RectSpec,
) -> Entity {
    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(spec.size.x, spec.size.y))),
            MeshMaterial2d(materials.add(spec.color)),
            Transform::from_translation(spec.position.extend(spec.z))
                .with_rotation(Quat::from_rotation_z(spec.rotation)),
        ))
        .id()
}

struct RectSpec {
    position: Vec2,
    size: Vec2,
    color: Color,
    z: f32,
    rotation: f32,
}

impl RectSpec {
    const fn new(position: Vec2, size: Vec2, color: Color, z: f32, rotation: f32) -> Self {
        Self {
            position,
            size,
            color,
            z,
            rotation,
        }
    }
}

struct SegmentSpec {
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
    color: Color,
    z: f32,
}

impl SegmentSpec {
    const fn new(ax: f64, ay: f64, bx: f64, by: f64, color: Color, z: f32) -> Self {
        Self {
            ax,
            ay,
            bx,
            by,
            color,
            z,
        }
    }
}

fn spawn_segment(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    board: &BoardDefinition,
    spec: SegmentSpec,
) -> Entity {
    let start = board_to_world(board, spec.ax, spec.ay);
    let end = board_to_world(board, spec.bx, spec.by);
    let delta = end - start;
    spawn_rect(
        commands,
        meshes,
        materials,
        RectSpec::new(
            start + delta * 0.5,
            Vec2::new(delta.length(), 4.0),
            spec.color,
            spec.z,
            delta.y.atan2(delta.x),
        ),
    )
}

fn board_to_world(board: &BoardDefinition, x: f64, y: f64) -> Vec2 {
    Vec2::new(
        ((x - board.size.x / 2.0) as f32) * BOARD_SCALE,
        ((board.size.y / 2.0 - y) as f32) * BOARD_SCALE,
    )
}

fn current_shot_input(aim_angle_radians: f64) -> ShotInput {
    ShotInput {
        aim_angle_radians,
        launch_speed: LAUNCH_SPEED,
        ball_id: BallId::new("ball/feel_test").expect("static id is valid"),
    }
}

fn event_position(event: &PhysicsEvent) -> Option<content_schema::Vec2> {
    match event {
        PhysicsEvent::BallHitPeg { position, .. }
        | PhysicsEvent::BallHitObstacle { position, .. } => Some(*position),
        PhysicsEvent::BallEnteredBucket { .. }
        | PhysicsEvent::BallExitedBoard { .. }
        | PhysicsEvent::ShotEnded { .. } => None,
    }
}
