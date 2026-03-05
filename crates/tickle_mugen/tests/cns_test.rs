use tickle_mugen::{CnsParser, StateTypeValue};
use std::path::PathBuf;

fn get_test_file_path() -> PathBuf {
    // Tests run from crate directory, need to go up to workspace root
    PathBuf::from("../../mugen_resources/KyoKusanagi[SuzukiInoue]/kyo.cns")
}

#[test]
fn test_cns_parser() {
    let path = get_test_file_path();

    // Skip test if file doesn't exist (e.g., in CI without resources)
    if !path.exists() {
        eprintln!("Skipping test: {} not found", path.display());
        return;
    }

    let cns = CnsParser::parse(&path).unwrap();

    // Test Data section
    assert_eq!(cns.data.life, 1000);
    assert_eq!(cns.data.attack, 100);
    assert_eq!(cns.data.defence, 100);
    assert_eq!(cns.data.liedown_time, 60);
    assert_eq!(cns.data.airjuggle, 15);

    // Test Size section
    assert_eq!(cns.size.xscale, 1.0);
    assert_eq!(cns.size.yscale, 1.0);
    assert_eq!(cns.size.ground_back, 20);
    assert_eq!(cns.size.ground_front, 20);
    assert_eq!(cns.size.height, 60);
    assert_eq!(cns.size.attack_dist, 160);

    // Test Velocity section
    assert_eq!(cns.velocity.walk_fwd.x, 3.0);
    assert_eq!(cns.velocity.walk_fwd.y, 0.0);
    assert_eq!(cns.velocity.walk_back.x, -2.5);
    assert_eq!(cns.velocity.walk_back.y, 0.0);
    assert_eq!(cns.velocity.run_fwd.x, 5.5);
    assert_eq!(cns.velocity.run_fwd.y, 0.0);
    assert_eq!(cns.velocity.jump_neu.x, 0.0);
    assert_eq!(cns.velocity.jump_neu.y, -9.0);

    // Test Movement section
    assert_eq!(cns.movement.airjump_num, 0);
    assert_eq!(cns.movement.airjump_height, 35.0);
    assert_eq!(cns.movement.yaccel, 0.55);
    assert_eq!(cns.movement.stand_friction, 0.8);
    assert_eq!(cns.movement.crouch_friction, 0.8);

    // Test statedefs count - Kyo has 158 statedefs
    assert!(cns.statedefs.len() >= 150, "Expected at least 150 statedefs, got {}", cns.statedefs.len());

    // Test specific state (state 0 - standing idle)
    let state_0 = cns.get_state(0);
    if let Some(state) = state_0 {
        assert_eq!(state.anim, Some(0));
        assert_eq!(state.state_type, StateTypeValue::Standing);
    }

    // Test state 40 (jump start)
    let state_40 = cns.get_state(40).unwrap();
    assert_eq!(state_40.state_type, StateTypeValue::Standing);
    assert_eq!(state_40.anim, Some(40));
    assert_eq!(state_40.ctrl, Some(0));
}
