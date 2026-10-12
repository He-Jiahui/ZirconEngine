//! 承载原生插件发现结果，供增量发现、报表投影和后续加载共享同一份包声明。
//! 候选保留的是发现时的元数据；实际执行仍需由宿主授权并验证当前文件代际。

use std::path::PathBuf;

use crate::plugin::PluginPackageManifest;

/// 供加载前筛选使用的包发现快照，也会保留在最终报表和热更新计划中。
///
/// 身份与路径字段可公开构造，不能代替执行许可；加载阶段会重新核对包声明与
/// 宿主授权，并保留通过校验的暂存镜像。包声明在发现后变化时，需要重新发现。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginCandidate {
    /// 用于选择、去重和诊断的发现身份，必须与包声明中的身份一致。
    pub plugin_id: String,
    /// 发现时解析的完整声明，供目标筛选、能力核对及报表投影使用。
    pub package_manifest: PluginPackageManifest,
    /// 发现索引的路径键，同时确定包资源根目录与加载阶段重新验证的文件。
    pub manifest_path: PathBuf,
    /// 发现时选出的代表库路径；加载会按请求的模块类别重新投影库集合。
    pub library_path: PathBuf,
}
