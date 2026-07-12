# Navigation M2 TiledBake 与异步产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

| 状态 | 日期 | 切片 | 产出与证据 |
|------|------|------|------------|
| 已完成 | 2026-07-12 | M2-T1 原生瓦片桥接与网格切分 | 新增 `zr_nav_recast_bake_tile` C ABI、带 border 的 Recast tile config、稳定 tile id/边界 DTO、Rust tile plan 与边界顶点去重合并；`tile_bake_matches_simple_bake_geometry` 已通过。 |
| 已完成 | 2026-07-12 | M2-T2 后台并行与收割 | `DefaultNavigationManager` 使用 Runtime async-compute `TaskPool`；`start_tiled_bake` 接收 owned `World` 快照并只在调用线程分配 per-surface generation/handle 与入队，几何收集、tile plan 和逐瓦片烘焙均在后台完成，公开 state/非阻塞 harvest；tile plan clone 通过 `Arc` 共享源 mesh、扁平顶点、area 与 tile 表，任务提交不复制完整输入；generation 只允许同 surface 最新 full/dirty bake 发布 snapshot、diagnostics 与统计，settings 更新和更新一代请求会 retire 对应旧 handle，superseded harvest 返回 typed error，worker panic 转为可收割的 `NavigationError`；`tiled_bake_does_not_block_main_thread`、`tile_boundary_paths_are_continuous`、多 surface 隔离、陈旧/retire/settings 交错与共享缓冲回归测试已通过。 |
| 已完成 | 2026-07-12 | M2-T3 局部脏瓦片重建 | `start_dirty_tile_rebuild`/`try_harvest_dirty_tile_rebuild` 复用后台准备、逐 tile 任务和收割路径；`NavMeshDirtyBounds` 将有限 AABB 扩展一圈邻居，只重新烘焙命中 tile id，未命中瓦片从上一份结果精确提取并保留；previous/current tile 以世界 XZ 边界对账，支持脏区内 tile 新增、腾空、全网格清空与稳定 id；快照绑定 surface entity、agent、surface descriptor 与 settings，身份或 tile size 变化要求显式全量重建，成功的非 tiled bake 清除旧快照；邻域保留、新增/腾空、全清空、身份漂移与生命周期回归测试已通过。 |
| 已通过 | 2026-07-12 | Windows native 验证 | 协调器 job `4cf932aef2af4b6ba2516a79936d59c2`：`cargo test -p zircon_plugin_navigation_recast --locked`，18 unit + 4 integration passed / 0 failed；doc tests 0。 |
| 已通过 | 2026-07-12 | Windows runtime 验证 | 协调器 job `6506b735385d4709b40cf734e1391980`：`cargo test -p zircon_plugin_navigation_runtime --locked`，37 passed / 0 failed；doc tests 0。 |
| 已完成 | 2026-07-12 | 结构约束处理 | `manager/bake.rs` 保持编排职责；tile plan、任务池/收割、dirty rebuild 分别落入 `tiled.rs`、`task_pool.rs`、`dirty.rs`；接近测试文件硬门槛的 M2 generation/context 用例已拆到 `tests/tiled_bake_context.rs`，`tests/bake.rs` 744 行、context 文件 192 行，所有生产 Rust 文件低于 1000 行且所有测试文件低于 800 行；未新增兼容 shim、扁平大文件或跨插件反向依赖。 |

| 里程碑 | 测试阶段 | 状态 |
|---|---|---|
| M2 | M2-Testing：Recast native、runtime async/continuity/dirty-rebuild 精确包级验收 | 通过 |
