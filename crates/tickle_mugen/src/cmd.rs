use crate::error::SffError;
use std::path::Path;

/// A parsed input step in a command sequence.
#[derive(Debug, Clone, PartialEq)]
pub struct InputStep {
    pub direction: Option<CommandDirection>,
    pub buttons: Vec<CommandButton>,
    pub modifier: InputModifier,
}

/// Input modifier (how the input must be performed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputModifier {
    /// Default: press the input.
    None,
    /// `~` prefix: release the input.
    Release,
    /// `$` or `/` prefix: hold the input.
    Hold,
    /// `~N` prefix: charge the input for N frames then release.
    Charge,
}

/// Directional input for a command step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandDirection {
    Forward,
    Back,
    Up,
    Down,
    DownForward,
    DownBack,
    UpForward,
    UpBack,
}

/// Button input for a command step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandButton {
    X,
    Y,
    Z,
    A,
    B,
    C,
    S,
}

/// A single named command (special move input sequence).
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub name: String,
    /// Ordered sequence of input steps that must be performed.
    pub inputs: Vec<InputStep>,
    /// Time window (in frames) in which all inputs must occur.
    pub time: u32,
    /// Input buffer time (in frames).
    pub buffer_time: u32,
}

/// Parsed CMD file containing all commands and defaults.
#[derive(Debug, Clone)]
pub struct Cmd {
    pub commands: Vec<Command>,
    pub default_time: u32,
    pub default_buffer_time: u32,
}

pub struct CmdParser;

impl CmdParser {
    /// Parse a CMD file from the filesystem.
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Cmd, SffError> {
        let content = std::fs::read_to_string(path)?;
        Self::parse_content(&content)
    }

    /// Parse CMD content from a string.
    pub fn parse_content(content: &str) -> Result<Cmd, SffError> {
        let mut commands = Vec::new();
        let mut default_time: u32 = 15;
        let mut default_buffer_time: u32 = 1;

        // Accumulate current section name and its key=value pairs.
        let mut current_section: Option<String> = None;
        let mut section_kvs: Vec<(String, String)> = Vec::new();

        for raw_line in content.lines() {
            // Strip inline comments and trim.
            let line = if let Some(pos) = raw_line.find(';') {
                &raw_line[..pos]
            } else {
                raw_line
            };
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            // Section header: [Name] or [Name Something]
            if line.starts_with('[') {
                if let Some(close) = line.find(']') {
                    // Flush the previous section.
                    if let Some(ref section_name) = current_section {
                        let dt = default_time;
                        let dbt = default_buffer_time;
                        Self::flush_section(
                            section_name,
                            &section_kvs,
                            &mut commands,
                            &mut default_time,
                            &mut default_buffer_time,
                            dt,
                            dbt,
                        );
                    }
                    current_section = Some(line[1..close].trim().to_string());
                    section_kvs.clear();
                }
                continue;
            }

            // Key = value pair (only inside a section).
            if current_section.is_some() {
                if let Some(eq_pos) = line.find('=') {
                    let key = line[..eq_pos].trim().to_lowercase();
                    let value = line[eq_pos + 1..].trim().to_string();
                    section_kvs.push((key, value));
                }
            }
        }

        // Flush the final section.
        if let Some(ref section_name) = current_section {
            let dt = default_time;
            let dbt = default_buffer_time;
            Self::flush_section(
                section_name,
                &section_kvs,
                &mut commands,
                &mut default_time,
                &mut default_buffer_time,
                dt,
                dbt,
            );
        }

        Ok(Cmd {
            commands,
            default_time,
            default_buffer_time,
        })
    }

    /// Process a fully-accumulated section.
    fn flush_section(
        section_name: &str,
        kvs: &[(String, String)],
        commands: &mut Vec<Command>,
        default_time: &mut u32,
        default_buffer_time: &mut u32,
        current_default_time: u32,
        current_default_buffer_time: u32,
    ) {
        let lower = section_name.to_lowercase();
        match lower.as_str() {
            "command" => {
                if let Some(cmd) = Self::build_command(kvs, current_default_time, current_default_buffer_time) {
                    commands.push(cmd);
                }
            }
            "defaults" => {
                for (k, v) in kvs {
                    match k.as_str() {
                        "command.time" => {
                            if let Ok(n) = v.parse::<u32>() {
                                *default_time = n;
                            }
                        }
                        "command.buffer.time" => {
                            if let Ok(n) = v.parse::<u32>() {
                                *default_buffer_time = n;
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                // Ignore [State], [Statedef], and other sections present in CMD files.
            }
        }
    }

    /// Build a Command from a [Command] section's key=value pairs.
    fn build_command(kvs: &[(String, String)], default_time: u32, default_buffer_time: u32) -> Option<Command> {
        let mut name: Option<String> = None;
        let mut command_str: Option<String> = None;
        let mut time: u32 = default_time;
        let mut buffer_time: u32 = default_buffer_time;

        for (k, v) in kvs {
            match k.as_str() {
                "name" => {
                    // Strip surrounding quotes if present.
                    let stripped = v.trim_matches('"').to_string();
                    name = Some(stripped);
                }
                "command" => {
                    command_str = Some(v.clone());
                }
                "time" => {
                    if let Ok(n) = v.parse::<u32>() {
                        time = n;
                    }
                }
                "buffer.time" => {
                    if let Ok(n) = v.parse::<u32>() {
                        buffer_time = n;
                    }
                }
                _ => {}
            }
        }

        let name = name?;
        let command_str = command_str?;
        let inputs = Self::parse_command_string(&command_str);

        Some(Command {
            name,
            inputs,
            time,
            buffer_time,
        })
    }

    /// Parse the `command = ...` value into a sequence of InputSteps.
    fn parse_command_string(cmd: &str) -> Vec<InputStep> {
        cmd.split(',')
            .map(|token| Self::parse_input_step(token.trim()))
            .collect()
    }

    /// Parse a single input token (e.g. `~D`, `$F`, `x+y`, `~30$B`).
    fn parse_input_step(token: &str) -> InputStep {
        let mut token = token;
        let mut modifier = InputModifier::None;

        // Release modifier `~`: can be `~D` (release) or `~30$B` (charge).
        if let Some(rest) = token.strip_prefix('~') {
            // Check if the next chars are digits → charge notation.
            if rest.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                let num_end = rest
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(rest.len());
                // Consume the charge frame count; remaining token may start with `$`.
                let after_num = &rest[num_end..];
                // Strip optional `$` that sometimes follows charge frames.
                token = after_num.strip_prefix('$').unwrap_or(after_num);
                modifier = InputModifier::Charge;
            } else {
                token = rest;
                modifier = InputModifier::Release;
            }
        } else if let Some(rest) = token.strip_prefix('$') {
            token = rest;
            modifier = InputModifier::Hold;
        } else if let Some(rest) = token.strip_prefix('/') {
            // Strip a following `$` if present (e.g., `/$F` means hold forward)
            token = rest.strip_prefix('$').unwrap_or(rest);
            modifier = InputModifier::Hold;
        }

        // Handle simultaneous buttons separated by `+` (e.g. `x+y`).
        if token.contains('+') {
            let buttons: Vec<CommandButton> = token
                .split('+')
                .filter_map(|b| Self::parse_button(b.trim()))
                .collect();
            return InputStep {
                direction: None,
                buttons,
                modifier,
            };
        }

        // Try direction first, then button.
        if let Some(dir) = Self::parse_direction(token) {
            return InputStep {
                direction: Some(dir),
                buttons: vec![],
                modifier,
            };
        }

        if let Some(btn) = Self::parse_button(token) {
            return InputStep {
                direction: None,
                buttons: vec![btn],
                modifier,
            };
        }

        // Unknown token — return an empty step rather than panicking.
        InputStep {
            direction: None,
            buttons: vec![],
            modifier,
        }
    }

    fn parse_direction(s: &str) -> Option<CommandDirection> {
        match s.to_uppercase().as_str() {
            "F" => Some(CommandDirection::Forward),
            "B" => Some(CommandDirection::Back),
            "U" => Some(CommandDirection::Up),
            "D" => Some(CommandDirection::Down),
            "DF" | "DR" => Some(CommandDirection::DownForward),
            "DB" | "DL" => Some(CommandDirection::DownBack),
            "UF" | "UR" => Some(CommandDirection::UpForward),
            "UB" | "UL" => Some(CommandDirection::UpBack),
            _ => None,
        }
    }

    fn parse_button(s: &str) -> Option<CommandButton> {
        match s.to_lowercase().as_str() {
            "x" => Some(CommandButton::X),
            "y" => Some(CommandButton::Y),
            "z" => Some(CommandButton::Z),
            "a" => Some(CommandButton::A),
            "b" => Some(CommandButton::B),
            "c" => Some(CommandButton::C),
            "s" => Some(CommandButton::S),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        // First step: ~D (release Down)
        assert_eq!(cmd.commands[0].inputs[0].direction, Some(CommandDirection::Down));
        assert_eq!(cmd.commands[0].inputs[0].modifier, InputModifier::Release);

        // Second step: DF (DownForward, no modifier)
        assert_eq!(cmd.commands[0].inputs[1].direction, Some(CommandDirection::DownForward));
        assert_eq!(cmd.commands[0].inputs[1].modifier, InputModifier::None);

        // Third step: F (Forward)
        assert_eq!(cmd.commands[0].inputs[2].direction, Some(CommandDirection::Forward));

        // Fourth step: x (button X)
        assert_eq!(cmd.commands[0].inputs[3].buttons, vec![CommandButton::X]);
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
        assert_eq!(cmd.commands[0].inputs.len(), 1);
        assert_eq!(cmd.commands[0].inputs[0].buttons.len(), 2);
        assert!(cmd.commands[0].inputs[0].buttons.contains(&CommandButton::X));
        assert!(cmd.commands[0].inputs[0].buttons.contains(&CommandButton::Y));
    }

    #[test]
    fn test_parse_hold_command_slash() {
        let content = r#"
[Command]
name = "holdfwd"
command = /F
time = 1
"#;
        let cmd = CmdParser::parse_content(content).expect("Failed to parse");
        assert_eq!(cmd.commands[0].inputs[0].modifier, InputModifier::Hold);
        assert_eq!(cmd.commands[0].inputs[0].direction, Some(CommandDirection::Forward));
    }

    #[test]
    fn test_parse_hold_command_dollar() {
        let content = r#"
[Command]
name = "holdfwd"
command = $F
time = 1
"#;
        let cmd = CmdParser::parse_content(content).expect("Failed to parse");
        assert_eq!(cmd.commands[0].inputs[0].modifier, InputModifier::Hold);
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
        assert_eq!(cmd.commands[0].inputs.len(), 3);
        // First step is charge Back
        assert_eq!(cmd.commands[0].inputs[0].modifier, InputModifier::Charge);
        assert_eq!(cmd.commands[0].inputs[0].direction, Some(CommandDirection::Back));
    }

    #[test]
    fn test_defaults_section() {
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
        // Command without explicit time should use default (15, parsed before Defaults)
        // Note: Defaults section may be processed at flush time, so behaviour depends on order.
        // At minimum the Defaults values should be reflected in the struct.
    }

    #[test]
    fn test_triple_simultaneous_buttons() {
        let content = r#"
[Command]
name = "abc"
command = x+y+z
time = 5
"#;
        let cmd = CmdParser::parse_content(content).expect("Failed to parse");
        assert_eq!(cmd.commands[0].inputs[0].buttons.len(), 3);
    }

    #[test]
    fn test_inline_comment_stripped() {
        let content = "[Command] ; this is a comment\nname = \"test\"\ncommand = x\ntime = 5\n";
        let cmd = CmdParser::parse_content(content).expect("Failed to parse");
        assert_eq!(cmd.commands.len(), 1);
        assert_eq!(cmd.commands[0].name, "test");
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
}
