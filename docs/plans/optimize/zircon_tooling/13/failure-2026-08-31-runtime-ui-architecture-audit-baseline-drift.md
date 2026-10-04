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

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 来源执行切片：Runtime Interface, Runtime UI and Editor asset-palette static contracts.
- 修复责任计划：`docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md`
- 交接原因：Tooling13 owns the structural audit vocabulary, inventory and mirrored guards.

## 失败现象与复现证据

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

## 最低共享层根因

The audit's exact inventory and source-scan metrics no longer match the reviewed
production owners. Raw legacy substring counting also conflates retired API debt
with benchmark comparison labels. Its mirrored Rust tests and documents must
share the reviewed meaning and current source inventory before acceptance.

## 架构修复验收

- Reconcile the three removed template source entries with the current template owner layout.
- Register the exact current `ui/` top-level owner map without allowing arbitrary additions.
- Diagnose the 331 production `legacy` hits and 34 production files. Do not merely promote those
  values to a green baseline; either narrow the scanner to the intended migration vocabulary or
  route real legacy debt to its owning plans.
- Reconcile the taffy scan with current intended production ownership and update mirrored Runtime09
  documentation only after the meaning of the metric is stable.
- Return `test_runtime_ui_architecture_boundary` green, then rerun the same 444-test static batch.

## 禁止临时方案

- RuntimeInterface03 will not expand the structural-audit engine or normalize migration debt.
- The two pre-existing current-source doc-anchor updates in the audit script must be preserved.
- This failure does not invalidate the 443 passing contracts or the eight repaired module-split
  guards; it blocks claiming the full static batch green.

## 修复结果与回传

State: `existing-source-repair_current-regression-and-union-validation-pending`.
This same lifecycle is now stored in its declared Tooling13 child directory.
Audited transfer
`9dbd6f2e428bd0f25ab9adc7c1dfef711aab50f0c6982c42f2f7c5cc62b7b98c`
adopted the original record from its archived owner; preimage snapshot 3206
preserves hash `106583c3a527f0c07beba3cc0d96945a1348bfa04a183e000f589ab53a502608`.
Origin/fixing identities, created_at, summary_slug and original evidence remain.
No failure return or lifecycle closure is implied by the placement repair.

The later [audit contract scope](failure-2026-09-01-runtime-ui-architecture-audit-contract-scope.md)
and [Rust guard contract](failure-2026-09-01-runtime-ui-architecture-rust-guard-contract.md)
retain their existing source snapshots and pending ticket owners. Current audit
hash is `1ac85653a6313872639adb974ceec02fdfcb1fa434f0135cc57ee82967340c1d`;
Python test hash is
`29c14c29edccdc09ee105a532aa16926263d16ad1aed1e1f705e77baae48a915`.
Both match the archived implementation owner's attribution; no source is changed
by this record correction.

Fresh current-source Python execution on 2026-09-08 ran four tests in 8.923
seconds: three passed, one failed. Log:
`.codex/tmp/tooling13-ui-architecture-current-20260908.log`.
The exact inventory rejects the new `ui/module` directory as an unexpected entry.
The retired-vocabulary and real Rust-declaration guards passed. The current owner
change and any remaining scan/mirror mismatches need diagnosis before updating
the baseline; the prior four-test pass cannot certify today's source.

## 2026-09-19 rolling repair: current-source baseline recheck

The stable fixing Session `failure-roll-01a084c8-tooling13-ui-baseline-r1` acquired
the record and its two executable audit inputs through the coordinator. Snapshot
`3559` froze the pre-update scope at baseline epoch `611`; the source manifest was:

- `ui_architecture_boundary.py`: `4afc45b2e89edd65980986a8c25f21b7b89427b9cb001c7af92315b37a7c54f8`;
- `test_runtime_ui_architecture_boundary.py`: `4549a11fc953718b813d4db0880f4d126a39ac67bc7765d6fbcd0f9b3da26481`;
- this failure record: `15edee3f5519b18bfe47a023972405b98d5b555dece1a696cd2b940881197895`.

No production or audit source was edited in this pass. Against those current hashes,
`python -B -m unittest tools.tests.test_runtime_ui_architecture_boundary -v`
executed all `4/4` tests in `8.975s` with exit `0`. The direct audit returned no
risks and the synchronized current metrics: source inventory `49`, `ui/` entries
`23`, `surface/` entries `45`, full legacy hits `15`, production legacy hits/files
`0/0`, and production Taffy hits/files `254/16`. The scoped `git diff --check`
returned exit `0` (only the repository's line-ending normalization warnings).

This recheck supersedes the stale 2026-09-08 observation that `ui/module` was an
unexpected entry for the current source. It is static evidence only: the complete
444-test union, Runtime09 Rust mirror/Cargo gates, independent review, canonical
failure return, and closeout remain pending. The lifecycle therefore stays `open`;
no fixed record or acceptance is claimed from the four-test recheck.
