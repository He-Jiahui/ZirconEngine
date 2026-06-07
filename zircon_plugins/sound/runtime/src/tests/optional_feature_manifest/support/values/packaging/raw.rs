pub(super) fn packaging_strategy_from_plugin_toml(
    value: String,
) -> zircon_runtime::plugin::ExportPackagingStrategy {
    match value.as_str() {
        "source_template" => zircon_runtime::plugin::ExportPackagingStrategy::SourceTemplate,
        "library_embed" => zircon_runtime::plugin::ExportPackagingStrategy::LibraryEmbed,
        "native_dynamic" => zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic,
        _ => panic!("unknown sound packaging strategy {value}"),
    }
}
