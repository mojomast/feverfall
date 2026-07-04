use content_schema::{BallId, ObstacleId, PegId, Scalar, Tick, Vec2};
use serde::{Deserialize, Serialize};

pub const PHYSICS_VERSION: &str = "physics_core/0.1.0-contracts";

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
    pub replay_hash: u64,
}

pub fn stable_hash_events(events: &[PhysicsEvent]) -> u64 {
    let mut hash = Fnv1a64::default();
    for event in events {
        hash_event(&mut hash, event);
    }
    hash.finish()
}

#[derive(Clone, Copy)]
struct Fnv1a64(u64);

impl Default for Fnv1a64 {
    fn default() -> Self {
        Self(0xcbf29ce484222325)
    }
}

impl Fnv1a64 {
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }

    fn finish(self) -> u64 {
        self.0
    }
}

fn hash_event(hash: &mut Fnv1a64, event: &PhysicsEvent) {
    match event {
        PhysicsEvent::BallHitPeg {
            ball,
            peg,
            position,
            normal,
            speed,
            tick,
        } => {
            hash.write(b"BallHitPeg");
            hash.write(ball.as_str().as_bytes());
            hash.write(peg.as_str().as_bytes());
            hash_vec2(hash, *position);
            hash_vec2(hash, *normal);
            hash_scalar(hash, *speed);
            hash.write(&tick.to_le_bytes());
        }
        PhysicsEvent::BallHitObstacle {
            ball,
            obstacle,
            position,
            normal,
            tick,
        } => {
            hash.write(b"BallHitObstacle");
            hash.write(ball.as_str().as_bytes());
            hash.write(obstacle.as_str().as_bytes());
            hash_vec2(hash, *position);
            hash_vec2(hash, *normal);
            hash.write(&tick.to_le_bytes());
        }
        PhysicsEvent::BallEnteredBucket { ball, tick } => {
            hash.write(b"BallEnteredBucket");
            hash.write(ball.as_str().as_bytes());
            hash.write(&tick.to_le_bytes());
        }
        PhysicsEvent::BallExitedBoard { ball, tick } => {
            hash.write(b"BallExitedBoard");
            hash.write(ball.as_str().as_bytes());
            hash.write(&tick.to_le_bytes());
        }
        PhysicsEvent::ShotEnded { summary } => {
            hash.write(b"ShotEnded");
            hash.write(&summary.ticks.to_le_bytes());
            for peg in &summary.pegs_hit {
                hash.write(peg.as_str().as_bytes());
            }
            hash.write(&[summary.caught_bucket as u8, summary.exited_board as u8]);
            hash.write(&summary.replay_hash.to_le_bytes());
        }
    }
}

fn hash_vec2(hash: &mut Fnv1a64, value: Vec2) {
    hash_scalar(hash, value.x);
    hash_scalar(hash, value.y);
}

fn hash_scalar(hash: &mut Fnv1a64, value: Scalar) {
    hash.write(&value.to_bits().to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

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
            ball: BallId::new("balls/basic").unwrap(),
            tick: 120,
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: PhysicsEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, event);
    }

    #[test]
    fn event_hash_is_stable_for_same_events() {
        let events = vec![PhysicsEvent::BallHitPeg {
            ball: BallId::new("balls/basic").unwrap(),
            peg: PegId::new("peg/orange_001").unwrap(),
            position: Vec2::new(10.0, 12.0),
            normal: Vec2::new(0.0, -1.0),
            speed: 17.5,
            tick: 7,
        }];

        assert_eq!(stable_hash_events(&events), stable_hash_events(&events));
    }
}
