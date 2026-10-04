//! 计划中的生成工件同时承载路径、用途和完整内容；验证报告可只公开摘要，也可单独输出内容工件。
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 尚未落盘的文本工件；path 相对导出根目录，消费者须经过 materialize 路径校验。
pub struct ExportGeneratedFile {
    pub path: String,
    pub purpose: String,
    pub contents: String,
}
