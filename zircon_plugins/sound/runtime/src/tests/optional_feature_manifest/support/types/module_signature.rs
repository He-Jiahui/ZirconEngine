pub(in crate::tests::optional_feature_manifest::support) type OptionalFeatureModuleSignature = (
    String,
    zircon_runtime::plugin::PluginModuleKind,
    String,
    Vec<zircon_runtime::RuntimeTargetMode>,
    Vec<String>,
);
