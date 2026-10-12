---
doc_type: source-adoption-record
title: Runtime44 oversized encoding and empty-prefix source repairs
date: 2026-10-12
task_id: runtime-editor-optimize
source_status: adopted_source_only
native_status: pending
performance_status: pending
---

# Runtime44 oversized encoding 与 empty-prefix Source 修复记录

本记录覆盖两个独立的 Runtime44 Source packet。两项均已采用 Source；这不表示 Native、完整 Runtime44 原合同或性能目标已验收。对应的 [Runtime44 Source 完成列表](../../../astra/features/runtime/2026-10-12-runtime44-encoding-filter-source-completion-list.md)保留这些边界。

## Oversized encoding：只减少临时编码峰值

v2 repair 让现有 formatter pieces 进入有界 writer requests，避免为 oversized first record 额外创建完整 encoded batch `Vec`。它保留完整记录内容、FIFO、时间戳、metrics、错误处理和输出所有权。控制台分支拥有 `StderrLock` 并在分支离开时释放，再进入文件输出；v2 修复了 v1 将 guard 保持到文件写入期间的 Source ownership 问题。

这项改动只减少这次编码产生的临时 buffer peak；它不实现 queue byte quota、record byte quota、queue/record memory bound 或 whole-logical-batch hard cap。Runtime44 P1-13 与 P1-24 仍开放。

新增五个普通、非 ignored cases，均待 Native 验证：

1. `diagnostic_log::sink::worker::oversized_encoding_tests::oversized_first_record_preserves_full_utf8_scope_message_and_fifo`
2. `diagnostic_log::sink::worker::oversized_encoding_tests::oversized_mirrors_preserve_console_then_file_single_prefix_and_short_writes`
3. `diagnostic_log::sink::worker::oversized_encoding_tests::oversized_partial_write_zero_and_flush_failures_attempt_next_sink_and_release_storage`
4. `diagnostic_log::sink::worker::oversized_encoding_tests::tiny_and_exact_byte_limits_reuse_one_prefix_and_preserve_variable_years`
5. `diagnostic_log::sink::worker::oversized_encoding_tests::oversized_console_guard_releases_before_file_delivery_on_success_and_failure`

| Canonical path | Raw SHA-256 | LF SHA-256 | Bytes |
| --- | --- | --- | ---: |
| `zircon_runtime/src/diagnostic_log/sink/worker.rs` | `5eaca01762934f62938a9caa4315e90d3a8dca18b901bfaf359c6ab8029881e3` | `349c2afa8ecf7e37254c667c81c3598ed098e305fd54b7ce89dc0b998a9bed07` | 25,697 |
| `zircon_runtime/src/diagnostic_log/sink.rs` | `5a10a21549eddbc2bafbd5f89b32f228bfca8295c233ac49c04f320e75c00c3a` | `23061599c03ead8f9fadac9c4c09e3007e32b613dcdb4d8b749c3abeba718d08` | 25,309 |
| `zircon_runtime/src/diagnostic_log/timestamp.rs` | `eb5cd27b82d8ccc1226f925fd2130786126e8d7512e12628b618f6f94fff697a` | `47e79ee51028fc3d57d00c6e36ac5d67c285ccb32a6c9c3dfa6270fdb29be5f6` | 4,198 |
| `zircon_runtime/src/diagnostic_log/sink/worker/oversized_encoding_tests.rs` (new) | `e4ca28fc048b23881afce0f00572fa97585592faf7da8668597ead0e9cba4189` | `e4ca28fc048b23881afce0f00572fa97585592faf7da8668597ead0e9cba4189` | 20,878 |

## Empty-prefix filter parity

The compiled filter now initializes the root predicate from the stored empty-prefix rule and uses it for the root-only return. This restores the public empty-prefix behavior while leaving insertion, traversal, parser and API shape unchanged. Matching remains byte-prefix based; duplicate empty-prefix rules keep last-wins behavior; literal `*` remains literal; and the parser still rejects an empty prefix. This does not close the separate Runtime44 P1-15 or P1-16 items.

Four ordinary, non-ignored cases were added and await Native execution:

1. `diagnostic_log::level::tests::compiled_empty_prefix_overrides_global_for_all_scopes_and_levels`
2. `diagnostic_log::level::tests::compiled_empty_prefix_is_overridden_only_by_longer_literal_byte_prefixes`
3. `diagnostic_log::level::tests::compiled_duplicate_empty_prefix_rules_preserve_last_wins`
4. `diagnostic_log::level::tests::filter_parser_rejects_empty_prefix_and_preserves_literal_wildcards`

| Canonical path | Raw / LF SHA-256 | Bytes |
| --- | --- | ---: |
| `zircon_runtime/src/diagnostic_log/level/compiled.rs` | `6ef39fcc548690d83ecda7bb12945eae6a691406c8f939e8c5f7d70b9dd848e6` | 2,400 |
| `zircon_runtime/src/diagnostic_log/tests/level.rs` | `b22eac5888c802a16d840cf3f99112e17589948262d5cebb26d66375154f7eea` | 11,235 |

## Actual Source adoption and remaining gates

All six actual afterimages in these two packets match their frozen raw SHA, LF SHA and byte count; the combined receipt records the 12-file strict apply with physical postimages matched and index unchanged. The two new Source files in the combined adoption are this oversized test module and the Editor binding-owner regression module.

- [x] Source adopted for the four oversized-encoding files and two empty-prefix files.
- [ ] Runtime07 Native selectors and the original Runtime44 ordinary/performance contracts; no Rust tests or performance samples are reported here.
- [ ] P1-13, P1-15, P1-16 and P1-24 scopes listed above remain open.
- [ ] Runtime products, Jenkins and whole-milestone acceptance.

## Frozen packets and combined receipt

- [Oversized v2 candidate patch](../../../../../.codex/outbox/runtime44-oversized-encoding-repair-v1/candidate-v2.patch) — SHA-256 `768527be4f2b95c1ad85693624d851992897cd9517ed10c1a4fd3bee7f9594b7`, 33,127 bytes.
- [Oversized v2 source binding](../../../../../.codex/outbox/runtime44-oversized-encoding-repair-v1/source-binding-v2.json) — SHA-256 `5be7e0139cfc5808778f37df037c4dab6a71c898661f24555a0660396eeeeabe`, 13,592 bytes.
- [Oversized v2 source proof](../../../../../.codex/outbox/runtime44-oversized-encoding-repair-v1/source-contract-proof-v2.json) — SHA-256 `0fd03a4e2c2b06356fd3c59c537e6a9c625b66080a18ec4cd773c423ca8da707`, 18,494 bytes.
- [Oversized v2 Native selectors](../../../../../.codex/outbox/runtime44-oversized-encoding-repair-v1/native-test-entrypoints-v2.json) — SHA-256 `72276998f713db43c0dd4f080c4247f0d7a486b673985d4a2dae22eb9353a6ae`, 5,296 bytes.
- [Independent v2 peer review](../../../../../.codex/outbox/runtime44-oversized-encoding-repair-v1/independent-review-v1.json) — SHA-256 `896b1f8dfec5c548aa0ddda044a7468acf82783c1352356971bfee1eb1075ed9`, 13,337 bytes; reviewed v2, not v1.
- [Empty-prefix candidate patch](../../../../../.codex/outbox/runtime44-empty-prefix-filter-parity-repair-v1/candidate-v1.patch) — SHA-256 `0e863da658e634eadbb4760a0fb0660cb203b4125ee278670964d594c3338bbd`, 8,245 bytes.
- [Empty-prefix source binding](../../../../../.codex/outbox/runtime44-empty-prefix-filter-parity-repair-v1/source-binding-v1.json) — SHA-256 `386c7907139c0ba2f8e29df88b6dfe82b4d67199f50f2ece307d095f1786df57`, 7,204 bytes.
- [Empty-prefix source proof](../../../../../.codex/outbox/runtime44-empty-prefix-filter-parity-repair-v1/source-contract-proof-v1.json) — SHA-256 `5a8e6435f88f907631d4857ea9bd1d1a9cfa542edb2cfab23c08b58848508c61`, 10,072 bytes.
- [Empty-prefix Native selectors](../../../../../.codex/outbox/runtime44-empty-prefix-filter-parity-repair-v1/native-test-entrypoints-v1.json) — SHA-256 `eb4b6205906a54bed6c4b82d4320d9e110772b42566791a511fd66201af5496d`, 6,478 bytes.
- [Empty-prefix independent peer review](../../../../../.codex/outbox/runtime44-empty-prefix-filter-parity-repair-v1/independent-review-v1.json) — SHA-256 `b73f9c0b538ad6344d1ec1b9af6a14b543b7262eb0967ff515f452c6129059fe`, 10,390 bytes.
- [Combined actual adoption receipt](../../../../../.codex/outbox/runtime-editor-public-contract-repairs-actual-adoption-20261012-v1/actual-canonical-apply-v1.json) — SHA-256 `e7c5bccb8e948bd5bf448266fb2e49c826245806b6b9bc9c1d5b9dcea2547e80`, 30,354 bytes.
