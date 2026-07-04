use content_schema::{
    BasketDef, BoardDefinition, BoardId, ContentId, ObstacleDef, ObstacleId, ObstacleKind, PegDef,
    PegId, PegKind, Scalar, Seed, ShapeDef, Vec2,
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const BOARD_WIDTH: Scalar = 20.0;
const BOARD_HEIGHT: Scalar = 35.56;
const BALL_RADIUS: Scalar = 0.22;
const MIN_ORANGE_PEGS: usize = 20;
const MAX_ORANGE_PEGS: usize = 25;
const MIN_BUCKET_WIDTH: Scalar = 2.2;
const MAX_BUCKET_WIDTH: Scalar = 4.4;
const AIM_SAMPLES: usize = 512;
const MAX_AIM_ANGLE_DEGREES: Scalar = 72.0;
const MAX_DEAD_ZONE_RATIO: Scalar = 0.15;
const MIN_CATCH_ANGLES: usize = 3;

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
    OrangeCountOutOfRange {
        count: usize,
        min: usize,
        max: usize,
    },
    PegOutsideBoard {
        peg: PegId,
    },
    ObstacleOutsideBoard {
        obstacle: ObstacleId,
    },
    BucketTooNarrow {
        width: Scalar,
    },
    BucketTooWide {
        width: Scalar,
    },
    BucketOutsideBoard,
    TooFewFirstShotTargets {
        reachable_oranges: usize,
    },
    StallRisk,
    ExcessiveDeadZone {
        ratio: Scalar,
    },
    PoorBucketOpportunity {
        viable_angles: usize,
    },
}

pub fn authored_boards_dir() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../game/assets/content/boards"
    ))
}

pub fn load_board_from_str(source: &str) -> Result<BoardDefinition, serde_json::Error> {
    serde_json::from_str(source)
}

pub fn load_board_from_path(path: impl AsRef<Path>) -> Result<BoardDefinition, BoardLoadError> {
    let source = fs::read_to_string(path.as_ref()).map_err(BoardLoadError::Io)?;
    load_board_from_str(&source).map_err(BoardLoadError::Json)
}

pub fn load_authored_boards(dir: impl AsRef<Path>) -> Result<Vec<BoardDefinition>, BoardLoadError> {
    let mut paths = Vec::new();
    collect_board_paths(dir.as_ref(), &mut paths)?;
    paths.sort();

    paths.into_iter().map(load_board_from_path).collect()
}

fn collect_board_paths(
    dir: &Path,
    paths: &mut Vec<std::path::PathBuf>,
) -> Result<(), BoardLoadError> {
    for entry in fs::read_dir(dir).map_err(BoardLoadError::Io)? {
        let path = entry.map_err(BoardLoadError::Io)?.path();
        if path.is_dir() {
            collect_board_paths(&path, paths)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum BoardLoadError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for BoardLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::Json(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for BoardLoadError {}

pub fn generate_board(params: &GenerationParams) -> BoardDefinition {
    let archetype = params
        .archetype
        .as_str()
        .rsplit('/')
        .next()
        .unwrap_or(params.archetype.as_str());
    let mut rng = Lcg::new(params.seed);
    let orange_count = match params.act {
        0 | 1 => 20 + usize::from(params.difficulty.min(5)),
        2 => 25,
        _ => 25,
    };
    let blue_count = usize::from(params.peg_budget)
        .saturating_sub(orange_count)
        .max(18);
    let mut pegs = Vec::with_capacity(orange_count + blue_count);
    let mut push_peg = |index: usize, kind: PegKind, x: Scalar, y: Scalar| {
        pegs.push(PegDef {
            id: PegId::new(format!("peg/{index:03}")).expect("formatted peg id is valid"),
            kind,
            shape: ShapeDef::Circle {
                center: Vec2::new(x.clamp(0.55, BOARD_WIDTH - 0.55), y.clamp(4.5, 32.5)),
                radius: 0.32,
            },
        });
    };

    for index in 0..orange_count {
        let t = if orange_count == 1 {
            0.0
        } else {
            index as Scalar / (orange_count - 1) as Scalar
        };
        let (x, y) = archetype_point(archetype, t, index, orange_count, &mut rng);
        push_peg(index, PegKind::Orange, x, y);
    }

    for index in 0..blue_count {
        let row = index / 7;
        let col = index % 7;
        let jitter_x = (rng.next_unit() - 0.5) * 0.45;
        let jitter_y = (rng.next_unit() - 0.5) * 0.45;
        let x = 2.2 + col as Scalar * 2.6 + (row % 2) as Scalar * 0.9 + jitter_x;
        let y = 9.0 + row as Scalar * 3.0 + jitter_y;
        push_peg(orange_count + index, PegKind::Blue, x, y);
    }

    let mut obstacles = Vec::new();
    if params.hazard_budget > 0 || archetype == "fortress" {
        obstacles.push(ObstacleDef {
            id: ObstacleId::new("obstacle/stone_gate").expect("static id is valid"),
            kind: ObstacleKind::Stone,
            shape: ShapeDef::Rect {
                center: Vec2::new(10.0, 19.5),
                half_extents: Vec2::new(1.2, 0.28),
            },
        });
    }

    BoardDefinition {
        id: BoardId::new(format!("generated/{archetype}/{:016x}", params.seed))
            .expect("formatted board id is valid"),
        size: Vec2::new(BOARD_WIDTH, BOARD_HEIGHT),
        cannon_position: Vec2::new(10.0, 1.5),
        kill_plane_y: 36.5,
        pegs,
        obstacles,
        bucket: BasketDef::spec_default(),
        tags: vec![params.archetype.clone()],
    }
}

pub fn validate_board(board: &BoardDefinition) -> BoardValidationReport {
    let mut issues = Vec::new();

    let orange_count = board
        .pegs
        .iter()
        .filter(|peg| peg.kind == PegKind::Orange)
        .count();
    if orange_count == 0 {
        issues.push(BoardValidationIssue::MissingOrangePegs);
    }
    let smoke_test_board = board.tags.iter().any(|tag| tag.as_str() == "test");
    let boss_board = board.tags.iter().any(|tag| tag.as_str() == "boss");
    let max_orange_pegs = if boss_board { 28 } else { MAX_ORANGE_PEGS };
    if !smoke_test_board && !(MIN_ORANGE_PEGS..=max_orange_pegs).contains(&orange_count) {
        issues.push(BoardValidationIssue::OrangeCountOutOfRange {
            count: orange_count,
            min: MIN_ORANGE_PEGS,
            max: max_orange_pegs,
        });
    }

    for peg in &board.pegs {
        if !shape_inside_board(&peg.shape, board.size, BALL_RADIUS) {
            issues.push(BoardValidationIssue::PegOutsideBoard {
                peg: peg.id.clone(),
            });
        }
    }

    for obstacle in &board.obstacles {
        if !shape_inside_board(&obstacle.shape, board.size, 0.0) {
            issues.push(BoardValidationIssue::ObstacleOutsideBoard {
                obstacle: obstacle.id.clone(),
            });
        }
    }

    if board.bucket.width < MIN_BUCKET_WIDTH {
        issues.push(BoardValidationIssue::BucketTooNarrow {
            width: board.bucket.width,
        });
    }
    if board.bucket.width > MAX_BUCKET_WIDTH {
        issues.push(BoardValidationIssue::BucketTooWide {
            width: board.bucket.width,
        });
    }
    if board.bucket.center.x - board.bucket.width * 0.5 < 0.0
        || board.bucket.center.x + board.bucket.width * 0.5 > board.size.x
        || board.bucket.center.y - board.bucket.height * 0.5 < 0.0
        || board.bucket.center.y + board.bucket.height * 0.5 > board.kill_plane_y
    {
        issues.push(BoardValidationIssue::BucketOutsideBoard);
    }

    if !smoke_test_board {
        let sample = sample_aims(board, AIM_SAMPLES);
        if sample.reachable_oranges == 0 {
            issues.push(BoardValidationIssue::TooFewFirstShotTargets {
                reachable_oranges: sample.reachable_oranges,
            });
        }
        if sample.dead_zone_ratio > MAX_DEAD_ZONE_RATIO {
            issues.push(BoardValidationIssue::ExcessiveDeadZone {
                ratio: sample.dead_zone_ratio,
            });
        }
        if sample.viable_catch_angles < MIN_CATCH_ANGLES {
            issues.push(BoardValidationIssue::PoorBucketOpportunity {
                viable_angles: sample.viable_catch_angles,
            });
        }
    }

    BoardValidationReport {
        board_id: board.id.clone(),
        issues,
    }
}

fn archetype_point(
    archetype: &str,
    t: Scalar,
    index: usize,
    count: usize,
    rng: &mut Lcg,
) -> (Scalar, Scalar) {
    let centered = t - 0.5;
    match archetype {
        "fan" => (10.0 + centered * 15.0, 7.0 + t.abs() * 12.0),
        "wave" => (
            2.0 + t * 16.0,
            14.0 + (t * std::f64::consts::TAU * 2.0).sin() * 4.0,
        ),
        "clusters" => {
            let centers = [(5.0, 11.0), (14.8, 13.0), (8.0, 22.0), (15.5, 25.0)];
            let (cx, cy) = centers[index % centers.len()];
            (
                cx + (rng.next_unit() - 0.5) * 2.0,
                cy + (rng.next_unit() - 0.5) * 2.0,
            )
        }
        "lanes" => (
            4.0 + (index % 4) as Scalar * 4.0,
            8.0 + (index / 4) as Scalar * 3.5,
        ),
        "rings" => {
            let angle = t * std::f64::consts::TAU;
            let radius = if index < count / 2 { 4.0 } else { 7.2 };
            (10.0 + angle.cos() * radius, 18.0 + angle.sin() * radius)
        }
        "spiral" => {
            let angle = t * std::f64::consts::TAU * 2.4;
            let radius = 1.0 + t * 7.2;
            (
                10.0 + angle.cos() * radius,
                10.0 + t * 16.0 + angle.sin() * 1.5,
            )
        }
        "fortress" => (
            3.0 + (index % 8) as Scalar * 2.0,
            10.0 + (index / 8) as Scalar * 5.0,
        ),
        _ => (2.0 + t * 16.0, 10.0 + (index % 5) as Scalar * 3.0),
    }
}

#[derive(Default)]
struct AimSample {
    reachable_oranges: usize,
    dead_zone_ratio: Scalar,
    viable_catch_angles: usize,
}

fn sample_aims(board: &BoardDefinition, samples: usize) -> AimSample {
    let mut orange_hits = Vec::<PegId>::new();
    let mut no_hit_count = 0usize;
    let mut catch_angles = 0usize;
    let (min_angle, max_angle) = playable_aim_angle_range(board);

    for index in 0..samples {
        let t = index as Scalar / (samples - 1) as Scalar;
        let angle = min_angle + t * (max_angle - min_angle);
        let dir = Vec2::new(angle.sin(), angle.cos());
        let first_hit = first_collision(board, board.cannon_position, dir);
        if let Some(hit) = first_hit.as_ref() {
            if hit.kind == Some(PegKind::Orange) && !orange_hits.iter().any(|id| id == &hit.id) {
                orange_hits.push(hit.id.clone());
            }
        } else {
            no_hit_count += 1;
        }

        if has_bucket_opportunity(board, board.cannon_position, dir, first_hit.as_ref()) {
            catch_angles += 1;
        }
    }

    AimSample {
        reachable_oranges: orange_hits.len(),
        dead_zone_ratio: no_hit_count as Scalar / samples as Scalar,
        viable_catch_angles: catch_angles,
    }
}

fn playable_aim_angle_range(board: &BoardDefinition) -> (Scalar, Scalar) {
    let vertical_span = (board.kill_plane_y - board.cannon_position.y).max(Scalar::EPSILON);
    let left = ((BALL_RADIUS - board.cannon_position.x) / vertical_span)
        .atan()
        .max(-MAX_AIM_ANGLE_DEGREES.to_radians());
    let right = ((board.size.x - BALL_RADIUS - board.cannon_position.x) / vertical_span)
        .atan()
        .min(MAX_AIM_ANGLE_DEGREES.to_radians());

    if left < right {
        (left, right)
    } else {
        let fallback = MAX_AIM_ANGLE_DEGREES.to_radians();
        (-fallback, fallback)
    }
}

#[derive(Clone)]
struct CollisionHit {
    id: PegId,
    kind: Option<PegKind>,
    distance: Scalar,
    point: Vec2,
    normal: Vec2,
}

fn first_collision(board: &BoardDefinition, origin: Vec2, dir: Vec2) -> Option<CollisionHit> {
    let mut best: Option<CollisionHit> = None;

    for peg in &board.pegs {
        if let Some((distance, point, normal)) = ray_shape_hit(origin, dir, &peg.shape, BALL_RADIUS)
        {
            if distance > 0.0 && best.as_ref().is_none_or(|hit| distance < hit.distance) {
                best = Some(CollisionHit {
                    id: peg.id.clone(),
                    kind: Some(peg.kind),
                    distance,
                    point,
                    normal,
                });
            }
        }
    }

    for obstacle in &board.obstacles {
        if let Some((distance, point, normal)) =
            ray_shape_hit(origin, dir, &obstacle.shape, BALL_RADIUS)
        {
            if distance > 0.0 && best.as_ref().is_none_or(|hit| distance < hit.distance) {
                best = Some(CollisionHit {
                    id: PegId::new(format!(
                        "peg/proxy_{}",
                        obstacle.id.as_str().replace('/', "_")
                    ))
                    .expect("formatted proxy id is valid"),
                    kind: None,
                    distance,
                    point,
                    normal,
                });
            }
        }
    }

    best
}

fn has_bucket_opportunity(
    board: &BoardDefinition,
    origin: Vec2,
    dir: Vec2,
    hit: Option<&CollisionHit>,
) -> bool {
    let (start, travel) = if let Some(hit) = hit {
        let reflected = reflect(dir, hit.normal);
        (hit.point, reflected)
    } else {
        (origin, dir)
    };
    if travel.y <= 0.05 {
        return false;
    }
    let time = (board.bucket.center.y - start.y) / travel.y;
    if time <= 0.0 {
        return false;
    }
    let x = start.x + travel.x * time;
    let half_travel = (board.size.x - board.bucket.width) * 0.5;
    let sweep_min =
        board.bucket.center.x - half_travel - board.bucket.width * 0.5 - board.bucket.catch_margin;
    let sweep_max =
        board.bucket.center.x + half_travel + board.bucket.width * 0.5 + board.bucket.catch_margin;
    (sweep_min..=sweep_max).contains(&x)
}

fn ray_shape_hit(
    origin: Vec2,
    dir: Vec2,
    shape: &ShapeDef,
    padding: Scalar,
) -> Option<(Scalar, Vec2, Vec2)> {
    match shape {
        ShapeDef::Circle { center, radius } => {
            ray_circle_hit(origin, dir, *center, radius + padding)
        }
        ShapeDef::Capsule { a, b, radius } => {
            ray_capsule_hit(origin, dir, *a, *b, radius + padding)
        }
        ShapeDef::Segment { a, b } => ray_capsule_hit(origin, dir, *a, *b, padding),
        ShapeDef::Rect {
            center,
            half_extents,
        } => ray_rect_hit(
            origin,
            dir,
            *center,
            Vec2::new(half_extents.x + padding, half_extents.y + padding),
        ),
    }
}

fn ray_circle_hit(
    origin: Vec2,
    dir: Vec2,
    center: Vec2,
    radius: Scalar,
) -> Option<(Scalar, Vec2, Vec2)> {
    let oc = sub(origin, center);
    let b = dot(oc, dir);
    let c = dot(oc, oc) - radius * radius;
    let discriminant = b * b - c;
    if discriminant < 0.0 {
        return None;
    }
    let distance = -b - discriminant.sqrt();
    if distance <= 0.0 {
        return None;
    }
    let point = add(origin, mul(dir, distance));
    Some((distance, point, normalize(sub(point, center))))
}

fn ray_capsule_hit(
    origin: Vec2,
    dir: Vec2,
    a: Vec2,
    b: Vec2,
    radius: Scalar,
) -> Option<(Scalar, Vec2, Vec2)> {
    let steps = 12;
    let mut best = ray_circle_hit(origin, dir, a, radius)
        .into_iter()
        .chain(ray_circle_hit(origin, dir, b, radius))
        .min_by(|left, right| left.0.total_cmp(&right.0));
    for step in 1..steps {
        let t = step as Scalar / steps as Scalar;
        let center = Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
        if let Some(hit) = ray_circle_hit(origin, dir, center, radius) {
            if best.as_ref().is_none_or(|current| hit.0 < current.0) {
                best = Some(hit);
            }
        }
    }
    best
}

fn ray_rect_hit(origin: Vec2, dir: Vec2, center: Vec2, half: Vec2) -> Option<(Scalar, Vec2, Vec2)> {
    let min = Vec2::new(center.x - half.x, center.y - half.y);
    let max = Vec2::new(center.x + half.x, center.y + half.y);
    let inv_x = if dir.x.abs() < 0.0001 {
        Scalar::INFINITY
    } else {
        1.0 / dir.x
    };
    let inv_y = if dir.y.abs() < 0.0001 {
        Scalar::INFINITY
    } else {
        1.0 / dir.y
    };
    let mut t_min = (min.x - origin.x) * inv_x;
    let mut t_max = (max.x - origin.x) * inv_x;
    if t_min > t_max {
        std::mem::swap(&mut t_min, &mut t_max);
    }
    let mut ty_min = (min.y - origin.y) * inv_y;
    let mut ty_max = (max.y - origin.y) * inv_y;
    if ty_min > ty_max {
        std::mem::swap(&mut ty_min, &mut ty_max);
    }
    if t_min > ty_max || ty_min > t_max {
        return None;
    }
    let distance = t_min.max(ty_min);
    if distance <= 0.0 {
        return None;
    }
    let point = add(origin, mul(dir, distance));
    let normal = if (point.x - min.x).abs() < 0.01 {
        Vec2::new(-1.0, 0.0)
    } else if (point.x - max.x).abs() < 0.01 {
        Vec2::new(1.0, 0.0)
    } else if (point.y - min.y).abs() < 0.01 {
        Vec2::new(0.0, -1.0)
    } else {
        Vec2::new(0.0, 1.0)
    };
    Some((distance, point, normal))
}

fn shape_inside_board(shape: &ShapeDef, size: Vec2, padding: Scalar) -> bool {
    let (min, max) = shape_bounds(shape, padding);
    min.x >= 0.0 && min.y >= 0.0 && max.x <= size.x && max.y <= size.y
}

fn shape_bounds(shape: &ShapeDef, padding: Scalar) -> (Vec2, Vec2) {
    match shape {
        ShapeDef::Circle { center, radius } => {
            let radius = radius + padding;
            (
                Vec2::new(center.x - radius, center.y - radius),
                Vec2::new(center.x + radius, center.y + radius),
            )
        }
        ShapeDef::Capsule { a, b, radius } => {
            let radius = radius + padding;
            (
                Vec2::new(a.x.min(b.x) - radius, a.y.min(b.y) - radius),
                Vec2::new(a.x.max(b.x) + radius, a.y.max(b.y) + radius),
            )
        }
        ShapeDef::Segment { a, b } => (
            Vec2::new(a.x.min(b.x) - padding, a.y.min(b.y) - padding),
            Vec2::new(a.x.max(b.x) + padding, a.y.max(b.y) + padding),
        ),
        ShapeDef::Rect {
            center,
            half_extents,
        } => (
            Vec2::new(
                center.x - half_extents.x - padding,
                center.y - half_extents.y - padding,
            ),
            Vec2::new(
                center.x + half_extents.x + padding,
                center.y + half_extents.y + padding,
            ),
        ),
    }
}

fn add(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x + b.x, a.y + b.y)
}

fn sub(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x - b.x, a.y - b.y)
}

fn mul(v: Vec2, scalar: Scalar) -> Vec2 {
    Vec2::new(v.x * scalar, v.y * scalar)
}

fn dot(a: Vec2, b: Vec2) -> Scalar {
    a.x * b.x + a.y * b.y
}

fn normalize(v: Vec2) -> Vec2 {
    let length = dot(v, v).sqrt();
    if length <= 0.0001 {
        Vec2::ZERO
    } else {
        Vec2::new(v.x / length, v.y / length)
    }
}

fn reflect(dir: Vec2, normal: Vec2) -> Vec2 {
    sub(dir, mul(normal, 2.0 * dot(dir, normal)))
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
    use physics_core::{simulate_shot, ShotInput};

    const BUCKET_DIAGNOSTIC_SAMPLES: usize = 181;
    const MIN_DIAGNOSTIC_BOARDS_WITH_CATCHES: usize = 10;
    const MIN_CATCHABLE_TRAJECTORIES_PER_BOARD: usize = 2;

    fn params(seed: Seed) -> GenerationParams {
        GenerationParams {
            act: 1,
            difficulty: 1,
            archetype: ContentId::new("archetypes/fan").unwrap(),
            seed,
            peg_budget: 44,
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
    fn board_with_more_than_fifteen_percent_dead_zones_is_rejected() {
        let mut board = generate_board(&params(11));
        board.pegs = (0..MIN_ORANGE_PEGS)
            .map(|index| PegDef {
                id: PegId::new(format!("peg/o{index:02}")).unwrap(),
                kind: PegKind::Orange,
                shape: ShapeDef::Circle {
                    center: Vec2::new(4.0 + (index % 3) as Scalar * 0.8, 9.0 + index as Scalar),
                    radius: 0.35,
                },
            })
            .collect();

        let sample = sample_aims(&board, AIM_SAMPLES);
        assert!(
            sample.dead_zone_ratio > MAX_DEAD_ZONE_RATIO,
            "test board dead zone ratio should exceed threshold: {}",
            sample.dead_zone_ratio
        );

        let report = validate_board(&board);

        assert!(
            report
                .issues
                .iter()
                .any(|issue| matches!(issue, BoardValidationIssue::ExcessiveDeadZone { ratio } if *ratio > 0.15)),
            "expected excessive dead zone issue, got {:?}",
            report.issues
        );
    }

    #[test]
    fn generation_params_round_trip_json() {
        let value = params(99);
        let json = serde_json::to_string(&value).unwrap();
        let parsed: GenerationParams = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, value);
    }

    #[test]
    fn authored_boards_load_and_validate() {
        let boards = load_authored_boards(authored_boards_dir()).unwrap();

        if boards.is_empty() {
            return;
        }

        for board in boards {
            let report = validate_board(&board);
            assert!(
                report.is_valid(),
                "{}: {:?}",
                report.board_id,
                report.issues
            );
        }
    }

    #[test]
    fn authored_board_bucket_catch_skillfulness_diagnostic_pre_human_feel_scene() {
        let boards = load_authored_boards(authored_boards_dir()).unwrap();
        let mut reports = Vec::new();

        for board in &boards {
            reports.push((
                board.id.clone(),
                count_catchable_trajectories(board, BUCKET_DIAGNOSTIC_SAMPLES),
            ));
        }

        let boards_with_catches = reports
            .iter()
            .filter(|(_, catches)| *catches >= MIN_CATCHABLE_TRAJECTORIES_PER_BOARD)
            .count();

        eprintln!("bucket catch diagnostic reports: {reports:?}");

        // Checkpoint 1 diagnostic only: this samples authored boards through the deterministic
        // simulator so the orchestrator can spot missing bucket opportunities before a playable
        // feel scene exists. It does not replace human bucket satisfaction testing.
        assert!(
            boards_with_catches >= MIN_DIAGNOSTIC_BOARDS_WITH_CATCHES,
            "expected at least {MIN_DIAGNOSTIC_BOARDS_WITH_CATCHES} authored boards with \
             {MIN_CATCHABLE_TRAJECTORIES_PER_BOARD}+ catchable trajectories from \
             {BUCKET_DIAGNOSTIC_SAMPLES} sampled shots; reports: {reports:?}"
        );
    }

    fn count_catchable_trajectories(board: &BoardDefinition, samples: usize) -> usize {
        let (min_angle, max_angle) = playable_aim_angle_range(board);
        let ball_id = content_schema::BallId::new("balls/basic").unwrap();

        (0..samples)
            .filter(|index| {
                let t = *index as Scalar / (samples - 1) as Scalar;
                let aim_offset_from_down = min_angle + t * (max_angle - min_angle);
                let input = ShotInput {
                    aim_angle_radians: std::f64::consts::FRAC_PI_2 - aim_offset_from_down,
                    launch_speed: 17.5,
                    ball_id: ball_id.clone(),
                };

                simulate_shot(*index as u64, board, &input)
                    .summary
                    .caught_bucket
            })
            .count()
    }
}
