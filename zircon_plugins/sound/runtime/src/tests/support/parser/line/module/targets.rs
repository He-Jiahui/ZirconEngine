// 模块字段仅在所属区段汇入当前待提交记录，避免静态对照把跨表同名字段视为同一声明。
#[path = "../../../../line/module/targets/support_field.rs"]
mod field;
#[path = "../../../../line/module/targets/support_modes.rs"]
mod modes;
#[path = "../../../../line/module/targets/support_state.rs"]
mod state;

pub(super) fn parse_module_target_modes_line(
    line: &str,
    target_modes: &mut Vec<zircon_runtime::core::framework::platform::RuntimeTargetMode>,
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
