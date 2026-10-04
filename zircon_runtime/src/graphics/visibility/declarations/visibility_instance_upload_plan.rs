/// 从空间索引结果导出的静态、动态和脏动态实例集合，表达预期的 GPU 场景更新范围。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityInstanceUploadPlan {
    pub static_instance_keys: Vec<u64>,
    pub dynamic_instance_keys: Vec<u64>,
    pub dirty_dynamic_instance_keys: Vec<u64>,
}
