use crate::{Result, SffError};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Character definition file parser
#[derive(Debug, Clone)]
pub struct CharacterDef {
    pub info: InfoSection,
    pub files: FilesSection,
}

#[derive(Debug, Clone)]
pub struct InfoSection {
    pub name: String,
    pub displayname: String,
    pub versiondate: Option<String>,
    pub mugen_version: Option<String>,
    pub author: Option<String>,
    pub pal_defaults: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct FilesSection {
    pub cmd: String,
    pub cns: String,
    pub st: Option<String>,
    pub stcommon: Option<String>,
    pub sff: String,  // sprite file
    pub air: String,  // animation file
    pub snd: String,  // sound file
    pub pal1: Option<String>,
    pub pal2: Option<String>,
    pub pal3: Option<String>,
    pub pal4: Option<String>,
    pub pal5: Option<String>,
    pub pal6: Option<String>,
}

impl CharacterDef {
    /// Parse a .def file from the given path
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())?;
        Self::parse_content(&content)
    }

    /// Parse DEF content from a string
    pub fn parse_content(content: &str) -> Result<Self> {
        let sections = parse_ini_sections(content);

        let info = parse_info_section(sections.get("Info"))?;
        let files = parse_files_section(sections.get("Files"))?;

        Ok(CharacterDef { info, files })
    }
}

/// Parse INI-style sections into a map
fn parse_ini_sections(content: &str) -> HashMap<String, HashMap<String, String>> {
    let mut sections: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current_section: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with(';') {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            let section_name = line[1..line.len() - 1].to_string();
            current_section = Some(section_name.clone());
            sections.entry(section_name).or_insert_with(HashMap::new);
            continue;
        }

        // Key-value pair
        if let Some(section_name) = &current_section {
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_lowercase();
                let value = line[pos + 1..].trim();

                // Remove quotes if present
                let value = if value.starts_with('"') && value.ends_with('"') {
                    &value[1..value.len() - 1]
                } else {
                    value
                };

                if let Some(section) = sections.get_mut(section_name) {
                    section.insert(key, value.to_string());
                }
            }
        }
    }

    sections
}

fn parse_info_section(section: Option<&HashMap<String, String>>) -> Result<InfoSection> {
    let section = section.ok_or_else(|| SffError::MissingField("Info section".to_string()))?;

    let name = section
        .get("name")
        .ok_or_else(|| SffError::MissingField("Info.name".to_string()))?
        .clone();

    let displayname = section
        .get("displayname")
        .cloned()
        .unwrap_or_else(|| name.clone());

    let versiondate = section.get("versiondate").cloned();
    let mugen_version = section.get("mugenversion").cloned();
    let author = section.get("author").cloned();

    // Parse pal.defaults (e.g., "1,2,3,4,5,6")
    let pal_defaults = section
        .get("pal.defaults")
        .map(|s| {
            s.split(',')
                .filter_map(|n| n.trim().parse::<u8>().ok())
                .collect()
        })
        .unwrap_or_else(|| vec![1, 2, 3, 4, 5, 6]);

    Ok(InfoSection {
        name,
        displayname,
        versiondate,
        mugen_version,
        author,
        pal_defaults,
    })
}

fn parse_files_section(section: Option<&HashMap<String, String>>) -> Result<FilesSection> {
    let section = section.ok_or_else(|| SffError::MissingField("Files section".to_string()))?;

    let cmd = section
        .get("cmd")
        .ok_or_else(|| SffError::MissingField("Files.cmd".to_string()))?
        .clone();

    let cns = section
        .get("cns")
        .ok_or_else(|| SffError::MissingField("Files.cns".to_string()))?
        .clone();

    let sff = section
        .get("sprite")
        .ok_or_else(|| SffError::MissingField("Files.sprite".to_string()))?
        .clone();

    let air = section
        .get("anim")
        .ok_or_else(|| SffError::MissingField("Files.anim".to_string()))?
        .clone();

    let snd = section
        .get("sound")
        .ok_or_else(|| SffError::MissingField("Files.sound".to_string()))?
        .clone();

    Ok(FilesSection {
        cmd,
        cns,
        st: section.get("st").cloned(),
        stcommon: section.get("stcommon").cloned(),
        sff,
        air,
        snd,
        pal1: section.get("pal1").cloned(),
        pal2: section.get("pal2").cloned(),
        pal3: section.get("pal3").cloned(),
        pal4: section.get("pal4").cloned(),
        pal5: section.get("pal5").cloned(),
        pal6: section.get("pal6").cloned(),
    })
}
