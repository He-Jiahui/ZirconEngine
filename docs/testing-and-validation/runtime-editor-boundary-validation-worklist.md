---
status: in-progress
validation_status: pending
related_code:
  - zircon_runtime/src/scene/dynamic/mod.rs
  - zircon_runtime/src/scene/dynamic/patch.rs
  - zircon_runtime/src/scene/dynamic/session/archive.rs
  - zircon_runtime/src/scene/tests/dynamic_scene.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_asset_reload.rs
  - zircon_runtime/src/tests/contracts/dynamic.rs
  - zircon_runtime/src/tests/contracts/dynamic/sources.rs
  - zircon_runtime/src/tests/contracts/dynamic/patch_preview_status_docs.rs
  - zircon_runtime/src/tests/contracts/dynamic/asset_reload_selection_status.rs
historical_record: docs/plans/_archive/zircon_runtime/runtime/05/2026-07-09-scene-editor-boundary-closeout-output-records.md
---

# Runtime 05 scene and editor boundary validation

This is the current validation worklist for the scene/editor ownership boundary. The dated Runtime 05 closeout retains its original completed status and observations. This worklist does not turn those observations into fresh Cargo, product-frame or migration acceptance. Revalidation of the current inputs is pending.

Runtime owns scene persistence, runtime inspection and world mutation. Editor owns authoring state: selection, gizmos, overlays and viewport configuration must stay outside persisted scene data and runtime inspection payloads. An editor host reference, a historical fixture and an authoring-state dependency have different roles; boundary reviews must classify each actual occurrence of editor or legacy terminology and reject newly introduced authoring dependencies. A rename alone does not establish that a dependency was removed.

The serialization exits and inspection surfaces must retain their boundary coverage. Keep the source inventory, the two naming-token inventories and the corresponding inspection checks aligned when ownership or a serialization entry changes. The dynamic-scene contract describes the runtime data and behavior; this worklist owns the outstanding validation and links it to the actual source and regression owners.

ScenePatch::preview_apply and DynamicScene::preview_spawn_into must report component types, resources and entity remaps without mutating the target world. Preflight failures must preserve the target. Preview/commit parity, ensure-backed resources and reflection preflight remain separate behavior checks.

Runtime session archives must resolve the selected slot before mutation and preserve metadata and retention protection. Preview operations must not write archive files or temporary files. Capture, restore, merge, copy, import and save checks must cover their actual commit and path behavior, including same-path rejection and atomic publication failures. A preview-only or source-anchor result does not prove a committed path operation.

Asset reload must retain the latest accepted revision, supersede older pending revisions, handle removed and failed reload events, and preserve the selected locator through renames. World and Level entry points require their own behavior evidence.

The two mounted documentation guards are patch_preview_status_docs::runtime_05_dynamic_scene_patch_preview_status_docs_stay_synced and asset_reload_selection_status::runtime_05_dynamic_scene_asset_reload_selection_and_status_anchors_stay_visible under tests::runtime_absorption::dynamic_scene. They continue to read current responsibilities together with the exact dated closeout through the strict history reader. Their historical status strings and counts stay in that fixed record; they are not reconstructed here.

| Validation responsibility | Current state |
| --- | --- |
| Current worklist and literal include target | Restored; structural verification pending |
| Both original mounted documentation guards and their assertions | Re-execution pending |
| Side-effect-free patch and spawn previews | Retain existing behavior regressions; affected checks pending |
| Archive selection, retention and atomic path operations | Retain existing behavior regressions; affected checks pending |
| Asset reload revision, selection and rename behavior | Retain existing behavior regressions; affected checks pending |
| Broader scene/editor migration or product acceptance | Requires its own current-source evidence |

Maintain this worklist with the source inventory and scoped validation results. Update an entry only from the matching command, source identity and actual outcome; preserve unsuccessful attempts and deferred gates. See [the validation policy](milestone-validation-policy.md) and [the scene contract](../crates/zircon_runtime/scene/dynamic_scene.md) for validation cadence and the scene contract.
