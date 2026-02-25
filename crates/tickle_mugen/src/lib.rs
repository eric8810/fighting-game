// MUGEN SFF/AIR parser and sprite atlas builder

mod error;
mod sff_v1;
mod air;
mod atlas;

pub use error::SffError;
pub use sff_v1::{SffV1, SpriteData};
pub use air::{Air, Action, Frame, FlipFlags, Clsn};
pub use atlas::{SpriteAtlas, SpriteInfo};

pub type Result<T> = std::result::Result<T, SffError>;
