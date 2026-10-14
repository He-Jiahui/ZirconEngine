use serde::{Deserialize, Serialize};

use crate::project::{ProjectGuid, ProjectTemplateReceipt};
use crate::serialization::Loaded;

use super::{ProjectManifestSummaryError, MAX_PROJECT_MANIFEST_BYTES};

/// Lightweight project identity consumed by Hub and other runtime-independent tools.
///
/// 解析入口产生的轻量项目身份快照，供 Hub 展示、Editor 预检和其它不依赖 Runtime 的消费者读取。
/// 该类型只携带摘要字段；直接构造或 serde 反序列化本身不会执行 parse_toml_* 的字段校验。
///
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectManifestSummary {
    /// 已通过共享项目名检查的显示/目录名称。
    pub name: String,
    #[serde(default)]
    /// 可选 semver 文本；语法有效不等于满足当前引擎版本。
    pub engine_version_req: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// 可选模板来源记录；解析时其身份与引擎要求会和清单字段交叉校验。
    pub template_receipt: Option<ProjectTemplateReceipt>,
    /// 清单中的非空场景引用；摘要解析不探测场景资源。
    pub default_scene: String,
    /// 迁移后的受支持清单结构版本。
    pub format_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// 持久化项目身份；旧格式迁移结果可暂缺，当前格式解析要求存在。
    pub project_guid: Option<ProjectGuid>,
}

impl ProjectManifestSummary {
    /// 解析并验证清单，且在 Loaded::migrated_from 中保留旧格式的来源供调用方显式处理。
    ///
    /// # 使用时机与生命周期
    /// 当调用方需要共享的展示或预检投影时使用；完整项目加载继续由 Runtime 负责。
    ///
    /// # 参数
    /// document 是调用方已经读入内存的 TOML；此入口按 UTF-8 字节数执行摘要清单限额。
    ///
    /// # 返回值
    /// 返回当前格式摘要以及 [`crate::serialization::Loaded`] 中记录的旧版来源；该来源应由准入调用方显式处理。
    ///
    /// # 约束
    /// 解析器不会从文件系统读取数据，也不验证场景、资产和 Runtime 专属字段；类型本身的 serde 解码不等同于此校验。
    ///
    /// # 参考
    /// 参考 [`crate::project::ProjectManifestSummary::parse_toml_bytes`]；Editor 路径为 zircon_editor/src/core/project/preflight_manifest_reader.rs。
    ///
    pub fn parse_toml_str(document: &str) -> Result<Loaded<Self>, ProjectManifestSummaryError> {
        super::parse::parse_str(document)
    }

    /// 从持久化字节生成相同摘要；大小与 UTF-8 检查先于 TOML 解析和迁移。
    ///
    /// # 使用时机与生命周期
    /// 接收已经取得的持久化字节，先检查字节上限和 UTF-8，再委托字符串入口执行同一迁移与摘要校验。
    ///
    /// # 参数
    /// document 是完整输入字节切片；恰好达到最大字节数允许。
    ///
    /// # 返回值
    /// 成功结果与 parse_toml_str 相同；超限或非法 UTF-8 时在 TOML 解析前返回对应错误。
    ///
    /// # 约束
    /// 字节片段已由调用方装入内存，此方法不负责有界文件读取。
    ///
    pub fn parse_toml_bytes(document: &[u8]) -> Result<Loaded<Self>, ProjectManifestSummaryError> {
        ensure_document_size(document.len())?;
        let document = std::str::from_utf8(document)
            .map_err(|source| ProjectManifestSummaryError::InvalidUtf8 { source })?;
        Self::parse_toml_str(document)
    }
}

/// 将原始文档长度与共享上限比较；等于上限时通过，大于上限时返回带 max/found 的错误。
///
pub(super) fn ensure_document_size(found: usize) -> Result<(), ProjectManifestSummaryError> {
    if found > MAX_PROJECT_MANIFEST_BYTES {
        return Err(ProjectManifestSummaryError::DocumentTooLarge {
            max: MAX_PROJECT_MANIFEST_BYTES,
            found,
        });
    }
    Ok(())
}
