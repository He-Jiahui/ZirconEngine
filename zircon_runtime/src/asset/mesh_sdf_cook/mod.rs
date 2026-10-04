//! 导入时按显式请求烘焙网格 SDF；预算超限可降级为无派生数据，其余错误中止导入。

mod acceleration;
mod budget;
mod cook;
mod distance;
mod error;
mod request;
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub use budget::MeshSdfCookBudget;
pub use cook::{
    cook_mesh_sdf_from_mesh, cook_mesh_sdf_from_mesh_with_budget,
    cook_mesh_sdf_from_mesh_with_budget_and_executor, cook_mesh_sdf_from_mesh_with_executor,
    cook_mesh_sdf_or_fallback, cook_mesh_sdf_or_fallback_single,
};
pub use error::MeshSdfCookError;
pub use request::MeshSdfCookRequest;
