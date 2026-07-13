---
related_code:
  - zircon_runtime_interface/src/serialization
  - zircon_runtime/src/scene/reflect/json_document
  - zircon_runtime/src/scene/dynamic_scene/document
  - zircon_runtime/src/scene/tests/ecs_reflect/foundation/versioned_json.rs
plan_sources:
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
status: focused_interface_green_runtime_profile_blocked_by_existing_cross_feature_failures
---

# Editor 11 M1 Scene Serialization Acceptance

Date: 2026-07-11

## Accepted evidence

The managed `zircon_runtime_interface` package validation passed after the canonical writer was introduced. The gate covers the shared envelope loader plus deterministic object ordering, shortest finite float output, one trailing newline, and typed binary-format rejection.

The managed target-server check compiled `zircon_runtime` as a library, which exercises the new reflected JSON and dynamic-scene serialization production code without graphics. The remaining command failure occurred only when Cargo continued into package binaries whose imports are not gated for the disabled `graphics`, `script`, and `dynamic-api` features.

The dedicated `plan11_scene_serialization_contract` binary passed 5/5: tag-shaped v0 JSON remains `ReflectedValue::Json`; project-world v0 data migrates without a retired DTO; the real dynamic-scene v0 fixture migrates and resaves byte-identically; runtime archives embed scene envelopes and normalize the temporary inner version; a future embedded scene header fails before invalid payload decoding.

## Blocked evidence

- Default-feature Windows validation stops in `wgpu-hal 29.0.4` because two `windows`/`windows-core` versions produce incompatible Direct3D 12 types before `zircon_runtime` is checked.
- The package-wide target-server contract fails in `zircon_shader_ide_env`, `zircon_host_reflection_docs`, and `zircon_shader_prewarm` because those binaries import modules disabled by the selected feature profile.
- A target-server lib-test attempt is also blocked before test execution by existing tests that unconditionally import disabled graphics, UI, script, dynamic-api, physics-contract, `wgpu`, and `naga` owners; Cargo reports 73 compile errors in those unrelated test owners.
- The earlier WSL mount sharing violation cleared without repository or virtual-disk mutation; Linux scene behavior validation remains a separate acceptance step.

These are retained as failure-owner evidence. They do not authorize compatibility paths or unrelated graphics/tooling edits in the scene serialization milestone.
