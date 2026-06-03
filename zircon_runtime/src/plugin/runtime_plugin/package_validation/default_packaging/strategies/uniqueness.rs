use crate::plugin::ExportPackagingStrategy;

pub(super) fn validate_runtime_plugin_default_packaging_strategy_uniqueness(
    owner: &str,
    strategy: ExportPackagingStrategy,
    seen: &mut Vec<ExportPackagingStrategy>,
    diagnostics: &mut Vec<String>,
) {
    if seen.contains(&strategy) {
        diagnostics.push(format!(
            "runtime plugin {owner} default_packaging strategy {strategy:?} must be unique"
        ));
        return;
    }
    seen.push(strategy);
}
