use tickle_mugen::CharacterDef;
use std::path::PathBuf;

fn get_test_file_path() -> PathBuf {
    // Tests run from crate directory, need to go up to workspace root
    PathBuf::from("../../mugen_resources/KyoKusanagi[SuzukiInoue]/KyoKusanagi[SuzukiInoue].def")
}

#[test]
fn test_def_parser() {
    let path = get_test_file_path();

    // Skip test if file doesn't exist (e.g., in CI without resources)
    if !path.exists() {
        eprintln!("Skipping test: {} not found", path.display());
        return;
    }

    let def = CharacterDef::parse(&path)
        .expect("Failed to parse DEF file");

    // Test Info section
    assert_eq!(def.info.name, "Kyo Kusanagi");
    assert_eq!(def.info.displayname, "Kyo Kusanagi");
    assert_eq!(def.info.author, Some("Suzuki Inoue".to_string()));
    assert_eq!(def.info.pal_defaults, vec![1, 2, 3, 4, 5, 6]);

    // Test Files section
    assert_eq!(def.files.sff, "kyo.sff");
    assert_eq!(def.files.air, "kyo.air");
    assert_eq!(def.files.cns, "kyo.cns");
    assert_eq!(def.files.cmd, "kyo.cmd");
    assert_eq!(def.files.snd, "kyo.snd");

    // Test optional files
    assert_eq!(def.files.st, Some("kyo.cns".to_string()));
    assert_eq!(def.files.stcommon, Some("common1.cns".to_string()));

    // Test palette files
    assert_eq!(def.files.pal1, Some("kyo01.act".to_string()));
    assert_eq!(def.files.pal2, Some("kyo02.act".to_string()));
    assert_eq!(def.files.pal3, Some("kyo03.act".to_string()));
    assert_eq!(def.files.pal4, Some("kyo04.act".to_string()));
    assert_eq!(def.files.pal5, Some("kyo05.act".to_string()));
    assert_eq!(def.files.pal6, Some("kyo06.act".to_string()));
}

#[test]
fn test_def_parser_minimal() {
    let content = r#"
[Info]
Name="Test Character"

[Files]
cmd=test.cmd
cns=test.cns
sprite=test.sff
anim=test.air
sound=test.snd
"#;

    let def = CharacterDef::parse_content(content).expect("Failed to parse minimal DEF");

    assert_eq!(def.info.name, "Test Character");
    assert_eq!(def.info.displayname, "Test Character");
    assert_eq!(def.files.cmd, "test.cmd");
    assert_eq!(def.files.cns, "test.cns");
    assert_eq!(def.files.sff, "test.sff");
    assert_eq!(def.files.air, "test.air");
    assert_eq!(def.files.snd, "test.snd");
}
