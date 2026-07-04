use content_schema::{
    BallId, BoardDefinition, ObstacleDef, ObstacleId, ObstacleKind, PegDef, PegId, PegKind, Scalar,
    ShapeDef, Tick, Vec2,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PHYSICS_VERSION: &str = "physics_core/0.2.0-fixed-step";
const BALL_RADIUS: Scalar = 0.22;
const CONTACT_SLOP: Scalar = 0.0001;
const MIN_REMAINING_FRACTION: Scalar = 0.000001;
const MAX_TICKS_PER_SHOT: Tick = 120 * 30;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShotInput {
    pub aim_angle_radians: Scalar,
    pub launch_speed: Scalar,
    pub ball_id: BallId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimConfig {
    pub timestep_seconds: Scalar,
    pub gravity: Vec2,
    pub air_damping_per_tick: Scalar,
    pub tangential_damping_on_collision: Scalar,
    pub min_active_speed: Scalar,
    pub max_speed_cap: Scalar,
    pub restitution: RestitutionTable,
    pub max_collision_iterations_per_tick: u8,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            timestep_seconds: 1.0 / 120.0,
            gravity: Vec2::new(0.0, 22.0),
            air_damping_per_tick: 0.9985,
            tangential_damping_on_collision: 0.995,
            min_active_speed: 0.25,
            max_speed_cap: 38.0,
            restitution: RestitutionTable::default(),
            max_collision_iterations_per_tick: 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RestitutionTable {
    pub peg: Scalar,
    pub wall: Scalar,
    pub bucket_rim: Scalar,
    pub stone_obstacle: Scalar,
    pub rubber_obstacle: Scalar,
}

impl Default for RestitutionTable {
    fn default() -> Self {
        Self {
            peg: 0.94,
            wall: 0.91,
            bucket_rim: 1.02,
            stone_obstacle: 0.78,
            rubber_obstacle: 1.08,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PhysicsEvent {
    BallHitPeg {
        ball: BallId,
        peg: PegId,
        position: Vec2,
        normal: Vec2,
        speed: Scalar,
        tick: Tick,
    },
    BallHitObstacle {
        ball: BallId,
        obstacle: ObstacleId,
        position: Vec2,
        normal: Vec2,
        tick: Tick,
    },
    BallEnteredBucket {
        ball: BallId,
        tick: Tick,
    },
    BallExitedBoard {
        ball: BallId,
        tick: Tick,
    },
    ShotEnded {
        summary: ShotSummary,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShotSummary {
    pub ticks: Tick,
    pub pegs_hit: Vec<PegId>,
    pub caught_bucket: bool,
    pub exited_board: bool,
    pub replay_hash: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShotResult {
    pub events: Vec<PhysicsEvent>,
    pub summary: ShotSummary,
    pub remaining_pegs: Vec<PegDef>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrajectorySample {
    pub tick: Tick,
    pub position: Vec2,
}

#[derive(Clone, Debug)]
struct BallState {
    position: Vec2,
    velocity: Vec2,
}

#[derive(Clone, Debug)]
enum ColliderKind<'a> {
    Peg(&'a PegDef),
    Obstacle(&'a ObstacleDef),
    BucketRim,
}

#[derive(Clone, Debug)]
struct Hit<'a> {
    t: Scalar,
    normal: Vec2,
    position: Vec2,
    kind: ColliderKind<'a>,
}

pub fn simulate_shot(seed: u64, board: &BoardDefinition, input: &ShotInput) -> ShotResult {
    simulate_shot_with_config(seed, board, input, &SimConfig::default())
}

pub fn predict_first_bounce(board: &BoardDefinition, input: &ShotInput) -> Option<PhysicsEvent> {
    predict_first_bounce_with_config(board, input, &SimConfig::default())
}

pub fn sample_shot_trajectory(
    board: &BoardDefinition,
    input: &ShotInput,
    sample_every_ticks: Tick,
) -> Vec<TrajectorySample> {
    sample_shot_trajectory_with_config(board, input, sample_every_ticks, &SimConfig::default())
}

pub fn sample_shot_trajectory_with_config(
    board: &BoardDefinition,
    input: &ShotInput,
    sample_every_ticks: Tick,
    config: &SimConfig,
) -> Vec<TrajectorySample> {
    let sample_every_ticks = sample_every_ticks.max(1);
    let mut ball = initial_ball_state(board, input, config);
    let mut hit_pegs = Vec::new();
    let mut samples = vec![TrajectorySample {
        tick: 0,
        position: ball.position,
    }];

    for tick in 1..=MAX_TICKS_PER_SHOT {
        let mut events = Vec::new();
        integrate_tick(
            board,
            input,
            config,
            tick,
            &mut ball,
            &mut events,
            &mut hit_pegs,
        );

        let shot_ended = !is_finite_vec(ball.position)
            || !is_finite_vec(ball.velocity)
            || bucket_contains_ball(board, tick, ball.position)
            || ball.position.y - BALL_RADIUS > board.kill_plane_y
            || (length(ball.velocity) < config.min_active_speed
                && ball.position.y > board.size.y * 0.75)
            || tick >= MAX_TICKS_PER_SHOT;

        if tick % sample_every_ticks == 0 || shot_ended {
            samples.push(TrajectorySample {
                tick,
                position: ball.position,
            });
        }

        if shot_ended {
            break;
        }
    }

    samples
}

pub fn predict_first_bounce_with_config(
    board: &BoardDefinition,
    input: &ShotInput,
    config: &SimConfig,
) -> Option<PhysicsEvent> {
    let mut ball = initial_ball_state(board, input, config);
    let mut hit_pegs = Vec::new();

    for tick in 1..=MAX_TICKS_PER_SHOT {
        let mut events = Vec::new();
        integrate_tick(
            board,
            input,
            config,
            tick,
            &mut ball,
            &mut events,
            &mut hit_pegs,
        );

        if let Some(event) = events.into_iter().find(is_collision_event) {
            return Some(event);
        }

        if !is_finite_vec(ball.position)
            || !is_finite_vec(ball.velocity)
            || bucket_contains_ball(board, tick, ball.position)
            || ball.position.y - BALL_RADIUS > board.kill_plane_y
            || (length(ball.velocity) < config.min_active_speed
                && ball.position.y > board.size.y * 0.75)
        {
            return None;
        }
    }

    None
}

pub fn simulate_shot_with_config(
    seed: u64,
    board: &BoardDefinition,
    input: &ShotInput,
    config: &SimConfig,
) -> ShotResult {
    let mut events = Vec::new();
    let mut hit_pegs = Vec::new();
    let mut hasher = ReplayHasher::new(seed, board, input, config);
    let mut ball = initial_ball_state(board, input, config);
    let mut caught_bucket = false;
    let mut exited_board = false;
    let mut tick = 0;

    while tick < MAX_TICKS_PER_SHOT {
        tick += 1;
        integrate_tick(
            board,
            input,
            config,
            tick,
            &mut ball,
            &mut events,
            &mut hit_pegs,
        );
        hasher.hash_tick(tick, &ball, &events);

        if !is_finite_vec(ball.position) || !is_finite_vec(ball.velocity) {
            exited_board = true;
            break;
        }

        if bucket_contains_ball(board, tick, ball.position) {
            events.push(PhysicsEvent::BallEnteredBucket {
                ball: input.ball_id.clone(),
                tick,
            });
            caught_bucket = true;
            hasher.hash_tick(tick, &ball, &events);
            break;
        }

        if ball.position.y - BALL_RADIUS > board.kill_plane_y {
            events.push(PhysicsEvent::BallExitedBoard {
                ball: input.ball_id.clone(),
                tick,
            });
            exited_board = true;
            hasher.hash_tick(tick, &ball, &events);
            break;
        }

        if length(ball.velocity) < config.min_active_speed && ball.position.y > board.size.y * 0.75
        {
            exited_board = true;
            break;
        }
    }

    if tick >= MAX_TICKS_PER_SHOT {
        exited_board = true;
    }

    let replay_hash = hasher.finish();
    let summary = ShotSummary {
        ticks: tick,
        pegs_hit: hit_pegs.clone(),
        caught_bucket,
        exited_board,
        replay_hash,
    };
    events.push(PhysicsEvent::ShotEnded {
        summary: summary.clone(),
    });

    let remaining_pegs = board
        .pegs
        .iter()
        .filter(|peg| !hit_pegs.contains(&peg.id) || !is_clearable(peg.kind))
        .cloned()
        .collect();

    ShotResult {
        events,
        summary,
        remaining_pegs,
    }
}

fn integrate_tick(
    board: &BoardDefinition,
    input: &ShotInput,
    config: &SimConfig,
    tick: Tick,
    ball: &mut BallState,
    events: &mut Vec<PhysicsEvent>,
    hit_pegs: &mut Vec<PegId>,
) {
    ball.velocity = add(ball.velocity, mul(config.gravity, config.timestep_seconds));
    ball.velocity = mul(ball.velocity, config.air_damping_per_tick);
    ball.velocity = cap_speed(ball.velocity, config.max_speed_cap);

    let mut remaining = 1.0;
    let mut iterations = 0;
    while remaining > MIN_REMAINING_FRACTION {
        let start = ball.position;
        let delta = mul(ball.velocity, config.timestep_seconds * remaining);
        let end = add(start, delta);
        let Some(hit) = earliest_hit(board, tick, start, end) else {
            ball.position = end;
            break;
        };

        iterations += 1;
        let travel_t = (hit.t - CONTACT_SLOP).max(0.0);
        ball.position = add(start, mul(delta, travel_t));

        match hit.kind {
            ColliderKind::Peg(peg) => {
                if !hit_pegs.contains(&peg.id) {
                    hit_pegs.push(peg.id.clone());
                }
                events.push(PhysicsEvent::BallHitPeg {
                    ball: input.ball_id.clone(),
                    peg: peg.id.clone(),
                    position: hit.position,
                    normal: hit.normal,
                    speed: length(ball.velocity),
                    tick,
                });
                if peg.kind != PegKind::Ghost {
                    ball.velocity =
                        reflect(ball.velocity, hit.normal, config.restitution.peg, config);
                }
            }
            ColliderKind::Obstacle(obstacle) => {
                events.push(PhysicsEvent::BallHitObstacle {
                    ball: input.ball_id.clone(),
                    obstacle: obstacle.id.clone(),
                    position: hit.position,
                    normal: hit.normal,
                    tick,
                });
                if obstacle.kind != ObstacleKind::Sensor {
                    ball.velocity = reflect(
                        ball.velocity,
                        hit.normal,
                        obstacle_restitution(obstacle.kind, config),
                        config,
                    );
                }
            }
            ColliderKind::BucketRim => {
                ball.velocity = reflect(
                    ball.velocity,
                    hit.normal,
                    config.restitution.bucket_rim,
                    config,
                );
            }
        }

        remaining *= 1.0 - hit.t.clamp(0.0, 1.0);
        if iterations >= config.max_collision_iterations_per_tick {
            ball.position = add(
                ball.position,
                mul(normalize_or_zero(ball.velocity), CONTACT_SLOP),
            );
            ball.velocity = mul(ball.velocity, 0.85);
            break;
        }
    }
}

fn initial_ball_state(board: &BoardDefinition, input: &ShotInput, config: &SimConfig) -> BallState {
    BallState {
        position: board.cannon_position,
        velocity: cap_speed(
            vec2(
                input.aim_angle_radians.cos() * input.launch_speed,
                input.aim_angle_radians.sin() * input.launch_speed,
            ),
            config.max_speed_cap,
        ),
    }
}

fn is_collision_event(event: &PhysicsEvent) -> bool {
    matches!(
        event,
        PhysicsEvent::BallHitPeg { .. } | PhysicsEvent::BallHitObstacle { .. }
    )
}

fn earliest_hit<'a>(
    board: &'a BoardDefinition,
    tick: Tick,
    start: Vec2,
    end: Vec2,
) -> Option<Hit<'a>> {
    let mut best: Option<Hit<'a>> = None;

    for peg in &board.pegs {
        for (a, b, radius) in shape_colliders(&peg.shape) {
            let hit =
                sweep_circle_vs_capsule(start, end, a, b, BALL_RADIUS + radius).map(|(t, n, p)| {
                    Hit {
                        t,
                        normal: n,
                        position: p,
                        kind: ColliderKind::Peg(peg),
                    }
                });
            best = choose_earlier(best, hit);
        }
    }

    for obstacle in &board.obstacles {
        for (a, b, radius) in shape_colliders(&obstacle.shape) {
            let hit =
                sweep_circle_vs_capsule(start, end, a, b, BALL_RADIUS + radius).map(|(t, n, p)| {
                    Hit {
                        t,
                        normal: n,
                        position: p,
                        kind: ColliderKind::Obstacle(obstacle),
                    }
                });
            best = choose_earlier(best, hit);
        }
    }

    let bucket = bucket_center(board, tick);
    let half_w = board.bucket.width * 0.5;
    let rim_y = bucket.y - board.bucket.height * 0.5;
    for rim_x in [bucket.x - half_w, bucket.x + half_w] {
        let a = vec2(rim_x, rim_y);
        let b = vec2(rim_x, rim_y + board.bucket.height);
        let hit =
            sweep_circle_vs_capsule(start, end, a, b, BALL_RADIUS + 0.06).map(|(t, n, p)| Hit {
                t,
                normal: n,
                position: p,
                kind: ColliderKind::BucketRim,
            });
        best = choose_earlier(best, hit);
    }

    best
}

fn choose_earlier<'a>(current: Option<Hit<'a>>, next: Option<Hit<'a>>) -> Option<Hit<'a>> {
    match (current, next) {
        (Some(a), Some(b)) if a.t <= b.t => Some(a),
        (_, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        (None, None) => None,
    }
}

fn shape_colliders(shape: &ShapeDef) -> Vec<(Vec2, Vec2, Scalar)> {
    match shape {
        ShapeDef::Circle { center, radius } => vec![(*center, *center, *radius)],
        ShapeDef::Capsule { a, b, radius } => vec![(*a, *b, *radius)],
        ShapeDef::Segment { a, b } => vec![(*a, *b, 0.0)],
        ShapeDef::Rect {
            center,
            half_extents,
        } => {
            let min = sub(*center, *half_extents);
            let max = add(*center, *half_extents);
            vec![
                (vec2(min.x, min.y), vec2(max.x, min.y), 0.0),
                (vec2(max.x, min.y), vec2(max.x, max.y), 0.0),
                (vec2(max.x, max.y), vec2(min.x, max.y), 0.0),
                (vec2(min.x, max.y), vec2(min.x, min.y), 0.0),
            ]
        }
    }
}

fn sweep_circle_vs_capsule(
    start: Vec2,
    end: Vec2,
    capsule_a: Vec2,
    capsule_b: Vec2,
    radius: Scalar,
) -> Option<(Scalar, Vec2, Vec2)> {
    let travel = sub(end, start);
    let mut best: Option<(Scalar, Vec2, Vec2)> = None;

    if distance_squared(capsule_a, capsule_b) <= Scalar::EPSILON {
        best = sweep_point_vs_circle(start, travel, capsule_a, radius);
    } else {
        let seg = sub(capsule_b, capsule_a);
        let seg_normal = normalize_or_zero(perp(seg));
        for normal in [seg_normal, mul(seg_normal, -1.0)] {
            let denom = dot(travel, normal);
            if denom < -Scalar::EPSILON {
                let t = (radius - dot(sub(start, capsule_a), normal)) / denom;
                if (0.0..=1.0).contains(&t) {
                    let center_at_hit = add(start, mul(travel, t));
                    let projected = dot(sub(center_at_hit, capsule_a), seg) / dot(seg, seg);
                    if (0.0..=1.0).contains(&projected) {
                        let contact = sub(center_at_hit, mul(normal, radius));
                        best = choose_earlier_sweep(best, Some((t, normal, contact)));
                    }
                }
            }
        }
        best = choose_earlier_sweep(
            best,
            sweep_point_vs_circle(start, travel, capsule_a, radius),
        );
        best = choose_earlier_sweep(
            best,
            sweep_point_vs_circle(start, travel, capsule_b, radius),
        );
    }

    best.filter(|(t, n, _)| *t >= 0.0 && *t <= 1.0 && is_finite_vec(*n))
}

fn choose_earlier_sweep(
    current: Option<(Scalar, Vec2, Vec2)>,
    next: Option<(Scalar, Vec2, Vec2)>,
) -> Option<(Scalar, Vec2, Vec2)> {
    match (current, next) {
        (Some(a), Some(b)) if a.0 <= b.0 => Some(a),
        (_, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        (None, None) => None,
    }
}

fn sweep_point_vs_circle(
    start: Vec2,
    travel: Vec2,
    center: Vec2,
    radius: Scalar,
) -> Option<(Scalar, Vec2, Vec2)> {
    let m = sub(start, center);
    let a = dot(travel, travel);
    if a <= Scalar::EPSILON {
        return None;
    }
    let b = 2.0 * dot(m, travel);
    let c = dot(m, m) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let root = discriminant.sqrt();
    let t = (-b - root) / (2.0 * a);
    if !(0.0..=1.0).contains(&t) {
        return None;
    }

    let center_at_hit = add(start, mul(travel, t));
    let normal = normalize_or_zero(sub(center_at_hit, center));
    Some((t, normal, sub(center_at_hit, mul(normal, radius))))
}

fn reflect(velocity: Vec2, normal: Vec2, restitution: Scalar, config: &SimConfig) -> Vec2 {
    let normal_speed = dot(velocity, normal);
    let reflected = sub(velocity, mul(normal, (1.0 + restitution) * normal_speed));
    let normal_component = mul(normal, dot(reflected, normal));
    let tangent_component = sub(reflected, normal_component);
    cap_speed(
        add(
            normal_component,
            mul(tangent_component, config.tangential_damping_on_collision),
        ),
        config.max_speed_cap,
    )
}

fn obstacle_restitution(kind: ObstacleKind, config: &SimConfig) -> Scalar {
    match kind {
        ObstacleKind::Wall => config.restitution.wall,
        ObstacleKind::Stone => config.restitution.stone_obstacle,
        ObstacleKind::Rubber => config.restitution.rubber_obstacle,
        ObstacleKind::Sensor => 0.0,
    }
}

fn bucket_center(board: &BoardDefinition, tick: Tick) -> Vec2 {
    let half_w = board.bucket.width * 0.5;
    let min_x = half_w;
    let max_x = board.size.x - half_w;
    let span = (max_x - min_x).max(0.0);
    if span <= Scalar::EPSILON || board.bucket.horizontal_speed == 0.0 {
        return board.bucket.center;
    }

    let dt = tick as Scalar / 120.0;
    let distance = (board.bucket.center.x - min_x) + board.bucket.horizontal_speed * dt;
    let period = span * 2.0;
    let wrapped = distance.rem_euclid(period);
    let x = if wrapped <= span {
        min_x + wrapped
    } else {
        max_x - (wrapped - span)
    };
    vec2(x, board.bucket.center.y)
}

fn bucket_contains_ball(board: &BoardDefinition, tick: Tick, position: Vec2) -> bool {
    let center = bucket_center(board, tick);
    let half_w = board.bucket.width * 0.5 + board.bucket.catch_margin;
    let top = center.y - board.bucket.height * 0.5 - board.bucket.catch_margin;
    let bottom = center.y + board.bucket.height * 0.5 + board.bucket.catch_margin;
    position.x >= center.x - half_w
        && position.x <= center.x + half_w
        && position.y >= top
        && position.y <= bottom
}

fn is_clearable(kind: PegKind) -> bool {
    !matches!(kind, PegKind::Stone)
}

pub fn stable_hash_events(events: &[PhysicsEvent]) -> String {
    let mut hasher = Sha256::new();
    for event in events {
        hash_event(&mut hasher, event);
    }
    hex(hasher.finalize().as_slice())
}

struct ReplayHasher {
    hasher: Sha256,
    event_cursor: usize,
}

impl ReplayHasher {
    fn new(seed: u64, board: &BoardDefinition, input: &ShotInput, config: &SimConfig) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"FeverfallReplayV1");
        hasher.update(seed.to_le_bytes());
        hash_str(&mut hasher, board.id.as_str());
        hash_vec2(&mut hasher, board.size);
        hash_vec2(&mut hasher, board.cannon_position);
        hash_scalar(&mut hasher, board.kill_plane_y);
        hash_str(&mut hasher, input.ball_id.as_str());
        hash_scalar(&mut hasher, input.aim_angle_radians);
        hash_scalar(&mut hasher, input.launch_speed);
        hash_scalar(&mut hasher, config.timestep_seconds);
        hash_vec2(&mut hasher, config.gravity);
        hash_scalar(&mut hasher, config.air_damping_per_tick);
        hash_scalar(&mut hasher, config.max_speed_cap);
        Self {
            hasher,
            event_cursor: 0,
        }
    }

    fn hash_tick(&mut self, tick: Tick, ball: &BallState, events: &[PhysicsEvent]) {
        self.hasher.update(b"tick");
        self.hasher.update(tick.to_le_bytes());
        hash_vec2(&mut self.hasher, ball.position);
        hash_vec2(&mut self.hasher, ball.velocity);
        while self.event_cursor < events.len() {
            hash_event(&mut self.hasher, &events[self.event_cursor]);
            self.event_cursor += 1;
        }
    }

    fn finish(self) -> String {
        hex(self.hasher.finalize().as_slice())
    }
}

fn hash_event(hash: &mut Sha256, event: &PhysicsEvent) {
    match event {
        PhysicsEvent::BallHitPeg {
            ball,
            peg,
            position,
            normal,
            speed,
            tick,
        } => {
            hash.update(b"BallHitPeg");
            hash_str(hash, ball.as_str());
            hash_str(hash, peg.as_str());
            hash_vec2(hash, *position);
            hash_vec2(hash, *normal);
            hash_scalar(hash, *speed);
            hash.update(tick.to_le_bytes());
        }
        PhysicsEvent::BallHitObstacle {
            ball,
            obstacle,
            position,
            normal,
            tick,
        } => {
            hash.update(b"BallHitObstacle");
            hash_str(hash, ball.as_str());
            hash_str(hash, obstacle.as_str());
            hash_vec2(hash, *position);
            hash_vec2(hash, *normal);
            hash.update(tick.to_le_bytes());
        }
        PhysicsEvent::BallEnteredBucket { ball, tick } => {
            hash.update(b"BallEnteredBucket");
            hash_str(hash, ball.as_str());
            hash.update(tick.to_le_bytes());
        }
        PhysicsEvent::BallExitedBoard { ball, tick } => {
            hash.update(b"BallExitedBoard");
            hash_str(hash, ball.as_str());
            hash.update(tick.to_le_bytes());
        }
        PhysicsEvent::ShotEnded { summary } => {
            hash.update(b"ShotEnded");
            hash.update(summary.ticks.to_le_bytes());
            for peg in &summary.pegs_hit {
                hash_str(hash, peg.as_str());
            }
            hash.update([summary.caught_bucket as u8, summary.exited_board as u8]);
            hash_str(hash, &summary.replay_hash);
        }
    }
}

fn hash_str(hash: &mut Sha256, value: &str) {
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value.as_bytes());
}

fn hash_vec2(hash: &mut Sha256, value: Vec2) {
    hash_scalar(hash, value.x);
    hash_scalar(hash, value.y);
}

fn hash_scalar(hash: &mut Sha256, value: Scalar) {
    hash.update(value.to_bits().to_le_bytes());
}

fn hex(bytes: &[u8]) -> String {
    const CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(CHARS[(byte >> 4) as usize] as char);
        out.push(CHARS[(byte & 0x0f) as usize] as char);
    }
    out
}

fn cap_speed(velocity: Vec2, max_speed: Scalar) -> Vec2 {
    let speed = length(velocity);
    if speed > max_speed && speed > Scalar::EPSILON {
        mul(velocity, max_speed / speed)
    } else {
        velocity
    }
}

fn vec2(x: Scalar, y: Scalar) -> Vec2 {
    Vec2::new(x, y)
}

fn add(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x + b.x, a.y + b.y)
}

fn sub(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x - b.x, a.y - b.y)
}

fn mul(a: Vec2, scalar: Scalar) -> Vec2 {
    vec2(a.x * scalar, a.y * scalar)
}

fn dot(a: Vec2, b: Vec2) -> Scalar {
    a.x * b.x + a.y * b.y
}

fn perp(a: Vec2) -> Vec2 {
    vec2(-a.y, a.x)
}

fn length(a: Vec2) -> Scalar {
    dot(a, a).sqrt()
}

fn distance_squared(a: Vec2, b: Vec2) -> Scalar {
    dot(sub(a, b), sub(a, b))
}

fn normalize_or_zero(a: Vec2) -> Vec2 {
    let len = length(a);
    if len > Scalar::EPSILON {
        mul(a, 1.0 / len)
    } else {
        Vec2::ZERO
    }
}

fn is_finite_vec(a: Vec2) -> bool {
    a.x.is_finite() && a.y.is_finite()
}

#[cfg(test)]
mod tests {
    use super::*;
    use content_schema::{minimal_test_board, BasketDef, BoardId, ObstacleDef, PegDef};

    fn ball_id() -> BallId {
        BallId::new("balls/basic").unwrap()
    }

    fn empty_board() -> BoardDefinition {
        BoardDefinition {
            id: BoardId::new("boards/physics_test").unwrap(),
            size: Vec2::new(20.0, 35.56),
            cannon_position: Vec2::new(10.0, 1.5),
            kill_plane_y: 36.5,
            pegs: Vec::new(),
            obstacles: Vec::new(),
            bucket: BasketDef {
                horizontal_speed: 0.0,
                ..BasketDef::spec_default()
            },
            tags: Vec::new(),
        }
    }

    fn assert_vec2_close(a: Vec2, b: Vec2, tolerance: Scalar) {
        assert!((a.x - b.x).abs() <= tolerance, "x: {} != {}", a.x, b.x);
        assert!((a.y - b.y).abs() <= tolerance, "y: {} != {}", a.y, b.y);
    }

    #[test]
    fn sim_config_defaults_match_initial_spec_values() {
        let config = SimConfig::default();

        assert_eq!(config.timestep_seconds, 1.0 / 120.0);
        assert_eq!(config.gravity, Vec2::new(0.0, 22.0));
        assert_eq!(config.max_speed_cap, 38.0);
        assert_eq!(config.restitution.peg, 0.94);
    }

    #[test]
    fn physics_event_round_trips_json() {
        let event = PhysicsEvent::BallEnteredBucket {
            ball: ball_id(),
            tick: 120,
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: PhysicsEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, event);
    }

    #[test]
    fn first_bounce_prediction_matches_simulated_circle_peg_collision() {
        let mut board = empty_board();
        board.pegs.push(PegDef {
            id: PegId::new("peg/prediction_circle").unwrap(),
            kind: PegKind::Blue,
            shape: ShapeDef::Circle {
                center: Vec2::new(10.0, 8.0),
                radius: 0.4,
            },
        });
        let input = ShotInput {
            aim_angle_radians: std::f64::consts::FRAC_PI_2,
            launch_speed: 17.5,
            ball_id: ball_id(),
        };

        let predicted = predict_first_bounce(&board, &input).unwrap();
        let simulated = simulate_shot(5, &board, &input)
            .events
            .into_iter()
            .find(is_collision_event)
            .unwrap();

        match (predicted, simulated) {
            (
                PhysicsEvent::BallHitPeg {
                    ball: predicted_ball,
                    peg: predicted_peg,
                    position: predicted_position,
                    normal: predicted_normal,
                    speed: predicted_speed,
                    tick: predicted_tick,
                },
                PhysicsEvent::BallHitPeg {
                    ball: simulated_ball,
                    peg: simulated_peg,
                    position: simulated_position,
                    normal: simulated_normal,
                    speed: simulated_speed,
                    tick: simulated_tick,
                },
            ) => {
                let tolerance = CONTACT_SLOP;
                assert_eq!(predicted_ball, simulated_ball);
                assert_eq!(predicted_peg, simulated_peg);
                assert_eq!(predicted_tick, simulated_tick);
                assert_vec2_close(predicted_position, simulated_position, tolerance);
                assert_vec2_close(predicted_normal, simulated_normal, tolerance);
                assert!((predicted_speed - simulated_speed).abs() <= tolerance);
            }
            (predicted, simulated) => panic!("unexpected events: {predicted:?} vs {simulated:?}"),
        }
    }

    #[test]
    fn event_hash_is_sha256_and_stable_for_same_events() {
        let events = vec![PhysicsEvent::BallHitPeg {
            ball: ball_id(),
            peg: PegId::new("peg/orange_001").unwrap(),
            position: Vec2::new(10.0, 12.0),
            normal: Vec2::new(0.0, -1.0),
            speed: 17.5,
            tick: 7,
        }];

        let hash = stable_hash_events(&events);
        assert_eq!(hash, stable_hash_events(&events));
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn circle_collision_reports_expected_normal() {
        let hit = sweep_circle_vs_capsule(
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(5.0, 0.0),
            1.0,
        )
        .unwrap();

        assert!((hit.0 - 0.4).abs() < 0.000001);
        assert_eq!(hit.1, Vec2::new(-1.0, 0.0));
    }

    #[test]
    fn capsule_side_collision_reports_expected_normal() {
        let hit = sweep_circle_vs_capsule(
            Vec2::new(5.0, 0.0),
            Vec2::new(5.0, 10.0),
            Vec2::new(3.0, 5.0),
            Vec2::new(7.0, 5.0),
            1.0,
        )
        .unwrap();

        assert!((hit.0 - 0.4).abs() < 0.000001);
        assert_eq!(hit.1, Vec2::new(0.0, -1.0));
    }

    #[test]
    fn segment_endpoint_collision_reports_expected_normal() {
        let hit = sweep_circle_vs_capsule(
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(5.0, 1.0),
            Vec2::new(8.0, 1.0),
            1.0,
        )
        .unwrap();

        assert!((hit.0 - 0.5).abs() < 0.000001);
        assert_eq!(hit.1, Vec2::new(0.0, -1.0));
    }

    #[test]
    fn no_tunneling_at_max_speed_against_thin_segment() {
        let mut board = empty_board();
        board.obstacles.push(ObstacleDef {
            id: ObstacleId::new("obstacles/wall").unwrap(),
            kind: ObstacleKind::Wall,
            shape: ShapeDef::Segment {
                a: Vec2::new(10.0, 5.0),
                b: Vec2::new(10.0, 30.0),
            },
        });
        let input = ShotInput {
            aim_angle_radians: 0.0,
            launch_speed: 38.0,
            ball_id: ball_id(),
        };
        board.cannon_position = Vec2::new(1.0, 10.0);

        let result = simulate_shot(1, &board, &input);

        assert!(result
            .events
            .iter()
            .any(|event| matches!(event, PhysicsEvent::BallHitObstacle { .. })));
    }

    #[test]
    fn bucket_catch_geometry_emits_event_and_ends_shot() {
        let mut board = empty_board();
        board.cannon_position = Vec2::new(10.0, 33.0);
        board.bucket.center = Vec2::new(10.0, 34.4);
        let input = ShotInput {
            aim_angle_radians: std::f64::consts::FRAC_PI_2,
            launch_speed: 4.0,
            ball_id: ball_id(),
        };

        let result = simulate_shot(2, &board, &input);

        assert!(result.summary.caught_bucket);
        assert!(result
            .events
            .iter()
            .any(|event| matches!(event, PhysicsEvent::BallEnteredBucket { .. })));
    }

    #[test]
    fn peg_lit_state_clears_clearable_pegs_only_at_shot_end() {
        let board = minimal_test_board();
        let input = ShotInput {
            aim_angle_radians: std::f64::consts::FRAC_PI_2,
            launch_speed: 17.5,
            ball_id: ball_id(),
        };

        let result = simulate_shot(3, &board, &input);

        assert!(result
            .summary
            .pegs_hit
            .contains(&PegId::new("peg/orange_001").unwrap()));
        assert!(!result
            .remaining_pegs
            .iter()
            .any(|peg| peg.id.as_str() == "peg/orange_001"));
    }

    #[test]
    fn simulation_produces_no_nan() {
        let board = minimal_test_board();
        let input = ShotInput {
            aim_angle_radians: 1.2,
            launch_speed: 22.0,
            ball_id: ball_id(),
        };

        let result = simulate_shot(4, &board, &input);

        for event in result.events {
            match event {
                PhysicsEvent::BallHitPeg {
                    position, normal, ..
                }
                | PhysicsEvent::BallHitObstacle {
                    position, normal, ..
                } => {
                    assert!(is_finite_vec(position));
                    assert!(is_finite_vec(normal));
                }
                _ => {}
            }
        }
    }

    #[test]
    fn deterministic_replay_hash_is_stable_for_same_shot() {
        let board = minimal_test_board();
        let input = ShotInput {
            aim_angle_radians: 1.18,
            launch_speed: 17.5,
            ball_id: ball_id(),
        };

        let a = simulate_shot(123, &board, &input);
        let b = simulate_shot(123, &board, &input);

        assert_eq!(a.summary.replay_hash, b.summary.replay_hash);
        assert_eq!(
            a.summary.replay_hash,
            "e5e9b4f955205cde05b28df55c8e03a0016220091c14925def3261d8687fc9e2"
        );
        assert_eq!(a.summary.replay_hash.len(), 64);
    }

    #[test]
    fn sampled_trajectory_is_deterministic_and_reaches_shot_end_tick() {
        let board = minimal_test_board();
        let input = ShotInput {
            aim_angle_radians: 1.18,
            launch_speed: 17.5,
            ball_id: ball_id(),
        };

        let a = sample_shot_trajectory(&board, &input, 12);
        let b = sample_shot_trajectory(&board, &input, 12);
        let result = simulate_shot(123, &board, &input);

        assert_eq!(a, b);
        assert_eq!(a.first().unwrap().tick, 0);
        assert_eq!(a.first().unwrap().position, board.cannon_position);
        assert_eq!(a.last().unwrap().tick, result.summary.ticks);
        assert!(a.iter().all(|sample| is_finite_vec(sample.position)));
    }

    #[test]
    fn stress_10000_randomish_shots_do_not_stick_or_loop_forever() {
        let mut board = minimal_test_board();
        board.pegs.push(PegDef {
            id: PegId::new("peg/blue_capsule").unwrap(),
            kind: PegKind::Blue,
            shape: ShapeDef::Capsule {
                a: Vec2::new(5.0, 18.0),
                b: Vec2::new(15.0, 19.0),
                radius: 0.25,
            },
        });
        board.obstacles.push(ObstacleDef {
            id: ObstacleId::new("obstacles/rubber_floor").unwrap(),
            kind: ObstacleKind::Rubber,
            shape: ShapeDef::Segment {
                a: Vec2::new(2.0, 28.0),
                b: Vec2::new(18.0, 31.0),
            },
        });

        let mut state = 0x5eed_u64;
        for _ in 0..10_000 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let unit = ((state >> 11) as f64) / ((1_u64 << 53) as f64);
            let angle = 0.15 + unit * (std::f64::consts::PI - 0.3);
            let speed = 14.0 + (1.0 - unit) * 8.0;
            let input = ShotInput {
                aim_angle_radians: angle,
                launch_speed: speed,
                ball_id: ball_id(),
            };
            let result = simulate_shot(state, &board, &input);
            assert!(result.summary.ticks <= MAX_TICKS_PER_SHOT);
            assert!(result.summary.caught_bucket || result.summary.exited_board);
            assert_eq!(result.summary.replay_hash.len(), 64);
        }
    }
}
