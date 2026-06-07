mod field;
mod modes;
mod state;

pub(super) fn parse_module_target_modes_line(
    line: &str,
    target_modes: &mut Vec<zircon_runtime::RuntimeTargetMode>,
) -> bool {
    let Some(value) = field::module_target_modes_value(line) else {
        return false;
    };
    state::set_module_target_modes(
        target_modes,
        modes::module_target_modes_from_plugin_toml(value),
    );
    true
}
