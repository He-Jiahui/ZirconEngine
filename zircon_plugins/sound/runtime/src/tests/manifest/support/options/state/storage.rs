// 暂存一个静态选项行；只有拿到 key 后才会形成运行时类型，必填元数据由完成阶段检查。
#[derive(Default)]
pub(in super::super) struct PendingOptionManifest {
    pub(in super::super) key: Option<String>,
    pub(in super::super) display_name: Option<String>,
    pub(in super::super) value_type: Option<String>,
    pub(in super::super) default_value: Option<String>,
    pub(in super::super) enum_values: Vec<String>,
    pub(in super::super) required_capability: Option<String>,
}
