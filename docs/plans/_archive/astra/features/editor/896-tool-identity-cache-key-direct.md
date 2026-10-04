---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/15/2026-09-21-tool-identity-cache-key-direct.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/export/inventory.rs
tests:
  - zircon_editor/src/core/export/inventory/tool_cache_key_tests.rs
  - tools/tests/test_editor896_tool_identity_cache_key_direct_performance_contract.py
---

# Editor896 Tool Identity Cache Key Direct

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor15 export tool-version inventory | Append version arguments directly to the final tool cache key instead of joining a full child string; preserve `OsStr` debug text, empty arguments, Unicode, and embedded NULs. | Combined RED `1/8` -> GREEN `8/8`; adjacent `48/48`. For 4,096 keys the deterministic model removes `4096` joined children. Lower byte-parity tests and ignored 101-pair `EDITOR896_TOOL_IDENTITY_CACHE_KEY_DIRECT_BENCH_V1` are wired. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/export/inventory.rs` | `2855644FA027FE094BC2DF1D2A79447F8259F5F687FEEC7D843EA83251A7720F` |
| `zircon_editor/src/core/export/inventory/tool_cache_key_tests.rs` | `EF25988EF88C00F47CFF0F355E89ED87B07DBAF50C6D442AA539F8EA14B40E25` |
| `tools/tests/test_editor896_tool_identity_cache_key_direct_performance_contract.py` | `8B2DDDF53E8BA95100F19DB20ABD8A15A1C40BF7ABB2008C873C899C4FCF2A6A` |

## Managed gate

Editor896 was submitted with Runtime877 in combined current-source v24 (PID
`33516`) at `2026-09-21T23:12:59.6906963+08:00`. One bounded receipt read
after independent work confirms Runtime compiled while Editor/App admissions
failed because the reuse pool was busy; no Editor source error is attributed.
Editor compilation, Rust lower/ignored Release tests, allocator measurement,
and export-product p50/p95/p99 remain pending.
