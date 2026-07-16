# Runtime 15 Overbudget Owner Splits Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the current Runtime 02 / Runtime 15 priority structure gates by accepting the six completed owner splits, correcting the remaining Physics guard anchor, and proving the current worktree passes both priority guard suites.

**Architecture:** Existing production and test owners already use folder-backed child modules and are below their budgets. The only source correction is in the structure-convention guard: it must validate the real `collider_shape.rs` prefix-plus-suffix projection instead of a nonexistent formatted variable name. Current-source standalone harnesses provide authoritative validation without compiling unrelated runtime dependencies; Runtime 02 evidence remains owned by its numbered child archive.

**Tech Stack:** Rust 2021 test harnesses, PowerShell, MSVC `VsDevCmd`, Zircon session coordinator, Markdown plan evidence.

---

## File map

- Modify `zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/scene_world_property_access.rs`: align the collider-shape child anchor with the current macro implementation while retaining all semantic variant checks.
- Verify `zircon_runtime/src/tests/runtime_absorption/structure_convention/render_graph_execution_record.rs`: keep the current parent/child routing that reads `compute_workload/tests.rs`; no edit unless a newly compiled harness contradicts the current source.
- Verify the six current owner roots and their child modules under `zircon_runtime/src/graphics/scene/scene_renderer` and `zircon_runtime/src/text/layout`: no reconstruction or duplicate split.
- Create temporarily and delete `tools/.tmp_runtime02_structure_harness.rs`: restore the crate module path required to compile the current structure-convention sources standalone.
- Create temporarily and delete `tools/.tmp_runtime02_review_harness.rs`: compile the current code-review-findings sources standalone.
- Update `docs/plans/zircon_runtime/runtime/02/2026-07-11-runtime02-current-cargo-baseline.md`: append one canonical current-state evidence record only after coordinator authorization and successful validation.
- Preserve `docs/superpowers/specs/2026-07-12-runtime15-overbudget-owner-splits-design.md`: approved design source; do not duplicate concrete Runtime 02 output rows here.

## Milestone M1: Current owner-boundary guard convergence

### Goal

Make the Runtime 15 structure guard describe the already-implemented folder-backed ownership accurately, without changing production behavior or weakening any budget or semantic requirement.

### In-scope behaviors

- The Physics entry parent delegates shape projection to `entries/collider_shape.rs`.
- The collider-shape child must still prove Box, Sphere, and Capsule coverage.
- The child must prove that property paths are built from the supplied prefix and the emitted suffix.
- The compute-workload guard must read test anchors from `compute_workload/tests.rs` and production anchors from `compute_workload.rs`.
- All six previously overbudget roots remain below their existing budgets.

### Dependencies

- The approved design in `docs/superpowers/specs/2026-07-12-runtime15-overbudget-owner-splits-design.md`.
- Current owner splits already present in the shared worktree.
- Exact-path leases before any edit.

### Implementation slices

- [ ] **M1-S1: Claim and re-read the Physics guard owner**

  Claim only:

  ```powershell
  python -m tools.session_coordinator lease claim `
    --session-id 019f48bc-360c-75b2-b57f-a093b467cf0c `
    zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/scene_world_property_access.rs
  ```

  Re-read both the guard and `scene/world/property_access/entries/collider_shape.rs`. Stop if a foreign lease or a newer equivalent fix is present.

- [ ] **M1-S2: Replace the stale collider path anchor**

  In `runtime_15_scene_world_property_access_physics_entries_are_child_owner`, retain the three required variants and replace the nonexistent `"{path_prefix}.kind"` anchor with the two real macro-contract anchors:

  ```rust
  assert_contains_all(
      "collider-shape child owns shape-specific property-entry projection",
      &collider_shape,
      &[
          "ColliderShape::Box",
          "ColliderShape::Sphere",
          "ColliderShape::Capsule",
          r#"let path = format!("{prefix}.{}", $suffix);"#,
          r#"push_shape_entry!("kind""#,
      ],
  );
  ```

  Do not modify `collider_shape.rs`; its current prefix-plus-suffix behavior is the production authority.

- [ ] **M1-S3: Confirm compute-workload parent/child routing without editing**

  Confirm `render_graph_execution_record.rs` reads:

  ```rust
  let compute_workload_tests = read_runtime_src(
      "graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload/tests.rs",
  );
  ```

  and applies the three moved test anchors to `&compute_workload_tests`, not `&compute_workload`. If this exact routing is already present, record the slice as verified and make no source change.

- [ ] **M1-S4: Lightweight syntax and whitespace checks**

  Run:

  ```powershell
  rustfmt --edition 2021 zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/scene_world_property_access.rs
  git diff --check -- zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/scene_world_property_access.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/render_graph_execution_record.rs
  ```

  Expected: both commands exit 0; no unrelated file is formatted.

### Testing stage: M1 structure acceptance

- [ ] **M1-T1: Compile the current structure harness**

  Create `tools/.tmp_runtime02_structure_harness.rs` with:

  ```rust
  #![allow(dead_code, unused_imports)]

  mod tests {
      pub mod runtime_absorption {
          #[path = "E:/Git/ZirconEngine/zircon_runtime/src/tests/runtime_absorption/structure_convention.rs"]
          pub mod structure_convention;
      }
  }
  ```

  Then run from a Windows MSVC environment:

  ```powershell
  $env:CARGO_MANIFEST_DIR='E:\Git\ZirconEngine\zircon_runtime'
  $cmd='call "E:\Visual Studio\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustc --crate-name runtime02_structure_harness --edition=2021 --test tools\.tmp_runtime02_structure_harness.rs -o D:\cargo-targets\runtime02-structure-standalone-20260712\runtime02_structure_current.exe'
  & $env:ComSpec /d /s /c $cmd
  ```

  Expected: exit 0 and a newly written `runtime02_structure_current.exe`. Delete the temporary wrapper with `apply_patch` immediately after compilation.

- [ ] **M1-T2: Run the seven historical exact gates**

  Run the current executable with `--exact` for:

  ```text
  tests::runtime_absorption::structure_convention::production_file_budget::global_budget::runtime_15_no_oversized_production_files
  tests::runtime_absorption::structure_convention::production_file_budget::render_pass_gpu_context_mesh_command_lists::runtime_15_render_pass_gpu_context_mesh_command_lists_are_child_owner
  tests::runtime_absorption::structure_convention::production_file_budget::render_shader_template_assembly::runtime_15_render_shader_template_assembly_is_folder_backed
  tests::runtime_absorption::structure_convention::production_file_budget::render_ui_screen_space_render::runtime_15_screen_space_ui_render_tests_are_child_owner_split
  tests::runtime_absorption::structure_convention::production_file_budget::scene_world_property_access::runtime_15_scene_world_property_access_physics_entries_are_child_owner
  tests::runtime_absorption::structure_convention::render_graph_execution_record::runtime_15_render_graph_execution_record_is_folder_backed
  tests::runtime_absorption::structure_convention::test_file_budget::global_budget::runtime_15_no_oversized_test_files
  ```

  Expected: 1 passed for each exact invocation and no failure.

- [ ] **M1-T3: Run the complete structure suite**

  Run:

  ```powershell
  & 'D:\cargo-targets\runtime02-structure-standalone-20260712\runtime02_structure_current.exe' --test-threads=1
  ```

  Expected: `1304 passed; 0 failed`.

- [ ] **M1-T4: Debug/correction loop**

  If any gate fails, use the panic's exact owner path and anchor as the lowest-layer diagnosis. Re-read the current child source, correct only the stale guard route or anchor, recompile the standalone harness, rerun the failed exact test, then rerun the complete suite. Do not change budgets, add allowlists, or alter production behavior to satisfy a textual guard.

### Exit evidence

- Physics and compute exact gates pass from a newly compiled current-source harness.
- The global production and test budgets pass.
- The complete structure suite reports 1304/1304.
- The touched guard diff contains no production code and no expectation weakening.

## Milestone M2: Priority review regression and Runtime 02 evidence

### Goal

Prove the structure repair preserved the priority code-review findings and record one authoritative Runtime 02 current-state result.

### In-scope behaviors

- Current code-review findings remain 80/80.
- The evidence distinguishes current-source passes from older stale binaries.
- Runtime 02 remains `in_progress` unless every broader Cargo/editor/plugin gate is also authoritative.

### Dependencies

- M1 complete with 1304/1304 structure evidence.
- Coordinator authorization for the numbered Runtime 02 output record.

### Implementation slices

- [ ] **M2-S1: Compile the current code-review harness**

  Create `tools/.tmp_runtime02_review_harness.rs` with:

  ```rust
  #![allow(dead_code, unused_imports)]

  mod tests {
      pub mod runtime_absorption {
          #[path = "E:/Git/ZirconEngine/zircon_runtime/src/tests/runtime_absorption/code_review_findings.rs"]
          pub mod code_review_findings;
      }
  }
  ```

  Compile it with:

  ```powershell
  $env:CARGO_MANIFEST_DIR='E:\Git\ZirconEngine\zircon_runtime'
  $cmd='call "E:\Visual Studio\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >nul && rustc --crate-name runtime02_review_harness --edition=2021 --test tools\.tmp_runtime02_review_harness.rs -o D:\cargo-targets\runtime02-structure-standalone-20260712\runtime02_review_current.exe'
  & $env:ComSpec /d /s /c $cmd
  ```

  Expected: exit 0. Delete the temporary wrapper with `apply_patch` immediately after compilation.

- [ ] **M2-S2: Run the complete priority review suite**

  Run:

  ```powershell
  & 'D:\cargo-targets\runtime02-structure-standalone-20260712\runtime02_review_current.exe' --test-threads=1
  ```

  Expected: `80 passed; 0 failed`.

- [ ] **M2-S3: Record canonical Runtime 02 evidence**

  Ask the coordinator to authorize `docs/plans/zircon_runtime/runtime/02/2026-07-11-runtime02-current-cargo-baseline.md`. Recount the Runtime 02 records first. Append exactly one dated record containing:

  ```text
  current structure harness: 1304/1304 passed
  current code-review-findings harness: 80/80 passed
  six former overbudget owners: below their existing budgets
  stale-guard corrections: Physics collider-shape child and compute-workload test child validated
  Runtime 02 status: in_progress pending broader Cargo/editor/plugin gates
  ```

  Do not write this concrete evidence into `runtime/index.md`, either `engine-code-*.md`, the approved design spec, or a session note.

### Testing stage: M2 evidence acceptance

- [ ] **M2-T1: Evidence and output-routing checks**

  Run:

  ```powershell
  python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:\Git\ZirconEngine
  git diff --check -- docs/plans/zircon_runtime/runtime/02 docs/superpowers/specs docs/superpowers/plans zircon_runtime/src/tests/runtime_absorption/structure_convention
  ```

  Expected: the output-record audit reports no new routing risk attributable to this slice, and `git diff --check` exits 0.

- [ ] **M2-T2: Acceptance review**

  Confirm the evidence names the newly compiled executables and current pass counts, does not claim the broader Runtime 02 milestone complete, and does not stage unrelated dirty paths. If an evidence audit finds a pre-existing unrelated issue, record it separately and keep this slice's claim limited to the two priority guard suites.

### Exit evidence

- Current-source structure suite: 1304/1304.
- Current-source code-review suite: 80/80.
- Runtime 02 numbered output record contains one current canonical result.
- Runtime 02 correctly remains `in_progress` until its remaining acceptance gates close.

## 状态与产出记录

执行时逐切片填写；完成一个切片更新一行，不许批量补记。具体执行证据的 canonical owner 是 Runtime 02 编号子计划/归档，本节只跟踪本实施计划的执行状态。

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
