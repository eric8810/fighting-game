use crate::components::{Facing, Hitbox, HitboxManager, Position};

#[allow(unused_imports)]
use crate::math::LogicRect;

/// Event generated when a hitbox intersects a hurtbox.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HitEvent {
    pub attacker: usize,
    pub defender: usize,
    pub hitbox: Hitbox,
}

/// Entity data needed for collision detection.
pub struct CollisionEntity {
    pub position: Position,
    pub facing: Facing,
    pub hitbox_manager: HitboxManager,
}

/// Checks all hitbox-vs-hurtbox pairs between different entities.
/// Returns a list of HitEvents for each collision found.
pub fn collision_system(entities: &[CollisionEntity]) -> Vec<HitEvent> {
    let mut events = Vec::new();

    for (i, attacker) in entities.iter().enumerate() {
        let world_hitboxes = attacker
            .hitbox_manager
            .world_hitboxes(attacker.position, attacker.facing);

        for (j, defender) in entities.iter().enumerate() {
            if i == j {
                continue;
            }

            let world_hurtboxes = defender
                .hitbox_manager
                .world_hurtboxes(defender.position, defender.facing);

            for (hi, world_hb) in world_hitboxes.iter().enumerate() {
                for world_hurt in &world_hurtboxes {
                    if world_hb.intersects(*world_hurt) {
                        events.push(HitEvent {
                            attacker: i,
                            defender: j,
                            hitbox: attacker.hitbox_manager.hitboxes[hi],
                        });
                    }
                }
            }
        }
    }

    events
}

/// Pushbox entity data for separation.
pub struct PushboxEntity {
    pub position: Position,
    pub facing: Facing,
    pub hitbox_manager: HitboxManager,
}

/// Separates overlapping pushboxes by pushing each entity half the overlap distance apart.
/// Returns the position adjustments applied (for testing).
pub fn pushbox_separation_system(entities: &mut [PushboxEntity]) -> Vec<(usize, usize, i32)> {
    let mut separations = Vec::new();
    let len = entities.len();

    for i in 0..len {
        for j in (i + 1)..len {
            let pb_a = entities[i]
                .hitbox_manager
                .world_pushbox(entities[i].position, entities[i].facing);
            let pb_b = entities[j]
                .hitbox_manager
                .world_pushbox(entities[j].position, entities[j].facing);

            if pb_a.intersects(pb_b) {
                // Calculate overlap on X axis
                let overlap_left = (pb_a.x + pb_a.w) - pb_b.x;
                let overlap_right = (pb_b.x + pb_b.w) - pb_a.x;
                let overlap = overlap_left.min(overlap_right);

                if overlap > 0 {
                    let half = overlap / 2;
                    let remainder = overlap - half * 2;

                    if entities[i].position.pos.x <= entities[j].position.pos.x {
                        entities[i].position.pos.x -= half + remainder;
                        entities[j].position.pos.x += half;
                    } else {
                        entities[i].position.pos.x += half;
                        entities[j].position.pos.x -= half + remainder;
                    }

                    separations.push((i, j, overlap));
                }
            }
        }
    }

    separations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Facing, Hurtbox, Pushbox};
    use crate::math::{LogicRect, LogicVec2};

    fn make_collision_entity(
        x: i32,
        y: i32,
        hitboxes: Vec<Hitbox>,
        hurtboxes: Vec<Hurtbox>,
    ) -> CollisionEntity {
        let mut mgr = HitboxManager::new(Pushbox {
            rect: LogicRect::new(-1500, -8000, 3000, 8000),
        });
        for h in hitboxes {
            mgr.add_hitbox(h);
        }
        for h in hurtboxes {
            mgr.add_hurtbox(h);
        }
        CollisionEntity {
            position: Position {
                pos: LogicVec2::new(x, y),
            },
            facing: Facing { dir: Facing::RIGHT },
            hitbox_manager: mgr,
        }
    }

    fn sample_hitbox() -> Hitbox {
        Hitbox {
            rect: LogicRect::new(2000, -5000, 6000, 3000),
            damage: 1000,
            hitstun: 15,
            blockstun: 10,
            knockback: LogicVec2::new(5000, 0),
            hit_type: crate::components::HitType::Mid,
        }
    }

    fn sample_hurtbox() -> Hurtbox {
        Hurtbox {
            rect: LogicRect::new(-2000, -8000, 4000, 8000),
        }
    }

    // -- PLACEHOLDER_TESTS --

    #[test]
    fn test_collision_hit_detected() {
        // Attacker at x=0 with hitbox extending right, defender at x=5000 with hurtbox
        let entities = vec![
            make_collision_entity(0, 0, vec![sample_hitbox()], vec![sample_hurtbox()]),
            make_collision_entity(5000, 0, vec![], vec![sample_hurtbox()]),
        ];
        let events = collision_system(&entities);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].attacker, 0);
        assert_eq!(events[0].defender, 1);
    }

    #[test]
    fn test_collision_no_hit_when_far_apart() {
        let entities = vec![
            make_collision_entity(0, 0, vec![sample_hitbox()], vec![sample_hurtbox()]),
            make_collision_entity(50000, 0, vec![], vec![sample_hurtbox()]),
        ];
        let events = collision_system(&entities);
        assert!(events.is_empty());
    }

    #[test]
    fn test_collision_self_not_hit() {
        // Entity with both hitbox and hurtbox should not hit itself
        let entities = vec![make_collision_entity(
            0,
            0,
            vec![sample_hitbox()],
            vec![sample_hurtbox()],
        )];
        let events = collision_system(&entities);
        assert!(events.is_empty());
    }

    #[test]
    fn test_pushbox_separation() {
        let mut entities = vec![
            PushboxEntity {
                position: Position {
                    pos: LogicVec2::new(0, 0),
                },
                facing: Facing { dir: Facing::RIGHT },
                hitbox_manager: HitboxManager::new(Pushbox {
                    rect: LogicRect::new(-1500, -8000, 3000, 8000),
                }),
            },
            PushboxEntity {
                position: Position {
                    pos: LogicVec2::new(2000, 0),
                },
                facing: Facing { dir: Facing::RIGHT },
                hitbox_manager: HitboxManager::new(Pushbox {
                    rect: LogicRect::new(-1500, -8000, 3000, 8000),
                }),
            },
        ];
        let seps = pushbox_separation_system(&mut entities);
        assert_eq!(seps.len(), 1);
        // After separation, pushboxes should no longer overlap
        let pb_a = entities[0]
            .hitbox_manager
            .world_pushbox(entities[0].position, entities[0].facing);
        let pb_b = entities[1]
            .hitbox_manager
            .world_pushbox(entities[1].position, entities[1].facing);
        assert!(!pb_a.intersects(pb_b));
    }

    #[test]
    fn test_pushbox_no_overlap() {
        let mut entities = vec![
            PushboxEntity {
                position: Position {
                    pos: LogicVec2::new(0, 0),
                },
                facing: Facing { dir: Facing::RIGHT },
                hitbox_manager: HitboxManager::new(Pushbox {
                    rect: LogicRect::new(-1500, -8000, 3000, 8000),
                }),
            },
            PushboxEntity {
                position: Position {
                    pos: LogicVec2::new(10000, 0),
                },
                facing: Facing { dir: Facing::RIGHT },
                hitbox_manager: HitboxManager::new(Pushbox {
                    rect: LogicRect::new(-1500, -8000, 3000, 8000),
                }),
            },
        ];
        let seps = pushbox_separation_system(&mut entities);
        assert!(seps.is_empty());
    }
}
