# Runtime02：事件发布预检计数栈存储记录

状态：源码已采用；功能、实际分配与性能验收待完成。日期：2026-10-09。

## 改动与原因

`EventBusState::admit` 的逐 subscriber 淘汰计数使用固定 8 项 `Retained` 数组。不超过 8 个 subscriber 时只借用对应前缀；超过 8 个时仍使用原逻辑长度的堆存储。`Retained` 仅含两个 Copy `usize` 计数，无 lease、析构或外部状态。源码结构因此消除了小扇出的这一个计数 Vec 分配；尚未进行实际 allocation 测量。

容量计算、拒绝原因、mixed-lossy 预检、预分配、ledger 与 queue 提交、退役释放、通知顺序均保持原实现。未增加 dependency、owner、公共 API、测试用例或修改任何性能阈值。

## 已完成的源码验证

一个 Rust 文件一次采用。采用前两次重新核对 35 个 raw/LF 依赖及直接前像；采用后 LF 后像完全匹配，34 个未修改依赖保持原值。实际 Rust 2021 格式检查与 `git diff --check` 均为 0；Cargo 文件、控制配置、共享索引及既有 index.lock 内容和元数据保持原样。

- [实际采用收据](../../../../../.codex/outbox/runtime02-inline-removal-preflight-v1/actual-canonical-apply.json)：SHA256 `314ed26fadf7b255762184c80cdefe0f1e8e048445452a54e94900308589bd27`。
- [核心独立审查](../../../../../.codex/outbox/runtime02-inline-removal-core-review-v1/review.json)：SHA256 `eda4a3f2fde9dc8e7a49de407795f42e8455c850fb3da6b682a7ecae29a79077`，源码接受，native/performance pending。
- [完成列表](../../../astra/features/runtime/2026-10-09-runtime02-inline-preflight-completion-list.md)。

首次 Root 采用预检误把独立报告的 `decision` 与 `status` 字段相比，在 Git 检查、应用及源码写入前拒绝。已记录 [预检历史](../../../../../.codex/outbox/runtime02-inline-removal-preflight-v1/initial-preflight-refusal.json)，按报告的实际 `decision=accepted`、`status=accepted_source_native_performance_pending` 校验；全部源码与依赖守卫保留。

## 批量异步验收

沿用当前 15 项批次的 Runtime 功能与原 Release EventBus 矩阵，零新命令、零新用例。原 1/2/5/100 fanout、64B/4KiB/256KiB payload、1/32/1K batch、diagnostics on/off 与 10K/100K/1M 负载保持原口径；还需测量原 allocation、CPU、延迟、retained bytes 与 RSS。

当前草稿 v7 为 `commands-draft-v7.json`，SHA256 `f4bbc71e97e21a6fb8cd332bdd3b3e1196affd697af6814b11aa9e99bb15fef3`。14 个原 Release 生产者 raw SHA 再次匹配。后续 Tools、Generated 与真实覆盖门采用后统一更新提交绑定。待可用协调器入口后批量提交一次，记录 request ID/receipt 入口，并继续其他功能修复；在实质检查点校验终态收据。

当前没有 Cargo 执行、协调器提交、native 通过或性能达标证据。
