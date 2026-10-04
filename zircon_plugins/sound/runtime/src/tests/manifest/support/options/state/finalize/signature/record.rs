// 测试中间值聚合选项的四个必填字段，最终构造 PluginOptionManifest 参与对照。
pub(in super::super) struct OptionManifestSignature {
    pub(in super::super) key: String,
    pub(in super::super) display_name: String,
    pub(in super::super) value_type: String,
    pub(in super::super) default_value: String,
}
