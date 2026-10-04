//! 定义学习资源打开动作的准入失败和恢复建议；动作端保存语义编号与位置参数，显示端再选择语言。
//! 稳定编号同时用于持久化恢复；新增条目需同步枚举全集、参数契约及两种语言模板。

use crate::settings::HubLanguage;

/// 学习资源打开动作的准入失败和恢复建议的语义身份；调用端按条目约定传入原始路径或错误参数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LearnMessageId {
    ChooseResource,
    OpenResourceTargetRequired,
    ResourceNotInCatalog,
    RefreshOrChooseLocalDocument,
    RefreshAndChooseLocalDocument,
    ResourceFileDoesNotExist,
    ResourcePathMustBeAbsolute,
}

impl LearnMessageId {
    /// 供持久化编号解析和双语模板全集检查使用；新增枚举成员须纳入此表。
    pub const ALL: &'static [Self] = &[
        Self::ChooseResource,
        Self::OpenResourceTargetRequired,
        Self::ResourceNotInCatalog,
        Self::RefreshOrChooseLocalDocument,
        Self::RefreshAndChooseLocalDocument,
        Self::ResourceFileDoesNotExist,
        Self::ResourcePathMustBeAbsolute,
    ];

    // 已保存记录以这些编号恢复，文案调整不应顺带重命名编号。
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::ChooseResource => "learn.choose-resource",
            Self::OpenResourceTargetRequired => "learn.open-resource-target-required",
            Self::ResourceNotInCatalog => "learn.resource-not-in-catalog",
            Self::RefreshOrChooseLocalDocument => "learn.refresh-or-choose-local-document",
            Self::RefreshAndChooseLocalDocument => "learn.refresh-and-choose-local-document",
            Self::ResourceFileDoesNotExist => "learn.resource-file-does-not-exist",
            Self::ResourcePathMustBeAbsolute => "learn.resource-path-must-be-absolute",
        }
    }

    // 这是模板的参数契约元数据，实际消息构造不会在这里校验参数数量。
    pub(super) fn param_count(self) -> usize {
        match self {
            Self::ResourceFileDoesNotExist | Self::ResourcePathMustBeAbsolute => 1,
            _ => 0,
        }
    }

    // 位置参数保持原始用户数据；语言投影负责周围的语句和标点。
    pub(super) fn template(self, language: HubLanguage) -> &'static str {
        match (language, self) {
            (HubLanguage::English, Self::ChooseResource) => {
                "Choose a resource from the current Learn catalog"
            }
            (HubLanguage::Chinese, Self::ChooseResource) => "从当前学习目录中选择资源",
            (HubLanguage::English, Self::OpenResourceTargetRequired) => {
                "Open Resource target is required"
            }
            (HubLanguage::Chinese, Self::OpenResourceTargetRequired) => "需要打开资源目标",
            (HubLanguage::English, Self::ResourceNotInCatalog) => {
                "Resource is not present in the current Learn catalog"
            }
            (HubLanguage::Chinese, Self::ResourceNotInCatalog) => "资源不在当前学习目录中",
            (HubLanguage::English, Self::RefreshOrChooseLocalDocument) => {
                "Refresh the Learn catalog or choose an existing local document"
            }
            (HubLanguage::Chinese, Self::RefreshOrChooseLocalDocument) => {
                "刷新学习目录或选择已有本地文档"
            }
            (HubLanguage::English, Self::RefreshAndChooseLocalDocument) => {
                "Refresh the Learn catalog and choose an existing local document"
            }
            (HubLanguage::Chinese, Self::RefreshAndChooseLocalDocument) => {
                "刷新学习目录并选择已有本地文档"
            }
            (HubLanguage::English, Self::ResourceFileDoesNotExist) => {
                "Resource file does not exist: {0}"
            }
            (HubLanguage::Chinese, Self::ResourceFileDoesNotExist) => "资源文件不存在：{0}",
            (HubLanguage::English, Self::ResourcePathMustBeAbsolute) => {
                "Resource path must be an absolute path: {0}"
            }
            (HubLanguage::Chinese, Self::ResourcePathMustBeAbsolute) => {
                "资源路径必须是绝对路径：{0}"
            }
        }
    }
}
