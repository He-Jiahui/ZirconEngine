//! 为神经模型准备 GPU compute pass 描述、张量布局和权重资源计划。
//! 此模块只生成规划数据，不编译 shader 或提交 GPU 工作。

#[path = "graph_executor.rs"]
mod graph_executor;
#[path = "shader_templates.rs"]
mod shader_templates;
#[path = "tensor_layout.rs"]
mod tensor_layout;
#[path = "weight_upload.rs"]
mod weight_upload;

pub use graph_executor::{NnGraphBuildError, NnGraphExecutor, NnGraphIo, NnGraphPassPlan};
pub use tensor_layout::{NnTensorLayout, NnTensorLayoutError};
pub use weight_upload::{NnWeightUploadPlan, NnWeightUploadPlanError};
