pub(in super::super) fn packaging_strategy_from_plugin_toml(
    value: String,
) -> zircon_runtime::core::framework::project::ExportPackagingStrategy {
    match value.as_str() {
        "source_template" => {
            zircon_runtime::core::framework::project::ExportPackagingStrategy::SourceTemplate
        }
        "library_embed" => {
            zircon_runtime::core::framework::project::ExportPackagingStrategy::LibraryEmbed
        }
        "native_dynamic" => {
            zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
        }
        _ => panic!("unknown sound packaging strategy {value}"),
    }
}
