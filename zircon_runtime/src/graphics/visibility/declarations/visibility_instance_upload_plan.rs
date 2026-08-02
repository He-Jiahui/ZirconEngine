#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityInstanceUploadPlan {
    pub static_instance_keys: Vec<u64>,
    pub dynamic_instance_keys: Vec<u64>,
    pub dirty_dynamic_instance_keys: Vec<u64>,
}
