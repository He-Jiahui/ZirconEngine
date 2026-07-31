---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: plugin-event-drain-frame-budget
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_editor/src/core/gateway/session.rs
tests:
  - cargo test -p zircon_runtime --lib plugin_event --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib runtime_event_consumer_bounded_pump --locked --jobs 1 -- --ignored --nocapture --test-threads=1
---

# Plugins01：plugin-event drain 帧预算交接

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源切片：Editor02 runtime event consumer bounded fair pump
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 生命周期键：`plugin-event-drain-frame-budget`

## 失败现象与复现证据

Editor02 已限制每 tick callback count、per-consumer count 与 elapsed time，并把未消费 delivery 放入 generation 绑定 pending queue；但生产 `drain_plugin_events` 仍在预算循环前一次构造全部 delivery、整批 JSON 序列化，并由 SessionGateway 完整解码。1k/10k backlog 的首 tick 因此仍可能在第一次 elapsed 检查前承担 O(backlog bytes) 的 transport 成本。

现有 fake gateway 只移动预构造 `Vec`，无法证明真实 ABI encode/decode 不突破 editor frame budget。该最低根因属于 Plugins01/runtime event transport；Editor02 不复制第二套 runtime mirror 或直接改写外部 owner。

## 架构修复验收

- transport 提供有界 drain（至少 count + encoded bytes，或等价 cursor/page contract），并保留 subscription/generation/sequence 身份。
- runtime 构造、wire 编码与 editor 解码的单次工作量都受同一请求预算约束；未取出的 delivery 留在 runtime authority，不丢失、不重复。
- 1k/10k backlog 记录每 tick drained count/bytes、encode/decode 时间、pending peak 与 p95；首 tick 不再随总 backlog 线性增长。
- Editor02 只消费该有界协议并保留自身 callback/fairness/pending 预算；不得长期并存 unbounded/bounded 两条 writer/reader 真相。
- 禁止用扩大 frame budget、预分配全量 Vec、test-only fake 分页或静默截断替代协议修复。

## 产出记录与时间

| 里程碑 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|
| Plugins01 transport / Editor02 M2 dependency handoff | `open / bounded_transport_required` | 2026-07-22 | 独立审查确认生产链 `dynamic_api/session/event_mirror.rs` 全量构造与 JSON encode、`core/gateway/session.rs` 全量 decode 均发生在 Editor02 callback elapsed budget 之前；现有 1k/10k fake gate不能证明真实帧预算。修复归 Plugins01，Editor02 failure 不得在 bounded transport 动态证据前 upward return。 |
| Plugins01 bounded-drain reference and current-cursor audit | `design_root_confirmed / implementation_pending` | 2026-07-22 | 对位当前仓内 Bevy `Messages` / `MessageCursor`：其 iterator 只在实际 `next`/`nth`/`count` 消费时推进 reader，而 Zircon `EventCursor::read` 在返回 iterator 前就把 cursor 推到队尾，因此直接在现有 iterator 上 `.take(limit)` 会把未编码事件误标为已读。Bevy 双缓冲又明确在两次 update 后丢弃滞后消息，不能充当 Editor 1k/10k backlog 的持久 transport authority。由此排除“现有 drain 后截断 Vec”与“仅给 iterator 加 take”两种伪修复；实现必须提供 subscription-owned persistent pending authority，并让 cursor advance、JSON bytes 与返回成功形成同一提交边界。冻结 `ZrRuntimeApiV3` 不增字段；若请求预算不能由等价固定 page contract 完整表达，则走新 API table version + host hard cut。 |
| Plugins01 bounded-drain transport authority | `implementation_complete / managed_focused_and_editor_evidence_pending` | 2026-07-22 | `EventCursor` 已把提交点硬切到 iterator 每次成功 `next()` 后；runtime mirror 已改为 send-boundary subscription-owned persistent queue，固定单页 `64 events / 128 KiB payload`、队列 `16K events / 64 MiB`，descriptor 与 ABI wire 分别限制为 `128 B / 256 KiB`，overflow / oversized payload 均显式报错且不丢弃已接受事件。精确当前生产源的隔离 MSVC 测试先复现旧实现 10K backlog 单页全量 drain，再在当前实现通过 `4 passed / 0 failed`（含 10K 跨 `World::update_events` 无损有序分页、overflow 保留与 oversized 拒绝）；scoped rustfmt、`git diff --check` 通过。合并 current-source reservation `443ed28a879c42228127596358928d6a` / job `a2f858fdd9894cb88df122fb92780da9` / run `255a862363d74b699f0b23cf308dfe3b` 已自然 `exit 0` 并生成 934 MiB lib-test 二进制，证明 bounded transport 当前源可在 Rust 1.94.1/core-min 下完整类型检查与链接。直接宽分组在先执行的 native access 用例发现独立 ECS id RED 后按 fail-fast 停止，因此尚未把 event-mirror 分组记为通过；其增量修复 reservation `ed339385ebc94690a60ef20d83a8be1a` GREEN 后继续执行。Runtime + Editor 真实 ABI 链的 1K/10K 每 tick count/bytes/time/p95 证据仍为 open，因此本 failure 尚不 upward return。 |
| Plugins01 editor ABI page observability | `source_implementation_complete / managed_validation_pending` | 2026-07-30 | `EditorRuntimePluginEventPage` 现在在 editor gateway 边界保留实际 wire bytes、runtime drain/encode 调用耗时和 editor decode/release 耗时；`EditorRuntimeEventPumpReport` 每 tick 聚合 count、bytes、两项总耗时与页级 p95。受管 1K/10K 用例已改为经 `SessionGateway` 的 C ABI 函数表返回 JSON 页，fixture 仅保存 subscription-owned remaining/sequence 并且每次最多构造 `64` 条，避免预构造全 backlog 或直接搬运 fake `Vec`。Rustfmt 1.94.1 check 与 scoped `git diff --check` 已通过；尚未启动 Cargo，因为外部 Shader06 job `077e5046727d4fa39c117af1cfb509d4` 仍为 `leased`，本条不把 managed evidence 记为 GREEN。 |
| Plugins01 fixed-page idle/backlog observability | `implementation_complete / managed_runtime_and_editor_validation_pending` | 2026-07-31 | Runtime queue page 现在返回 remaining delivery count 与 oldest pending age；idle drain 直接返回 empty owned buffer，wire loop 直接拼接 subscription-owned raw JSON payload bytes，未把 payload 重解为 `Value`。ABI batch 的新增字段使用 serde default 以保持旧 wire 默认零值；Editor gateway/pump 传递并汇总 backlog 指标。1K/10K C ABI fixture 继续每页最多生成 `64` 条；scoped `rustfmt +1.94.1 --check` 与 `git diff --check` 通过，现有 Editor managed reservation 的 7-file manifest 逐项匹配。独立审阅发现 `scene/tests/ecs_event_mirror.rs` 的旧 `erased.drain()` 守卫已漂移；现已绑定 raw-page `erased.drain_payloads()` 并禁止该路径恢复 `serde_json::from_slice`，whitespace/source guard 通过。Runtime 默认-feature gate 仍先被 Frameworks01 `scene/level_system` compile boundary 阻断，且 Editor reservation 尚未轮到 CPU FIFO；本 handoff 继续保持 `open`，不宣称动态 GREEN。 |

后续已单独执行同一 `r9` 不可变二进制的 `scene::tests::ecs_event_mirror`，结果
`9 passed / 0 failed`；覆盖 current-only hard cut、10K 跨 update 持久分页、queue overflow、
oversized payload/descriptor、schema/duplicate、unsubscribe rollback 与 drain 不 clone event id。
因此 runtime focused 状态更新为 GREEN；Editor 真实 ABI 预算证据仍保持 open。

Render17 的 current-source lib-test 编译随后在运行任何 Render17 测试前暴露 Plugins01 边界错误：
`dynamic_api/tests/linked_plugins.rs` 直接导入私有 `session::event_mirror`，产生 E0603。修复保持
`event_mirror` 私有，仅由 `dynamic_api::session` 以 `pub(in crate::dynamic_api)` 重导出两项 page
预算常量，测试改为消费 session 边界；scoped rustfmt、`git diff --check` 与禁止私有模块穿透的
源码守卫已通过。当前 SHA256 为 `session.rs=49F8B059434935B941951A3E36460A481A989D8FC15EEE221A9803FD33A97D9F`、
`session/event_mirror.rs=A5E00D8BE7D9EBBB713220173E8F0B1AB04EE3540B96263449807067A6105A07`、
`tests/linked_plugins.rs=8C83533147101C19AA5AE8AEB07DE2436E71D139CA4A0260A5886A83057FABC2`。
包含该边界与 eager ECS access-id 修复、并启用默认 feature 以实际编译 linked-plugin tests 的
managed current-source reservation `41a0329396404c4d830c6987fe663225` 正按 FIFO 等待；在它自然
GREEN 前不把 E0603 写为 fixed return。

该 reservation 后续已按原审计命令消费为 job `f156748ce96a4cfd9940221340fc04e9` / run
`e19d2b8efd89450888f5c03d6839eb44`，67 路 source manifest 在启动前复核为
`changed=[] / missing=[]`。默认 feature 的 `zircon_runtime --lib` 已越过 linked-plugin test
类型检查与主 crate codegen，原 E0603 未再出现；job 于 `2026-07-22T17:11:09Z` 终态
`exit 101`，唯一 6 项错误均为 Performance01 exact3 owner 路径
`rhi/tests/device_contract/transfer_and_fences.rs` 六处未导入 `WgpuRenderDevice`（E0433）。
根因已确认不是恢复 compatibility import，而是把遗留类型/测试前缀硬切到
`DeterministicRhiContractDevice` / `deterministic_rhi_contract_*`；修复已路由其 active lease owner。
本切片不修改该路径，也不把跨计划阻断伪记为 Plugins01 GREEN；owner 修复后，
复用已建立的 default-feature target 做一次增量终态复核。plugin-event runtime focused 已有 9/9，
Editor 真实 ABI 的 1K/10K 帧预算证据仍 open，因此本 failure 继续保持 `open`。

## 2026-07-30 Performance01 current-source reconciliation

Performance01当前源复读确认Plugins01的固定page/queue实现仍是有效的最低层进展，Editor02旧记录中的“生产全量无界Vec”不得继续作为当前根因。剩余跨计划边界有两项：其一，runtime对空page仍序列化owned JSON、Editor逐active consumer每tick解码，稳定空载工作不为零；其二，queued JSON payload在runtime先解为Value、整batch再编码，Editor再解Value并typed `from_value`，descriptor String也逐delivery clone。两项继续归PERF-MVP-432/Plugins01+Runtime10，不能由Editor复制第二transport。

Editor host无上限pending和逐event双active-map锁归Editor02/PERF-MVP-069；stable capability全量reconcile归PERF-MVP-565。Plugins01验收补充empty page encode/decode/alloc=0、stable descriptor clone=0、payload单一owner，并向Editor报告remaining/oldest age。若固定256KiB wire page的current-source decode p95超出4ms，才升级新API table的request-aware count/bytes/deadline或Runtime11统一single-flight decode ticket；不得新增plugin私有线程池。当前没有新的managed Cargo结果，failure保持open。
