---
related_code:
  - zircon_runtime/src/scene/world/compiled_binding/property_path.rs
  - zircon_runtime/src/scene/world/compiled_binding/property_path/model.rs
  - zircon_runtime/src/scene/world/compiled_binding/property_path/compile.rs
  - zircon_runtime/src/scene/world/compiled_binding/property_path/read.rs
  - zircon_runtime/src/scene/world/compiled_binding/property_path/write.rs
  - zircon_runtime/src/scene/tests/property_paths/read_paths.rs
implementation_files:
  - zircon_runtime/src/scene/world/compiled_binding/property_path.rs
  - zircon_runtime/src/scene/world/compiled_binding/property_path/model.rs
  - zircon_runtime/src/scene/world/compiled_binding/property_path/compile.rs
  - zircon_runtime/src/scene/world/compiled_binding/property_path/read.rs
  - zircon_runtime/src/scene/world/compiled_binding/property_path/write.rs
  - zircon_runtime/src/scene/tests/property_paths/read_paths.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-scene-property-path-compiled-dispatch.md
tests:
  - zircon_runtime/src/scene/tests/property_paths/read_paths.rs::compiled_sequence_apply_keeps_path_resolution_and_string_dispatch_at_compile_boundary
  - zircon_runtime/src/scene/world/compiled_binding/tests.rs::compiled_scene_property_writer_stable_access_avoids_text_and_entry_work
  - zircon_runtime/src/scene/world/compiled_binding/tests.rs::compiled_scene_property_writer_scales_stable_access_without_text_work
  - rustfmt --edition 2021 --check
  - git diff --check
doc_type: milestone-detail
---

# Runtime 08 compiled property-path owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| M2/M3 | Compiled property-path folder-backed owner split | `runtime_08_compiled_property_path_owner_split_implemented_static_passed_managed_validation_deferred` | 2026-08-26 | Root 700 -> 8 lines; four production child owners 218/135/162/219 lines; 23/23 functions, 7/7 types, and all typed dispatch variants retained. |

Completed:

- Kept `property_path.rs` as a narrow private mount and compatible re-export facade.
- Split stable identity, generation-currentness, and typed writer variants into `model.rs`.
- Split canonicalization, interning, entity resolution, and writer compilation into `compile.rs`.
- Split stable-frame typed reads into `read.rs`.
- Split stable-frame typed writes and dirty-generation publication into `write.rs`.
- Preserved the existing `compiled_binding` construction/access visibility explicitly after nesting.
- Updated the existing source-contract regression to inspect all new owners while preserving its pre-existing dirty edits.

## Review basis

Unreal's local `FPropertyPathSegment`/`FCachedPropertyPath` implementation separates authored path data, resolution, and cached fast access. Bevy separately owns stable animation target identity and typed property evaluation. Zircon's existing generation-owned IDs and typed writer enum already match that compile-once/apply-many direction; this slice makes those lifecycle boundaries explicit without replacing a correct algorithm.

There is no compatibility module, duplicate implementation, public API expansion, dispatch fallback, algorithm replacement, or hotpath instrumentation change.

## Verification

- Scoped `rustfmt --edition 2021 --check` passed for all six touched Rust files.
- Scoped `git diff --check` passed, apart from LF/CRLF checkout notices.
- Static migration comparison retained all 23 old function definitions and all 7 old type definitions with no missing or duplicate item.
- Static variant comparison retained identical transform, mesh, animation, camera, light, and dynamic dispatch coverage.
- The facade contains only four child mounts and the existing four-type public re-export; production child owners are all at or below 219 lines.
- Production files contain no new `allow`, `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` escape path.
- Managed Cargo, runtime throughput, allocation, and power validation were not run while bypassing the current validation blocker. They remain required before accepted milestone closeout.
- No CPU, memory, energy, or power improvement is claimed because this slice changes ownership rather than the compiled-binding algorithm.

## Open scope

Runtime 08 and the full runtime architecture remain `in_progress`. This record closes only the compiled property-path source ownership implementation. Managed compile/test, existing 10k stable-access gates, representative profiling, milestone commit, coordinator integration receipt, and WeCom publication remain open.
