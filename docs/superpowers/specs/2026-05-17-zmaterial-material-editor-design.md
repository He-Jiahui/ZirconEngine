# ZMaterial Material Editor Design

## Summary

`zirconEngine` will hard-cut material source assets from `.material.toml` to `.zmaterial`. Shader packages keep `.zshader` as the shader contract owner for WGSL files, entry points, material property schema, texture slots, editor hints, and WGSL capture expectations. `.zmaterial` becomes the material instance format: it references one `.zshader`, stores scalar/vector overrides, stores texture slot references in a dedicated table, and never redeclares extra fields.

The first implementation milestone focuses on runtime asset correctness: `.zshader` property/texture-slot schema, `.zmaterial` parsing, dependency capture, diagnostics, and readiness fallback reports. A dedicated Material Editor window is the target editor surface, but M1 editor work is structural: open/project a material editor view, show shader/override/texture-slot/diagnostic data, and keep Unity-style field MessageBox diagnostics for a later editor-inspector design.

## Architecture

- Owner crate and layer: `zircon_runtime::asset` owns `.zshader` and `.zmaterial` document parsing/import; `zircon_runtime::core::framework::render::material` owns neutral readiness diagnostics; `zircon_editor` owns authoring UI and never becomes runtime world authority.
- Shader contract: `.zshader` declares `properties` and `texture_slots`. WGSL capture validates same-name uniforms/resources and records diagnostics rather than failing import.
- Material instance: `.zmaterial` contains `shader`, `[overrides]`, and `[textures.<slot>]`. Unknown overrides or texture slots are diagnostics, not new fields.
- Asset identity: `.zmeta` remains UUID/URL and dependency authority. `.zmaterial` does not introduce a second database.
- Hard cutover: `.material.toml` importer support is removed from built-in material import registration, fixtures are moved to `.zmaterial`, and tests assert the old suffix is not selected.

## Reference Evidence

- Bevy: `dev/bevy/crates/bevy_render/src/render_resource/bind_group.rs` documents field-level `uniform`, `texture`, and `sampler` binding expectations plus retry/fallback behavior when textures are unavailable.
- Bevy: `dev/bevy/crates/bevy_pbr/src/pbr_material.rs` shows material fields, texture dependencies, default values, and shader-facing flags as a material contract.
- Godot: `dev/godot/scene/resources/material.cpp` `ShaderMaterial` exposes shader parameters from the shader, resets to shader defaults, and updates property lists when shader changes.
- Fyrox: `dev/Fyrox/editor/src/scene/commands/material.rs` separates commands for shader selection, material bindings, and material property values.
- Zircon: `.codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md`, `docs/zircon_runtime/asset/zmeta-shader-material.md`, and `docs/zircon_runtime/asset/render-assets.md` already establish `.zmeta`, `.zshader`, `MaterialAsset.property_values`, and readiness reporting as the current landing zone.

## `.zshader` Shape

`.zshader` remains TOML and continues to live inside a compound shader package:

```toml
version = 1
name = "Unlit Surface"
wgsl_files = ["unlit.wgsl"]

[[entry_points]]
name = "vs_main"
stage = "vertex"

[[entry_points]]
name = "fs_main"
stage = "fragment"

[[properties]]
name = "base_color"
kind = "color4"
default = [1.0, 1.0, 1.0, 1.0]
group = "Surface"
label = "Base Color"

[[properties]]
name = "roughness"
kind = "float"
default = 0.5
range = [0.0, 1.0]
group = "Surface"

[[texture_slots]]
name = "base_color"
kind = "texture2d"
default = "white"
sampler = "linear_repeat"
group = "Textures"
```

M1 only requires parsing and storing the schema. WGSL capture can be conservative: it records diagnostics for missing same-name symbols using source text scanning or Naga reflection where already available. The key behavior is structured diagnostics, not perfect shader reflection.

## `.zmaterial` Shape

`.zmaterial` is TOML and only stores instance state:

```toml
version = 1
name = "Hero Paint"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/unlit_shader"

[overrides]
base_color = [0.9, 0.2, 0.1, 1.0]
roughness = 0.72

[textures.base_color]
uuid = "00000000-0000-0000-0000-000000000002"
url = "res://textures/hero_albedo.png"
```

Rules:

- `overrides` keys must correspond to `.zshader.properties[*].name`; unknown keys become diagnostics.
- `textures` keys must correspond to `.zshader.texture_slots[*].name`; unknown keys become diagnostics.
- Missing overrides use shader defaults.
- Missing textures use the texture slot fallback class such as `white`, `black`, `normal`, or `missing`.
- Shader, property, WGSL capture, texture, and dependency mismatch never hard-fail import unless the `.zmaterial` TOML itself is invalid. They become diagnostics and readiness fallback usage.

## Runtime Data Model

- `ShaderAsset` gains `texture_slots` and keeps `property_schema`, `editor`, and `validation_diagnostics`.
- `MaterialAsset` shifts from fixed PBR-first fields to shader-driven instance fields. Existing PBR fields can remain only as transitional Rust struct data while code is cut over, but the source format must be `.zmaterial` and new behavior must use shader defaults plus material overrides.
- Texture slots use a dedicated map from slot name to an asset reference. M1 does not need sampler/UV overrides beyond preserving the shape for future additions.
- Direct dependencies include the shader reference and every texture slot reference.
- `RenderMaterialReadinessReport` gains enough diagnostic/source information to represent shader schema, WGSL capture, material override, texture slot, and dependency resolution problems.

## Import And Readiness

- Built-in material importer registers `.zmaterial` only.
- `.zmaterial` importer parses the document into `MaterialAsset` and records direct dependencies.
- If shader resolution is available in the import/project manager path, material validation uses the referenced `ShaderAsset`; otherwise readiness validation can run later with a resolver and report unresolved shader.
- Runtime readiness produces a resolved material contract from shader defaults, material overrides, and texture slot fallbacks.
- Resource streamer continues to prepare fallback materials when dependencies fail, but stores the structured report so renderer/editor consumers can display exact causes.

## Material Editor

M1:

- Dedicated Material Editor window descriptor remains the primary entry.
- Asset Browser double-click/open behavior targets the dedicated window for `.zmaterial` assets.
- Structural preview shows shader identity, effective overrides, texture slots, fallback state, and diagnostics.
- Editor-side inline inspector integration is limited to an extension seam or open button.
- Field-level Unity-style MessageBox diagnostics are explicitly deferred.

M2:

- Runtime sphere/plane preview uses runtime/render facades and a local editor preview scene.
- Preview state stays editor-host local and is not serialized into runtime world/project scene.
- Preview uses fallback material plus readiness diagnostics when shader/material resources are not ready.

## Validation

- Runtime asset tests cover `.zshader` texture slots, `.zmaterial` parsing, `.zmaterial` roundtrip, shader defaults, override diagnostics, texture slot diagnostics, direct references, and `.material.toml` hard cutover.
- Project `.zmeta` tests cover material dependencies in `entries[*].dependencies` and UUID/URL reference roundtrip.
- Render readiness tests cover missing shader, missing texture, unknown property, unknown slot, fallback class, and report source classification.
- Editor tests cover Material Editor descriptor/open route and M1 structural projection.

## Scope Boundaries

- No shader graph implementation in this design.
- No `.material.toml` compatibility.
- No complete WGSL reflection if source-level symbol capture is sufficient for M1 diagnostics.
- No runtime sphere/plane preview until M2.
- No Unity-style per-field MessageBox rendering until the later inspector diagnostic design.
