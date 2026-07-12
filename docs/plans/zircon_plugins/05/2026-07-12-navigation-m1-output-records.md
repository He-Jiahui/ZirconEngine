# Navigation M1 SimpleBake 产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

| 状态 | 日期 | 切片 | 产出与证据 |
|------|------|------|------------|
| 已完成 | 2026-07-12 | M1-T1 physics-first 输入 | `manager/bake/geometry.rs` 在 physics 输入无几何且未被 modifier/obstacle 主动清空时退化到 render mesh；`diagnostics.rs` 输出明确 fallback 诊断；精确测试 `bake_input_falls_back_to_render_mesh_without_physics` 已通过。 |
| 已完成 | 2026-07-12 | M1-T2 bake-load-query 闭环 | 单块 Recast bake 生成 `NavMeshAsset`，加载到 manager 后完成 Detour path query；`golden_level_bake_then_path_length_within_tolerance` 以路径长度容差守护闭环。 |
| 已完成 | 2026-07-12 | M1-T3 modifier area volume | 新增 `manager/bake/area_volume.rs`，空节点 modifier volume 按 world-space AABB 为重叠来源赋 area id；`modifier_volume_marks_area_id_in_polymesh` 已通过。 |
| 已通过 | 2026-07-12 | Windows runtime 验证 | pinned-HEAD validation copy：`cargo test -p zircon_plugin_navigation_runtime --lib --locked --offline --jobs 1 -- --nocapture --test-threads=1`，24 passed / 0 failed。 |
| 已通过 | 2026-07-12 | Windows native 验证 | 同一 validation copy：`cargo test -p zircon_plugin_navigation_recast --locked --offline --jobs 1 -- --nocapture --test-threads=1`，16 unit + 4 integration passed / 0 failed；doc tests 0。 |
| 已通过 | 2026-07-12 | 共享源码 runtime 复验 | 协调器托管 Windows lane：`cargo test -p zircon_plugin_navigation_runtime --locked`，24 passed / 0 failed；doc tests 0。 |
| 已通过 | 2026-07-12 | 共享源码 native 复验 | 同一协调器托管 Windows lane：`cargo test -p zircon_plugin_navigation_recast --locked`，16 unit + 4 integration passed / 0 failed；doc tests 0。 |
| 已完成 | 2026-07-12 | M1 closeout | 共享源码精确包级验证已解除原 Frameworks 阻塞，M1-T1..T3 代码、文档与测试证据齐全，进入 M2 TiledBake 与异步。 |

| 里程碑 | 测试阶段 | 状态 |
|---|---|---|
| M1 | M1-Testing：runtime、Recast native、bake-load-query 精确包级验收 | 通过 |
