use crate::project::RelPath;

/// 模板渲染后交给 Editor 项目创建事务写入暂存目录的单个文件。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedProjectTemplateEntry {
    /// 只保证词法上的项目相对路径；落盘调用方仍须保证目标位于暂存根内。
    pub path: RelPath,
    pub bytes: Vec<u8>,
}
