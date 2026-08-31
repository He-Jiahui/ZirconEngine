---
handoff_kind: failure
status: open
failure_scope: cross_plan
created_at: 2026-08-29
summary_slug: validation-copy-missing-copy-command-resource
origin_plan: docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
fixing_plan: docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime/07
fixing_child_dir: docs/plans/optimize/zircon_runtime/90
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_rhi_wgpu/src/command_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/command_validation/copy_commands.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/command_validation/render_state.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/capabilities.rs
tests:
  - cargo +1.94.1 test -p zr_rhi_wgpu --lib --locked --jobs 1
  - cargo +1.94.1 test -p zircon_runtime -p zircon_plugin_sdk -p zircon_plugin_zr_vm_language_runtime --features zircon_plugin_zr_vm_language_runtime/backend-zr-vm --lib --locked --jobs 1
---

# Runtime90: validation copy missing copy-command resource

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md`
- 来源执行切片：Runtime07 three-slice managed validation ticket
  `f6268d8516e34299995f2032e34d6b15`
- 修复责任计划：`docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md`
- 交接原因：Runtime90 owns `zr_rhi_wgpu` command validation, its test-only deterministic
  execution contract, and the active parent-to-child copy-command split.

## 失败现象与复现证据

- The Runtime07 managed ticket failed before Cargo started. Coordinator event `5949` records
  `validation_copy_compile_time_resource_missing` during `closure_planning`.
- The copy-complete retry tickets `beac06b552a04502b7faf55ae4518bb3` and
  `fbe6fd96a8e54b33ab1396f09854cce5` failed at the same stage (events `5965` and `5985`) even
  after snapshot `2347` and all four Runtime90 paths were added as validation dependencies.
- The reported source is
  `zircon_runtime/crates/zr_rhi_wgpu/src/tests/capabilities.rs`; the missing compile-time resource
  is `zircon_runtime/crates/zr_rhi_wgpu/src/command_validation/copy_commands.rs`.
- In current source, `capabilities.rs` has two `include_str!` references to that child file and
  `command_validation.rs` declares `mod copy_commands`. The child file exists but is untracked,
  was untracked when the failures were recorded.
- The parent, sibling state owner, and capability test are also modified as one active Runtime90
  command-validation union. The failed Runtime07 coverage snapshots contain the referring test
  source but no snapshot that materializes the untracked child resource.

## 最低共享层根因

The Runtime90 parent-to-child command-validation split is not copy-complete at closure-planning
time. `CargoInputClosurePlanner` discovers compile-time resources only through `git ls-files`
before ticket overlays are materialized. A tracked test therefore cannot `include_str!` an
untracked child, even when that child has an exact snapshot and appears in the ticket source
manifest. This is not a Runtime07 compile or test failure and Cargo produced no result.

## 架构修复验收

- Reclaim the complete Runtime90 command-validation union without discarding current render,
  compute, indirect, dynamic-offset, and copy-region changes.
- Keep child-specific source-shape guards inside the child owner so tracked sources do not require
  an untracked compile-time resource before overlays are applied. Preserve the same no-clone,
  parent delegation, public helper, and file-budget assertions.
- Attribute and snapshot the child resource together with every referring/owning Runtime90 path.
- Run the focused managed `zr_rhi_wgpu --lib` suite through that copy-complete snapshot.
- Re-run the original Runtime07 managed command with the accepted Runtime90 dependency snapshot;
  it must reach Cargo and pass rather than fail in closure planning.

## 禁止临时方案

- Do not remove or weaken the ownership guards to hide an absent resource; relocating those guards
  into the child owner is allowed only when every assertion remains executable.
- Do not copy the child implementation back into the parent, create a duplicate compatibility
  file, or change coordinator closure planning to ignore missing compile-time resources.
- Do not submit the untracked child alone as an integration candidate detached from its active
  Runtime90 parent/test union.

## 修复结果与回传

Current-source repair state: `tracked closure fixed / managed validation pending`.

- `command_copy_execution_does_not_clone_whole_source_resources` and
  `copy_command_validation_stays_in_its_child_owner` now live in `copy_commands.rs` and retain all
  original assertions.
- Tracked Rust sources no longer contain
  `include_str!("../command_validation/copy_commands.rs")`, so closure planning does not require
  the untracked child before the exact overlay is materialized.
- Exact-file rustfmt, scoped diff checks, and moved-contract static checks passed. No Runtime07 or
  Runtime90 Cargo pass is claimed until the next managed batch reaches Cargo.
