mod degrade_ladder;
mod memory_budget;

pub(in crate::graphics::runtime::render_framework) use degrade_ladder::{
    BudgetDegradeLadder, BudgetDegradeSettings,
};
pub(in crate::graphics::runtime::render_framework) use memory_budget::{
    is_memory_over_budget, memory_budget_warning_count,
};
pub(in crate::graphics::runtime::render_framework) use zr_rhi::GpuMemoryBudget;
