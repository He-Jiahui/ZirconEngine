//! 保留构建输出路径的语义编号和双语模板，供总消息编号解析和全集契约检查。
//! 当前源码中尚未找到该编号的动作生产者；若为归档兼容，应明确保留它的边界。

use crate::settings::HubLanguage;

// TODO: [CR-HUBSTATE-0007] 确认输出路径编号仅为归档兼容还是遗漏了动作生产者；当前调用只见路由和全集测试，需追查构建输出状态约定。
/// 构建状态中的输出路径消息的语义身份；调用端按条目约定传入原始路径或错误参数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildMessageId {
    BuildOutputPath,
}

impl BuildMessageId {
    /// 供持久化编号解析和双语模板全集检查使用；新增枚举成员须纳入此表。
    pub const ALL: &'static [Self] = &[Self::BuildOutputPath];

    // 已保存记录以这些编号恢复，文案调整不应顺带重命名编号。
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::BuildOutputPath => "build.output-path",
        }
    }

    // 这是模板的参数契约元数据，实际消息构造不会在这里校验参数数量。
    pub(super) fn param_count(self) -> usize {
        match self {
            Self::BuildOutputPath => 1,
        }
    }

    // 位置参数保持原始用户数据；语言投影负责周围的语句和标点。
    pub(super) fn template(self, language: HubLanguage) -> &'static str {
        match (language, self) {
            (HubLanguage::English, Self::BuildOutputPath) => "{0}",
            (HubLanguage::Chinese, Self::BuildOutputPath) => "{0}",
        }
    }
}
