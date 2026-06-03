pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn push_fatal_diagnostic(
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
    diagnostic: String,
) {
    diagnostics.push(diagnostic.clone());
    fatal_diagnostics.push(diagnostic);
}
