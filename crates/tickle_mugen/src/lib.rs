// MUGEN SFF (Sprite File Format) parser
// Supports SFF v1 (PCX-based) format

mod error;
mod sff_v1;
mod air;

pub use error::SffError;
pub use sff_v1::{SffV1, SpriteData};
pub use air::{Air, Action, Frame, FlipFlags, Clsn};

pub type Result<T> = std::result::Result<T, SffError>;
