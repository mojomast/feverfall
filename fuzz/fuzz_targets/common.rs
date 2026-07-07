#![allow(dead_code)]

use arbitrary::Arbitrary;
use content_schema::{
    BallId, BasketDef, BasketMotion, BoardDefinition, BoardId, ObstacleDef, ObstacleId,
    ObstacleKind, PegDef, PegId, PegKind, Scalar, ShapeDef, Vec2,
};
use physics_core::{PhysicsEvent, ShotInput};

const MAX_PEGS: usize = 16;
const MAX_OBSTACLES: usize = 8;

#[derive(Clone, Debug, Arbitrary)]
pub struct FuzzShot {
    pub seed: u64,
    angle: u16,
    speed: u8,
    sample_every: u8,
}

impl FuzzShot {
    pub fn input(&self) -> ShotInput {
        let angle_unit = self.angle as Scalar / u16::MAX as Scalar;
        ShotInput {
            // Keep launches generally down-board while still allowing wall grazes.
            aim_angle_radians: 0.05 + angle_unit * (std::f64::consts::PI - 0.10),
            launch_speed: 4.0 + (self.speed as Scalar / u8::MAX as Scalar) * 34.0,
            ball_id: BallId::new("balls/fuzz_basic").unwrap(),
        }
    }

    pub fn sample_every_ticks(&self) -> u64 {
        1 + (self.sample_every as u64 % 60)
    }
}

#[derive(Clone, Debug, Arbitrary)]
pub struct FuzzCase {
    pub board: FuzzBoard,
    pub shot: FuzzShot,
}

#[derive(Clone, Debug, Arbitrary)]
pub struct FuzzBoard {
    width: u8,
    height: u8,
    cannon_x: u8,
    bucket_x: u8,
    bucket_speed: i8,
    pegs: [RawPeg; MAX_PEGS],
    obstacles: [RawObstacle; MAX_OBSTACLES],
}

impl FuzzBoard {
    pub fn board_definition(&self, id_suffix: &str) -> BoardDefinition {
        let width = 8.0 + unit_u8(self.width) * 24.0;
        let height = 18.0 + unit_u8(self.height) * 30.0;
        let cannon_x = 0.75 + unit_u8(self.cannon_x) * (width - 1.5);
        let bucket_width = (1.5 + width * 0.10).min(4.5);
        let half_bucket = bucket_width * 0.5;
        let bucket_x = half_bucket + unit_u8(self.bucket_x) * (width - bucket_width);

        BoardDefinition {
            id: BoardId::new(format!("boards/fuzz_{id_suffix}")).unwrap(),
            size: Vec2::new(width, height),
            cannon_position: Vec2::new(cannon_x, 1.25),
            kill_plane_y: height + 1.5,
            pegs: self
                .pegs
                .iter()
                .enumerate()
                .map(|(idx, peg)| peg.to_def(idx, width, height))
                .collect(),
            obstacles: self
                .obstacles
                .iter()
                .enumerate()
                .map(|(idx, obstacle)| obstacle.to_def(idx, width, height))
                .collect(),
            bucket: BasketDef {
                center: Vec2::new(bucket_x, height - 1.2),
                width: bucket_width,
                height: 0.6,
                horizontal_speed: (self.bucket_speed as Scalar / i8::MAX as Scalar) * 8.0,
                motion: BasketMotion::PingPong,
                catch_margin: 0.18,
            },
            tags: Vec::new(),
            objectives: Vec::new(),
            boss_mechanic: None,
        }
    }
}

#[derive(Clone, Debug, Arbitrary)]
struct RawPeg {
    kind: u8,
    shape: u8,
    x: u8,
    y: u8,
    x2: u8,
    y2: u8,
    radius: u8,
}

impl RawPeg {
    fn to_def(&self, idx: usize, width: Scalar, height: Scalar) -> PegDef {
        PegDef {
            id: PegId::new(format!("peg/fuzz_{idx:02}")).unwrap(),
            kind: match self.kind % 8 {
                0 => PegKind::Blue,
                1 => PegKind::Orange,
                2 => PegKind::Purple,
                3 => PegKind::Green,
                4 => PegKind::Stone,
                5 => PegKind::Ghost,
                6 => PegKind::Bomb,
                _ => PegKind::Splitter,
            },
            shape: bounded_shape(
                self.shape,
                self.x,
                self.y,
                self.x2,
                self.y2,
                self.radius,
                width,
                height,
            ),
        }
    }
}

#[derive(Clone, Debug, Arbitrary)]
struct RawObstacle {
    kind: u8,
    shape: u8,
    x: u8,
    y: u8,
    x2: u8,
    y2: u8,
    radius: u8,
}

impl RawObstacle {
    fn to_def(&self, idx: usize, width: Scalar, height: Scalar) -> ObstacleDef {
        ObstacleDef {
            id: ObstacleId::new(format!("obstacles/fuzz_{idx:02}")).unwrap(),
            kind: match self.kind % 4 {
                0 => ObstacleKind::Wall,
                1 => ObstacleKind::Stone,
                2 => ObstacleKind::Rubber,
                _ => ObstacleKind::Sensor,
            },
            shape: bounded_shape(
                self.shape,
                self.x,
                self.y,
                self.x2,
                self.y2,
                self.radius,
                width,
                height,
            ),
        }
    }
}

fn bounded_shape(
    shape: u8,
    x: u8,
    y: u8,
    x2: u8,
    y2: u8,
    radius: u8,
    width: Scalar,
    height: Scalar,
) -> ShapeDef {
    let p1 = bounded_point(x, y, width, height);
    let p2 = bounded_point(x2, y2, width, height);
    let r = 0.08 + unit_u8(radius) * 0.55;
    match shape % 4 {
        0 => ShapeDef::Circle {
            center: p1,
            radius: r,
        },
        1 => ShapeDef::Capsule {
            a: p1,
            b: p2,
            radius: r.min(0.35),
        },
        2 => ShapeDef::Segment { a: p1, b: p2 },
        _ => ShapeDef::Rect {
            center: p1,
            half_extents: Vec2::new(0.15 + unit_u8(x2) * 1.2, 0.08 + unit_u8(y2) * 0.8),
        },
    }
}

fn bounded_point(x: u8, y: u8, width: Scalar, height: Scalar) -> Vec2 {
    Vec2::new(
        0.6 + unit_u8(x) * (width - 1.2),
        3.0 + unit_u8(y) * (height - 7.0),
    )
}

fn unit_u8(value: u8) -> Scalar {
    value as Scalar / u8::MAX as Scalar
}

pub fn first_collision_signature(event: &PhysicsEvent) -> Option<(&'static str, String, u64)> {
    match event {
        PhysicsEvent::BallHitPeg { peg, tick, .. } => Some(("peg", peg.as_str().to_owned(), *tick)),
        PhysicsEvent::BallHitObstacle { obstacle, tick, .. } => {
            Some(("obstacle", obstacle.as_str().to_owned(), *tick))
        }
        _ => None,
    }
}

pub fn finite_event_positions(event: &PhysicsEvent) -> bool {
    match event {
        PhysicsEvent::BallHitPeg {
            position,
            normal,
            speed,
            ..
        } => finite_vec(*position) && finite_vec(*normal) && speed.is_finite(),
        PhysicsEvent::BallHitObstacle {
            position, normal, ..
        } => finite_vec(*position) && finite_vec(*normal),
        _ => true,
    }
}

pub fn finite_vec(value: Vec2) -> bool {
    value.x.is_finite() && value.y.is_finite()
}
