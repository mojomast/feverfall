use content_schema::{
    BasketDef, BoardDefinition, BoardId, ContentId, ObstacleDef, PegDef, PegId, PegKind, Scalar,
    Seed, ShapeDef, Vec2,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenerationParams {
    pub act: u8,
    pub difficulty: u8,
    pub archetype: ContentId,
    pub seed: Seed,
    pub peg_budget: u16,
    pub hazard_budget: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoardValidationReport {
    pub board_id: BoardId,
    pub issues: Vec<BoardValidationIssue>,
}

impl BoardValidationReport {
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BoardValidationIssue {
    MissingOrangePegs,
    PegOutsideBoard { peg: PegId },
    BucketTooNarrow { width: Scalar },
    TooFewFirstShotTargets,
    StallRisk,
    ExcessiveDeadZone,
    PoorBucketOpportunity,
}

pub fn generate_board(params: &GenerationParams) -> BoardDefinition {
    let mut rng = Lcg::new(params.seed);
    let peg_count = params.peg_budget.max(1);
    let mut pegs = Vec::with_capacity(usize::from(peg_count));

    for index in 0..peg_count {
        let kind = if index % 4 == 0 {
            PegKind::Orange
        } else {
            PegKind::Blue
        };
        let x = 2.0 + rng.next_unit() * 16.0;
        let y = 6.0 + rng.next_unit() * 22.0;
        pegs.push(PegDef {
            id: PegId::new(format!("peg/{index:03}")).expect("formatted peg id is valid"),
            kind,
            shape: ShapeDef::Circle {
                center: Vec2::new(x, y),
                radius: 0.32,
            },
        });
    }

    BoardDefinition {
        id: BoardId::new(format!("generated/{:016x}", params.seed))
            .expect("formatted board id is valid"),
        size: Vec2::new(20.0, 35.56),
        cannon_position: Vec2::new(10.0, 1.5),
        kill_plane_y: 36.5,
        pegs,
        obstacles: Vec::<ObstacleDef>::new(),
        bucket: BasketDef::spec_default(),
        tags: vec![params.archetype.clone()],
    }
}

pub fn validate_board(board: &BoardDefinition) -> BoardValidationReport {
    let mut issues = Vec::new();

    if !board.pegs.iter().any(|peg| peg.kind == PegKind::Orange) {
        issues.push(BoardValidationIssue::MissingOrangePegs);
    }

    for peg in &board.pegs {
        if let Some(center) = shape_center(&peg.shape) {
            if center.x < 0.0
                || center.y < 0.0
                || center.x > board.size.x
                || center.y > board.size.y
            {
                issues.push(BoardValidationIssue::PegOutsideBoard {
                    peg: peg.id.clone(),
                });
            }
        }
    }

    if board.bucket.width < 2.2 {
        issues.push(BoardValidationIssue::BucketTooNarrow {
            width: board.bucket.width,
        });
    }

    BoardValidationReport {
        board_id: board.id.clone(),
        issues,
    }
}

fn shape_center(shape: &ShapeDef) -> Option<Vec2> {
    match shape {
        ShapeDef::Circle { center, .. } | ShapeDef::Rect { center, .. } => Some(*center),
        ShapeDef::Capsule { .. } | ShapeDef::Segment { .. } => None,
    }
}

#[derive(Clone, Copy)]
struct Lcg(u64);

impl Lcg {
    fn new(seed: Seed) -> Self {
        Self(seed ^ 0x9e3779b97f4a7c15)
    }

    fn next_unit(&mut self) -> Scalar {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 11) as Scalar) / ((1u64 << 53) as Scalar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(seed: Seed) -> GenerationParams {
        GenerationParams {
            act: 1,
            difficulty: 1,
            archetype: ContentId::new("archetypes/test").unwrap(),
            seed,
            peg_budget: 12,
            hazard_budget: 0,
        }
    }

    #[test]
    fn generation_is_deterministic_by_seed() {
        assert_eq!(generate_board(&params(42)), generate_board(&params(42)));
    }

    #[test]
    fn generated_board_validates() {
        let board = generate_board(&params(7));
        let report = validate_board(&board);

        assert!(report.is_valid(), "unexpected issues: {:?}", report.issues);
    }

    #[test]
    fn generation_params_round_trip_json() {
        let value = params(99);
        let json = serde_json::to_string(&value).unwrap();
        let parsed: GenerationParams = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, value);
    }
}
