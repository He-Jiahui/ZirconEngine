---
owner_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
milestone: M2
slice: world-sync-watch-map
status: review_clean_validation_copy_rebuild_pending
related_code:
  - zircon_editor/src/core/sync/mod.rs
  - zircon_editor/src/core/sync/watch_map.rs
tests:
  - zircon_editor/src/core/sync/watch_map/tests.rs
  - zircon_editor/tests/editor_world_sync_watch_map.rs
  - tools/tests/test_editor02_world_sync_watch_map_contract.py
---

# Editor02 world sync watch map

Plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
Milestone: M2
Status: review_clean_validation_copy_rebuild_pending
Files: ["docs/zircon_editor/core/world_sync.md", "tools/tests/test_editor02_world_sync_watch_map_contract.py", "zircon_editor/src/core/mod.rs", "zircon_editor/src/core/sync/mod.rs", "zircon_editor/src/core/sync/watch_map.rs", "zircon_editor/src/core/sync/watch_map/tests.rs", "zircon_editor/tests/editor_world_sync_watch_map.rs"]

本切片只完成 M2.1 的 editor `watch_map` 支持层。当前 gateway 文件仍属于 Editor01 的已归档 current-hash attribution，本切片没有吸收或改写这些文件，也不把 gateway、session owner、frame pump 或 hierarchy diff 写成完成。

## Scope delivered

- `WorldWatchMap` 原子维护 token authority 与 view reverse lifecycle index；同 token 重绑会先清理旧 view 关系。
- view close 可一次取得排序后的全部 runtime token；session teardown 可 `drain_tokens` 并清空两侧状态。
- 零 token 与空 invalidation mask 返回 typed error，失败注册不改变索引。
- `InvalidationBatch` 投影只遍历 dirty token，按 view 合并 mask，不随全部 watch 数量线性扫描。
- 重复 token 与未知 token 形成确定性诊断；未知 token 不创建隐式 view 状态。
- 无效重绑在修改任一索引前失败，既有 token/view 关系保持；`unbind_token` 同步清理反向索引，未知 token 为 no-op。
- 公开方法已补 replacement、sorted cleanup 与 `into_dirty` 丢弃诊断语义的 Rustdoc。

## Fresh testing evidence

- TDD RED：实现前静态合同 `5 errors`，均为新 authority/记录尚不存在。
- Rust 回归覆盖 token 重绑两侧一致性、view 批量注销排序、mask 合并、重复/未知 token 诊断、session drain 和无效注册原子拒绝。
- r1 静态合同最终 `5/5 GREEN`；r2 integration gate 合同先得到 `1 error / 5 passed`，公开 production-surface gate 落盘后为 `6/6 GREEN`。
- exact5 Rust 文件通过 Rust 1.94.1 rustfmt，exact8 通过 scoped `git diff --check`。
- 独立首审 Critical/Important/Minor=`0/1/2`。两个 Minor 已以测试优先补齐 invalid rebind/unbind 生命周期与公开 API 文档，静态合同由新增 1 个 RED 收敛为 `7/7 GREEN`。
- 唯一 Important 属 Editor13 facade 闭包：snapshot 753 的 `core/mod.rs` 含 `pub mod script_build;`，冻结基线却无 `core/script_build/mod.rs`，当前副本必然 E0583。已写入 `13/failure-2026-07-22-script-build-facade-validation-copy-closure.md`；不得删除或吸收外部 facade，需等待 Editor13 managed SHA 后重建副本/snapshot。
- 状态合同同步修复后最终独立复审 Critical/Important/Minor=`0/0/0`；外部 Editor13 blocker 不冒充 watch-map 业务 finding 已解决。

## Remaining M2 work

- gateway query/watch/unwatch/drain 的 InProcess 与 Session 实现，以及 session teardown unwatch 接线。
- runtime mutation throat、frame-end flush 与 editor 每帧至多一次 drain。
- hierarchy subtree diff、5k 单节点修改重建行数和 retained-host dirty 消费。
- PERF-MVP-468 的 typed direct index、bounded fact coalesce 与 100k scale evidence。

## 产出记录与时间

| 里程碑 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|
| M2.1 editor watch-map support slice r3 | `review_clean_waiting_editor13_base` | 2026-07-22 | 独立首审 0/1/2；新增 invalid rebind 原子性、token unbind 反向清理/unknown no-op 回归与公共 Rustdoc，状态合同修复后静态门 `7/7 GREEN`，最终独立复审 0/0/0。Important 已按 owner 写入 Editor13 facade closure failure；等待其 managed SHA 后重建 validation copy/snapshot，再做 Cargo 与 managed commit。父 M2 保持 pending。 |
| M2.1 performance follow-up | `static_green_dynamic_pending` | 2026-07-22 | `ViewDirtySet::mark_ref`让同view多token只首次拥有ViewInstanceId，watch-map源码守卫与Editor02 Python合同7/7。normal batch的三套BTreeSet/transport bytes预算仍归PERF-MVP-468，未取得Cargo/100k/F4证据，父状态不变。 |
| M2.1 snapshot820 三路径增量复核 | `review_clean_validation_copy_rebuild_pending` | 2026-07-22 | snapshot820 后仅本记录、`watch_map.rs`、`watch_map/tests.rs` 合法漂移；其余 5/8 路径哈希未变。当前静态合同 `7/7 GREEN`，exact Rust 以 `skip_children=true` scoped rustfmt check 通过，增量独立复审 `Critical/Important/Minor=0/0/0`：同 view 多 token 只首次拥有 ViewInstanceId，duplicate/unknown/matched 诊断及 mask union 均未回退。Editor13 closure 文件当前存在，但 validation copy 必须先校验其 current-hash attribution；本切片不越权归因外部 owner，不宣称 Cargo 或 M2 完成。 |
