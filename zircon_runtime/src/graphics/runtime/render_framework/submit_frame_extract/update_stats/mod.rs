//! 成功提交后从场景报告、provider 反馈及共享产品汇总视口统计，供编辑器和运行时诊断。
mod base_stats;
mod hybrid_gi_stats;
mod light_grid_stats;
mod particle_stats;
mod quality_profile;
mod shared_product_reports;
mod ui_stats;
mod update;
mod virtual_geometry_stats;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use shared_product_reports::SharedViewportProductReports;
pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use update::update_stats;
