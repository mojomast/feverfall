use serde::{Deserialize, Serialize};
use std::fmt;

pub type Scalar = f64;
pub type Seed = u64;
pub type Tick = u64;
pub type Score = i64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: Scalar,
    pub y: Scalar,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: Scalar, y: Scalar) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContentIdError {
    Empty,
    InvalidStart(char),
    InvalidChar(char),
}

impl fmt::Display for ContentIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "content id cannot be empty"),
            Self::InvalidStart(c) => write!(
                f,
                "content id must start with ascii alphanumeric, got {c:?}"
            ),
            Self::InvalidChar(c) => write!(f, "content id contains invalid character {c:?}"),
        }
    }
}

impl std::error::Error for ContentIdError {}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentId(String);

impl ContentId {
    pub fn new(value: impl Into<String>) -> Result<Self, ContentIdError> {
        let value = value.into();
        validate_content_id(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub fn validate_content_id(value: &str) -> Result<(), ContentIdError> {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(ContentIdError::Empty);
    };

    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return Err(ContentIdError::InvalidStart(first));
    }

    for c in chars {
        if !(c.is_ascii_lowercase()
            || c.is_ascii_digit()
            || matches!(c, '_' | '-' | '.' | '/' | ':'))
        {
            return Err(ContentIdError::InvalidChar(c));
        }
    }

    Ok(())
}

macro_rules! id_newtype {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub ContentId);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ContentIdError> {
                ContentId::new(value).map(Self)
            }

            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl From<ContentId> for $name {
            fn from(value: ContentId) -> Self {
                Self(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

id_newtype!(BoardId);
id_newtype!(BallId);
id_newtype!(PegId);
id_newtype!(ObstacleId);
id_newtype!(RelicId);
id_newtype!(SkillId);
id_newtype!(GearId);
id_newtype!(ContentPackId);
id_newtype!(ShopItemId);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoardDefinition {
    pub id: BoardId,
    pub size: Vec2,
    pub cannon_position: Vec2,
    pub kill_plane_y: Scalar,
    pub pegs: Vec<PegDef>,
    pub obstacles: Vec<ObstacleDef>,
    pub bucket: BasketDef,
    pub tags: Vec<ContentId>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PegDef {
    pub id: PegId,
    pub kind: PegKind,
    pub shape: ShapeDef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PegKind {
    Blue,
    Orange,
    Purple,
    Green,
    Stone,
    Ghost,
    Bomb,
    Splitter,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObstacleDef {
    pub id: ObstacleId,
    pub kind: ObstacleKind,
    pub shape: ShapeDef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObstacleKind {
    Wall,
    Stone,
    Rubber,
    Sensor,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ShapeDef {
    Circle { center: Vec2, radius: Scalar },
    Capsule { a: Vec2, b: Vec2, radius: Scalar },
    Segment { a: Vec2, b: Vec2 },
    Rect { center: Vec2, half_extents: Vec2 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BasketDef {
    pub center: Vec2,
    pub width: Scalar,
    pub height: Scalar,
    pub horizontal_speed: Scalar,
    pub motion: BasketMotion,
    pub catch_margin: Scalar,
}

impl BasketDef {
    pub fn spec_default() -> Self {
        Self {
            center: Vec2::new(10.0, 34.4),
            width: 3.0,
            height: 0.55,
            horizontal_speed: 6.0,
            motion: BasketMotion::PingPong,
            catch_margin: 0.18,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BasketMotion {
    PingPong,
    Sine,
    Scripted,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContentManifest {
    pub id: ContentPackId,
    pub version: String,
    pub entries: Vec<ContentId>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RelicDefinition {
    pub id: RelicId,
    pub name: String,
    pub category: RelicCategory,
    pub rarity: Rarity,
    pub act: u8,
    pub description: String,
    pub effects: Vec<ContentId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelicCategory {
    Ball,
    Peg,
    Basket,
    Board,
    EconomyCombo,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BallVariantDefinition {
    pub id: BallId,
    pub name: String,
    pub family: ContentId,
    pub rarity: Rarity,
    pub description: String,
    pub stats: BallStats,
    pub effects: Vec<ContentId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BallStats {
    pub radius: Scalar,
    pub mass: Scalar,
    pub launch_speed_multiplier: Scalar,
    pub restitution_multiplier: Scalar,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShopItemDefinition {
    pub id: ShopItemId,
    pub name: String,
    pub act: u8,
    pub item_type: ShopItemType,
    pub price_coins: u16,
    pub stock_weight: u16,
    pub description: String,
    pub grants: ContentId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShopItemType {
    Relic,
    Ball,
    Service,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
}

pub fn minimal_test_board() -> BoardDefinition {
    BoardDefinition {
        id: BoardId::new("boards/minimal_test").expect("static id is valid"),
        size: Vec2::new(20.0, 35.56),
        cannon_position: Vec2::new(10.0, 1.5),
        kill_plane_y: 36.5,
        pegs: vec![
            PegDef {
                id: PegId::new("peg/orange_001").expect("static id is valid"),
                kind: PegKind::Orange,
                shape: ShapeDef::Circle {
                    center: Vec2::new(10.0, 14.0),
                    radius: 0.35,
                },
            },
            PegDef {
                id: PegId::new("peg/blue_001").expect("static id is valid"),
                kind: PegKind::Blue,
                shape: ShapeDef::Circle {
                    center: Vec2::new(8.0, 16.0),
                    radius: 0.35,
                },
            },
        ],
        obstacles: Vec::new(),
        bucket: BasketDef::spec_default(),
        tags: vec![ContentId::new("test").expect("static id is valid")],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_id_accepts_documented_namespace_style() {
        let id = ContentId::new("relics/act1:first-bounce_bonus").unwrap();

        assert_eq!(id.as_str(), "relics/act1:first-bounce_bonus");
    }

    #[test]
    fn content_id_rejects_uppercase_and_spaces() {
        assert!(matches!(
            ContentId::new("Bad Id"),
            Err(ContentIdError::InvalidStart('B'))
        ));
        assert!(matches!(
            ContentId::new("_bad_start"),
            Err(ContentIdError::InvalidStart('_'))
        ));
    }

    #[test]
    fn board_definition_round_trips_json() {
        let board = minimal_test_board();

        let json = serde_json::to_string(&board).unwrap();
        let parsed: BoardDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, board);
    }

    #[test]
    fn relic_definition_round_trips_json() {
        let relic = RelicDefinition {
            id: RelicId::new("relics/act1/test_relic").unwrap(),
            name: "Test Relic".to_string(),
            category: RelicCategory::Ball,
            rarity: Rarity::Common,
            act: 1,
            description: "A schema smoke relic.".to_string(),
            effects: vec![ContentId::new("effects/test").unwrap()],
        };

        let json = serde_json::to_string(&relic).unwrap();
        let parsed: RelicDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, relic);
    }
}
