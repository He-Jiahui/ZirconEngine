//! Editor command registry, keymap, and command-palette projection.

mod context;
mod descriptor;
mod key_chord;
mod keymap;
mod palette;
mod registry;

pub use context::{EditorCommandContext, EditorCommandEnablement};
pub use descriptor::{EditorCommandAction, EditorCommandCategory, EditorCommandDescriptor};
pub use key_chord::{EditorKeyChord, EditorKeyChordParseError};
pub use keymap::{EditorKeyBinding, EditorKeymap, EditorKeymapError};
pub use palette::EditorCommandPaletteEntry;
pub use registry::{EditorCommandDispatchError, EditorCommandRegistry};
