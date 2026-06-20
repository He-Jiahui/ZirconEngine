use crate::scene::LevelMetadata;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionLevelRestoreReport {
    pub slot_id: String,
    pub metadata: LevelMetadata,
    pub entity_count: usize,
}
