//! 从产品视角组织相机、可见性、材质、灯光及 UI 指标，串联画面产生链路。
#[path = "product/camera.rs"]
mod camera;
#[path = "../product/effect_stack.rs"]
mod effect_stack;
#[path = "../product/gpu_scene.rs"]
mod gpu_scene;
#[path = "product/hzb.rs"]
mod hzb;
#[path = "product/light.rs"]
mod light;
#[path = "../product/light_grid.rs"]
mod light_grid;
#[path = "../product/material.rs"]
mod material;
#[path = "../product/mesh_queue.rs"]
mod mesh_queue;
#[path = "../product/readback.rs"]
mod readback;
#[path = "product/sprite.rs"]
mod sprite;
#[path = "product/ui.rs"]
mod ui;
#[path = "../product/visibility.rs"]
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
#[path = "../product/tests/cases.rs"]
mod tests;
