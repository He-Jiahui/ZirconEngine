---
handoff_kind: failure
status: open
created_at: 2026-08-31
summary_slug: runtime-ui-architecture-audit-baseline-drift
origin_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
fixing_plan: docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime_interface/03
fixing_child_dir: docs/plans/optimize/zircon_tooling/13
plan_link_mode: child_record_only
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
tests:
  - python -m unittest tools.tests.test_runtime_ui_architecture_boundary
---

# RuntimeInterface03: Runtime UI architecture audit baselines are stale

## Failure receipt

RuntimeInterface03 ran the batched static Runtime Interface, Runtime UI and Editor asset-palette
contract suite after repairing eight module-split guard paths. The batch completed 443/444 tests;
the only failure was
`test_runtime_ui_architecture_boundary.py::test_current_text_surface_leaves_and_index_route_are_mirrored`.

The first failure was a fixed `surface/` inventory that omitted 18 current top-level owners. The
inventory was synchronized exactly from 26 to 44 entries without weakening the unexpected-entry
guard. That exposed three older risks previously hidden by the first assertion:

- source inventory still names three removed flat files:
  `ui/template/pipeline.rs`, `ui/template/loader.rs`, and `ui/template/validate.rs`;
- `ui/` inventory omits `secure_text_policy.rs` and the `style/` owner directory;
- source-scan baselines have materially diverged: full `legacy` hits `70 -> 1663`, production
  `legacy` hits `0 -> 331`, production legacy files `0 -> 34`, production `taffy` hits
  `175 -> 200`, and production taffy files `10 -> 14`.

Current hashes after the exact surface inventory synchronization:

- audit script SHA-256:
  `75928811C1A55BFC73475705DA06748CFE47E8F4328AB8281EDFBBD6D5269A53`;
- static test SHA-256:
  `A9C767B4423910B2423F227AE24C2F970EF1BB1BA2C24F614106FB3824DBDE98`.

## Acceptance

- Reconcile the three removed template source entries with the current template owner layout.
- Register the exact current `ui/` top-level owner map without allowing arbitrary additions.
- Diagnose the 331 production `legacy` hits and 34 production files. Do not merely promote those
  values to a green baseline; either narrow the scanner to the intended migration vocabulary or
  route real legacy debt to its owning plans.
- Reconcile the taffy scan with current intended production ownership and update mirrored Runtime09
  documentation only after the meaning of the metric is stable.
- Return `test_runtime_ui_architecture_boundary` green, then rerun the same 444-test static batch.

## Constraints

- RuntimeInterface03 will not expand the structural-audit engine or normalize migration debt.
- The two pre-existing current-source doc-anchor updates in the audit script must be preserved.
- This failure does not invalidate the 443 passing contracts or the eight repaired module-split
  guards; it blocks claiming the full static batch green.
