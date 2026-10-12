// 保存当前选项与完成列表；状态仅用于静态清单测试投影，不参与运行时注册。
use super::super::super::state::PendingOptionManifest;

// Preserves the static plugin.toml scanner's table-boundary behavior for option rows.
#[derive(Default)]
pub(in super::super) struct OptionManifestParserState {
    pub(in super::super) options: Vec<zircon_runtime::plugin::PluginOptionManifest>,
    pub(in super::super) pending: PendingOptionManifest,
    pub(in super::super) inside_option: bool,
}
