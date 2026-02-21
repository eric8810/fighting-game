use crate::components::{FighterState, Position, StateType, Velocity};

#[allow(unused_imports)]
use crate::math::LogicVec2;

/// Default gravity acceleration per frame (in logic units)
/// -80 per frame^2 = 0.8 pixels/frame^2 downward
pub const DEFAULT_GRAVITY: i32 = -80;

/// Ground level Y coordinate
pub const GROUND_Y: i32 = 0;

/// Default ground friction deceleration per frame
pub const DEFAULT_FRICTION: i32 = 50;

/// Applies velocity to position for all entities.
pub fn velocity_system(entities: &mut [(Position, Velocity)]) {
    for (pos, vel) in entities.iter_mut() {
        pos.pos.x += vel.vel.x;
        pos.pos.y += vel.vel.y;
    }
}

/// Applies gravity to airborne entities (position.y > GROUND_Y).
pub fn gravity_system(entities: &mut [(Position, Velocity)], gravity: i32) {
    for (pos, vel) in entities.iter_mut() {
        if pos.pos.y > GROUND_Y || vel.vel.y > 0 {
            vel.vel.y += gravity;
        }
    }
}

/// Detects landing: if an entity has fallen below ground, snap to ground and zero Y velocity.
/// Returns indices of entities that just landed.
pub fn ground_detection_system(entities: &mut [(Position, Velocity, FighterState)]) -> Vec<usize> {
    let mut landed = Vec::new();
    for (i, (pos, vel, state)) in entities.iter_mut().enumerate() {
        if pos.pos.y < GROUND_Y {
            pos.pos.y = GROUND_Y;
            vel.vel.y = 0;
            // Transition from Jump to Idle on landing
            if state.current_state == StateType::Jump {
                state.change_state(StateType::Idle);
            }
            landed.push(i);
        }
    }
    landed
}

/// Applies ground friction to entities on the ground, decelerating X velocity toward zero.
pub fn friction_system(entities: &mut [(Position, Velocity)], friction: i32) {
    for (pos, vel) in entities.iter_mut() {
        if pos.pos.y == GROUND_Y && vel.vel.x != 0 {
            if vel.vel.x > 0 {
                vel.vel.x = (vel.vel.x - friction).max(0);
            } else {
                vel.vel.x = (vel.vel.x + friction).min(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::LogicVec2;

    fn pos(x: i32, y: i32) -> Position {
        Position {
            pos: LogicVec2::new(x, y),
        }
    }

    fn vel(x: i32, y: i32) -> Velocity {
        Velocity {
            vel: LogicVec2::new(x, y),
        }
    }

    #[test]
    fn test_velocity_system_applies_velocity() {
        let mut entities = [(pos(1000, 2000), vel(500, -80))];
        velocity_system(&mut entities);
        assert_eq!(entities[0].0.pos, LogicVec2::new(1500, 1920));
    }

    #[test]
    fn test_velocity_system_zero_velocity() {
        let mut entities = [(pos(1000, 0), vel(0, 0))];
        velocity_system(&mut entities);
        assert_eq!(entities[0].0.pos, LogicVec2::new(1000, 0));
    }

    #[test]
    fn test_gravity_system_airborne() {
        let mut entities = [(pos(0, 5000), vel(0, 0))];
        gravity_system(&mut entities, DEFAULT_GRAVITY);
        assert_eq!(entities[0].1.vel.y, -80);
    }

    #[test]
    fn test_gravity_system_on_ground_no_effect() {
        let mut entities = [(pos(0, GROUND_Y), vel(0, 0))];
        gravity_system(&mut entities, DEFAULT_GRAVITY);
        assert_eq!(entities[0].1.vel.y, 0);
    }

    #[test]
    fn test_gravity_system_rising_entity() {
        // Entity at ground but with upward velocity should still get gravity
        let mut entities = [(pos(0, GROUND_Y), vel(0, 500))];
        gravity_system(&mut entities, DEFAULT_GRAVITY);
        assert_eq!(entities[0].1.vel.y, 420);
    }

    #[test]
    fn test_ground_detection_snaps_to_ground() {
        let mut entities = [(pos(0, -100), vel(0, -200), FighterState::new())];
        entities[0].2.change_state(StateType::Jump);
        let landed = ground_detection_system(&mut entities);
        assert_eq!(entities[0].0.pos.y, GROUND_Y);
        assert_eq!(entities[0].1.vel.y, 0);
        assert_eq!(entities[0].2.current_state, StateType::Idle);
        assert_eq!(landed, vec![0]);
    }

    #[test]
    fn test_ground_detection_no_landing() {
        let mut entities = [(pos(0, 5000), vel(0, -80), FighterState::new())];
        entities[0].2.change_state(StateType::Jump);
        let landed = ground_detection_system(&mut entities);
        assert!(landed.is_empty());
        assert_eq!(entities[0].2.current_state, StateType::Jump);
    }

    #[test]
    fn test_friction_decelerates_positive() {
        let mut entities = [(pos(0, GROUND_Y), vel(200, 0))];
        friction_system(&mut entities, DEFAULT_FRICTION);
        assert_eq!(entities[0].1.vel.x, 150);
    }

    #[test]
    fn test_friction_decelerates_negative() {
        let mut entities = [(pos(0, GROUND_Y), vel(-200, 0))];
        friction_system(&mut entities, DEFAULT_FRICTION);
        assert_eq!(entities[0].1.vel.x, -150);
    }

    #[test]
    fn test_friction_clamps_to_zero() {
        let mut entities = [(pos(0, GROUND_Y), vel(30, 0))];
        friction_system(&mut entities, DEFAULT_FRICTION);
        assert_eq!(entities[0].1.vel.x, 0);
    }

    #[test]
    fn test_friction_no_effect_airborne() {
        let mut entities = [(pos(0, 5000), vel(200, 0))];
        friction_system(&mut entities, DEFAULT_FRICTION);
        assert_eq!(entities[0].1.vel.x, 200); // unchanged
    }
}
