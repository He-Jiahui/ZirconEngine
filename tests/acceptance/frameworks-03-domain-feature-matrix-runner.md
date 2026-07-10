# Frameworks 03 Domain Feature Matrix Runner Acceptance

## Scope

This slice implements the M1 single-domain check runner and the documented profile-to-Cargo preset mapping. It does not claim that the full domain matrix passes.

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

## Negative Execution Evidence

`tools/check-runtime-domain-features.ps1 -Feature physics-contracts -Offline` currently reaches Cargo, reports that `zircon_runtime` does not contain `physics-contracts`, records the failed feature, and exits 1. This is the expected M1 RED state and proves the runner does not silently omit the pending Physics domain.

## Remaining Gate

- Land the `physics-contracts` hard cutover after the active Physics manager/Jolt and shared scene/asset owners release their files.
- Run all twelve WSL nightly domain checks and record their individual results.
- Complete the Runtime/App package test gates before declaring Frameworks 03 M1 complete.
