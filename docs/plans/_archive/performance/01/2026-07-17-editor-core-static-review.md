---
related_code:
  - zircon_editor/src/core
  - zircon_editor/src/ui/host/startup
  - zircon_editor/src/ui/host/editor_manager_plugins_export
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
reference_sources:
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
  - dev/godot/editor/export/editor_export_platform.cpp
tests:
  - editor core source guards listed below
  - current-source Windows zircon_editor focused tests pending
  - editor idle/interaction WPR trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor core 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

本轮逐文件阅读当前工作树 `zircon_editor/src/core` 的 **184/184** 个 Rust 文件，并追到 `ui/host/startup` 与 plugin-manager/export 直接调用方。Git 基线账本中的该行仍为 178；当前另外 6 个 untracked core 文件是 `commandlet` 3 个与 gateway 3 个，已纳入本轮阅读，但在形成可复现 Git tree 和动态证据前不改写 17,706 基线守恒。

| 模块 | 文件数 | 静态状态 | 主要结论 |
|---|---:|---|---|
| root | 7 | 已读 | operation path 临时 Vec 已修；plugin catalog generation 缺失已移交 |
| asset | 15 | 已读 | builtin lookup owned id 已修；augment clone-on-write 已移交 Editor09 |
| commandlet | 3 | 已读 | 启动/显式命令路径，未发现 MVP frame hotspot |
| commands | 16 | 已读 | enablement clause 反复物化、path segment 分配已修 |
| context | 3 | 已读 | snapshot ownership 清晰；频率需由 UI trace 验证 |
| editing | 18 | 已读 | UpdateNode 重复 capture/全字段 apply 与 nested cancel front-remove 已修 |
| editor_event | 27 | 已读 | prefix 规范化已修；journal/listener 无界保留与锁内 fanout 已移交 Editor02 |
| editor_extension | 1 | 已读 | 883 行多域 owner；注册期重复校验待结合 Editor12 generation 处理，不在本轮继续堆叠修改 |
| editor_message | 25 | 已读 | 无界 inbox/fanout 已在 PERF-MVP-019 移交 Editor02 |
| editor_plugin_sdk | 3 | 已读 | SDK facade/lifecycle 非帧路径；catalog owner 是主要问题 |
| export | 8 | 已读 | 重叠递归 digest/工具探测是 warm export P0，已移交 Editor15 |
| gateway | 8 | 已读 | 稳态 read-lock/Arc clone/capability deep clone 已移交 Editor01 |
| jobs | 26 | 已读 | storm/无界 pump 与 pending promotion 已在 PERF-MVP-018/020 移交 Editor14 |
| play | 2 | 已读 | session bridge 无新增简单热点；受 gateway/event transport 共同影响 |
| project | 17 | 已读 | successful open 后重复 reopen/parse recent summary 已修 |
| runtime_event_consumer | 5 | 已读 | active mutex 跨 ABI/decode/plugin callback 且无配额，已移交 Editor02/Plugins01 |

## 直接优化与静态 RED→GREEN

- command enablement 直接求值 stored `when`、required capabilities 与 `AssetWritable`，不再构造/排序/去重 owned effective clause；document/menu/schema/operation identifiers 改为单遍 segment 校验。
- `UpdateNodeCommand` 只 capture 一次 before/after 并按 delta 调用 setter；nested cancel 改用尾部 `pop` 保持逆序恢复；batch command 去掉再 collect。
- builtin asset lookup 使用 static canonical resource-kind id 与 borrowed map lookup，不再构造 `AssetTypeId(String)`。
- listener prefix 在 descriptor 创建/更新时 normalize 一次，逐 event accept 不再 lowercase/format。
- successful project open 直接复用已解析 manifest summary 更新 recent list，不再 reopen/parse 同一项目。

结构守卫：

- `command_enablement_does_not_materialize_an_effective_when_clause`
- `document_kind_validation_streams_segments`
- `command_descriptor_validation_streams_path_segments`
- `editor_operation_path_validation_streams_segments_without_collecting`
- `editor_commands_avoid_recollecting_batches_and_reapplying_unchanged_node_fields`
- `nested_cancel_does_not_remove_from_the_front_of_a_vec`
- `builtin_lookup_does_not_construct_an_owned_asset_type_id`
- `listener_acceptance_does_not_normalize_prefixes_per_record`
- `listener_filter_normalizes_operation_prefixes_once`
- `opened_project_is_not_reopened_just_to_update_recents`

## 跨计划性能交接

- Editor01：`failure-2026-07-17-gateway-stable-call-lock-and-clone.md`。
- Editor02：`failure-2026-07-17-editor-event-journal-listener-unbounded-retention.md` 与 `failure-2026-07-17-runtime-event-consumer-unbounded-pump-lock.md`。
- Editor09：`failure-2026-07-17-asset-type-registry-clone-on-augment.md`。
- Editor12：`failure-2026-07-17-editor-plugin-catalog-rebuild-and-deep-copy.md`。
- Editor15：`failure-2026-07-17-export-overlapping-recursive-digests.md`。

failure graph import 已成功，node count 219；唯一 diagnostic 是既有且无关的 tooling `maintenance-held-cpu-reservation-consumption-gap` origin workflow metadata。

## 参考引擎对照

Bevy `Messages` 用每 update 双缓冲与 per-reader cursor 明确消息寿命，并明确警告不 update 会无限增长。Zircon editor journal 需要 durable replay/undo，不能直接限制为两帧，但应采用同样明确的 retention class/cursor/清理契约，而不是所有消息永久 owned clone。

Godot export `FileExportCache` 持久保存 source mtime、MD5 与 saved path，mtime 未变时复用，变化后再校验内容。Zircon 应保留 BLAKE3 内容正确性，同时增加 export-generation inventory、metadata 快筛、重叠 Merkle projection 与持久 cache，避免每 stage 全树重读。

## 未通过项

当前仅完成静态阅读、源码守卫、`rustfmt` 与 diff 检查。尚缺当前源码 `zircon_editor` 聚焦 Cargo、打开项目/选择/修改/保存产品脚本、idle 30 秒 WPR、事件/job storm、export warm bytes-read/hash-count 和插件管理 recompute build-count。因此 `docs/plans/performance/review.md` 保持 **0**，`zircon_editor/src/core` 继续留在 `pending.md`。
