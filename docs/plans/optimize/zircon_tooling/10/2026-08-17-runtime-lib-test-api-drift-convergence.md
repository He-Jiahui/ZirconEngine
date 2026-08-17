---
record_kind: milestone
status: completed
created_at: 2026-08-17
plan: docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
milestone: M2
---

Plan: docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
Milestone: M2
Status: completed
Files: ["zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs", "zircon_runtime/src/core/framework/render/mod.rs", "zircon_runtime/src/core/framework/render/post_process/pass_graph.rs", "zircon_runtime/src/core/framework/render/view_family.rs", "zircon_runtime/src/core/runtime/handle/core_handle.rs", "zircon_runtime/src/core/runtime/tests/activation/behavior/activation.rs", "zircon_runtime/src/dynamic_api/session/tests/foundation_render.rs", "zircon_runtime/src/dynamic_api/session/tests/frame_demand.rs", "zircon_runtime/src/dynamic_api/session/tests/runtime_ui_surface.rs", "zircon_runtime/src/dynamic_api/session/tests/vampire_gameplay.rs", "zircon_runtime/src/dynamic_api/session/tests/vampire_hud.rs", "zircon_runtime/src/foundation/runtime/config_path.rs", "zircon_runtime/src/foundation/runtime/mod.rs", "zircon_runtime/src/foundation/tests.rs", "zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/mod.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs", "zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/generic_compute_executor.rs", "zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler/tests.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/draw_plan.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests/prepare_report.rs", "zircon_runtime/src/graphics/shader/template/tests/environment/provider_guards.rs", "zircon_runtime/src/graphics/tests/render_product_post_process_full_chain/visual_export.rs", "zircon_runtime/src/script/vm/host/host_export_registry.rs", "zircon_runtime/src/text/native_bitmap_atlas/tests.rs", "zircon_runtime/src/text/native_bitmap_atlas/tests/frame.rs", "zircon_runtime/src/text/native_bitmap_atlas/tests/source_cache.rs", "zircon_runtime/src/text/native_bitmap_atlas/tests/storage.rs", "zircon_runtime/src/text/shaping/tests.rs", "zircon_runtime/src/ui/tests/asset/loader_validation.rs", "zircon_runtime/src/ui/tests/text_pipeline/render_extract_prewarm.rs", "zircon_runtime/src/ui/text/layout_engine/tests/performance.rs", "docs/plans/optimize/zircon_tooling/10/fixed-2026-08-17-environment-only-forward-retains-generic-provider-bindings.md"]

# M2 runtime lib-test API drift convergence

## Scope Delivered

- 将 `zircon_runtime` lib-test 基线从 326 个编译错误收敛到 0；测试调用面重新对齐当前 render graph、runtime UI、HZB、text atlas、shader template、script host 与动态 runtime API。
- 测试配置路径改为线程局部、作用域化覆盖；默认测试路径按进程、时间与原子序号隔离，替代进程级环境变量变更，并使 override guard 保持 `!Send`，避免并行测试互相污染或死锁。
- IBL journal 仅对策略比较使用规范化目录，恢复层继续接收原始路径并保留符号链接拒绝合同；浮点 texel roundtrip 使用显式数值容差。
- environment-only provider guard 改为验证真实 Standard-PBR 最终 assembly；错误的自定义 surface 测试请求已通过 Render08 failure lifecycle 回传并关闭。

## Fresh Testing Evidence

- Windows 托管验证全部为 `Dry run: off` 和 `[OK] Cargo test`：10 个过滤器共发现 126 个测试实例，实际执行并通过 125 个，另有 1 个 `#[ignore]`。
- 首次完整 test-profile 编译耗时 397 秒；热缓存 10-filter 验收墙钟总计 161.7 秒，单过滤器耗时 14.37-17.61 秒。
- 发现项覆盖：environment-only 1、自定义 surface 1、配置持久化 1、多 session 1、IBL 14、runtime UI 4、HZB 11、provider guards 13、native bitmap 79（其中 1 项忽略）、broad compile 1。
- 本里程碑恢复测试可靠性与可执行基线，不修改生产热路径，因此不声明运行时吞吐或帧时收益。

## Review

- 独立复审：Critical 0，Important 0；确认测试路径无进程级环境变量修改、IBL 原始恢复路径仍保留安全校验、shader guard 检查真实最终 assembly。
- `git diff --check` 通过；其余 32 个修改 Rust 文件通过 `rustfmt --check`。`runtime_ui_surface.rs` 保留既有布局以避免无关全文件格式化，提交仅保留 29 行 API 对齐差异。
