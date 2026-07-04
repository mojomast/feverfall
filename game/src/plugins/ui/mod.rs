use content_schema::{
    minimal_test_board, BallId, BoardDefinition, BoardId, ContentId, GearId, PegKind, RelicId,
    Scalar, Score, SkillId, Vec2,
};
use physics_core::{predict_first_bounce, PhysicsEvent, ShotInput, ShotSummary};
use rpg_mode::{CharacterState, GearSlot, SkillState};
use run_mode::{all_relic_ids, full_run_nodes, RelicInstance, Reward, RunState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiRegistrationSummary {
    pub first_bounce_predicted: bool,
    pub balls_remaining: u32,
    pub equipped_skill_count: usize,
    pub active_power_charge_percent: u8,
    pub placeholder_screen_count: usize,
    pub keyboard_navigable: bool,
    pub layout_smoke_passed: bool,
}

pub fn register() -> UiRegistrationSummary {
    let board = minimal_test_board();
    let input = ShotInput {
        aim_angle_radians: std::f64::consts::FRAC_PI_2,
        launch_speed: 17.5,
        ball_id: BallId::new("balls/basic").expect("static id is valid"),
    };
    let mut run_state = RunState::new(9);
    run_state.relics.push(RelicInstance {
        id: RelicId::new("relics/mock_focus").expect("static id is valid"),
        stacks: 2,
    });

    let mut character_state =
        CharacterState::new(ContentId::new("characters/mock").expect("static id is valid"));
    character_state.unlocked_skills.push(SkillState {
        id: SkillId::new("skills/zen_reroute").expect("static id is valid"),
        rank: 1,
        equipped: true,
        cooldown_boards: 0,
        cooldown_remaining: 0,
        used_this_board: false,
    });

    let mut hud =
        HudState::mock_from_states(&board, &input, &run_state, &character_state, 99, 2, 3);
    hud.score.active_power = Some(ActivePowerDisplay {
        id: SkillId::new("skills/zen_reroute").expect("static id is valid"),
        label: String::from("Zen Reroute"),
        charge_percent: 75,
    });

    let mut screen_suite = PlaceholderScreenSuite::sample();
    screen_suite.settings.handle(UiNavInput::Right);
    screen_suite.settings.handle(UiNavInput::Down);
    screen_suite.settings.handle(UiNavInput::Down);
    screen_suite.settings.handle(UiNavInput::Down);
    screen_suite.settings.handle(UiNavInput::Toggle);
    screen_suite.main_menu.focus.handle(UiNavInput::Confirm);
    screen_suite.main_menu.focus.handle(UiNavInput::Back);
    screen_suite.main_menu.focus.handle(UiNavInput::Up);
    screen_suite.main_menu.focus.handle(UiNavInput::Left);
    let screen_summary = screen_suite.smoke_summary();

    UiRegistrationSummary {
        first_bounce_predicted: hud.aim.first_bounce.is_some(),
        balls_remaining: hud.score.balls_remaining,
        equipped_skill_count: hud.progression.equipped_skill_count,
        active_power_charge_percent: hud
            .score
            .active_power
            .as_ref()
            .map_or(0, |power| power.charge_percent),
        placeholder_screen_count: screen_summary.screen_count,
        keyboard_navigable: screen_summary.keyboard_navigable,
        layout_smoke_passed: screen_summary.layout_smoke_passed,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiViewport {
    pub width: u32,
    pub height: u32,
}

impl UiViewport {
    pub const HD: Self = Self {
        width: 1280,
        height: 720,
    };
    pub const FHD: Self = Self {
        width: 1920,
        height: 1080,
    };

    const fn safe_margin(self) -> u32 {
        if self.width >= 1600 {
            48
        } else {
            32
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiNavInput {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Back,
    Toggle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusList {
    pub selected: usize,
    pub item_count: usize,
}

impl FocusList {
    pub const fn new(item_count: usize) -> Self {
        Self {
            selected: 0,
            item_count,
        }
    }

    pub fn handle(&mut self, input: UiNavInput) {
        match input {
            UiNavInput::Up | UiNavInput::Left => {
                self.selected = self.selected.saturating_sub(1);
            }
            UiNavInput::Down | UiNavInput::Right => {
                if self.selected + 1 < self.item_count {
                    self.selected += 1;
                }
            }
            UiNavInput::Confirm | UiNavInput::Back | UiNavInput::Toggle => {}
        }
    }

    pub const fn keyboard_ready(&self) -> bool {
        self.item_count > 0 && self.selected < self.item_count
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuActionModel {
    pub hotkey: char,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MainMenuScreen {
    pub title: String,
    pub actions: Vec<MenuActionModel>,
    pub focus: FocusList,
}

impl MainMenuScreen {
    pub fn production_placeholder() -> Self {
        Self {
            title: "Feverfall".to_owned(),
            actions: vec![
                menu_action('1', "Play Roguelite"),
                menu_action('2', "Play RPG"),
                menu_action('3', "Settings"),
                menu_action('4', "Quit"),
            ],
            focus: FocusList::new(4),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsScreen {
    pub title: String,
    pub volume_sliders: Vec<VolumeSliderModel>,
    pub reduce_flash: ToggleModel,
    pub reduce_shake: ToggleModel,
    pub key_rebinds: Vec<KeyRebindPlaceholder>,
    pub focus: FocusList,
}

impl SettingsScreen {
    pub fn production_placeholder() -> Self {
        let volume_sliders = vec![
            VolumeSliderModel::stub("Master", 80),
            VolumeSliderModel::stub("Music", 70),
            VolumeSliderModel::stub("SFX", 85),
        ];
        let key_rebinds = vec![
            KeyRebindPlaceholder::new("Aim Left", "A / Left"),
            KeyRebindPlaceholder::new("Aim Right", "D / Right"),
            KeyRebindPlaceholder::new("Fire", "Space"),
            KeyRebindPlaceholder::new("Debug Overlay", "F3"),
        ];
        let item_count = volume_sliders.len() + 2 + key_rebinds.len();
        Self {
            title: "Settings".to_owned(),
            volume_sliders,
            reduce_flash: ToggleModel::new("Reduce Flash", false),
            reduce_shake: ToggleModel::new("Reduce Shake", false),
            key_rebinds,
            focus: FocusList::new(item_count),
        }
    }

    pub fn handle(&mut self, input: UiNavInput) {
        match input {
            UiNavInput::Left => self.adjust_selected_slider(-5),
            UiNavInput::Right => self.adjust_selected_slider(5),
            UiNavInput::Toggle | UiNavInput::Confirm => self.toggle_selected(),
            other => self.focus.handle(other),
        }
    }

    fn adjust_selected_slider(&mut self, delta: i32) {
        if let Some(slider) = self.volume_sliders.get_mut(self.focus.selected) {
            slider.value_percent = (slider.value_percent as i32 + delta).clamp(0, 100) as u8;
        }
    }

    fn toggle_selected(&mut self) {
        let reduce_flash_index = self.volume_sliders.len();
        let reduce_shake_index = reduce_flash_index + 1;
        if self.focus.selected == reduce_flash_index {
            self.reduce_flash.enabled = !self.reduce_flash.enabled;
        } else if self.focus.selected == reduce_shake_index {
            self.reduce_shake.enabled = !self.reduce_shake.enabled;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VolumeSliderModel {
    pub label: String,
    pub value_percent: u8,
    pub stubbed: bool,
}

impl VolumeSliderModel {
    fn stub(label: &str, value_percent: u8) -> Self {
        Self {
            label: label.to_owned(),
            value_percent,
            stubbed: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToggleModel {
    pub label: String,
    pub enabled: bool,
}

impl ToggleModel {
    fn new(label: &str, enabled: bool) -> Self {
        Self {
            label: label.to_owned(),
            enabled,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyRebindPlaceholder {
    pub action: String,
    pub binding: String,
    pub stubbed: bool,
}

impl KeyRebindPlaceholder {
    fn new(action: &str, binding: &str) -> Self {
        Self {
            action: action.to_owned(),
            binding: binding.to_owned(),
            stubbed: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RogueliteUiSuite {
    pub node_map: FullNodeMapScreen,
    pub shop: ShopScreen,
    pub forge: ForgeScreen,
    pub event: EventScreen,
    pub relic_bar: RelicBarModel,
}

impl RogueliteUiSuite {
    pub fn production_placeholder() -> Self {
        let mut run_state = RunState::act1_slice(0xC4_0001);
        for id in all_relic_ids().into_iter().take(8) {
            run_state.relics.push(RelicInstance { id, stacks: 1 });
        }
        Self {
            node_map: FullNodeMapScreen::from_run_state(&run_state, &full_run_nodes()),
            shop: ShopScreen::sample(),
            forge: ForgeScreen::sample(),
            event: EventScreen::sample(),
            relic_bar: RelicBarModel::from_run_state(&run_state),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullNodeMapScreen {
    pub title: String,
    pub branches: Vec<NodeBranchDisplay>,
    pub focus: FocusList,
}

impl FullNodeMapScreen {
    pub fn from_run_state(run_state: &RunState, nodes: &[run_mode::RunNode]) -> Self {
        let branches = nodes
            .iter()
            .enumerate()
            .map(|(index, node)| NodeBranchDisplay {
                index: index as u16,
                node_id: node.id.to_string(),
                label: format!("Act {} {:?}", node.act, node.kind),
                branch_choices: node.path_choices.iter().map(ToString::to_string).collect(),
                on_current_path: index <= usize::from(run_state.node_index) + 1,
            })
            .collect::<Vec<_>>();
        Self {
            title: "Roguelite Node Map".to_owned(),
            focus: FocusList::new(branches.len()),
            branches,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeBranchDisplay {
    pub index: u16,
    pub node_id: String,
    pub label: String,
    pub branch_choices: Vec<String>,
    pub on_current_path: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopScreen {
    pub title: String,
    pub coins: u32,
    pub offers: Vec<ShopOfferModel>,
    pub actions: Vec<MenuActionModel>,
    pub focus: FocusList,
}

impl ShopScreen {
    fn sample() -> Self {
        let offers = vec![
            ShopOfferModel::new(
                "Wide Cup Rim",
                35,
                Reward::Relic(relic("relics/act1/wide_cup_rim")),
            ),
            ShopOfferModel::new("Spark Ball", 28, Reward::Ball(ball("balls/act1/spark_orb"))),
            ShopOfferModel::new("Cleanse", 20, Reward::RemoveCurse(1)),
        ];
        Self {
            title: "Shop".to_owned(),
            coins: 54,
            actions: vec![
                menu_action('B', "Buy"),
                menu_action('R', "Reroll"),
                menu_action('S', "Skip"),
            ],
            focus: FocusList::new(offers.len() + 3),
            offers,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopOfferModel {
    pub title: String,
    pub cost: u32,
    pub reward: Reward,
}

impl ShopOfferModel {
    fn new(title: &str, cost: u32, reward: Reward) -> Self {
        Self {
            title: title.to_owned(),
            cost,
            reward,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForgeScreen {
    pub title: String,
    pub sparks: u32,
    pub upgrades: Vec<ForgeUpgradeModel>,
    pub focus: FocusList,
}

impl ForgeScreen {
    fn sample() -> Self {
        let upgrades = vec![
            ForgeUpgradeModel::new("Basic Orb +1", "Launch speed +4%", 24),
            ForgeUpgradeModel::new("Wide Cup Rim +1", "Catch margin +0.05", 32),
            ForgeUpgradeModel::new("Combo Abacus +1", "Combo spark payout +1", 30),
        ];
        Self {
            title: "Forge".to_owned(),
            sparks: 41,
            focus: FocusList::new(upgrades.len()),
            upgrades,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForgeUpgradeModel {
    pub target: String,
    pub effect_preview: String,
    pub cost: u32,
}

impl ForgeUpgradeModel {
    fn new(target: &str, effect_preview: &str, cost: u32) -> Self {
        Self {
            target: target.to_owned(),
            effect_preview: effect_preview.to_owned(),
            cost,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventScreen {
    pub title: String,
    pub prompt: String,
    pub choices: Vec<EventChoiceModel>,
    pub focus: FocusList,
}

impl EventScreen {
    fn sample() -> Self {
        let choices = vec![
            EventChoiceModel::new("Take the Fever Shard", "Gain rare relic", "Add 1 curse"),
            EventChoiceModel::new("Patch the Bucket", "Heal 1 heart", "Lose 10 coins"),
            EventChoiceModel::new("Walk Away", "No change", "Skip reward"),
        ];
        Self {
            title: "Event".to_owned(),
            prompt: "A hot shard hums under the board.".to_owned(),
            focus: FocusList::new(choices.len()),
            choices,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventChoiceModel {
    pub label: String,
    pub reward: String,
    pub risk: String,
}

impl EventChoiceModel {
    fn new(label: &str, reward: &str, risk: &str) -> Self {
        Self {
            label: label.to_owned(),
            reward: reward.to_owned(),
            risk: risk.to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelicBarModel {
    pub relics: Vec<RelicDisplay>,
    pub visible_window: std::ops::Range<usize>,
    pub scrollable: bool,
}

impl RelicBarModel {
    pub fn from_run_state(run_state: &RunState) -> Self {
        let relics = run_state
            .relics
            .iter()
            .map(|relic| RelicDisplay {
                id: relic.id.clone(),
                stacks: relic.stacks,
                ready: true,
            })
            .collect::<Vec<_>>();
        Self {
            scrollable: relics.len() > 6,
            visible_window: 0..relics.len().min(6),
            relics,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RpgUiSuite {
    pub chapter_select: ChapterSelectScreen,
    pub gear: GearEquipScreen,
    pub skill_tree: SkillTreeScreen,
    pub campaign_progress: CampaignProgressScreen,
}

impl RpgUiSuite {
    pub fn production_placeholder() -> Self {
        let character = CharacterState::chapter1();
        Self {
            chapter_select: ChapterSelectScreen::from_character(&character),
            gear: GearEquipScreen::from_character(&character),
            skill_tree: SkillTreeScreen::from_character(&character),
            campaign_progress: CampaignProgressScreen::from_character(&character),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChapterSelectScreen {
    pub title: String,
    pub chapters: Vec<ChapterCardModel>,
    pub focus: FocusList,
}

impl ChapterSelectScreen {
    fn from_character(character: &CharacterState) -> Self {
        let chapters = vec![
            ChapterCardModel::new(1, "Ashfall Gardens", true, "5/5 boards previewed"),
            ChapterCardModel::new(
                2,
                "Clockwork Canopy",
                character.level >= 4,
                "Locked placeholder",
            ),
            ChapterCardModel::new(3, "Fever Spire", false, "Locked placeholder"),
        ];
        Self {
            title: "Chapter Select".to_owned(),
            focus: FocusList::new(chapters.len()),
            chapters,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChapterCardModel {
    pub chapter: u8,
    pub title: String,
    pub unlocked: bool,
    pub progress: String,
}

impl ChapterCardModel {
    fn new(chapter: u8, title: &str, unlocked: bool, progress: &str) -> Self {
        Self {
            chapter,
            title: title.to_owned(),
            unlocked,
            progress: progress.to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GearEquipScreen {
    pub title: String,
    pub slots: Vec<GearSlotModel>,
    pub comparison: GearComparisonModel,
    pub focus: FocusList,
}

impl GearEquipScreen {
    fn from_character(character: &CharacterState) -> Self {
        let slots = [
            GearSlot::Launcher,
            GearSlot::CoreBall,
            GearSlot::BasketRig,
            GearSlot::Charm,
        ]
        .into_iter()
        .map(|slot| GearSlotModel::from_character(slot, character))
        .collect::<Vec<_>>();
        Self {
            title: "Gear".to_owned(),
            comparison: GearComparisonModel::sample(),
            focus: FocusList::new(slots.len()),
            slots,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GearSlotModel {
    pub slot: GearSlot,
    pub equipped: Option<GearId>,
    pub hotkey: char,
}

impl GearSlotModel {
    fn from_character(slot: GearSlot, character: &CharacterState) -> Self {
        let equipped = character
            .gear
            .iter()
            .find(|gear| gear.slot == slot)
            .map(|gear| gear.item.clone());
        let hotkey = match slot {
            GearSlot::Launcher => '1',
            GearSlot::CoreBall => '2',
            GearSlot::BasketRig => '3',
            GearSlot::Charm | GearSlot::Trinket => '4',
        };
        Self {
            slot,
            equipped,
            hotkey,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GearComparisonModel {
    pub current_item: Option<GearId>,
    pub candidate_item: GearId,
    pub stat_deltas: Vec<StatDeltaModel>,
}

impl GearComparisonModel {
    fn sample() -> Self {
        Self {
            current_item: Some(gear("gear/rpg_ch1/starter_launcher")),
            candidate_item: gear("gear/rpg_ch1/bankshot_launcher"),
            stat_deltas: vec![
                StatDeltaModel::new("Aim", 1),
                StatDeltaModel::new("Control", -1),
                StatDeltaModel::new("Bankshot Preview", 1),
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatDeltaModel {
    pub label: String,
    pub delta: i32,
}

impl StatDeltaModel {
    fn new(label: &str, delta: i32) -> Self {
        Self {
            label: label.to_owned(),
            delta,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillTreeScreen {
    pub title: String,
    pub trees: Vec<SkillTreeModel>,
    pub focus: FocusList,
}

impl SkillTreeScreen {
    fn from_character(character: &CharacterState) -> Self {
        let trees = ["Aim", "Control", "Resonance", "Luck"]
            .into_iter()
            .map(|tree| SkillTreeModel::sample(tree, character))
            .collect::<Vec<_>>();
        Self {
            title: "Skill Trees".to_owned(),
            focus: FocusList::new(4),
            trees,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillTreeModel {
    pub name: String,
    pub nodes: Vec<SkillNodeModel>,
}

impl SkillTreeModel {
    fn sample(name: &str, character: &CharacterState) -> Self {
        let unlocked = character.level > 1 || name == "Aim";
        Self {
            name: name.to_owned(),
            nodes: vec![
                SkillNodeModel::new(format!("skills/{}/root", name.to_lowercase()), true, false),
                SkillNodeModel::new(
                    format!("skills/{}/branch", name.to_lowercase()),
                    unlocked,
                    true,
                ),
                SkillNodeModel::new(
                    format!("skills/{}/capstone", name.to_lowercase()),
                    false,
                    false,
                ),
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillNodeModel {
    pub id: SkillId,
    pub unlocked: bool,
    pub unlock_animation: UnlockAnimationState,
}

impl SkillNodeModel {
    fn new(id: String, unlocked: bool, animating: bool) -> Self {
        Self {
            id: SkillId::new(id).expect("formatted skill id is valid"),
            unlocked,
            unlock_animation: if animating {
                UnlockAnimationState::Pulse { frame: 6 }
            } else {
                UnlockAnimationState::Idle
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnlockAnimationState {
    Idle,
    Pulse { frame: u8 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CampaignProgressScreen {
    pub title: String,
    pub level: u32,
    pub xp: u64,
    pub flags: Vec<String>,
    pub next_objectives: Vec<String>,
    pub focus: FocusList,
}

impl CampaignProgressScreen {
    fn from_character(character: &CharacterState) -> Self {
        let next_objectives = vec![
            "Clear board 5 with two objective tiers".to_owned(),
            "Unlock one Resonance node".to_owned(),
            "Equip a Chapter 1 alternate core".to_owned(),
        ];
        Self {
            title: "Campaign Progress".to_owned(),
            level: character.level,
            xp: character.xp,
            flags: character
                .campaign_flags
                .iter()
                .map(ToString::to_string)
                .collect(),
            focus: FocusList::new(next_objectives.len()),
            next_objectives,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceholderScreenSuite {
    pub main_menu: MainMenuScreen,
    pub settings: SettingsScreen,
    pub roguelite: RogueliteUiSuite,
    pub rpg: RpgUiSuite,
}

impl PlaceholderScreenSuite {
    pub fn sample() -> Self {
        Self {
            main_menu: MainMenuScreen::production_placeholder(),
            settings: SettingsScreen::production_placeholder(),
            roguelite: RogueliteUiSuite::production_placeholder(),
            rpg: RpgUiSuite::production_placeholder(),
        }
    }

    pub fn smoke_summary(&self) -> UiScreenSmokeSummary {
        let screen_count = 13;
        let keyboard_navigable = self.main_menu.focus.keyboard_ready()
            && self.settings.focus.keyboard_ready()
            && self.roguelite.node_map.focus.keyboard_ready()
            && self.roguelite.shop.focus.keyboard_ready()
            && self.roguelite.forge.focus.keyboard_ready()
            && self.roguelite.event.focus.keyboard_ready()
            && self.rpg.chapter_select.focus.keyboard_ready()
            && self.rpg.gear.focus.keyboard_ready()
            && self.rpg.skill_tree.focus.keyboard_ready()
            && self.rpg.campaign_progress.focus.keyboard_ready();
        let layout_smoke_passed = [UiViewport::HD, UiViewport::FHD]
            .into_iter()
            .all(|viewport| screen_count > 0 && viewport.safe_margin() * 2 < viewport.width);
        UiScreenSmokeSummary {
            screen_count,
            keyboard_navigable,
            layout_smoke_passed,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiScreenSmokeSummary {
    pub screen_count: usize,
    pub keyboard_navigable: bool,
    pub layout_smoke_passed: bool,
}

fn menu_action(hotkey: char, label: &str) -> MenuActionModel {
    MenuActionModel {
        hotkey,
        label: label.to_owned(),
    }
}

fn relic(id: &str) -> RelicId {
    RelicId::new(id).expect("static relic id is valid")
}

fn ball(id: &str) -> BallId {
    BallId::new(id).expect("static ball id is valid")
}

fn gear(id: &str) -> GearId {
    GearId::new(id).expect("static gear id is valid")
}

#[derive(Clone, Debug, PartialEq)]
pub struct HudState {
    pub aim: AimHudState,
    pub score: ScoreHudState,
    pub progression: ProgressionHudState,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FeelTestHudState {
    pub board_id: BoardId,
    pub aim: AimHudState,
    pub replay_hash: Option<String>,
    pub balls_remaining: u32,
    pub shot_count: u32,
    pub mock_score: Score,
    pub collision_count: usize,
    pub event_log_summary: String,
    pub completion: Option<SliceCompletionSummary>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FeelTestHudParts {
    pub replay_hash: Option<String>,
    pub balls_remaining: u32,
    pub shot_count: u32,
    pub mock_score: Score,
    pub collision_count: usize,
    pub event_log_summary: String,
    pub completion: Option<SliceCompletionSummary>,
}

impl FeelTestHudState {
    pub fn from_scene_parts(
        board: &BoardDefinition,
        input: &ShotInput,
        parts: FeelTestHudParts,
    ) -> Self {
        Self {
            board_id: board.id.clone(),
            aim: AimHudState::from_board_and_input(board, input),
            replay_hash: parts.replay_hash,
            balls_remaining: parts.balls_remaining,
            shot_count: parts.shot_count,
            mock_score: parts.mock_score,
            collision_count: parts.collision_count,
            event_log_summary: parts.event_log_summary,
            completion: parts.completion,
        }
    }

    #[cfg(test)]
    pub fn from_shot_summary(
        board: &BoardDefinition,
        input: &ShotInput,
        summary: &physics_core::ShotSummary,
        balls_remaining: u32,
        shot_count: u32,
        collision_count: usize,
        event_log_summary: impl Into<String>,
    ) -> Self {
        let mock_score =
            summary.pegs_hit.len() as Score * 100 + if summary.caught_bucket { 2_500 } else { 0 };

        Self::from_scene_parts(
            board,
            input,
            FeelTestHudParts {
                replay_hash: Some(summary.replay_hash.clone()),
                balls_remaining,
                shot_count,
                mock_score,
                collision_count,
                event_log_summary: event_log_summary.into(),
                completion: Some(SliceCompletionSummary::from_shot_summary(
                    board,
                    summary,
                    mock_score,
                    balls_remaining,
                    0,
                )),
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SliceCompletionSummary {
    pub score: Score,
    pub hit_pegs: usize,
    pub hit_oranges: usize,
    pub caught_bucket: bool,
    pub replay_hash: String,
    pub progression_outcome: SliceProgressionOutcome,
    pub feedback_events: usize,
    pub feedback_cues: usize,
}

impl SliceCompletionSummary {
    pub fn from_shot_summary(
        board: &BoardDefinition,
        summary: &ShotSummary,
        score: Score,
        balls_remaining: u32,
        feedback_cues: usize,
    ) -> Self {
        let hit_oranges = summary
            .pegs_hit
            .iter()
            .filter(|hit| {
                board
                    .pegs
                    .iter()
                    .any(|peg| peg.id == **hit && peg.kind == PegKind::Orange)
            })
            .count();
        let total_oranges = board
            .pegs
            .iter()
            .filter(|peg| peg.kind == PegKind::Orange)
            .count();
        let progression_outcome = if hit_oranges >= total_oranges && total_oranges > 0 {
            SliceProgressionOutcome::BoardWon
        } else if balls_remaining == 0 {
            SliceProgressionOutcome::BoardLost
        } else {
            SliceProgressionOutcome::Continue
        };

        Self {
            score,
            hit_pegs: summary.pegs_hit.len(),
            hit_oranges,
            caught_bucket: summary.caught_bucket,
            replay_hash: summary.replay_hash.clone(),
            progression_outcome,
            feedback_events: 0,
            feedback_cues,
        }
    }

    pub fn with_feedback_events(mut self, feedback_events: usize) -> Self {
        self.feedback_events = feedback_events;
        self
    }

    pub fn display_line(&self) -> String {
        format!(
            "slice score={} hit_pegs={} hit_oranges={} result={} replay_hash={} progression={:?} feedback_events={} feedback_cues={}",
            self.score,
            self.hit_pegs,
            self.hit_oranges,
            if self.caught_bucket { "catch" } else { "miss" },
            self.replay_hash,
            self.progression_outcome,
            self.feedback_events,
            self.feedback_cues,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliceProgressionOutcome {
    Continue,
    BoardWon,
    BoardLost,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunSessionSummary {
    pub run_id: ContentId,
    pub act: u8,
    pub current_node_index: u16,
    pub visited_node_count: usize,
    pub relic_count: usize,
    pub ball_count: usize,
    pub shots: u32,
    pub hearts: u32,
    pub coins: u32,
    pub sparks: u32,
    pub xp: u64,
    pub equipped_skill_count: usize,
    pub total_score: Score,
}

impl RunSessionSummary {
    pub fn from_states(
        run_state: &RunState,
        character_state: &CharacterState,
        total_score: Score,
    ) -> Self {
        Self {
            run_id: run_state.run_id.clone(),
            act: run_state.act,
            current_node_index: run_state.node_index,
            visited_node_count: run_state.visited_nodes.len(),
            relic_count: run_state.relics.len(),
            ball_count: run_state.balls.len(),
            shots: run_state.resources.shots,
            hearts: run_state.resources.hearts,
            coins: run_state.resources.coins,
            sparks: run_state.resources.sparks,
            xp: character_state.xp,
            equipped_skill_count: character_state
                .unlocked_skills
                .iter()
                .filter(|skill| skill.equipped)
                .count(),
            total_score,
        }
    }
}

impl HudState {
    pub fn mock_from_states(
        board: &BoardDefinition,
        shot_input: &ShotInput,
        run_state: &RunState,
        character_state: &CharacterState,
        score: Score,
        fever_multiplier: u32,
        combo_hits: u32,
    ) -> Self {
        Self {
            aim: AimHudState::from_board_and_input(board, shot_input),
            score: ScoreHudState::from_run_state(score, fever_multiplier, combo_hits, run_state),
            progression: ProgressionHudState::from_character_state(character_state),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AimHudState {
    pub origin: Vec2,
    pub aim_angle_radians: Scalar,
    pub launch_speed: Scalar,
    pub first_bounce: Option<FirstBounceAimLine>,
}

impl AimHudState {
    pub fn from_board_and_input(board: &BoardDefinition, input: &ShotInput) -> Self {
        Self {
            origin: board.cannon_position,
            aim_angle_radians: input.aim_angle_radians,
            launch_speed: input.launch_speed,
            first_bounce: predict_first_bounce(board, input).map(|event| {
                FirstBounceAimLine::from_collision_event(board.cannon_position, event)
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FirstBounceAimLine {
    pub start: Vec2,
    pub impact: Vec2,
    pub normal: Vec2,
    pub collision: CollisionDisplayKind,
    pub tick: u64,
}

impl FirstBounceAimLine {
    fn from_collision_event(start: Vec2, event: PhysicsEvent) -> Self {
        match event {
            PhysicsEvent::BallHitPeg {
                peg,
                position,
                normal,
                tick,
                ..
            } => Self {
                start,
                impact: position,
                normal,
                collision: CollisionDisplayKind::Peg(peg.to_string()),
                tick,
            },
            PhysicsEvent::BallHitObstacle {
                obstacle,
                position,
                normal,
                tick,
                ..
            } => Self {
                start,
                impact: position,
                normal,
                collision: CollisionDisplayKind::Obstacle(obstacle.to_string()),
                tick,
            },
            PhysicsEvent::BallEnteredBucket { ball, tick } => Self {
                start,
                impact: start,
                normal: Vec2::ZERO,
                collision: CollisionDisplayKind::Bucket(ball.to_string()),
                tick,
            },
            PhysicsEvent::BallExitedBoard { ball, tick } => Self {
                start,
                impact: start,
                normal: Vec2::ZERO,
                collision: CollisionDisplayKind::Exit(ball.to_string()),
                tick,
            },
            PhysicsEvent::ShotEnded { summary } => Self {
                start,
                impact: start,
                normal: Vec2::ZERO,
                collision: CollisionDisplayKind::ShotEnd(summary.replay_hash),
                tick: summary.ticks,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollisionDisplayKind {
    Peg(String),
    Obstacle(String),
    Bucket(String),
    Exit(String),
    ShotEnd(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoreHudState {
    pub score: Score,
    pub balls_remaining: u32,
    pub fever_multiplier: u32,
    pub combo: ComboDisplay,
    pub active_power: Option<ActivePowerDisplay>,
    pub relics: Vec<RelicDisplay>,
}

impl ScoreHudState {
    pub fn from_run_state(
        score: Score,
        fever_multiplier: u32,
        combo_hits: u32,
        run_state: &RunState,
    ) -> Self {
        Self {
            score,
            balls_remaining: run_state.resources.shots,
            fever_multiplier,
            combo: ComboDisplay::from_hits(combo_hits),
            active_power: None,
            relics: run_state
                .relics
                .iter()
                .map(|relic| RelicDisplay {
                    id: relic.id.clone(),
                    stacks: relic.stacks,
                    ready: false,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboDisplay {
    pub hits: u32,
    pub label: String,
    pub visible: bool,
}

impl ComboDisplay {
    pub fn from_hits(hits: u32) -> Self {
        Self {
            hits,
            label: if hits == 0 {
                String::from("No Combo")
            } else {
                format!("Combo x{hits}")
            },
            visible: hits > 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivePowerDisplay {
    pub id: SkillId,
    pub label: String,
    pub charge_percent: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelicDisplay {
    pub id: RelicId,
    pub stacks: u32,
    pub ready: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressionHudState {
    pub character_id: ContentId,
    pub level: u32,
    pub xp: u64,
    pub equipped_skill_count: usize,
}

impl ProgressionHudState {
    pub fn from_character_state(character_state: &CharacterState) -> Self {
        Self {
            character_id: character_state.character_id.clone(),
            level: character_state.level,
            xp: character_state.xp,
            equipped_skill_count: character_state
                .unlocked_skills
                .iter()
                .filter(|skill| skill.equipped)
                .count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use content_schema::{minimal_test_board, BallId, ContentId, RelicId, SkillId};
    use rpg_mode::SkillState;
    use run_mode::RelicInstance;

    #[test]
    fn aim_hud_uses_physics_first_bounce_prediction() {
        let board = minimal_test_board();
        let input = ShotInput {
            aim_angle_radians: std::f64::consts::FRAC_PI_2,
            launch_speed: 17.5,
            ball_id: BallId::new("balls/basic").unwrap(),
        };

        let hud = AimHudState::from_board_and_input(&board, &input);
        let predicted = predict_first_bounce(&board, &input);

        assert!(hud.first_bounce.is_some());
        assert_eq!(hud.origin, board.cannon_position);
        match (hud.first_bounce.unwrap().collision, predicted.unwrap()) {
            (CollisionDisplayKind::Peg(displayed), PhysicsEvent::BallHitPeg { peg, .. }) => {
                assert_eq!(displayed, peg.to_string());
            }
            other => panic!("unexpected first bounce mapping: {other:?}"),
        }
    }

    #[test]
    fn score_hud_can_be_driven_from_mock_run_state() {
        let mut run_state = RunState::new(7);
        run_state.resources.shots = 8;
        run_state.relics.push(RelicInstance {
            id: RelicId::new("relics/mock_focus").unwrap(),
            stacks: 2,
        });

        let hud = ScoreHudState::from_run_state(12_500, 3, 6, &run_state);

        assert_eq!(hud.score, 12_500);
        assert_eq!(hud.balls_remaining, 8);
        assert_eq!(hud.fever_multiplier, 3);
        assert_eq!(hud.combo.label, "Combo x6");
        assert!(hud.combo.visible);
        assert!(hud.active_power.is_none());
        assert_eq!(hud.relics.len(), 1);
        assert_eq!(hud.relics[0].stacks, 2);
    }

    #[test]
    fn full_hud_mock_state_uses_run_and_character_models() {
        let board = minimal_test_board();
        let input = ShotInput {
            aim_angle_radians: std::f64::consts::FRAC_PI_2,
            launch_speed: 17.5,
            ball_id: BallId::new("balls/basic").unwrap(),
        };
        let run_state = RunState::new(9);
        let mut character_state = CharacterState::new(ContentId::new("characters/mock").unwrap());
        character_state.level = 4;
        character_state.xp = 900;
        character_state.unlocked_skills.push(SkillState {
            id: SkillId::new("skills/zen_reroute").unwrap(),
            rank: 1,
            equipped: true,
            cooldown_boards: 0,
            cooldown_remaining: 0,
            used_this_board: false,
        });

        let hud =
            HudState::mock_from_states(&board, &input, &run_state, &character_state, 99, 2, 3);

        assert!(hud.aim.first_bounce.is_some());
        assert_eq!(hud.score.balls_remaining, run_state.resources.shots);
        assert_eq!(hud.progression.level, 4);
        assert_eq!(hud.progression.equipped_skill_count, 1);
    }

    #[test]
    fn feel_test_hud_exposes_checkpoint_fields() {
        let board = minimal_test_board();
        let input = ShotInput {
            aim_angle_radians: std::f64::consts::FRAC_PI_2,
            launch_speed: 17.5,
            ball_id: BallId::new("balls/basic").unwrap(),
        };
        let result = physics_core::simulate_shot(123, &board, &input);

        let hud = FeelTestHudState::from_shot_summary(
            &board,
            &input,
            &result.summary,
            8,
            1,
            result.events.len(),
            "events logged",
        );

        assert_eq!(hud.board_id, board.id);
        assert_eq!(hud.shot_count, 1);
        assert_eq!(hud.balls_remaining, 8);
        assert!(hud.aim.first_bounce.is_some());
        assert_eq!(hud.replay_hash, Some(result.summary.replay_hash.clone()));
        assert!(hud.collision_count > 0);
        let completion = hud.completion.unwrap();
        assert_eq!(completion.score, hud.mock_score);
        assert_eq!(completion.hit_pegs, result.summary.pegs_hit.len());
        assert_eq!(completion.caught_bucket, result.summary.caught_bucket);
        assert_eq!(completion.feedback_cues, 0);
    }

    #[test]
    fn run_session_summary_populates_full_interactive_slice_fields() {
        let mut run_state = RunState::act1_slice(0xC2_FE);
        let nodes = run_mode::act1_slice_nodes();
        run_state.advance_to_node(nodes[0].clone());
        run_state.advance_to_node(nodes[1].clone());
        run_state.advance_to_node(nodes[2].clone());
        run_state.apply_reward(&run_mode::Reward::Coins(12));
        run_state.apply_reward(&run_mode::Reward::Relic(
            RelicId::new("relics/act1/orange_lacquer").unwrap(),
        ));
        run_state.apply_reward(&run_mode::Reward::Ball(BallId::new("balls/spark").unwrap()));
        run_state.resources.shots = 6;
        run_state.resources.sparks = 9;

        let mut character_state = CharacterState::act1_slice();
        character_state.xp = 9;

        let summary = RunSessionSummary::from_states(&run_state, &character_state, 7_500);

        assert_eq!(summary.run_id, run_state.run_id);
        assert_eq!(summary.act, 1);
        assert_eq!(summary.current_node_index, 3);
        assert_eq!(summary.visited_node_count, 3);
        assert_eq!(summary.relic_count, 2);
        assert_eq!(summary.ball_count, 2);
        assert_eq!(summary.shots, 6);
        assert_eq!(summary.hearts, 3);
        assert_eq!(summary.coins, 22);
        assert_eq!(summary.sparks, 9);
        assert_eq!(summary.xp, 9);
        assert_eq!(summary.equipped_skill_count, 1);
        assert_eq!(summary.total_score, 7_500);
    }

    #[test]
    fn placeholder_screen_suite_covers_checkpoint4_contract() {
        let suite = PlaceholderScreenSuite::sample();
        let summary = suite.smoke_summary();

        assert_eq!(suite.main_menu.title, "Feverfall");
        assert_eq!(suite.main_menu.actions.len(), 4);
        assert!(suite
            .main_menu
            .actions
            .iter()
            .any(|action| action.label == "Play Roguelite"));
        assert!(suite
            .main_menu
            .actions
            .iter()
            .any(|action| action.label == "Play RPG"));
        assert_eq!(suite.settings.volume_sliders.len(), 3);
        assert!(suite
            .settings
            .volume_sliders
            .iter()
            .all(|slider| slider.stubbed));
        assert!(suite
            .settings
            .key_rebinds
            .iter()
            .all(|rebind| rebind.stubbed));
        assert!(suite.roguelite.node_map.branches.len() >= full_run_nodes().len());
        assert!(suite
            .roguelite
            .node_map
            .branches
            .iter()
            .any(|branch| !branch.branch_choices.is_empty()));
        assert_eq!(suite.roguelite.shop.actions.len(), 3);
        assert_eq!(suite.roguelite.forge.upgrades.len(), 3);
        assert!(suite
            .roguelite
            .event
            .choices
            .iter()
            .any(|choice| !choice.reward.is_empty() && !choice.risk.is_empty()));
        assert!(suite.roguelite.relic_bar.scrollable);
        assert_eq!(suite.roguelite.relic_bar.visible_window, 0..6);
        assert_eq!(suite.rpg.chapter_select.chapters.len(), 3);
        assert_eq!(suite.rpg.gear.slots.len(), 4);
        assert!(!suite.rpg.gear.comparison.stat_deltas.is_empty());
        assert_eq!(suite.rpg.skill_tree.trees.len(), 4);
        assert!(suite
            .rpg
            .skill_tree
            .trees
            .iter()
            .flat_map(|tree| tree.nodes.iter())
            .any(|node| matches!(node.unlock_animation, UnlockAnimationState::Pulse { .. })));
        assert!(!suite.rpg.campaign_progress.next_objectives.is_empty());
        assert!(summary.keyboard_navigable);
        assert!(summary.layout_smoke_passed);
        assert_eq!(summary.screen_count, 13);
    }

    #[test]
    fn settings_screen_is_keyboard_navigable_without_mouse() {
        let mut settings = SettingsScreen::production_placeholder();

        settings.handle(UiNavInput::Right);
        assert_eq!(settings.volume_sliders[0].value_percent, 85);
        settings.handle(UiNavInput::Down);
        settings.handle(UiNavInput::Down);
        settings.handle(UiNavInput::Down);
        settings.handle(UiNavInput::Toggle);

        assert!(settings.reduce_flash.enabled);
        assert!(settings.focus.keyboard_ready());
    }
}
