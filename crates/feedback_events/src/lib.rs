use content_schema::{Score, Vec2};
use game_rules::GameEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeedbackEvent {
    pub kind: FeedbackKind,
    pub intensity: f32,
    pub position: Vec2,
    pub combo: u32,
    pub value: Score,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeedbackKind {
    PegHit,
    OrangeHit,
    PurpleHit,
    GreenHit,
    BucketCatch,
    NearBucketMiss,
    ComboThreshold,
    FinalOrangeTension,
    ExtremeFever,
    Loss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessibilityFeedbackFlags {
    pub reduce_shake: bool,
    pub reduce_flash: bool,
    pub mute_high_frequency_layers: bool,
}

impl AccessibilityFeedbackFlags {
    pub const DEFAULT: Self = Self {
        reduce_shake: false,
        reduce_flash: false,
        mute_high_frequency_layers: false,
    };
}

pub fn map_game_event(event: &GameEvent) -> Option<FeedbackEvent> {
    match event {
        GameEvent::BucketCatchAwarded { points, .. } => Some(FeedbackEvent {
            kind: FeedbackKind::BucketCatch,
            intensity: 0.8,
            position: Vec2::ZERO,
            combo: 0,
            value: *points,
        }),
        GameEvent::BoardWon { final_score, .. } => Some(FeedbackEvent {
            kind: FeedbackKind::ExtremeFever,
            intensity: 1.0,
            position: Vec2::ZERO,
            combo: 0,
            value: *final_score,
        }),
        GameEvent::BoardLost { .. } => Some(FeedbackEvent {
            kind: FeedbackKind::Loss,
            intensity: 0.2,
            position: Vec2::ZERO,
            combo: 0,
            value: 0,
        }),
        GameEvent::PegScored { points, .. } => Some(FeedbackEvent {
            kind: FeedbackKind::PegHit,
            intensity: 0.35,
            position: Vec2::ZERO,
            combo: 0,
            value: *points,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use content_schema::BoardId;
    use game_rules::LossReason;

    #[test]
    fn loss_feedback_is_not_victory_like() {
        let event = GameEvent::BoardLost {
            board: BoardId::new("boards/minimal_test").unwrap(),
            reason: LossReason::OutOfShots,
        };

        let feedback = map_game_event(&event).unwrap();

        assert_eq!(feedback.kind, FeedbackKind::Loss);
        assert!(feedback.intensity < 0.5);
    }

    #[test]
    fn feedback_event_round_trips_json() {
        let event = FeedbackEvent {
            kind: FeedbackKind::BucketCatch,
            intensity: 0.8,
            position: Vec2::new(10.0, 34.4),
            combo: 2,
            value: 2_500,
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: FeedbackEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, event);
    }
}
