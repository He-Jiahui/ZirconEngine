---
handoff_kind: failure
status: open
created_at: 2026-08-25
summary_slug: engine-compatibility-caret-range-exhaustiveness
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/optimize/zircon_editor/51
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/project/engine_compatibility/directional_range.rs
tests:
  - .\tools\build-editor.ps1 -Ephemeral
---

# Editor51: engine compatibility caret range match is not exhaustive

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M6 current-source Windows Editor product build and visual-acceptance gate
- 修复责任计划：`docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md`
- 交接原因：Editor51 owns project engine/BuildSet compatibility preflight. The failing file is a new untracked current-worktree implementation under that contract; UI12 does not own semantic-version range parsing.

## 失败现象与复现证据

On 2026-08-25, `\.\tools\build-editor.ps1 -Ephemeral` entered the managed Windows Cargo lane at target `F:\cargo-targets\zircon-engine\ephemeral\check\ba774352932c4e7daea94f508ab8064d` and failed with one compiler error:

```text
error[E0004]: non-exhaustive patterns: `(1_u64..=u64::MAX, Some(_), _)` not covered
  --> zircon_runtime_interface\src\project\engine_compatibility\directional_range.rs:178:23
```

The upper-layer expected path was the normal Editor product build through `zircon_app`; no UI-specific fallback is acceptable. Plausible lower support layers were comparator parsing, comparator-to-range projection, caret upper-bound selection, and Editor51 project preflight integration. The compiler proves the lowest failing layer is `caret_range` upper-bound selection before any Editor/UI code can be validated.

## 最低共享层根因

`caret_range` matches `(base.major, minor, patch)` and uses a guard arm `(major, Some(_), _) if major > 0`. Rust exhaustiveness checking does not treat that guard as covering every `major >= 1` value, so the match remains non-exhaustive for `(1..=u64::MAX, Some(_), _)`. The repair must express the nonzero-major domain as an exhaustive pattern while preserving SemVer caret behavior for `^1`, `^1.2`, `^0.2`, `^0.0.3`, and overflow boundaries.

## 架构修复验收

- Add focused lower-layer range tests for nonzero-major caret comparators, zero-major/minor/patch cases, and checked overflow behavior.
- Compile and run the focused `zircon_runtime_interface` engine-compatibility tests through the managed Windows validator.
- Re-run `\.\tools\build-editor.ps1 -Ephemeral`; UI12 can resume the current-source product screenshot gate only after the normal `zircon_app` path passes this lower layer.

## 禁止临时方案

- Do not add a wildcard that silently returns `None`, weaken compatibility decisions, or bypass preflight in Editor/App.
- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or UI12 product acceptance criteria to hide the failure.

## 修复结果与回传

### 2026-08-27 current-source continuation

- The untracked current-source implementation now expresses the nonzero-major domain as the exhaustive pattern `(1.., Some(_), _)`, with no `if major > 0` guard. The zero-major nonzero-minor, zero-minor patch, omitted patch, and checked `u64::MAX` overflow branches remain explicit.
- Focused source inspection confirms four regression tests cover `^1.2`, `^0.2`, `^0.0.3`, and maximum-major overflow. `rustfmt +1.94.1 --edition 2021 --check zircon_runtime_interface/src/project/engine_compatibility/directional_range.rs` passes, and a structured static contract verifies every required branch/test marker.
- Managed compilation cannot yet be claimed: the shared validator currently fails closed before Cargo acquire with `unmanaged_artifacts_detected` for the foreign path `D:\\ZirconBuilds\\tooling15-local-benchmarks` and an empty `cleanupReservations` list. This continuation does not delete or adopt that Tooling15 artifact and does not retry under unchanged governance state.

Open state: `部分推进，待受管编译与 Editor 产品回归`; no fixed/return claim is made.

### 2026-08-27 managed focused validation

- Artifact governance is no longer the admission blocker: the fresh audit returned
  `unmanaged: []`, and the managed validator started Cargo job
  `afdeb83a911b4aff83f04a719aad658b`.
- The job ran `cargo test -p zircon_runtime_interface --locked --lib caret_`, reached
  real `rustc`, then released normally with wrapper exit `1` / Cargo exit `101`.
- Compilation stopped before the caret tests on nine foreign current-source lib-test
  errors: missing public projections for `UiHitRouteNode` and
  `UiTextShapeArtifact`, crate-name imports inside `binding_value_contracts.rs`, two
  `ProjectManifestSummary` initializers missing `project_guid`, and one stale
  `UiTextShapeArtifact::as_ref` call. No diagnostic names `directional_range.rs`.
- The focused caret test count is therefore zero. This is forward compile evidence,
  not a GREEN result; the failure remains open until those lower-layer owners restore
  the package test boundary and the same managed filter executes successfully.
