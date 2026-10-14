use crate::project::ProjectManifestSummary;

use super::{ProjectTemplateDescriptor, RenderedProjectTemplateEntry};

/// Fully rendered, filesystem-independent template payload consumed by Editor and Hub.
///
/// 持有完成身份改写与清单校验后的模板描述符、清单摘要和文件项，供上层项目创建流程消费。
///
/// # 生命周期
/// 所有文件字节都由此值拥有；它不借用嵌入资源或源目录。交给 Editor 落盘时仍须把相对路径限制在目标 staging 根内。
#[derive(Clone, Debug, PartialEq, Eq)]
/// 渲染结果在离开接口层后由 Editor 创建事务逐项写入并再次预检。
pub struct RenderedProjectTemplate {
    /// 原始嵌入包的规范版本、摘要和目标准入要求。
    pub descriptor: ProjectTemplateDescriptor,
    /// 身份字段已改写且已通过共享摘要解析的清单视图。
    pub summary: ProjectManifestSummary,
    /// 经过 RelPath 校验、可直接写入 staging 目录的文件项。
    pub entries: Vec<RenderedProjectTemplateEntry>,
}
