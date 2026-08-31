use crate::core::framework::project::ExportPackagingStrategy;

pub(super) fn validate_runtime_plugin_default_packaging_strategy_uniqueness(
    owner: &str,
    strategy: ExportPackagingStrategy,
    seen: &mut u8,
    diagnostics: &mut Vec<String>,
) {
    let strategy_bit = match strategy {
        ExportPackagingStrategy::SourceTemplate => 0b001,
        ExportPackagingStrategy::LibraryEmbed => 0b010,
        ExportPackagingStrategy::NativeDynamic => 0b100,
    };
    if *seen & strategy_bit != 0 {
        diagnostics.push(format!(
            "runtime plugin {owner} default_packaging strategy {strategy:?} must be unique"
        ));
        return;
    }
    *seen |= strategy_bit;
}
