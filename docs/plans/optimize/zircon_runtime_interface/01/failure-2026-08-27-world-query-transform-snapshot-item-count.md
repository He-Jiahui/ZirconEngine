---
handoff_kind: failure
status: open
created_at: 2026-08-27
summary_slug: world-query-transform-snapshot-item-count
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_host/src/foreign_output/item_count.rs
  - zircon_runtime_interface/src/world_sync/query.rs
  - zircon_runtime/src/dynamic_api/frame.rs
tests:
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime_host -SkipBuild -LibTests -TestFilter world_query_item_count -VerboseOutput
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests -TestFilter viewport -VerboseOutput
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_navigation_editor -SkipBuild -VerboseOutput
---

# RuntimeInterface01: TransformSnapshot lacks foreign-output item accounting

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 来源执行切片：Editor05 viewport `Arc<[T]>` consumer focused validation
- 修复责任计划：`docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md`
- 交接原因：the failure is in RuntimeHost's bounded foreign-output projection for a
  RuntimeInterface world-query result, before any Editor05 viewport test executes.

## 失败现象与复现证据

Managed Editor05 job `62f609ebdaeb4db985b582882dd4876a` ran
`cargo test -p zircon_editor --locked --lib viewport`. It compiled the ordinary
`zircon_runtime_interface` library, then stopped in
`zircon_runtime_host/src/foreign_output/item_count.rs:80` with E0004 because
`WorldQueryResult::TransformSnapshot` is not covered. The viewport test count was
zero and no diagnostic named the Editor05-owned consumer.

This is independently reproduced by managed Plugins05 job
`0d26b703ac164fc082c9369ab38a7b6b`, which stopped on the same E0004 before its
Navigation editor tests. Both jobs released normally with Cargo exit 101.

The current worktree is a mixed uncommitted migration: `query.rs` adds
`ComponentRows`, `HierarchyRows`, `InspectionFields`, `TransformSnapshot`, and
`EntityMissing`, while `item_count.rs` already carries foreign receipt accounting
and the other new query-result arms. Therefore an isolated one-line HEAD patch would
not compile and must not be used to sever the current-source union.

## 最低共享层根因

The world-query result hard cut was not applied atomically to every bounded-output
consumer. RuntimeHost's explicit structural accounting was updated for component,
hierarchy, inspection, missing, and not-modified results but omitted the transform
snapshot variant. Rust correctly rejects the non-exhaustive match before upper-layer
tests can run.

## 架构修复验收

- Add an explicit `TransformSnapshot` arm to RuntimeHost item accounting in the same
  owner commit as the current world-query result migration; preserve the other mixed
  receipt/query changes.
- Match the existing runtime-side authority in
  `zircon_runtime/src/dynamic_api/frame.rs`, which counts a transform snapshot as one
  bounded result item. Do not count individual scalar transform fields independently.
- Add a focused RuntimeHost regression that constructs every `WorldQueryResult`
  variant and proves item-count parity for the transform snapshot and existing rows.
- Pass the RuntimeHost focused managed gate, then replay the Editor05 viewport and
  Plugins05 Navigation gates so both upstream test suites actually execute.

## 禁止临时方案

- Do not add a wildcard arm, suppress E0004, or reduce the foreign-output budget.
- Do not commit only `item_count.rs` against a HEAD where `TransformSnapshot` does not
  exist, and do not discard its existing module-composition receipt/query migration.
- Do not claim either upstream failure fixed from static source inspection or a test
  run that stops before its selected tests.

## 修复结果与回传

Current-source repair state: `source exhaustive / managed focused validation pending`.

- `zircon_runtime_host/src/foreign_output/item_count.rs` now preserves the existing
  `ComponentRows` / `HierarchyRows` / `InspectionFields` / `EntityMissing` /
  `NotModified` migration and adds the explicit
  `WorldQueryResult::TransformSnapshot { .. } => 1` arm. Current SHA-256 is
  `7E643D75BC34A77A5A596A59351184AD679A10197B613A7C7255B7D00F66E322`.
- `zircon_runtime_host/src/foreign_output/tests.rs` constructs all six current
  variants and checks their exact structural counts. Current SHA-256 is
  `10AC7626F4E51874388F2EC9C96FB23F09E69C423EBD958C6529513AB0E87B5C`.
- Managed focused request `6518997dfee14750a4334d1c5682b7bf` stopped before Cargo
  admission, so this record does not claim RuntimeHost GREEN and does not return the
  blocker to Editor05/UI12 yet. The Editor and Navigation replay gates remain pending
  behind a successful current-source RuntimeHost focused receipt.

### 2026-08-30 current-source focused validation

- Current source `rustfmt --edition 2021 --check` passes for `item_count.rs` and its focused
  foreign-output tests. A source-bound exhaustive check finds all six current
  `WorldQueryResult` arms, including the explicit `TransformSnapshot { .. } => 1`; no wildcard
  arm was added.
- The focused regression source constructs and checks `ComponentRows`, `HierarchyRows`,
  `InspectionFields`, `TransformSnapshot`, `EntityMissing`, and `NotModified`. Current hashes
  remain `item_count.rs = 7E643D75BC34A77A5A596A59351184AD679A10197B613A7C7255B7D00F66E322` and
  `tests.rs = 10AC7626F4E51874388F2EC9C96FB23F09E69C423EBD958C6529513AB0E87B5C`.
- The copy-complete union was reclaimed under lease request
  `471e9b32da004734a06f00e4c7b6d8e3` and attributed at its current hashes. In addition to the
  two RuntimeHost hashes above, `query.rs =
  56F357DDDD79E119CA894B195966658025731B67BC102377A386FDA62A7AAA47` and `frame.rs =
  0EAF943FF6C0CA122AD18F29150A4EDB090DD69115F1F156330CB8322D3B3DFC`.
- Windows managed validation ticket `32c6ac74ba4247e6b4c34c4e42fd9739` was submitted with the
  exact five-path source manifest and focused filter `world_query_item_count`; initial state is
  `queued`. The ticket runs asynchronously, so this record does not claim RuntimeHost GREEN and
  the Editor05/UI12 return remains pending until a terminal Cargo receipt exists.

### 2026-08-31 current-source ownership reconciliation

The complete current-source union was reclaimed into the active RuntimeInterface03 owner session;
the four source files and this canonical failure were transferred together, preserving all existing
world-query migration edits:

- transfer-preview request `8f2dccacd8174a498e47405900d118bd`, fingerprint
  `b456222b33376b324aea3e5107933aed9f6d257fe638fb0a8f3068604b636d0c`;
- transfer-apply request `c4dc43e193784872829d095d0f2ab110`;
- exact union lease request `13eaac312089422eb9d2b20f35e23620`;
- owner Session `root-runtime-interface03-activate-link-failure-20260831`.

Current hashes remain unchanged: `item_count.rs =
7e643d75bc34a77a5a596a59351184ad679a10197b613a7c7255b7d00f66e322`, `tests.rs =
10ac7626f4e51874388f2ec9c96fb23f09e69c423ebd958c6529513ab0e87b5c`, `query.rs =
56f357dddd79e119ca894b195966658025731b67bc102377a386fda62a7aaa47`, and `frame.rs =
0eaf943ff6c0ca122ad18f29150a4edb090dd69115f1f156330cb8322d3b3dfc`.

The existing managed ticket `32c6ac74ba4247e6b4c34c4e42fd9739` remains the validation receipt for
this exact source union; it is not polled or reinterpreted here. Status remains open until a
terminal RuntimeHost receipt and the two upstream replay gates are available.

The current-source RuntimeHost retry was deduplicated into managed ticket
`7f4763e1c3b7458faee271b5c5dd0ee6` (submit request
`runtime-interface01-world-query-recovery-20260831-r2`, coordinator receipt
`a66b5a506141489db1519275df799e54`). It runs the Windows Rust 1.94.1 release command with the
five-path union manifest; no Cargo status is polled here.
