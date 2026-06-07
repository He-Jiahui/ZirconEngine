mod field;
mod state;
mod value;

pub(super) fn parse_module_kind_line(
    line: &str,
    kind: &mut Option<zircon_runtime::plugin::PluginModuleKind>,
) -> bool {
    let Some(value) = field::module_kind_value(line) else {
        return false;
    };
    state::set_module_kind(kind, value::module_kind_from_plugin_toml(value));
    true
}
