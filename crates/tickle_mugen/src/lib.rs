// MUGEN SFF/AIR parser and sprite atlas builder

mod error;
mod sff_v1;
mod air;
mod atlas;
mod act;
mod def;
mod cns;
mod trigger;
pub mod cmd;
pub mod snd;
pub mod command_recognizer;
pub mod mugen_controller;

pub use error::SffError;
pub use sff_v1::{SffV1, SpriteData};
pub use air::{Air, Action, Frame, FlipFlags, Clsn};
pub use atlas::{SpriteAtlas, SpriteInfo};
pub use act::{ActPalette, ActParser, Rgb};
pub use def::{CharacterDef, InfoSection, FilesSection};
pub use cns::{
    Cns, CnsParser, CnsData, CnsSize, CnsVelocity, CnsMovement,
    StateDef, StateTypeValue, MoveType, Physics, Vec2,
    StateController, Controller, merge_statedefs,
};
pub use trigger::{TriggerExpr, TriggerParser, TriggerContext, TriggerValue};
pub use cmd::{Cmd, CmdParser, Command, InputStep, InputModifier, CommandDirection, CommandButton};
pub use snd::{Snd, SndParser, SoundEntry};
pub use command_recognizer::MugenCommandRecognizer;
pub use mugen_controller::{
    mugen_controller_system, mugen_global_controller_system, mugen_tick, mugen_tick_with_p2,
    MugenFighterState, MugenSoundEvent, ActiveHitDef, MugenCollisionFighter, MugenHitResult,
    CombatFrameResult, mugen_hitdef_collision, mugen_combat_frame, hitdef_to_hitbox,
    populate_p2_context, reset_combo_if_recovered,
};

pub type Result<T> = std::result::Result<T, SffError>;
