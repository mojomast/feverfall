use content_schema::{RelicId, Vec2};
use feedback_events::{FeedbackEvent, FeedbackKind};
use run_mode::{Reward, RewardOffer, RewardRarity, RunState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RewardUiRegistrationSummary {
    pub cards: usize,
    pub has_relic_metadata: bool,
    pub smoke_auto_selects_first: bool,
}

pub fn register() -> RewardUiRegistrationSummary {
    let offer = sample_offer();
    let screen = RewardChoiceScreen::from_offer(&offer);
    let mut keyboard_run_state = RunState::act1_slice(0xC2_0D);
    let mut keyboard_controller = RewardChoiceController::new(false);
    let keyboard_transition = keyboard_controller.handle_key('1', &mut keyboard_run_state, &offer);
    debug_assert!(matches!(
        keyboard_transition,
        RewardChoiceTransition::Selected { index: 0, .. }
    ));
    let mut run_state = RunState::act1_slice(0xC2_0E);
    let mut controller = RewardChoiceController::new(true);
    let transition = controller.tick(&mut run_state, &offer);

    RewardUiRegistrationSummary {
        cards: screen.cards.len(),
        has_relic_metadata: screen.cards.iter().any(|card| card.relic_id.is_some()),
        smoke_auto_selects_first: matches!(
            transition,
            RewardChoiceTransition::Selected { index: 0, .. }
        ),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardChoiceScreen {
    pub title: String,
    pub subtitle: String,
    pub cards: Vec<RewardCardModel>,
}

impl RewardChoiceScreen {
    pub fn from_offer(offer: &RewardOffer) -> Self {
        Self {
            title: "Choose a Reward".to_owned(),
            subtitle: "Press 1, 2, or 3".to_owned(),
            cards: offer
                .choices
                .iter()
                .take(3)
                .enumerate()
                .map(|(index, reward)| RewardCardModel::from_reward(index, reward, offer.rarity))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardCardModel {
    pub index: usize,
    pub hotkey: char,
    pub relic_id: Option<RelicId>,
    pub title: String,
    pub description: String,
    pub rarity: RewardRarity,
    pub shape: RewardCardShape,
}

impl RewardCardModel {
    fn from_reward(index: usize, reward: &Reward, rarity: RewardRarity) -> Self {
        let (title, description, relic_id) = match reward {
            Reward::Relic(id) => {
                let metadata = relic_metadata(id);
                (
                    metadata.name.to_owned(),
                    metadata.description.to_owned(),
                    Some(id.clone()),
                )
            }
            Reward::Ball(id) => (
                label_from_id(id.as_str()),
                format!("Add {} to your ball pouch.", label_from_id(id.as_str())),
                None,
            ),
            Reward::Coins(amount) => (
                format!("{amount} Coins"),
                "Spend coins at shops and forge nodes.".to_owned(),
                None,
            ),
            Reward::Heal(amount) => (
                format!("Heal {amount}"),
                "Restore hearts for the rest of this run.".to_owned(),
                None,
            ),
            Reward::RemoveCurse(amount) => (
                format!("Cleanse {amount}"),
                "Remove curse pressure from future rewards.".to_owned(),
                None,
            ),
        };

        Self {
            index,
            hotkey: char::from_digit((index + 1) as u32, 10).unwrap_or('?'),
            relic_id,
            title,
            description,
            rarity,
            shape: RewardCardShape::RoundedRect,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewardCardShape {
    RoundedRect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RewardChoiceController {
    smoke_mode: bool,
    ticks_visible: u32,
    selected: Option<usize>,
}

impl RewardChoiceController {
    pub const fn new(smoke_mode: bool) -> Self {
        Self {
            smoke_mode,
            ticks_visible: 0,
            selected: None,
        }
    }

    pub fn handle_key(
        &mut self,
        key: char,
        run_state: &mut RunState,
        offer: &RewardOffer,
    ) -> RewardChoiceTransition {
        match key {
            '1' => self.select(0, run_state, offer),
            '2' => self.select(1, run_state, offer),
            '3' => self.select(2, run_state, offer),
            _ => RewardChoiceTransition::Waiting,
        }
    }

    pub fn tick(
        &mut self,
        run_state: &mut RunState,
        offer: &RewardOffer,
    ) -> RewardChoiceTransition {
        self.ticks_visible = self.ticks_visible.saturating_add(1);
        if self.smoke_mode && self.ticks_visible >= 1 {
            self.select(0, run_state, offer)
        } else {
            RewardChoiceTransition::Waiting
        }
    }

    pub fn select(
        &mut self,
        index: usize,
        run_state: &mut RunState,
        offer: &RewardOffer,
    ) -> RewardChoiceTransition {
        if self.selected.is_some() {
            return RewardChoiceTransition::AlreadySelected;
        }

        let Some(reward) = offer.choices.get(index).cloned() else {
            return RewardChoiceTransition::InvalidSelection(index);
        };

        run_state.apply_reward(&reward);
        self.selected = Some(index);
        RewardChoiceTransition::Selected {
            index,
            reward,
            feedback: confirmation_feedback(index),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RewardChoiceTransition {
    Waiting,
    Selected {
        index: usize,
        reward: Reward,
        feedback: FeedbackEvent,
    },
    AlreadySelected,
    InvalidSelection(usize),
}

pub fn confirmation_feedback(index: usize) -> FeedbackEvent {
    FeedbackEvent {
        kind: FeedbackKind::BucketCatch,
        intensity: 0.6,
        position: Vec2::ZERO,
        combo: (index + 1) as u32,
        value: 0,
    }
}

struct RelicMetadata {
    name: &'static str,
    description: &'static str,
}

fn relic_metadata(id: &RelicId) -> RelicMetadata {
    match id.as_str() {
        "relics/act1/spark_catcher" => RelicMetadata {
            name: "Spark Catcher",
            description: "Bucket catches preserve extra spark energy.",
        },
        "relics/act1/steady_bucket" => RelicMetadata {
            name: "Steady Bucket",
            description: "The bucket holds a wider reliable catch lane.",
        },
        "relics/act1/orange_echo" => RelicMetadata {
            name: "Orange Echo",
            description: "Orange hits leave a faint scoring echo.",
        },
        "relics/act1/stonebreaker" => RelicMetadata {
            name: "Stonebreaker",
            description: "Cracked stones become less punishing on future boards.",
        },
        "relics/act1/feverheart" => RelicMetadata {
            name: "Feverheart",
            description: "Board clears carry more fever into the next node.",
        },
        _ => RelicMetadata {
            name: "Unknown Relic",
            description: "A relic from the current reward pool.",
        },
    }
}

fn label_from_id(id: &str) -> String {
    id.rsplit('/')
        .next()
        .unwrap_or(id)
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn sample_offer() -> RewardOffer {
    act1_feel_test_relic_offer()
}

pub fn act1_feel_test_relic_offer() -> RewardOffer {
    RewardOffer {
        choices: vec![
            Reward::Relic(RelicId::new("relics/act1/orange_echo").expect("static id is valid")),
            Reward::Relic(RelicId::new("relics/act1/steady_bucket").expect("static id is valid")),
            Reward::Relic(RelicId::new("relics/act1/stonebreaker").expect("static id is valid")),
        ],
        rarity: RewardRarity::Uncommon,
        source: content_schema::ContentId::new("runs/act1/feel_test_reward")
            .expect("static id is valid"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reward_selection_applies_correct_relic_to_run_state() {
        let offer = RewardOffer {
            choices: vec![
                Reward::Relic(RelicId::new("relics/act1/orange_echo").unwrap()),
                Reward::Relic(RelicId::new("relics/act1/steady_bucket").unwrap()),
                Reward::Relic(RelicId::new("relics/act1/stonebreaker").unwrap()),
            ],
            rarity: RewardRarity::Rare,
            source: content_schema::ContentId::new("runs/test/reward").unwrap(),
        };
        let mut run_state = RunState::act1_slice(7);
        let mut controller = RewardChoiceController::new(false);

        let transition = controller.handle_key('2', &mut run_state, &offer);

        assert!(matches!(
            transition,
            RewardChoiceTransition::Selected { index: 1, .. }
        ));
        assert!(run_state
            .relics
            .iter()
            .any(|relic| relic.id.as_str() == "relics/act1/steady_bucket" && relic.stacks == 1));
    }

    #[test]
    fn smoke_mode_auto_select_is_deterministic() {
        let offer = sample_offer();
        let mut first = RunState::act1_slice(99);
        let mut second = RunState::act1_slice(99);
        let mut first_controller = RewardChoiceController::new(true);
        let mut second_controller = RewardChoiceController::new(true);

        let first_transition = first_controller.tick(&mut first, &offer);
        let second_transition = second_controller.tick(&mut second, &offer);

        assert!(matches!(
            first_transition,
            RewardChoiceTransition::Selected { index: 0, .. }
        ));
        assert_eq!(first_transition, second_transition);
        assert_eq!(first, second);
        assert!(first
            .relics
            .iter()
            .any(|relic| relic.id.as_str() == "relics/act1/orange_echo" && relic.stacks == 1));
    }

    #[test]
    fn primitive_card_model_exposes_labels_descriptions_and_rarity() {
        let offer = RewardOffer {
            choices: vec![
                Reward::Relic(RelicId::new("relics/act1/orange_echo").unwrap()),
                Reward::Relic(RelicId::new("relics/act1/steady_bucket").unwrap()),
                Reward::Relic(RelicId::new("relics/act1/stonebreaker").unwrap()),
            ],
            rarity: RewardRarity::Uncommon,
            source: content_schema::ContentId::new("runs/test/cards").unwrap(),
        };

        let screen = RewardChoiceScreen::from_offer(&offer);

        assert_eq!(screen.cards.len(), 3);
        assert_eq!(screen.cards[0].title, "Orange Echo");
        assert!(screen.cards[0].description.contains("Orange hits"));
        assert_eq!(screen.cards[0].rarity, RewardRarity::Uncommon);
        assert_eq!(screen.cards[0].shape, RewardCardShape::RoundedRect);
        assert_eq!(screen.cards[1].hotkey, '2');
        assert_eq!(
            screen.cards[2].relic_id.as_ref().unwrap().as_str(),
            "relics/act1/stonebreaker"
        );
    }
}
