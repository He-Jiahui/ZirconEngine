//! 成功提交后从场景报告、provider 反馈及共享产品汇总视口统计，供编辑器和运行时诊断。
#[path = "base_stats.rs"]
mod base_stats;
#[path = "../../../extract/update_stats/hybrid_gi_stats.rs"]
mod hybrid_gi_stats;
#[path = "../../../extract/update_stats/light_grid_stats.rs"]
mod light_grid_stats;
#[path = "../../../extract/update_stats/particle_stats.rs"]
mod particle_stats;
#[path = "../../../extract/update_stats/quality_profile.rs"]
mod quality_profile;
#[path = "../../../../extract/update_stats/shared_product_reports.rs"]
mod shared_product_reports;
#[path = "ui_stats.rs"]
mod ui_stats;
#[path = "update.rs"]
mod update;
#[path = "../../../../extract/update_stats/virtual_geometry_stats.rs"]
mod virtual_geometry_stats;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use shared_product_reports::SharedViewportProductReports;
pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use update::update_stats;
