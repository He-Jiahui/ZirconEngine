#[path = "id.rs"]
mod id;
#[path = "index.rs"]
mod index;
#[path = "record.rs"]
mod record;
#[path = "signature.rs"]
mod signature;
#[path = "table/mod.rs"]
mod table;

pub use id::ArchetypeId;
pub use index::{
    ArchetypeIndex, ArchetypeIndexPerformanceStats,
    ECS_ARCHETYPE_COMPONENT_INDEX_PROBES_DIAGNOSTIC, ECS_ARCHETYPE_ROW_APPENDS_DIAGNOSTIC,
    ECS_ARCHETYPE_SIGNATURE_MEMBERSHIP_CHECKS_DIAGNOSTIC,
};
pub use record::ArchetypeRecord;
pub use signature::ArchetypeSignature;
pub(crate) use table::{
    ArchetypePreflightedRow, ArchetypeTable, ArchetypeTableError, ArchetypeTakenRow,
};
