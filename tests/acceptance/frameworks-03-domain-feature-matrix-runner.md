---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/graphics/text
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/rich_text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/sdf_fallback.rs
  - zircon_runtime/src/graphics/types/mod.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
implementation_files:
  - tools/check-runtime-domain-features.ps1
  - tools/tests/test_frameworks_03_domain_feature_matrix.py
  - tools/tests/test_frameworks_03_contract_feature_boundary.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python -m unittest tools.tests.test_frameworks_03_contract_feature_boundary -v
  - tools/check-runtime-domain-features.ps1 -Toolchain nightly -TargetDir F:\cargo-targets\zircon-frameworks03-domain-matrix-0711 -Jobs 1 -Offline
doc_type: acceptance-evidence
status: green-twelve-domain-matrix-runtime-app-full-tests-pending
---

# Frameworks 03 Domain Feature Matrix Runner Acceptance

## Scope

This slice implements and validates the M1 single-domain check runner and the documented profile-to-Cargo preset mapping. The complete twelve-domain matrix now passes; this record does not claim the Runtime/App full test gates or Frameworks 03 M1 are complete.

Implemented surfaces:

- `tools/check-runtime-domain-features.ps1` owns the exact twelve-domain list and runs each selected domain as `core-min + one domain` with `--lib`, `--no-default-features`, and `--locked`.
- The runner accumulates failed domains and exits nonzero. There is no missing-feature skip path.
- `CLAUDE.md` exposes the server check and domain runner as common commands.
- `docs/runtime-plugins/profile-selection.md` maps the six runtime profiles to the current M1 Cargo presets and explicitly leaves M2 single-source generation pending.
- `tools/tests/test_frameworks_03_domain_feature_matrix.py` fixes the exact feature list, Cargo command shape, and failure behavior.

## TDD Evidence

RED:

- `python -m unittest tools.tests.test_frameworks_03_domain_feature_matrix` failed because `tools/check-runtime-domain-features.ps1` did not exist.

GREEN:

- The same focused suite passes 3/3 after the runner was added.
- The combined Frameworks 03/05 static suite passes 24/24:
  - server boundary: 5
  - contract boundary: 13
  - domain matrix runner: 3
  - asset/UI boundary: 3
- PowerShell parses the runner successfully through `[scriptblock]::Create(...)`.
- Python compilation and scoped `git diff --check` pass.

## Initial Negative Execution Evidence

Before the Physics hard cut, `tools/check-runtime-domain-features.ps1 -Feature physics-contracts -Offline` reached Cargo, reported that `zircon_runtime` did not contain `physics-contracts`, recorded the failed feature, and exited 1. This proved the runner did not silently omit a pending domain.

## Full Matrix Evidence

Fresh 2026-07-11 execution used the dedicated external target `F:\cargo-targets\zircon-frameworks03-domain-matrix-0711` with nightly, locked, offline, no-default-features, one job, and `core-min + one domain` per invocation.

The first complete run passed 11/12 domains and failed only standalone `graphics`. UI had passed because its feature preset also activates Graphics, which had hidden four reverse dependencies from Graphics into `crate::ui`:

- font render-mode resolution;
- bidi direction resolution;
- rich inline layout;
- public runtime-frame conversion.

The production cut removed those reverse dependencies at their lowest owners: shared render-mode resolution is now a neutral runtime-interface contract; Graphics bidi/rich layout consumes `graphics::text`; the obsolete UI shaper bridge was deleted; public runtime-frame conversion is mounted only when `ui` is enabled. No alias, compatibility module, or feature widening was added.

Fresh verification after the cut:

- standalone `core-min,graphics`: passed in 2m39s;
- Frameworks 03 contract boundary suite: 20/20 passed in 26.433s;
- complete domain runner: 12/12 passed in 25m02.4s for AI, Animation, Diagnostic Log, Dynamic API, Graphics, Navigation, Net, Physics, Script, Sound, Text, and UI.

## Remaining Gate

- Complete the Runtime/App package test gates before declaring Frameworks 03 M1 complete.
- Implement M2 profile-preset single source and CI matrix after the M1 testing gate is closed.
