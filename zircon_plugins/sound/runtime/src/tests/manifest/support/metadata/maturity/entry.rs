// 固定清单当前只有根级 maturity；取首个声明与运行时描述符比较，缺失或未知值让测试失败。
use super::{field, required, value};

pub(super) fn static_maturity_from_plugin_toml(
    manifest: &str,
) -> zircon_runtime::plugin::PluginMaturity {
    for line in manifest.lines().map(str::trim) {
        let Some(value) = field::maturity_value(line) else {
            continue;
        };
        return value::maturity_from_static_plugin_toml(value);
    }
    required::missing_static_maturity()
}
