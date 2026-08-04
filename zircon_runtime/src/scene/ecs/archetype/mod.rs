mod id;
mod index;
mod move_result;
mod record;
mod signature;
mod table;

pub use id::ArchetypeId;
pub use index::ArchetypeIndex;
pub use move_result::ArchetypeMove;
pub use record::ArchetypeRecord;
pub use signature::ArchetypeSignature;
pub(crate) use table::{ArchetypeTable, ArchetypeTableError, ArchetypeTakenRow};
