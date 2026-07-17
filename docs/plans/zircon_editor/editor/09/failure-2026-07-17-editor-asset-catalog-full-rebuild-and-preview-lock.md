---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: editor-asset-catalog-full-rebuild-and-preview-lock
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/09
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/catalog_snapshot.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/broadcast.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/subscribe_editor_asset_changes.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/request_preview_refresh.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/events/runtime.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/snapshots.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces
reference_sources:
  - dev/godot/editor/editor_file_system.cpp
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
tests:
  - unchanged project refresh build/read/clone-count regression
  - preview worker and generation-safe commit race matrix
  - paused subscriber and 10000-change storm budget
---

# Editor09：asset catalog 全量重建、preview 锁内 I/O 与无界 change bus

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_editor/src/ui/host/editor_asset_manager` 36/36 Rust 文件逐文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 交接原因：catalog、reference graph、folder projection、preview 与 change stream 必须共享同一 project/asset generation authority。

## 失败现象与复现证据

`sync_from_project` 每次 refresh 遍历完整 registry；每个 ready asset 都重新加载 artifact 并提取 direct references，随后全量重建两个 catalog maps、reference graph、preview scheduler，并在只要存在 ready shader 时重写 shader IDE environment。unchanged project refresh 没有 source/import generation 快路。

`catalog_snapshot_record` 持有 state `RwLock` read guard，期间为全部 assets 深复制 String/diagnostics/references、排序并从零构建 folder tree。调用方得到 owned snapshot，因此稳定 workbench reflection 可重复支付完整 catalog projection。`request_preview_refresh` 更严重：持有 state write lock 跨 source image decode、resize、PNG encode/write 与 `.zmeta` save，任何 catalog/details/snapshot 查询都被单张 preview I/O 串行阻塞。

Workbench 侧已让 Activity/Browser 在同一个 editor snapshot 内共享一次 folder/filter/sort projection，并把 search normalization 与 parent locator 临时分配降到一次/流式；但第二表面仍需要复制 owned snapshot strings/rows。最终解法仍是本计划的 immutable catalog/projection generation，不能在 UI caller 再永久缓存一份 catalog authority。

change subscription 使用 `crossbeam_channel::unbounded`；broadcast 在 subscriber mutex 内为每个 receiver clone change 并同步 send。暂停 consumer 可无限增长，慢/重入路径也扩大全局锁持有时间。该问题与 UI asset watcher 的 PERF-MVP-083 不同：这里是 editor asset catalog authority 自身的 change stream。

Retained host consumer 也没有帧预算：每个 tick 对 asset/editor/resource 三条 receiver 使用 `while try_recv` 全量 drain；任意事件先 `refresh_from_runtime_project`，随后为 selected UUID 构建完整 editor snapshot。visible preview refresh 又构建完整 chrome 并逐项重发 refresh；preview change 可以把 `sync_catalog` 置真，令 paint-only 结果升级为全 catalog sync。此次审查已把 asset content/tree/reference 与 asset-details pointer 改为消费 committed `Arc<AssetWorkspaceSnapshot>`，但 ingress/coalescing/full-refresh 根因必须由同一 Editor09 generation authority 修复。

## 最低共享层根因

Editor asset manager 没有 immutable catalog generation、增量 source/import change set、cached consumer projection 或 preview job generation；可变 state 同时承担 authority、长任务 scratch、UI DTO 构建与消息分发。

## 架构修复验收

- 从 runtime project/import generation 生成 typed asset delta；unchanged refresh 的 artifact load、reference rebuild、folder build 与 shader IDE write count 均为 0。
- catalog publish 为 immutable generation snapshot；UUID/locator/reference/folder indexes 每 generation 构建一次，workbench/details 查询复用 projection，长构建不持有 live state lock。
- preview decode/resize/encode/meta persistence 在 worker 上执行并绑定 `{asset_uuid, source_hash, catalog_generation}`；主线程短锁验证 generation 后提交，陈旧结果不得覆盖新资源。
- change stream 按 CatalogChanged、PreviewChanged、ReferenceChanged 定义 durable/latest/coalesced retention；锁内只维护 subscriber owner，锁外 fanout shared immutable payload。
- 10k asset catalog/paused subscriber/preview storm 下 memory、queue age、lock wait、snapshot p95 有明确预算；顺序、revision、错误、remove/rename 与 shutdown 语义不变。
- retained host 每帧三类 change drain 有 count/time budget 与 queue-age 指标；同 asset/revision 的 catalog/details/preview delta 合并，preview-only 不触发 catalog full sync，visible UUID 每 generation 最多请求一次。

## 禁止临时方案

- 不得在 UI caller 层缓存另一份 catalog authority。
- 不得通过在 write lock 外 clone 完整 `EditorAssetState` 来掩盖锁内 I/O。
- 不得静默丢弃 CatalogChanged 或最终 Preview/Error 状态。

## 修复结果与回传

Open state: `待 Editor09 实现 generation snapshot、增量 refresh、preview worker commit 与有界/coalesced change stream，并回传 build/read/clone/lock/queue 证据`。
