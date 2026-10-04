---
title: Runtime15 Facade Unused-Import Suppression Hard Cut
date: 2026-09-01
status: source_implemented_static_scope_checked_managed_validation_pending
origin_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
---

# Runtime15 facade unused-import suppression hard cut

## Result

Four production `#[allow(unused_imports)]` attributes were removed from the Asset, Graphics backend,
and Scene facade roots. The cut follows the repository facade rule rather than mechanically keeping
every import:

- `asset/mod.rs` deleted the six-entry `pipeline::types` forwarding group because tracked and
  current-source root-path consumers are zero; existing code already imports the named
  `asset::types` owner.
- The Asset project forwarding group was reduced from nine entries to the five live root-path
  contracts: `AssetMetaDocument`, `AssetMetaError`, `AssetSourceUnit`, `ProjectManager`, and
  `ProjectPaths`. `AssetMetaEntry`, `AssetMetaResult`, `PackageAssetRegistry`, and `PreviewState`
  remain available from their named project owner instead of a stale parent alias.
- The Graphics IBL WGPU readback bridge remains intact because the scene renderer consumes all
  three entries through `crate::graphics::backend::{...}`.
- The public Scene component facade remains intact; it is a deliberate cross-crate API surface and
  does not require an unused-import suppression.

No compatibility alias, replacement wrapper, runtime branch, or behavior change was added.

## Static evidence

- production suppression sites in the three reviewed facades: `4 -> 0`;
- stale Asset root forwarding entries removed: `10`;
- live Asset project root forwarding entries retained: `5`;
- live Graphics bridge entries retained: `3`;
- live Scene public component entries retained: `4`;
- `runtime_15_facades_do_not_hide_unused_imports_or_stale_forwarding_exports` rejects suppression
  restoration and checks the curated surfaces;
- isolated `rustfmt --check --config skip_children=true` passes for the modified Scene root and
  focused regression file;
- `git diff --check` passes for the complete owned source manifest;
- isolated whole-file `rustfmt --check` for the Asset and Graphics backend roots still reports
  pre-existing import-order drift outside this hard cut. Reformatting those large foreign-owned
  surfaces is intentionally excluded from this scoped change.

The source slice does not claim a runtime performance or power improvement. Managed Windows compile
and focused structure behavior validation remain required before coordinator integration, milestone
commit, or WeCom completion notification.

## Completion state

- [x] Reviewed every production `allow(unused_imports)` occurrence in `zircon_runtime/src`.
- [x] Classified actual root-path consumers before editing.
- [x] Deleted stale crate-private Asset forwarding exports.
- [x] Preserved live Graphics and public Scene curated facades.
- [x] Added a focused structural regression.
- [x] Passed owned diff checks and focused formatting for the changed leaf/Scene surfaces.
- [ ] Managed current-source Runtime structure validation accepted.
- [ ] Independent review and coordinator integration completed.
