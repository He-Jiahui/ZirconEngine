//! 此模块统一挂载运行时吸收测试，子模块各自拥有具体断言。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "asset_pipeline.rs"]
mod asset_pipeline;
#[path = "asset_surface.rs"]
mod asset_surface;
#[path = "asset_worker_policy.rs"]
mod asset_worker_policy;
#[path = "builtin_modules.rs"]
mod builtin_modules;
#[path = "review.rs"]
mod code_review_findings;
#[path = "compatibility_shells.rs"]
mod compatibility_shells;
#[path = "core_spine_root_generated.rs"]
mod core_spine_root_generated;
#[path = "current_source_fixture.rs"]
mod current_source_fixture;
#[path = "dynamic_api_session.rs"]
mod dynamic_api_session;
#[path = "dynamic.rs"]
mod dynamic_scene;
#[path = "ecs_kernel_data.rs"]
mod ecs_kernel_data;
#[path = "generated_code_guard.rs"]
mod generated_code_guard;
#[path = "input_stack.rs"]
mod input_stack;
#[path = "job_system.rs"]
mod job_system;
#[path = "naming_boundary.rs"]
mod naming_boundary;
#[path = "performance.rs"]
mod performance_hotspots;
#[path = "plugin_surface_lifecycle.rs"]
mod plugin_surface_lifecycle;
#[path = "rayon_boundary.rs"]
mod rayon_boundary;
#[path = "resources.rs"]
mod resource_foundation;
#[path = "root_entries.rs"]
mod root_entries;
#[path = "root_surface.rs"]
mod root_surface;
#[path = "schedule_frame_loop.rs"]
mod schedule_frame_loop;
#[path = "script_absorption.rs"]
mod script_absorption;
#[path = "script_binding.rs"]
mod script_binding;
#[cfg(feature = "script")]
#[path = "script_host_ledger.rs"]
mod script_host_ledger;
#[path = "service_registry_lifecycle.rs"]
mod service_registry_lifecycle;
#[path = "service_registry_ownership.rs"]
mod service_registry_ownership;
#[path = "structure.rs"]
mod structure_convention;
#[path = "tech_stack.rs"]
mod tech_stack;
#[path = "ui_architecture.rs"]
mod ui_architecture;
