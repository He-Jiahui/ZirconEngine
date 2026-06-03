mod charset;
mod start;
mod underscore;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_id(
    context: &str,
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    charset::validate_runtime_plugin_package_id_charset(context, field_name, value, diagnostics);
    start::validate_runtime_plugin_package_id_start(context, field_name, value, diagnostics);
    underscore::validate_runtime_plugin_package_id_underscore(
        context,
        field_name,
        value,
        diagnostics,
    );
}
