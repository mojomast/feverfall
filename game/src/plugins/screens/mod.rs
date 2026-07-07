//! Real Bevy ECS screen rendering for Checkpoint 5.
//!
//! The gameplay crates remain the source of deterministic state.  This module
//! adapts the existing screen models into renderable Bevy entities with a
//! single tagged root per active screen so state exits can cleanly despawn the
//! whole visual tree.

#[cfg(feature = "bevy_feel_test")]
use bevy::prelude::*;

#[cfg(feature = "bevy_feel_test")]
use crate::plugins::{
    reward_ui::{act1_feel_test_relic_offer, RewardChoiceScreen},
    ui::PlaceholderScreenSuite,
};

#[allow(dead_code)]
pub const PRODUCTION_SCREEN_COUNT: usize = 13;

#[allow(dead_code)]
pub const RENDER_ASSET_MANIFEST: &[&str] = &[
    "game/assets/sprites/feverfall_ui_atlas.png",
    "game/assets/sprites/feverfall_icons.png",
    "game/assets/sprites/ATTRIBUTION.md",
    "game/assets/fonts/OFL.txt",
    "game/assets/fonts/ATTRIBUTION.md",
];

#[allow(dead_code)]
pub const FONT_LICENSE_FILES: &[&str] = &[
    "game/assets/fonts/OFL.txt",
    "game/assets/fonts/ATTRIBUTION.md",
];

#[cfg(feature = "bevy_feel_test")]
#[derive(Clone, Copy, Debug, Default, Resource, PartialEq, Eq)]
pub struct CurrentScreen(pub ScreenKind);

#[cfg(feature = "bevy_feel_test")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ScreenKind {
    #[default]
    MainMenu,
    Settings,
    BoardHudRelicBar,
    RewardChoice,
    NodeMap,
    Shop,
    Forge,
    Event,
    RunSummaryFailure,
    RpgChapterSelect,
    RpgGear,
    RpgSkillTree,
    CampaignProgress,
}

#[cfg(feature = "bevy_feel_test")]
#[allow(dead_code)]
impl ScreenKind {
    pub const ALL: [Self; PRODUCTION_SCREEN_COUNT] = [
        Self::MainMenu,
        Self::Settings,
        Self::BoardHudRelicBar,
        Self::RewardChoice,
        Self::NodeMap,
        Self::Shop,
        Self::Forge,
        Self::Event,
        Self::RunSummaryFailure,
        Self::RpgChapterSelect,
        Self::RpgGear,
        Self::RpgSkillTree,
        Self::CampaignProgress,
    ];
}

#[cfg(feature = "bevy_feel_test")]
#[derive(Component)]
#[allow(dead_code)]
pub struct ScreenRoot {
    pub kind: ScreenKind,
}

#[cfg(feature = "bevy_feel_test")]
#[derive(Clone, Copy, Debug, Component, PartialEq, Eq)]
pub enum ScreenElementKind {
    Panel,
    Button,
    Card,
    ReachableNode,
    LockedNode,
    RelicIcon,
    SkillNode,
    Label,
}

#[cfg(feature = "bevy_feel_test")]
#[derive(Component)]
#[allow(dead_code)]
pub struct ScreenElement {
    pub kind: ScreenElementKind,
}

#[cfg(feature = "bevy_feel_test")]
#[allow(dead_code)]
pub fn plugin(app: &mut App) {
    app.init_resource::<CurrentScreen>()
        .add_systems(Startup, spawn_current_screen);
}

#[cfg(feature = "bevy_feel_test")]
pub fn spawn_current_screen(
    mut commands: Commands,
    screen: Res<CurrentScreen>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_screen(&mut commands, screen.0, &mut meshes, &mut materials);
}

#[cfg(feature = "bevy_feel_test")]
#[allow(dead_code)]
pub fn despawn_screen_entities(mut commands: Commands, roots: Query<Entity, With<ScreenRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

#[cfg(feature = "bevy_feel_test")]
pub fn spawn_screen(
    commands: &mut Commands,
    kind: ScreenKind,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> Entity {
    let root = commands
        .spawn((
            ScreenRoot { kind },
            Name::new(format!("screen::{kind:?}")),
            Transform::default(),
            Visibility::Visible,
        ))
        .id();

    let suite = PlaceholderScreenSuite::sample();
    match kind {
        ScreenKind::MainMenu => {
            spawn_menu(commands, root, &suite.main_menu.title, 4, meshes, materials)
        }
        ScreenKind::Settings => {
            spawn_menu(commands, root, &suite.settings.title, 9, meshes, materials)
        }
        ScreenKind::BoardHudRelicBar => spawn_board_hud(commands, root, meshes, materials),
        ScreenKind::RewardChoice => {
            let screen = RewardChoiceScreen::from_offer(&act1_feel_test_relic_offer());
            spawn_reward_cards(commands, root, screen.cards.len(), meshes, materials);
        }
        ScreenKind::NodeMap => spawn_node_map(commands, root, meshes, materials),
        ScreenKind::Shop => spawn_menu(
            commands,
            root,
            &suite.roguelite.shop.title,
            suite.roguelite.shop.offers.len(),
            meshes,
            materials,
        ),
        ScreenKind::Forge => spawn_menu(
            commands,
            root,
            &suite.roguelite.forge.title,
            suite.roguelite.forge.upgrades.len(),
            meshes,
            materials,
        ),
        ScreenKind::Event => spawn_menu(
            commands,
            root,
            &suite.roguelite.event.title,
            suite.roguelite.event.choices.len(),
            meshes,
            materials,
        ),
        ScreenKind::RunSummaryFailure => {
            spawn_menu(commands, root, "Run Summary", 5, meshes, materials)
        }
        ScreenKind::RpgChapterSelect => spawn_menu(
            commands,
            root,
            &suite.rpg.chapter_select.title,
            suite.rpg.chapter_select.chapters.len(),
            meshes,
            materials,
        ),
        ScreenKind::RpgGear => spawn_menu(
            commands,
            root,
            &suite.rpg.gear.title,
            suite.rpg.gear.slots.len(),
            meshes,
            materials,
        ),
        ScreenKind::RpgSkillTree => spawn_skill_tree(commands, root, meshes, materials),
        ScreenKind::CampaignProgress => spawn_menu(
            commands,
            root,
            &suite.rpg.campaign_progress.title,
            suite.rpg.campaign_progress.next_objectives.len(),
            meshes,
            materials,
        ),
    }
    root
}

#[cfg(feature = "bevy_feel_test")]
fn spawn_menu(
    commands: &mut Commands,
    root: Entity,
    title: &str,
    rows: usize,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    spawn_rect(
        commands,
        root,
        meshes,
        materials,
        Vec2::new(0.0, 0.0),
        Vec2::new(680.0, 460.0),
        Color::srgb(0.06, 0.08, 0.14),
        ScreenElementKind::Panel,
    );
    spawn_label(commands, root, title, Vec2::new(-280.0, 180.0), 32.0);
    for row in 0..rows.max(1) {
        spawn_rect(
            commands,
            root,
            meshes,
            materials,
            Vec2::new(0.0, 110.0 - row as f32 * 58.0),
            Vec2::new(520.0, 42.0),
            Color::srgb(0.13, 0.17, 0.28),
            ScreenElementKind::Button,
        );
    }
}

#[cfg(feature = "bevy_feel_test")]
fn spawn_board_hud(
    commands: &mut Commands,
    root: Entity,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    spawn_rect(
        commands,
        root,
        meshes,
        materials,
        Vec2::new(0.0, -20.0),
        Vec2::new(520.0, 560.0),
        Color::srgb(0.03, 0.08, 0.12),
        ScreenElementKind::Panel,
    );
    spawn_label(
        commands,
        root,
        "Board / HUD",
        Vec2::new(-245.0, 285.0),
        26.0,
    );
    for col in 0..6 {
        spawn_rect(
            commands,
            root,
            meshes,
            materials,
            Vec2::new(-225.0 + col as f32 * 90.0, 245.0),
            Vec2::splat(38.0),
            Color::srgb(0.90, 0.47, 0.17),
            ScreenElementKind::RelicIcon,
        );
    }
}

#[cfg(feature = "bevy_feel_test")]
fn spawn_reward_cards(
    commands: &mut Commands,
    root: Entity,
    count: usize,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    spawn_label(
        commands,
        root,
        "Choose a Reward",
        Vec2::new(-260.0, 185.0),
        30.0,
    );
    for index in 0..count {
        spawn_rect(
            commands,
            root,
            meshes,
            materials,
            Vec2::new(-220.0 + index as f32 * 220.0, 0.0),
            Vec2::new(170.0, 255.0),
            Color::srgb(0.18, 0.12, 0.28),
            ScreenElementKind::Card,
        );
    }
}

#[cfg(feature = "bevy_feel_test")]
fn spawn_node_map(
    commands: &mut Commands,
    root: Entity,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    spawn_label(
        commands,
        root,
        "Roguelite Node Map",
        Vec2::new(-300.0, 230.0),
        30.0,
    );
    for index in 0..8 {
        let reachable = index < 3;
        spawn_rect(
            commands,
            root,
            meshes,
            materials,
            Vec2::new(
                -300.0 + index as f32 * 85.0,
                if index % 2 == 0 { 40.0 } else { -40.0 },
            ),
            Vec2::splat(44.0),
            if reachable {
                Color::srgb(0.20, 0.82, 0.55)
            } else {
                Color::srgb(0.16, 0.18, 0.24)
            },
            if reachable {
                ScreenElementKind::ReachableNode
            } else {
                ScreenElementKind::LockedNode
            },
        );
    }
}

#[cfg(feature = "bevy_feel_test")]
fn spawn_skill_tree(
    commands: &mut Commands,
    root: Entity,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    spawn_label(
        commands,
        root,
        "Skill Trees",
        Vec2::new(-280.0, 220.0),
        30.0,
    );
    for tree in 0..4 {
        for node in 0..3 {
            spawn_rect(
                commands,
                root,
                meshes,
                materials,
                Vec2::new(-240.0 + tree as f32 * 160.0, 120.0 - node as f32 * 105.0),
                Vec2::splat(42.0),
                Color::srgb(0.35, 0.45, 0.95),
                ScreenElementKind::SkillNode,
            );
        }
    }
}

#[cfg(feature = "bevy_feel_test")]
fn spawn_label(commands: &mut Commands, root: Entity, text: &str, position: Vec2, font_size: f32) {
    let child = commands
        .spawn((
            Text2d::new(text.to_owned()),
            TextFont {
                font_size: FontSize::Px(font_size),
                ..default()
            },
            TextColor(Color::srgb(0.92, 0.94, 1.0)),
            Transform::from_translation(position.extend(5.0)),
            ScreenElement {
                kind: ScreenElementKind::Label,
            },
        ))
        .id();
    commands.entity(root).add_child(child);
}

#[cfg(feature = "bevy_feel_test")]
#[allow(clippy::too_many_arguments)]
fn spawn_rect(
    commands: &mut Commands,
    root: Entity,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    position: Vec2,
    size: Vec2,
    color: Color,
    kind: ScreenElementKind,
) {
    let child = commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(size.x, size.y))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_translation(position.extend(0.0)),
            ScreenElement { kind },
        ))
        .id();
    commands.entity(root).add_child(child);
}

#[cfg(all(test, feature = "bevy_feel_test"))]
mod tests {
    use super::*;

    fn test_app(kind: ScreenKind) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .insert_resource(CurrentScreen(kind))
            .add_systems(Update, spawn_current_screen);
        app
    }

    #[test]
    fn screen_state_spawns_one_root() {
        for kind in ScreenKind::ALL {
            let mut app = test_app(kind);
            app.update();
            let roots = app
                .world_mut()
                .query_filtered::<&ScreenRoot, With<ScreenRoot>>()
                .iter(app.world())
                .count();
            assert_eq!(roots, 1, "{kind:?} should spawn one screen root");
        }
    }

    #[test]
    fn screen_exit_despawns_screen_entities() {
        let mut app = test_app(ScreenKind::MainMenu);
        app.add_systems(Update, despawn_screen_entities.after(spawn_current_screen));
        app.update();
        let roots = app
            .world_mut()
            .query_filtered::<&ScreenRoot, With<ScreenRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(roots, 0);
    }

    #[test]
    fn reward_screen_spawns_three_cards() {
        let mut app = test_app(ScreenKind::RewardChoice);
        app.update();
        let cards = app
            .world_mut()
            .query::<&ScreenElement>()
            .iter(app.world())
            .filter(|element| element.kind == ScreenElementKind::Card)
            .count();
        assert_eq!(cards, 3);
    }

    #[test]
    fn node_map_screen_spawns_reachable_nodes() {
        let mut app = test_app(ScreenKind::NodeMap);
        app.update();
        let reachable = app
            .world_mut()
            .query::<&ScreenElement>()
            .iter(app.world())
            .filter(|element| element.kind == ScreenElementKind::ReachableNode)
            .count();
        assert!(reachable >= 3);
    }

    #[test]
    fn rpg_skill_tree_spawns_skill_nodes() {
        let mut app = test_app(ScreenKind::RpgSkillTree);
        app.update();
        let nodes = app
            .world_mut()
            .query::<&ScreenElement>()
            .iter(app.world())
            .filter(|element| element.kind == ScreenElementKind::SkillNode)
            .count();
        assert_eq!(nodes, 12);
    }

    #[test]
    fn render_asset_manifest_paths_exist() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        for path in RENDER_ASSET_MANIFEST {
            assert!(root.join(path).exists(), "missing render asset {path}");
        }
    }

    #[test]
    fn font_license_files_exist() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        for path in FONT_LICENSE_FILES {
            assert!(root.join(path).exists(), "missing font license {path}");
        }
    }
}
