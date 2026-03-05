use crate::components::{FighterState, Health, PowerGauge, Velocity};
use crate::math::LogicVec2;
use crate::state_constants::*;
use crate::systems::collision::HitEvent;

/// Combo scaling: damage multiplier decreases with each hit in a combo.
/// Returns scaled damage as integer. combo_count starts at 1 for the first hit.
pub fn combo_scaled_damage(base_damage: i32, combo_count: u32) -> i32 {
    // Scaling: 100%, 100%, 80%, 70%, 60%, 50% (minimum 50%)
    let scale = match combo_count {
        0 | 1 => 100,
        2 => 100,
        3 => 80,
        4 => 70,
        5 => 60,
        _ => 50,
    };
    base_damage * scale / 100
}

/// Data needed for hit resolution on the defender side.
pub struct CombatEntity {
    pub health: Health,
    pub power_gauge: PowerGauge,
    pub velocity: Velocity,
    pub state: FighterState,
    pub combo_count: u32,
}

/// Result of processing a single hit event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HitResult {
    pub damage_dealt: i32,
    pub hitstun_applied: i32,
    pub knockback_applied: LogicVec2,
    pub attacker_gauge_gain: i32,
    pub defender_gauge_gain: i32,
}

/// Gauge gain constants
const ATTACKER_GAUGE_ON_HIT: i32 = 20;
const DEFENDER_GAUGE_ON_HIT: i32 = 10;

/// Processes a HitEvent: applies damage, hitstun, knockback, and power gauge changes.
pub fn hit_resolution_system(
    event: &HitEvent,
    attacker_gauge: &mut PowerGauge,
    defender: &mut CombatEntity,
) -> HitResult {
    let scaled_damage = combo_scaled_damage(event.hitbox.damage, defender.combo_count);

    // Apply damage
    defender.health.take_damage(scaled_damage);

    // Apply hitstun
    defender.state.change_state(STATE_HIT_STAND_LIGHT);

    // Apply knockback
    defender.velocity.vel = event.hitbox.knockback;

    // Update power gauges
    attacker_gauge.add(ATTACKER_GAUGE_ON_HIT);
    defender.power_gauge.add(DEFENDER_GAUGE_ON_HIT);

    // Increment combo
    defender.combo_count += 1;

    HitResult {
        damage_dealt: scaled_damage,
        hitstun_applied: event.hitbox.hitstun,
        knockback_applied: event.hitbox.knockback,
        attacker_gauge_gain: ATTACKER_GAUGE_ON_HIT,
        defender_gauge_gain: DEFENDER_GAUGE_ON_HIT,
    }
}

/// Applies blockstun to a defending entity.
pub fn apply_blockstun(defender: &mut CombatEntity) {
    defender.state.change_state(STATE_GUARD_STAND);
}

/// Updates power gauge by a given amount (can be used for special moves, etc.).
pub fn update_power_gauge(gauge: &mut PowerGauge, amount: i32) {
    gauge.add(amount);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{HitType, Hitbox};
    use crate::math::LogicVec2;

    fn make_hit_event(damage: i32, hitstun: i32, knockback_x: i32) -> HitEvent {
        HitEvent {
            attacker: 0,
            defender: 1,
            hitbox: Hitbox {
                rect: crate::math::LogicRect::new(0, 0, 100, 100),
                damage,
                hitstun,
                blockstun: 10,
                knockback: LogicVec2::new(knockback_x, 0),
                hit_type: HitType::Mid,
            },
        }
    }

    fn make_defender(hp: i32, combo_count: u32) -> CombatEntity {
        CombatEntity {
            health: Health::new(hp),
            power_gauge: PowerGauge::new(),
            velocity: Velocity {
                vel: LogicVec2::ZERO,
            },
            state: FighterState::new(),
            combo_count,
        }
    }

    // -- PLACEHOLDER_COMBAT_TESTS --

    #[test]
    fn test_hit_resolution_applies_damage() {
        let event = make_hit_event(1000, 15, 5000);
        let mut attacker_gauge = PowerGauge::new();
        let mut defender = make_defender(10000, 1);

        let result = hit_resolution_system(&event, &mut attacker_gauge, &mut defender);

        assert_eq!(result.damage_dealt, 1000);
        assert_eq!(defender.health.current, 9000);
    }

    #[test]
    fn test_hit_resolution_applies_hitstun() {
        let event = make_hit_event(1000, 15, 5000);
        let mut attacker_gauge = PowerGauge::new();
        let mut defender = make_defender(10000, 1);

        hit_resolution_system(&event, &mut attacker_gauge, &mut defender);

        assert_eq!(defender.state.state_num, STATE_HIT_STAND_LIGHT);
    }

    #[test]
    fn test_hit_resolution_applies_knockback() {
        let event = make_hit_event(1000, 15, 5000);
        let mut attacker_gauge = PowerGauge::new();
        let mut defender = make_defender(10000, 1);

        hit_resolution_system(&event, &mut attacker_gauge, &mut defender);

        assert_eq!(defender.velocity.vel, LogicVec2::new(5000, 0));
    }

    #[test]
    fn test_hit_resolution_updates_gauges() {
        let event = make_hit_event(1000, 15, 5000);
        let mut attacker_gauge = PowerGauge::new();
        let mut defender = make_defender(10000, 1);

        hit_resolution_system(&event, &mut attacker_gauge, &mut defender);

        assert_eq!(attacker_gauge.current, 20);
        assert_eq!(defender.power_gauge.current, 10);
    }

    #[test]
    fn test_combo_scaling() {
        assert_eq!(combo_scaled_damage(1000, 1), 1000); // 100%
        assert_eq!(combo_scaled_damage(1000, 2), 1000); // 100%
        assert_eq!(combo_scaled_damage(1000, 3), 800); // 80%
        assert_eq!(combo_scaled_damage(1000, 4), 700); // 70%
        assert_eq!(combo_scaled_damage(1000, 5), 600); // 60%
        assert_eq!(combo_scaled_damage(1000, 6), 500); // 50%
        assert_eq!(combo_scaled_damage(1000, 10), 500); // 50% floor
    }

    #[test]
    fn test_combo_scaling_applied_in_resolution() {
        let event = make_hit_event(1000, 15, 5000);
        let mut attacker_gauge = PowerGauge::new();
        // combo_count=3 means 80% scaling
        let mut defender = make_defender(10000, 3);

        let result = hit_resolution_system(&event, &mut attacker_gauge, &mut defender);

        assert_eq!(result.damage_dealt, 800);
        assert_eq!(defender.health.current, 9200);
    }

    #[test]
    fn test_blockstun() {
        let mut defender = make_defender(10000, 1);
        apply_blockstun(&mut defender);
        assert_eq!(defender.state.state_num, STATE_GUARD_STAND);
    }

    #[test]
    fn test_update_power_gauge() {
        let mut gauge = PowerGauge::new();
        update_power_gauge(&mut gauge, 500);
        assert_eq!(gauge.current, 500);
        update_power_gauge(&mut gauge, 2600);
        assert_eq!(gauge.current, 3000); // capped at max
    }
}
