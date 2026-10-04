//! 从产品视角组织相机、可见性、材质、灯光及 UI 指标，串联画面产生链路。
mod camera;
mod effect_stack;
mod gpu_scene;
mod hzb;
mod light;
mod light_grid;
mod material;
mod mesh_queue;
mod readback;
mod sprite;
mod ui;
mod visibility;

use crate::core::framework::render::RenderStats;

use super::{record_bool, record_bytes, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    camera::record(store, stats);
    visibility::record(store, stats);
    hzb::record(store, stats);
    light_grid::record(store, stats);
    material::record(store, stats);
    light::record(store, stats);
    mesh_queue::record(store, stats);
    readback::record(store, stats);
    gpu_scene::record(store, stats);
    sprite::record(store, stats);
    effect_stack::record(store, stats);
    ui::record(store, stats);
}

#[cfg(test)]
#[path = "product/tests/cases.rs"]
mod tests;
