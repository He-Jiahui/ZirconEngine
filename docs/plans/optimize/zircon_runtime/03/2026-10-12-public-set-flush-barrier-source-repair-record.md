---
doc_type: source-adoption-record
title: Runtime03 public set flush barrier source repair
date: 2026-10-12
task_id: runtime-editor-optimize
source_status: adopted_source_only
native_status: pending
performance_status: pending
---

# Runtime03 公共 set/flush 发布屏障 Source 修复记录

本记录登记共享 Config persistence owner 上已采用的 Source 修复。对应的 [Runtime03 Source 完成列表](../../../astra/features/runtime/2026-10-12-runtime03-public-set-flush-source-completion-list.md)只勾选 Source；Native、性能和产品验收仍待后续验证。

## Source 修复范围

冻结的 P2 finding 描述了同一 `DefaultConfigManager` persistence owner 的两个 clone：一个 clone 发布新值后暂停在 dirty request 前，另一个 clone 对相同值的 setter 返回 `Ok`，随后 `flush` 捕获旧 dirty generation 并在 worker 写盘前返回。修复在既有 `ConfigPersistenceWorker` owner 上共享 publication gate，覆盖 `store_config_value_if_changed` 到 `request_persistence` 的区间，让同 owner 的公开 set/flush 不能越过尚未登记的发布。

清洁的相同值仍不新增 generation；失败 dirty generation 的相同值重试仍保留。`flush` 继续使用原 worker state 和既有目标 generation 语义。该修复没有声称完成 raw Core writes、独立 persistence owners、typed layer 或多 key transaction 的其他边界，也没有测量 gate contention、Value retirement latency 或 setter throughput。

## 实际采用的 postimages

Root combined adoption receipt 的 12 项 `actualAfter` 与冻结 `expectedAfter` 在 raw SHA-256、LF SHA-256、字节数、存在性和 no-reparse 状态上逐项相符。本修复对应下列三项；receipt 记录 index unchanged，且本记录未运行 Rust 或 Native 测试。

| Canonical path | Raw / LF SHA-256 | Bytes |
| --- | --- | ---: |
| `zircon_runtime/src/foundation/runtime/config_manager.rs` | `bf5393c78fb8896d7139cdf2f428c13edb0fb64a08693a3db1e6767f3e9a7e08` | 8,577 |
| `zircon_runtime/src/foundation/runtime/config_manager/worker.rs` | `1618a689c607673e37afa4a4eafe025f68878bd96cffe79454c8445dc7cd9134` | 15,414 |
| `zircon_runtime/src/foundation/runtime/tests/config_manager_tests.rs` | `caffcf0c66acf22acdcebe07ff1856cc7fa59bf2d72830b1548afa96b940edab` | 29,146 |

新增一个普通、非 ignored 的 case：`foundation::runtime::config_manager_tests::equal_set_flush_waits_for_an_in_progress_publication`。这是待 Native 运行的回归用例，不是已执行测试或性能样本。

## 验收边界

- [x] Source：三项 canonical postimages 与冻结 afterimages 匹配，采用结果登记在 combined receipt。
- [ ] Native：新回归 case 和原 Runtime03 Config matrix 尚未运行。
- [ ] 原性能合同：setter throughput、锁竞争和 Value retirement 等结果未测量，原阈值未放宽。
- [ ] Runtime 产品、Jenkins 与 whole-milestone acceptance。

## 冻结 packet 与采用回执

- [Frozen candidate patch](../../../../../.codex/outbox/runtime03-public-set-flush-barrier-repair-v1/candidate-v1.patch) — SHA-256 `8a1c5b3fbfac47ea1d8b410447b90105ccbb28826f0511d22ffe962f2636459f`, 9,777 bytes.
- [Source binding](../../../../../.codex/outbox/runtime03-public-set-flush-barrier-repair-v1/source-binding-v1.json) — SHA-256 `e9c04009bd4a1b2064a22d614a31e789a26be99d7bc4791deffd4fa161ad20e9`, 4,778 bytes.
- [Source contract proof](../../../../../.codex/outbox/runtime03-public-set-flush-barrier-repair-v1/source-contract-proof-v1.json) — SHA-256 `7865d46ec77fe3566e1b33d47c54f7f5e2e60052dbf157791801a2b01d19d2ce`, 11,857 bytes.
- [Native test selectors](../../../../../.codex/outbox/runtime03-public-set-flush-barrier-repair-v1/native-test-entrypoints-v1.json) — SHA-256 `7078cc88f1dccf9d87ea9f244fc071b6cb8edfb956cfe97632d6de0de14d8090`, 6,308 bytes.
- [Independent peer review](../../../../../.codex/outbox/runtime03-public-set-flush-barrier-repair-v1/independent-review-v1.json) — SHA-256 `09242a54853f233c279443ddb60e2e818bbb138b1b674d90d460671dabb4e6fd`, 27,377 bytes; Source adoption approved, Native/performance pending.
- [Combined actual adoption receipt](../../../../../.codex/outbox/runtime-editor-public-contract-repairs-actual-adoption-20261012-v1/actual-canonical-apply-v1.json) — SHA-256 `e7c5bccb8e948bd5bf448266fb2e49c826245806b6b9bc9c1d5b9dcea2547e80`, 30,354 bytes.
