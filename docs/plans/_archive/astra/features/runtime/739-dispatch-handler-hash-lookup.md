---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/77-runtime-ui-input-dispatch-routing-focus-navigation-pointer-capture-gesture-drag-drop-ime-window-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_runtime/77/2026-09-13-dispatch-handler-hash-lookup.md
related_code:
  - zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs
  - zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs
  - zircon_runtime_interface/src/ui/surface/pointer/event_kind.rs
  - zircon_runtime_interface/src/ui/surface/navigation/event_kind.rs
  - zircon_runtime_interface/src/ui/dispatch/input/reply.rs
tests:
  - tools/tests/test_runtime_ui_dispatch_hash_lookup_performance_contract.py
  - zircon_runtime/src/ui/tests/event_routing
  - zircon_runtime/src/ui/tests/runtime_input_manager
---

# Runtime Pointer / Navigation Dispatch Handler Hash Lookup

Runtime 的 pointer 与 navigation dispatcher 只按完整 key 查找 handler，原先
`BTreeMap` 的有序查找没有被用于投递顺序，因此改为 `HashMap`。每个 key 下的
handler `Vec` 顺序、route candidate 顺序和 phase 链保持不变；接口枚举新增
`Hash` 派生但保留原有 `Ord` 语义。

## 计划完成列表

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime77 / 11A dispatch hot path | Pointer phase/unqualified and navigation handler tables use exact-key hash lookup | implemented_pending_validation | TDD RED/GREEN source contract `4/4`; batched Runtime UI/input contracts loaded 113 modules and passed `492/492` in `19.899s`, while the corresponding Editor asset/hierarchy/reference batch passed `89/89` across 18 modules in `0.499s`; managed Cargo, Release allocation, and p50/p95/p99 evidence remain pending. |

## 性能边界

注册 key 数为 `H` 时，单次 map probe 从 `O(log H)` 降为 expected `O(1)`；回调执行、
route traversal 与结果构造不变。该记录不把静态合同当作产品延迟达标证据。

## 源码快照

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs` | `2E97DF1D32C54B018AA6E712DF7F765444F98A28E8D91F4F7279B26EF9EE602F` |
| `zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs` | `9CF842AD5A705414A8450A34C43864B099F683BC131FF08DBBCEB8E70ADC6220` |
| `zircon_runtime_interface/src/ui/surface/pointer/event_kind.rs` | `FDCB15656F0C2D5B163819F6DF72239E086A48E6371CC66D7835F6E56A3AF9B6` |
| `zircon_runtime_interface/src/ui/surface/navigation/event_kind.rs` | `539E8C960743EC634124D1EC65AF7174B29DC6CAE0011E272F397A8BCBF5CA61` |
| `zircon_runtime_interface/src/ui/dispatch/input/reply.rs` | `9FC2692AE0E8AAAF1919EE0965C050D9C9ABCD8518F72790B936D58FAE994DA5` |
| `tools/tests/test_runtime_ui_dispatch_hash_lookup_performance_contract.py` | `63BABB083E682FE8E7F866D404D7F34E28A9B5139B94E2E4FF2A7027BAE80AD5` |

## 受管验证

该切片加入现有 Runtime/Editor 合批，不单独启动 Cargo。此前协调器在
`E:\\Git\\zr_vm` 外部脏工作树和 static overlay ownership 门禁处拒绝，故当前
仍为 `implemented_pending_validation`。
