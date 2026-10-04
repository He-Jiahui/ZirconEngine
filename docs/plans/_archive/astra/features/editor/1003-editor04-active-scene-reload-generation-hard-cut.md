---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/04/2026-08-27-active-scene-reload-generation-hard-cut.md
related_records:
  - docs/plans/astra/features/editor/1002-editor04-capability-sort-completion-list.md
  - docs/plans/astra/features/runtime/940-runtime02-mesh-sdf-executor-capacity.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_editor/src/ui/retained_host/app/assets/workspace/active_scene_reload_conflict.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime/tests.rs
tests:
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime/tests.rs
---

# Editor1003 · active-scene reload generation hard cut

The retained active-scene refresh now consumes the lifecycle-owned scene
identity and a Runtime-owned prepared project-generation snapshot. It no
longer reopens the project or performs a full `scan_and_import` for an already
committed asset event. Scene preparation remains outside the read gate, while
the conditional generation commit retains the fence through the short world
replacement; stale A-to-B-to-A work, dirty state, and bounded admission retry
are rejected or routed through the existing conflict owner.

| Boundary | Before | Current contract | Status |
| --- | ---: | ---: | --- |
| Project manager reopen on matching active-scene event | 1 | 0 | implemented_pending_validation |
| Full project `scan_and_import` | 1 | 0 | implemented_pending_validation |
| Lifecycle-owned active identity | 0 | 1 | implemented_pending_validation |
| Unconditional stale-world replacement | 1 | 0 | implemented_pending_validation |

The workspace source contract and Runtime generation-fence tests cover stale
tokens, fence retention, dirty conflict routing, and the rule that an
unpublished preparation epoch does not invalidate the active generation.
Managed Runtime/Editor behavior execution and product F0/F4 traces remain
pending; historical timeout/failure logs are not counted as passes.

## Source snapshots

| Owner | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/app/assets/workspace.rs` | `FF50386BE6FBAF8184DBADFFD67528D821532714686D68406A647C8C0F384825` |
| `zircon_editor/src/ui/retained_host/app/assets/workspace/active_scene_reload_conflict.rs` | `0E17B19802DB63F7BF5A5911DED8A0C2092FFB01039A6E1A3D841D8EE1F3F33A` |
| `zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs` | `7FA51EECA7F5D4D119833FE42AAD9C4D339A5865080A27DB1BA075C93EB739D7` |
| `zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime/tests.rs` | `E75BD82FB35B9388F408BD8A032BB0D603FBBB56FF7F57796F672500B645726DB` |
