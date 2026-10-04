//! 为神经模型准备 GPU compute pass 描述、张量布局和权重资源计划。
//! 此模块只生成规划数据，不编译 shader 或提交 GPU 工作。

mod graph_executor;
mod shader_templates;
mod tensor_layout;
mod weight_upload;

pub use graph_executor::{NnGraphBuildError, NnGraphExecutor, NnGraphIo, NnGraphPassPlan};
pub use tensor_layout::{NnTensorLayout, NnTensorLayoutError};
pub use weight_upload::{NnWeightUploadPlan, NnWeightUploadPlanError};
