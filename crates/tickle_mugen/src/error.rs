use thiserror::Error;

#[derive(Debug, Error)]
pub enum SffError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid SFF signature: expected 'ElecbyteSpr', got {0:?}")]
    InvalidSignature([u8; 12]),

    #[error("Unsupported SFF version: {0}.{1}")]
    UnsupportedVersion(u8, u8),

    #[error("Sprite not found: group={0}, image={1}")]
    SpriteNotFound(u16, u16),

    #[error("Invalid PCX data at offset {0}")]
    InvalidPcx(u64),

    #[error("Palette not found for sprite group={0}, image={1}")]
    PaletteNotFound(u16, u16),

    #[error("DEF parse error: {0}")]
    DefParse(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}
