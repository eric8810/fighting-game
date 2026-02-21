use crate::math::{LogicRect, LogicVec2};
use serde::{Deserialize, Serialize};

/// Position component
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub pos: LogicVec2,
}

/// Previous position for rendering interpolation
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviousPosition {
    pub pos: LogicVec2,
}

/// Velocity component
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Velocity {
    pub vel: LogicVec2,
}

/// Facing direction (1 = right, -1 = left)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Facing {
    pub dir: i32,
}

impl Facing {
    pub const RIGHT: i32 = 1;
    pub const LEFT: i32 = -1;

    pub fn flip(&mut self) {
        self.dir = -self.dir;
    }
}

/// Health component
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.current = (self.current - damage).max(0);
    }

    pub fn heal(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0
    }

    pub fn percentage(&self) -> f32 {
        self.current as f32 / self.max as f32
    }
}

/// Power gauge (3 stocks, 0-3000 points)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerGauge {
    pub current: i32,
    pub max: i32,
}

impl PowerGauge {
    pub const MAX: i32 = 3000;
    pub const STOCK_SIZE: i32 = 1000;

    pub fn new() -> Self {
        Self {
            current: 0,
            max: Self::MAX,
        }
    }

    pub fn add(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn consume(&mut self, amount: i32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    pub fn stocks(&self) -> i32 {
        self.current / Self::STOCK_SIZE
    }

    pub fn can_use_super(&self) -> bool {
        self.stocks() >= 1
    }
}

impl Default for PowerGauge {
    fn default() -> Self {
        Self::new()
    }
}

/// Fighter state type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateType {
    Idle,
    WalkForward,
    WalkBackward,
    Run,
    Crouch,
    Jump,
    Attack(u32), // Attack ID
    Hitstun,
    Blockstun,
    Knockdown,
}

/// Fighter state component
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FighterState {
    pub current_state: StateType,
    pub state_frame: u32,
}

impl FighterState {
    pub fn new() -> Self {
        Self {
            current_state: StateType::Idle,
            state_frame: 0,
        }
    }

    pub fn change_state(&mut self, new_state: StateType) {
        self.current_state = new_state;
        self.state_frame = 0;
    }

    pub fn advance_frame(&mut self) {
        self.state_frame += 1;
    }
}

impl Default for FighterState {
    fn default() -> Self {
        Self::new()
    }
}

/// Hit type for attack classification
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HitType {
    High,  // Standing guard only
    Mid,   // Standing or crouching guard
    Low,   // Crouching guard only
    Throw, // Unblockable
}

/// Hitbox (attack box)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hitbox {
    pub rect: LogicRect,
    pub damage: i32,
    pub hitstun: u32,
    pub blockstun: u32,
    pub knockback: LogicVec2,
    pub hit_type: HitType,
}

/// Hurtbox (vulnerable area)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hurtbox {
    pub rect: LogicRect,
}

/// Pushbox (collision box for character separation)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pushbox {
    pub rect: LogicRect,
}

/// Hitbox manager component
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HitboxManager {
    pub hitboxes: Vec<Hitbox>,
    pub hurtboxes: Vec<Hurtbox>,
    pub pushbox: Pushbox,
}

impl HitboxManager {
    pub fn new(pushbox: Pushbox) -> Self {
        Self {
            hitboxes: Vec::new(),
            hurtboxes: Vec::new(),
            pushbox,
        }
    }

    pub fn clear_hitboxes(&mut self) {
        self.hitboxes.clear();
    }

    pub fn clear_hurtboxes(&mut self) {
        self.hurtboxes.clear();
    }

    pub fn add_hitbox(&mut self, hitbox: Hitbox) {
        self.hitboxes.push(hitbox);
    }

    pub fn add_hurtbox(&mut self, hurtbox: Hurtbox) {
        self.hurtboxes.push(hurtbox);
    }

    /// Transform hitboxes to world space
    pub fn world_hitboxes(&self, pos: Position, facing: Facing) -> Vec<LogicRect> {
        self.hitboxes
            .iter()
            .map(|h| {
                let mut rect = h.rect.translate(pos.pos);
                if facing.dir == Facing::LEFT {
                    rect = rect.flip_x(pos.pos.x);
                }
                rect
            })
            .collect()
    }

    /// Transform hurtboxes to world space
    pub fn world_hurtboxes(&self, pos: Position, facing: Facing) -> Vec<LogicRect> {
        self.hurtboxes
            .iter()
            .map(|h| {
                let mut rect = h.rect.translate(pos.pos);
                if facing.dir == Facing::LEFT {
                    rect = rect.flip_x(pos.pos.x);
                }
                rect
            })
            .collect()
    }

    /// Transform pushbox to world space
    pub fn world_pushbox(&self, pos: Position, facing: Facing) -> LogicRect {
        let mut rect = self.pushbox.rect.translate(pos.pos);
        if facing.dir == Facing::LEFT {
            rect = rect.flip_x(pos.pos.x);
        }
        rect
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health() {
        let mut health = Health::new(10000);
        assert_eq!(health.current, 10000);
        assert!(health.is_alive());

        health.take_damage(3000);
        assert_eq!(health.current, 7000);
        assert_eq!(health.percentage(), 0.7);

        health.heal(1000);
        assert_eq!(health.current, 8000);

        health.take_damage(10000);
        assert_eq!(health.current, 0);
        assert!(!health.is_alive());
    }

    #[test]
    fn test_power_gauge() {
        let mut gauge = PowerGauge::new();
        assert_eq!(gauge.stocks(), 0);
        assert!(!gauge.can_use_super());

        gauge.add(1500);
        assert_eq!(gauge.stocks(), 1);
        assert!(gauge.can_use_super());

        assert!(gauge.consume(1000));
        assert_eq!(gauge.current, 500);

        assert!(!gauge.consume(1000));
        assert_eq!(gauge.current, 500);
    }

    #[test]
    fn test_facing() {
        let mut facing = Facing { dir: Facing::RIGHT };
        assert_eq!(facing.dir, 1);

        facing.flip();
        assert_eq!(facing.dir, -1);

        facing.flip();
        assert_eq!(facing.dir, 1);
    }

    #[test]
    fn test_fighter_state() {
        let mut state = FighterState::new();
        assert_eq!(state.current_state, StateType::Idle);
        assert_eq!(state.state_frame, 0);

        state.advance_frame();
        assert_eq!(state.state_frame, 1);

        state.change_state(StateType::Jump);
        assert_eq!(state.current_state, StateType::Jump);
        assert_eq!(state.state_frame, 0);
    }

    #[test]
    fn test_hitbox_manager_world_transform() {
        let mut manager = HitboxManager::new(Pushbox {
            rect: LogicRect::new(-1500, -8000, 3000, 8000),
        });

        manager.add_hitbox(Hitbox {
            rect: LogicRect::new(2000, -5000, 6000, 3000),
            damage: 1000,
            hitstun: 15,
            blockstun: 10,
            knockback: LogicVec2::new(5000, 0),
            hit_type: HitType::Mid,
        });

        let pos = Position {
            pos: LogicVec2::new(50000, 0),
        };
        let facing = Facing { dir: Facing::RIGHT };

        let world_hitboxes = manager.world_hitboxes(pos, facing);
        assert_eq!(world_hitboxes.len(), 1);
        assert_eq!(world_hitboxes[0].x, 52000); // 50000 + 2000

        // Test flip
        let facing_left = Facing { dir: Facing::LEFT };
        let world_hitboxes_flipped = manager.world_hitboxes(pos, facing_left);
        assert_eq!(world_hitboxes_flipped[0].x, 42000); // Flipped around pivot
    }
}
