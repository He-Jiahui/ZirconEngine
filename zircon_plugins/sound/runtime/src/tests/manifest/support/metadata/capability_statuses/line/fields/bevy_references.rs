// 保留能力状态引用的参考路径，使静态清单与运行时描述符比较的不只是状态枚举。
mod field;
mod state;
mod values;

use super::super::super::state::CapabilityStatusParserState;

pub(super) fn parse_bevy_references_field(line: &str, parser: &mut CapabilityStatusParserState) {
    let Some(value) = field::capability_status_bevy_references_value(line) else {
        return;
    };
    state::set_capability_status_bevy_references(
        parser,
        values::capability_status_bevy_references_from_plugin_toml(value),
    );
}
