// 静态清单的最小对照视图；只暴露 metadata 用例真正与运行时描述符比较的字段。
pub(in crate::tests::manifest) struct StaticSoundPluginMetadata {
    pub(in crate::tests::manifest) maturity: zircon_runtime::plugin::PluginMaturity,
    pub(in crate::tests::manifest) capability_statuses:
        Vec<zircon_runtime::plugin::CapabilityStatusManifest>,
}
