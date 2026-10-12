//! 图形资源和渲染后端的死代码清理守卫入口；子模块将生产源码与迁移记录对照。
#[path = "../../../structure/graphics_dead_code/backend_owners.rs"]
mod backend_owners;
#[path = "../../../structure/graphics_dead_code/gpu_resource_owners.rs"]
mod gpu_resource_owners;
#[path = "module_layout.rs"]
mod module_layout;
#[path = "../../../graphics_dead_code/renderer_output_accessors.rs"]
mod renderer_output_accessors;
#[path = "../../../graphics_dead_code/resource_streamer_cleanup.rs"]
mod resource_streamer_cleanup;

use super::{repo_path, runtime_src_path};

pub(super) const DEAD_CODE_ALLOW_ATTRIBUTE: &str = concat!("#[allow(", "dead_code", ")]");

fn read_runtime_src(relative: &str) -> String {
    let path = runtime_src_path(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("runtime source should exist at {}: {error}", path.display())
    })
}

fn read_repo(relative: &str) -> String {
    let path = repo_path(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("repo source should exist at {}: {error}", path.display()))
}
