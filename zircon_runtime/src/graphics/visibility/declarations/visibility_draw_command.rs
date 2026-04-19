use super::visibility_batch_key::VisibilityBatchKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisibilityDrawCommand {
    pub key: VisibilityBatchKey,
    pub visible_instance_offset: u32,
    pub visible_instance_count: u32,
}
