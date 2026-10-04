# Runtime UI source-entry closure

Status: `source_applied`  
Managed validation: `managed_tests_pending`

The applied source correction separates the 45 raw worktree surface paths from the 43 Rust-source-backed surface modules in the planned v36 composite. The local `navigation/` and `pointer/` directories do not contain Rust source and are not counted as source modules.

The planned composite uses the exact current bytes for all 20 repository-relative source reads in the architecture boundary test, all selected `include_str!` bytes, root-applied G30 v3 and Runtime51 v4 source/record outputs, and root-applied Runtime76 and F2 v8 source/record outputs. Static source metrics are recalculated in the v36 manifest. Existing legacy and product-cache assertions remain unchanged.

No Cargo tests or performance measurements were run. Foreign input ownership/admission and the audit script raw-45 versus sealed source-backed-43 execution gate remain pending.

## Completion list

- [x] Preserve all existing legacy and product-cache assertions; distinguish 45 raw paths from 43 source-backed modules.
- [x] Apply the two reviewed Rust guard corrections and three compile-time document mirrors with exact preimage and lease checks.
- [x] Keep the immutable 768-entry v36 audit map as a historical composite inventory; remove its 253 historical record-only paths from the Root compile-input envelope (515 selected source inputs before application).
- [ ] Admit remaining required foreign current inputs; seal the final current source/record inventories.
- [ ] Run the grouped managed package checks and tests.
- [ ] Resolve the unchanged raw-45 Python audit execution gate on the sealed source tree.

The source count is an inventory metric, not a performance measurement. No compiler result or product performance acceptance is claimed. Root apply receipt: `2026-09-30-runtime-ui-v36-reviewed-exact-guarded-apply.json`.
