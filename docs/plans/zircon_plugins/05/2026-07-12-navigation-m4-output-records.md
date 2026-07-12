---
related_code:
  - zircon_plugins/navigation/native/native/detour_query.cpp
  - zircon_plugins/navigation/native/native/detour_tile_cache.cpp
  - zircon_plugins/navigation/native/native/detour_tile_cache_raster.cpp
  - zircon_plugins/navigation/native/native/detour_tile_cache_raster.h
  - zircon_plugins/navigation/native/src/tile_cache.rs
  - zircon_plugins/navigation/runtime/src/runtime_obstacles.rs
  - zircon_runtime/src/core/framework/navigation/query.rs
implementation_files:
  - zircon_plugins/navigation/native/native/recast_bridge.h
  - zircon_plugins/navigation/native/native/detour_query.cpp
  - zircon_plugins/navigation/native/native/detour_tile_cache.cpp
  - zircon_plugins/navigation/native/src/ffi.rs
  - zircon_plugins/navigation/native/src/detour.rs
  - zircon_plugins/navigation/native/src/tile_cache.rs
  - zircon_plugins/navigation/runtime/src/runtime_obstacles.rs
  - zircon_plugins/navigation/runtime/src/manager/state.rs
  - zircon_plugins/navigation/runtime/src/manager/tick.rs
  - zircon_runtime/src/core/framework/navigation/manager.rs
  - zircon_runtime/src/core/framework/navigation/query.rs
plan_sources:
  - docs/plans/zircon_plugins/05-navigation.md
tests:
  - zircon_plugins/navigation/native/src/tests/path.rs
  - zircon_plugins/navigation/native/src/tests/tile_cache.rs
  - zircon_plugins/navigation/runtime/src/tests/manager.rs
doc_type: milestone-detail
---

# Navigation M4 Obstacle / Modifier 产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M4 | M4-T1：TileCache obstacle add/remove bridge 与 per-navmesh carving 闭环 | `完成` | 2026-07-12 | bridge ABI v3；持久 owner；cache-scoped handle；128 runtime obstacle 容量；64-request 队列自动分批 flush；remove/update/add/update 替换顺序；carve/remove/restore、65 incremental/batch、跨 cache 拒绝与满 4 替换回归通过 |
| M4 | M4-T2：NavQueryFilter area cost / include-exclude flags 贯通 | `完成` | 2026-07-12 | 64-entry caller cost table、显式 serde、16-bit Detour flags、asset walkability；`NavigationManager` 服务 trait 可调用；native/fallback 双向成本与 flag 回归通过 |
| M4 | M4-Review：独立 Critical/Important 复审与结构收束 | `通过` | 2026-07-12 | 两轮共修复 5 个 Important；TileCache 栅格职责拆到 `detour_tile_cache_raster.{h,cpp}`，主实现由 906 行降至 761 行；最终复审无 Critical/Important |
| M4 | M4-Testing：native/runtime 精确包级 Windows 验收 | 通过 | 2026-07-12 | managed validator job `5c1a96ab19e54cb1bb47d091979e17d7`: native 31 unit + 4 integration + doctests；job `567fc95691f44ec9a43ca895aabcfcc3`: runtime 50 unit + doctests；均 0 failed |

## 设计结论

- 动态 obstacle 的所有权固定在 runtime per-navmesh world，C bridge 只暴露 opaque owner 与稳定 obstacle ref；移除组件不会重新构建一份无关联的临时查询。
- Rust safe wrapper 用 cache identity 约束 obstacle handle，按 64 个 request 水位自动提交 Detour 更新；native batch create 使用同一分批原则；runtime 在替换障碍前先提交 removal，避免满容量时复用尚未释放的 native slot。
- 区域成本从资产默认值与查询覆盖值分离；普通路径继续继承 baked costs，`NavQueryFilter` 通过 `NavigationManager::find_path_with_filter` 服务合同使用，不污染 `NavPathQuery` DTO。
- area mask 继续表达 64 个 area 的语义过滤；polygon flags 使用 Detour 16-bit include/exclude 语义，高位自定义 area 汇入明确的 overflow flag，而不是静默截断为零。
- fallback shared edge 按方向分别使用 source polygon area 计费，与 native Detour `cur_poly` 成本语义一致。
