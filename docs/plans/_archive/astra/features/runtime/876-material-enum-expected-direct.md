---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/91/2026-09-21-material-enum-expected-direct.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/core/framework/render/shader/material_property_layout.rs
tests:
  - zircon_runtime/src/core/framework/render/shader/material_property_layout/enum_description_tests.rs
  - tools/tests/test_runtime876_material_enum_expected_direct_performance_contract.py
---

# Runtime876 Material Enum Expected Direct

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime91 material-schema mismatch text | Append borrowed enum choices and delimiters into one exact-size `one of ...` output instead of joining a temporary child and formatting again; preserve invalid-value reporting, empty entries, Unicode, bool, and empty-enum cases. | Combined RED `2/10` → GREEN `10/10`; adjacent source contracts `49/49`. The 4,096-invalid-enum deterministic model eliminates `4096` joined child strings; lower parity and ignored 101-pair `RUNTIME876_MATERIAL_ENUM_EXPECTED_DIRECT_BENCH_V1` wired. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/core/framework/render/shader/material_property_layout.rs` | `B3059B36C7944EB9227E67E017AE5BEF995B5213D0D49C62482D81B045BE8DCF` |
| `zircon_runtime/src/core/framework/render/shader/material_property_layout/enum_description_tests.rs` | `19F468DCCCEFED106E7E109267309C4CDFDEC8DF85457B5C0BBEC31ED02331C6` |
| `tools/tests/test_runtime876_material_enum_expected_direct_performance_contract.py` | `0323C299EFAB362079F27A4CD1DC4815620CC4250E5683BD37860D032DBF3418` |

## Managed gate

Runtime876 was submitted with Editor895 in combined current-source v23 (PID
`23296`) at `2026-09-21T22:58:13.7141657+08:00`. No v22/v23 receipt was read
or monitored. Current-source Runtime compilation, lower/ignored Release tests,
allocator measurement, and material-schema product p50/p95/p99 remain pending.
