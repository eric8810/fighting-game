use tickle_mugen::{CmdParser, CommandDirection, CommandButton, InputModifier};

#[test]
fn test_parse_kyo_cmd() {
    let path = "../../mugen_resources/KyoKusanagi[SuzukiInoue]/kyo.cmd";
    if !std::path::Path::new(path).exists() {
        eprintln!("Skipping test: {} not found", path);
        return;
    }
    let cmd = CmdParser::parse(path).expect("Failed to parse kyo.cmd");
    assert!(!cmd.commands.is_empty(), "kyo.cmd should have commands");

    // Kyo should have QCF moves.
    let has_qcf = cmd.commands.iter().any(|c| {
        c.name.to_lowercase().contains("qcf")
    });
    assert!(has_qcf, "Kyo should have QCF commands");

    // Kyo should have a FF (forward dash) command.
    let has_ff = cmd.commands.iter().any(|c| c.name == "FF");
    assert!(has_ff, "Kyo should have FF dash command");

    println!("kyo.cmd: loaded {} commands", cmd.commands.len());
    for c in cmd.commands.iter().take(5) {
        println!("  {:?} ({} steps, time={})", c.name, c.inputs.len(), c.time);
    }
}

#[test]
fn test_parse_qcf_command() {
    let content = r#"
[Command]
name = "QCF_a"
command = ~D, DF, F, x
time = 15

[Defaults]
command.time = 15
command.buffer.time = 1
"#;
    let cmd = CmdParser::parse_content(content).expect("Failed to parse");
    assert_eq!(cmd.commands.len(), 1);
    assert_eq!(cmd.commands[0].name, "QCF_a");
    assert_eq!(cmd.commands[0].time, 15);

    // Should have 4 input steps: ~D, DF, F, x
    assert_eq!(cmd.commands[0].inputs.len(), 4);

    let steps = &cmd.commands[0].inputs;

    // ~D = release Down
    assert_eq!(steps[0].direction, Some(CommandDirection::Down));
    assert_eq!(steps[0].modifier, InputModifier::Release);

    // DF = DownForward, press
    assert_eq!(steps[1].direction, Some(CommandDirection::DownForward));
    assert_eq!(steps[1].modifier, InputModifier::None);

    // F = Forward, press
    assert_eq!(steps[2].direction, Some(CommandDirection::Forward));
    assert_eq!(steps[2].modifier, InputModifier::None);

    // x = button X, press
    assert_eq!(steps[3].buttons, vec![CommandButton::X]);
    assert_eq!(steps[3].modifier, InputModifier::None);
}

#[test]
fn test_parse_simultaneous_buttons() {
    let content = r#"
[Command]
name = "2buttons"
command = x+y
time = 5
"#;
    let cmd = CmdParser::parse_content(content).expect("Failed to parse");
    assert_eq!(cmd.commands.len(), 1);
    // One step with two simultaneous buttons.
    assert_eq!(cmd.commands[0].inputs.len(), 1);
    assert_eq!(cmd.commands[0].inputs[0].buttons.len(), 2);
    assert!(cmd.commands[0].inputs[0].buttons.contains(&CommandButton::X));
    assert!(cmd.commands[0].inputs[0].buttons.contains(&CommandButton::Y));
}

#[test]
fn test_parse_hold_command() {
    let content = r#"
[Command]
name = "holdfwd"
command = /$F
time = 1
"#;
    let cmd = CmdParser::parse_content(content).expect("Failed to parse");
    // `/` prefix = hold modifier
    assert_eq!(cmd.commands[0].inputs[0].modifier, InputModifier::Hold);
    assert_eq!(cmd.commands[0].inputs[0].direction, Some(CommandDirection::Forward));
}

#[test]
fn test_parse_charge_command() {
    let content = r#"
[Command]
name = "charge_back_a"
command = ~30$B, F, x
time = 40
"#;
    let cmd = CmdParser::parse_content(content).expect("Failed to parse");
    assert_eq!(cmd.commands.len(), 1);
    // Three steps: charge-Back, Forward, x
    assert_eq!(cmd.commands[0].inputs.len(), 3);
    // First step is Charge modifier targeting Back direction.
    assert_eq!(cmd.commands[0].inputs[0].modifier, InputModifier::Charge);
    assert_eq!(cmd.commands[0].inputs[0].direction, Some(CommandDirection::Back));
    // Second step: Forward press
    assert_eq!(cmd.commands[0].inputs[1].direction, Some(CommandDirection::Forward));
    assert_eq!(cmd.commands[0].inputs[1].modifier, InputModifier::None);
}

#[test]
fn test_defaults_override() {
    let content = r#"
[Defaults]
command.time = 20
command.buffer.time = 3

[Command]
name = "test"
command = x
"#;
    let cmd = CmdParser::parse_content(content).expect("Failed to parse");
    assert_eq!(cmd.default_time, 20);
    assert_eq!(cmd.default_buffer_time, 3);
}

#[test]
fn test_ff_dash_command() {
    let content = r#"
[Command]
name = "FF"
command = F, F
time = 10
"#;
    let cmd = CmdParser::parse_content(content).expect("Failed to parse");
    assert_eq!(cmd.commands[0].name, "FF");
    assert_eq!(cmd.commands[0].inputs.len(), 2);
    assert_eq!(cmd.commands[0].inputs[0].direction, Some(CommandDirection::Forward));
    assert_eq!(cmd.commands[0].inputs[1].direction, Some(CommandDirection::Forward));
}

#[test]
fn test_triple_simultaneous() {
    let content = r#"
[Command]
name = "abc"
command = x+y+z
time = 5
"#;
    let cmd = CmdParser::parse_content(content).expect("Failed to parse");
    assert_eq!(cmd.commands[0].inputs[0].buttons.len(), 3);
    assert!(cmd.commands[0].inputs[0].buttons.contains(&CommandButton::Z));
}
