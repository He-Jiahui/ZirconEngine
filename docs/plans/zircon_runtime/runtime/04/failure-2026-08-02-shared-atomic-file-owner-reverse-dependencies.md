---
handoff_kind: failure
status: open
created_at: 2026-08-02
summary_slug: shared-atomic-file-owner-reverse-dependencies
origin_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/05
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/resource/io/mod.rs
  - zircon_runtime/src/core/resource/io/resource_io.rs
  - zircon_runtime/src/core/resource/io/error.rs
  - zircon_runtime/src/core/resource/io/atomic_file.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/migration/transaction/journal.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/manager/targeted_transaction.rs
  - zircon_runtime/src/asset/registry/incremental.rs
  - zircon_runtime/src/asset/registry/persistence.rs
  - zircon_runtime/src/asset/project/meta_preview_state.rs
  - zircon_runtime/src/graphics/pipeline/pipeline_cache_gate.rs
  - zircon_runtime/src/platform/preferences/atomic_file.rs
  - zircon_runtime/src/scene/module/level_manager_project_io.rs
  - zircon_runtime/src/scene/world/project_io.rs
tests:
  - cargo test -p zircon_runtime --lib atomic_write --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib asset_meta_preview_state_cas_ --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib render_perf_pipeline_cache_ --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib platform_preference_storage --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib project_asset_manager --locked --jobs 1 -- --nocapture --test-threads=1
---

# Runtime04: shared atomic-file implementation has a concrete Foundation owner

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 来源执行切片：M4 production dependency follow-up for the recorded `asset -> foundation` concrete seam
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：the lowest shared implementation is file/resource I/O used by Asset, Platform, Scene, and Foundation config. Runtime04 already owns `core::resource` and the asset/resource state boundary; Frameworks05 cannot move that owner while Runtime04 has an active resource generation/event slice, and it must not duplicate the writer in each business domain.

## 失败现象与复现证据

The initial 2026-08-02 current-source inspection found the complete atomic commit implementation in
`zircon_runtime/src/foundation/persistence/atomic_file.rs`. The implementation owns staging,
write/flush/fsync, Windows `ReplaceFileW`, Unix backup durability, recovery, fault injection, and
cleanup behavior, but its consumers are not Foundation-only:

- Asset has **9 production references across 6 files** to `crate::foundation::persistence`, plus 3
  test files that import `AtomicWriteFault` from the same concrete path.
- Platform preferences imports `stage_atomic_write` directly from the Foundation implementation.
- Scene project I/O has 2 production imports of the Foundation `atomic_write` projection.
- Foundation config remains a valid consumer, not the authority that makes the implementation a
  Foundation business concern.

The Frameworks05 plan already records the concrete `asset -> foundation` seam as unfinished. At
handoff creation, several callers contained unattributed or other-session dirty changes while
`core::resource` simultaneously contained Runtime04 resource event/readiness/manager work.
Frameworks05 therefore published the fixing-plan contract without moving source or adding a
compatibility layer.

## 最低共享层根因

Generic crash-safe file replacement was absorbed under the concrete Foundation runtime domain even
though it is shared supporting I/O. This makes Asset, Platform, and Scene depend on a neighboring
business implementation and forces Foundation to expose test fault types for unrelated domains.
The converged runtime spine already designates `core::resource` as the canonical shared resource
foundation; its failure-time flat `core/resource/io.rs` also mixed the `ResourceIo` trait and
`ResourceIoError`, so the scalable owner is a folder-backed `core::resource::io` implementation
boundary rather than another top-level package or a framework contract.

Reference routing supports this placement: Godot separates common file behavior under `core/io`,
while Bevy/Fyrox keep recurring resource/I/O support below asset and editor/runtime consumers. The
local landing zone remains `zircon_runtime::core::resource` to obey Zircon's fixed internal spine.

## 架构修复验收

- Convert `core/resource/io.rs` into a structural `core/resource/io/mod.rs`; move `ResourceIo` and
  `ResourceIoError` into focused declaration files and move the single atomic-file implementation
  into `core/resource/io/atomic_file.rs`.
- Preserve the current staging, durability, rollback/recovery, Windows/Unix behavior, fault
  injection, and cleanup tests. Do not create a second writer or change persistence semantics as
  part of the owner move.
- Hard-cut every Foundation, Asset, Platform, Scene, and test consumer to the new direct owner in
  the same milestone. Delete `foundation/persistence.rs` and
  `foundation/persistence/atomic_file.rs` after the final consumer moves.
- Fresh production scans must report `foundation::persistence = 0` and `asset -> foundation = 0`.
  No old-path `pub use`, alias module, wrapper function, conversion shim, or caller exception may
  survive.
- Run the focused atomic writer tests, Frameworks05 asset/platform upward filters, a package-scoped
  `zircon_runtime --lib` check, and Runtime04 resource/asset gates on one immutable current-source
  snapshot.

## 禁止临时方案

- Do not copy `atomic_file.rs` into Asset, Platform, Scene, or a generic `utils` module.
- Do not keep `foundation::persistence` as a forwarding path or re-export the new owner from it.
- Do not move concrete filesystem implementation into `core::framework`; that layer remains
  contracts/neutral DTO only.
- Do not absorb the currently dirty Runtime04 resource slice or unrelated Asset/Platform blobs into
  a Frameworks05 commit.
- Do not weaken durability/fault tests or replace atomic persistence with plain `fs::write`.

## 修复结果与回传

Current state: `runtime04_owner_move_source_complete_validation_pending`. The Runtime04 successor
created the folder-backed `core/resource/io/{mod,resource_io,error,atomic_file}.rs` owner, migrated
all 20 direct current-source consumers, and physically deleted the flat `core/resource/io.rs` plus
both Foundation persistence owner files. The two previously missed untracked consumers,
`asset/project/meta_preview_state.rs` and `graphics/pipeline/pipeline_cache_gate.rs`, now also
import `core::resource::io::atomic_file` directly. A fresh recursive `zircon_runtime/src` scan
reports `foundation::persistence = 0`; `git diff --check` across the migrated tracked paths reports
no whitespace errors. No Foundation facade, alias, or re-export was retained, and atomic-write
behavior/fault injection code moved intact rather than being copied or rewritten.

Focused current-source tests, independent review, managed Runtime package gates, fixed return, and
Frameworks05 M4 acceptance remain pending, so this artifact stays `open` and does not claim an
accepted source pass.
