# Color Foundation Hard Cutover Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current split color representations (`hex String`, `Vec4`, `[u8; 4]`, `[f32; 4]`) with canonical `Srgba` / `LinearRgba` / `Hsla` / `Hsva` color contracts, explicit sRGB-linear conversion, straight-alpha semantics, and shared UI/material/asset-import/editor usage.

**Architecture:** `zircon_runtime_interface::math::color` owns canonical shared color DTOs, parsing, formatting, serialization, and conversion because these values cross runtime, editor, UI, asset, and plugin-facing boundaries. `zircon_runtime::core::math` re-exports the same types; UI/editor/asset/render code must consume the shared types directly instead of keeping local string or array color parsers. This is a hard cutover: do not keep old array/string color fields or compatibility readers for old serialized color shapes.

**Tech Stack:** Rust 2021, `serde`, `toml`, `serde_json`, `glam`, existing `zircon_runtime_interface`, `zircon_runtime`, `zircon_editor`, `gltf`, `image`, Slint host painter integration, milestone-first Cargo validation.

---

## Current Baseline

- The branch is `main`; repository policy forbids feature branches and worktrees for this work.
- User selected full migration and hard cutover. Do not add tolerant legacy readers for old color arrays/strings unless a later user instruction explicitly changes this.
- Current split representations found in code:
  - `zircon_runtime_interface/src/ui/component/value.rs`: `UiValue::Color(String)`.
  - `zircon_runtime_interface/src/ui/surface/render/resolved_style.rs`: `background_color`, `foreground_color`, and `border_color` are `Option<String>`.
  - `zircon_runtime_interface/src/ui/surface/render/brush.rs`: brush colors, tints, gradient stops, and material fallback colors are `String` / `Option<String>`.
  - `zircon_runtime_interface/src/ui/surface/render/text_shape.rs`: text paint colors and decoration colors are strings.
  - `zircon_runtime_interface/src/ui/text.rs`: `UiTextCursorStyle.color` is `Option<String>`.
  - `zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs`: local `parse_style_color(...)` converts `#rgb/#rgba/#rrggbb/#rrggbbaa` to `[u8; 4]`.
  - `zircon_runtime/src/asset/assets/material.rs`: `MaterialAsset.base_color` is `[f32; 4]`, `emissive` is `[f32; 3]`.
  - `zircon_runtime/src/asset/assets/texture.rs`: `TextureAsset.rgba` has no texture color-space metadata.
  - `zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs`: runtime material colors are `Vec4` / `Vec3`.
  - `zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs`: GPU texture creation uses one format path and ignores source color space.
  - `zircon_editor/src/scene/viewport/edit_mode_projection/build.rs`: Inspector projects mesh tints as `SceneInspectorFieldValue::Vec4(...)` fields labelled `Color`.
- glTF importer currently returns a `ModelAsset` only; it does not emit `MaterialAsset` or `TextureAsset` entries from glTF material definitions.
- Active coordination note for this lane: `.codex/sessions/20260508-2057-color-foundation-design.md`.
- Active nearby session `.codex/sessions/20260508-2036-reflection-type-registry.md` owns reflection/type-registry design. Do not broaden this color plan into reflection registry implementation. If Inspector typed color metadata needs reflection, leave only the direct color DTO projection and document the follow-up.

## Reference Evidence

- `dev/bevy/crates/bevy_color/src/color.rs`: Bevy uses a shared `Color` enum with `Srgba`, `LinearRgba`, `Hsla`, and `Hsva`, and provides `to_linear()` / `to_srgba()` conversions.
- `dev/bevy/crates/bevy_color/src/srgba.rs`: Bevy owns CSS-style hex parsing and formatting in `Srgba`, including `#RGB`, `#RGBA`, `#RRGGBB`, and `#RRGGBBAA`, plus exact sRGB transfer functions.
- `dev/bevy/crates/bevy_color/src/linear_rgba.rs`: Bevy keeps render-linear colors as a distinct type instead of generic vectors.
- `dev/godot/core/math/color.h`: Godot keeps HTML/CSS color parsing, sRGB/linear conversion, HSV helpers, and straight-alpha blend behavior in the engine foundation color type.
- `dev/Fyrox/fyrox-core/src/color.rs`: Fyrox uses a shared core color type consumed by UI/editor code and includes parse/conversion tests.
- `dev/Graphics/com.unity.postprocessing/PostProcessing/Runtime/PostProcessRenderContext.cs`: Unity Graphics explicitly carries sRGB vs Linear render texture read/write decisions instead of making sampling implicit.

## File Structure

### Shared Color Foundation

- Modify: `zircon_runtime_interface/src/math.rs`
  - Convert this root math file to structural wiring for the new `math::color` subtree while preserving existing vector aliases and transform helpers.
- Create: `zircon_runtime_interface/src/math/color/mod.rs`
  - Structural module declarations and public re-exports only.
- Create: `zircon_runtime_interface/src/math/color/srgba.rs`
  - `Srgba`, `Srgba8`, constructors, constants, byte conversion, hex formatting, `FromStr` via shared parser.
- Create: `zircon_runtime_interface/src/math/color/linear_rgba.rs`
  - `LinearRgba`, constructors, constants, finite checks, `to_vec4`, `from_vec4`, `From<Srgba>`, straight-alpha premultiply helper entry point, and fallible unpremultiply support.
- Create: `zircon_runtime_interface/src/math/color/hsla.rs`
  - `Hsla` declaration and `Srgba` conversion helpers.
- Create: `zircon_runtime_interface/src/math/color/hsva.rs`
  - `Hsva` declaration and `Srgba` conversion helpers.
- Create: `zircon_runtime_interface/src/math/color/color.rs`
  - Canonical tagged `Color` enum with variants `Srgba(Srgba)`, `LinearRgba(LinearRgba)`, `Hsla(Hsla)`, and `Hsva(Hsva)` plus `to_srgba()` and `to_linear_rgba()`.
- Create: `zircon_runtime_interface/src/math/color/alpha.rs`
  - `AlphaMode` remains material-owned, but this file owns color alpha semantics: `AlphaRepresentation::{Straight, Premultiplied}` and `PremultipliedLinearRgba`.
- Create: `zircon_runtime_interface/src/math/color/format.rs`
  - CSS-like parse/format support for `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb(...)`, and `rgba(...)`.
- Create: `zircon_runtime_interface/src/math/color/error.rs`
  - `ColorParseError` and `ColorComponentError` with exact error variants.
- Create: `zircon_runtime_interface/src/math/color/serde.rs`
  - Canonical hard-cutover serialization helpers.
- Create: `zircon_runtime_interface/src/math/color/texture_color_space.rs`
  - `TextureColorSpace::{Srgb, Linear, NonColor}` and helper predicates.
- Modify: `zircon_runtime/src/core/math/mod.rs`
  - Continue re-exporting `zircon_runtime_interface::math::*`; no local color implementation.
- Create: `zircon_runtime_interface/src/tests/color_contracts.rs`
  - Conversion, parse, format, serialization, alpha, and texture-color-space contract tests.
- Modify: `zircon_runtime_interface/src/tests/mod.rs`
  - Add `mod color_contracts;`.

### UI Contract Cutover

- Modify: `zircon_runtime_interface/src/ui/component/value.rs`
  - Change `UiValue::Color(String)` to `UiValue::Color(Color)`.
- Modify: `zircon_runtime_interface/src/ui/surface/render/resolved_style.rs`
  - Change color fields to `Option<Color>`.
- Modify: `zircon_runtime_interface/src/ui/surface/render/brush.rs`
  - Change brush payload color fields, image/vector tints, gradient stop colors, and material fallback color to `Color` / `Option<Color>`.
- Modify: `zircon_runtime_interface/src/ui/surface/render/text_shape.rs`
  - Change text paint, run, and decoration color fields to `Option<Color>` / `Color`.
- Modify: `zircon_runtime_interface/src/ui/surface/render/command.rs`
  - Replace string color constants with `Color` constructors and update brush/text construction.
- Modify: `zircon_runtime_interface/src/ui/surface/render/visualizer.rs`
  - Replace debug overlay `Option<String>` color with `Option<Color>`.
- Modify: `zircon_runtime_interface/src/ui/text.rs`
  - Change `UiTextCursorStyle.color` to `Option<Color>`.
- Modify: `zircon_runtime_interface/src/tests/ui_contract_spine.rs`
  - Update JSON/TOML roundtrip expectations to the canonical color serialization.
- Modify: all UI template/build code that constructs or parses `UiValue::Color`, `UiResolvedStyle`, `UiBrushPayload`, `UiTextPaint`, or cursor colors. Expected hot spots from previous scans include `zircon_runtime/src/ui/template/build/*`, `zircon_runtime/src/ui/template/asset/*`, `zircon_runtime/src/ui/surface/render/*`, and editor UI template runtime adapters.

### Editor Painter And Inspector Cutover

- Modify: `zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs`
  - Delete local `parse_style_color`, `parse_hex_pair`, and `parse_nibble`; replace painter conversion with shared `Color::to_srgba().to_u8_array()` or `Srgba::to_u8_array()`.
- Modify: `zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs`
  - Keep `[u8; 4]` only as final pixel framebuffer representation; do not expose it as semantic color input.
- Modify: `zircon_editor/src/ui/slint_host/host_contract/painter/text.rs`
  - Keep `[u8; 4]` only at rasterization boundary.
- Modify: `zircon_editor/src/ui/slint_host/host_contract/painter/theme.rs`
  - Convert palette constants to `Srgba`/`Color` or isolate `[u8; 4]` as final framebuffer constants with shared conversion helpers.
- Modify: `zircon_editor/src/ui/slint_host/ui/template_node_conversion.rs`
  - Convert `UiValue::Color(Color)` to `slint::Color` via shared `Srgba8` conversion.
- Modify: `zircon_editor/src/ui/slint_host/host_contract/data/template_nodes.rs`
  - Keep `slint::Color` only as Slint boundary representation.
- Modify: `zircon_editor/src/ui/template_runtime/component_adapter/*`
  - Replace string color payload handling with `Color`.
- Modify: `zircon_editor/src/ui/template_runtime/showcase_demo_state*`
  - Replace demo string colors with canonical `Color` constructors.
- Modify: `zircon_editor/src/scene/viewport/edit_mode_projection/build.rs`
  - Stop projecting color inspector fields as raw `Vec4` where the semantic field is a color. Use `SceneInspectorFieldValue::Color(Color)` for mesh tint and material color fields.
- Modify: `zircon_editor/src/scene/viewport/edit_mode_projection/scene_inspector_field_value.rs`
  - Add `SceneInspectorFieldValue::Color(Color)` and import the canonical shared color contract from `zircon_runtime_interface::math`.
- Modify: any affected editor tests under `zircon_editor/src/tests/**` and `zircon_editor/src/ui/**/tests.rs` that assert `#fedcba`, `#ffcc33`, `pub value_color: Color`, or raw Vec4 color values.

### Asset, Material, Texture, And glTF Cutover

- Modify: `zircon_runtime/src/asset/assets/material.rs`
  - Change `MaterialAsset.base_color` to `LinearRgba`.
  - Change `MaterialAsset.emissive` to `LinearRgba` or a dedicated `LinearRgb` is not introduced in this plan; use `LinearRgba` with alpha fixed to `1.0` for serialization simplicity.
  - Keep material blend mode in `AlphaMode`; do not duplicate it in color alpha semantics.
- Modify: `zircon_runtime/src/asset/assets/texture.rs`
  - Add `pub color_space: TextureColorSpace` to `TextureAsset`.
  - Rename only `TextureAsset.rgba` to `rgba8` in this milestone. Leave framebuffer/readback payloads such as `CapturedFrame.rgba`, `ViewportFrame.rgba`, and `CpuTexturePayload.rgba` unchanged because those are final pixel buffers, not semantic texture asset color contracts.
- Modify: `zircon_runtime/src/asset/importer/ingest/import_texture.rs`
  - Populate `TextureAsset.color_space` from import settings: default `Srgb` for image textures; allow explicit `color_space = "linear"` or `"non_color"` in importer settings for normal/roughness/occlusion-style image assets.
- Modify: `zircon_runtime/src/asset/importer/ingest/import_material.rs`
  - Rely on new `MaterialAsset` serde; do not accept old array fields.
- Modify: `zircon_runtime/src/asset/importer/ingest/import_gltf.rs`
  - Emit material entries for glTF materials in addition to the root model entry.
  - Emit texture entries or dependencies for glTF material textures where data is accessible from `gltf::import` outputs.
  - Set glTF `baseColorTexture` and emissive texture color space to `Srgb`.
  - Set glTF metallic-roughness, normal, and occlusion textures to `NonColor`.
  - Treat glTF `base_color_factor` as a linear PBR multiplier and construct `LinearRgba::new(r, g, b, a)` directly. The implementation comment must state that base color textures are sRGB, while numeric factors are consumed as linear multipliers.
  - Convert glTF emissive factor into `LinearRgba::new(r, g, b, 1.0)` with explicit documentation.
- Modify: `zircon_runtime/src/asset/importer/ingest/asset_importer.rs`
  - For test fixture glTF importer descriptors, add `AssetKind::Material` and `AssetKind::Texture` as additional output kinds when import emits them.
- Modify: `zircon_runtime/src/asset/pipeline/manager/builtins/builtin_resources.rs`
  - Use `LinearRgba` constructors for built-in material base colors and emissive colors.
- Modify: all runtime asset tests that construct `MaterialAsset` or `TextureAsset`, including `zircon_runtime/src/asset/tests/assets/material.rs`, `artifact_store.rs`, `facade.rs`, `project/manager.rs`, `pipeline/manager.rs`, `support.rs`, and scene/graphics test support.

### Runtime Graphics Cutover

- Modify: `zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs`
  - Store semantic runtime fields as `base_color: LinearRgba` and `emissive: LinearRgba`. Final `Vec4` values are produced only inside GPU packing or CPU shading upload helpers.
- Modify: `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs`
  - Remove `Vec4::from_array(material.base_color)` and `Vec3::from_array(material.emissive)`; pass canonical linear color values from `MaterialAsset`.
- Modify: shader/material GPU packing paths that currently read `MaterialRuntime.base_color` / `emissive` as vectors. Convert to `Vec4` at the final uniform/push-constant/CPU shading boundary using `LinearRgba::to_vec4()`.
- Modify: `zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs`
  - Choose GPU format with `texture_format_for_color_space(TextureColorSpace) -> wgpu::TextureFormat`: `TextureColorSpace::Srgb` maps to `wgpu::TextureFormat::Rgba8UnormSrgb`; `Linear` and `NonColor` map to `wgpu::TextureFormat::Rgba8Unorm`.
- Modify: any CPU preview/readback paths that sample texture RGBA and combine with material base color; convert sRGB texture samples to linear before lighting/material math.
- Modify: graphics tests under `zircon_runtime/src/graphics/tests/**` that construct material colors or assert final color behavior.

### Documentation

- Create: `docs/zircon_runtime_interface/math/color.md`
  - Module detail doc for canonical color types, serialization, parse rules, conversion rules, and alpha semantics.
- Create: `docs/zircon_runtime/asset/color-and-material-import.md`
  - Asset/material/glTF/texture color-space import rules.
- Update: `docs/zircon_runtime_interface/ui/contract-spine.md`
  - UI color fields now use canonical `Color`, not strings.
- Update: `docs/assets-and-rendering/render-framework-architecture.md`
  - Material/runtime rendering consumes `LinearRgba`; texture color-space metadata controls sRGB sampling.
- Update: `docs/ui-and-layout/slate-style-ui-surface-frame.md`
  - UI brushes/text/visualizer colors use shared color contracts.
- Update: `.codex/sessions/20260508-2057-color-foundation-design.md`
  - Track implementation status, blockers, validation evidence, and archive on completion.

## Serialization Contract

All persisted semantic colors use this hard-cutover shape. Do not keep old raw array or bare string color readers.

```toml
[base_color]
space = "linear_rgba"
components = [0.8, 0.8, 0.8, 1.0]

[theme.accent]
space = "srgba"
hex = "#4D89FF"

[ui.demo_color]
space = "hsva"
components = [210.0, 0.7, 1.0, 1.0]
```

JSON serialization follows the same field names:

```json
{"space":"srgba","hex":"#4D89FF"}
```

Rules:

- `Color::Srgba` serializes as `{ "space": "srgba", "hex": "#RRGGBB" }` when alpha is `1.0`, and `#RRGGBBAA` when alpha is not `1.0`.
- `Color::LinearRgba` serializes as `{ "space": "linear_rgba", "components": [r, g, b, a] }`.
- `Color::Hsla` serializes as `{ "space": "hsla", "components": [h_degrees, s, l, a] }`.
- `Color::Hsva` serializes as `{ "space": "hsva", "components": [h_degrees, s, v, a] }`.
- `Srgba`, `LinearRgba`, `Hsla`, and `Hsva` serialize as the same shape as their corresponding `Color` variant, not as tuple structs.
- Deserialization rejects missing `space`, unknown `space`, malformed hex, wrong component length, non-finite floats, negative alpha, and alpha above `1.0`.

## Alpha And Color-Space Rules

- Canonical color structs store straight alpha.
- `PremultipliedLinearRgba` is a distinct final-boundary type used only at GPU/painter upload or blending boundaries.
- `Color::to_linear_rgba()` converts only RGB channels from sRGB to linear; alpha is copied unchanged.
- `Srgba::to_u8_array()` rounds channel values with clamp to `[0, 255]` for editor/painter boundaries.
- UI authored colors are `Srgba` by default when parsed from CSS-like strings.
- Material authored colors are `LinearRgba` by default in material TOML.
- glTF texture color-space rules:
  - `baseColorTexture`: `TextureColorSpace::Srgb`.
  - `emissiveTexture`: `TextureColorSpace::Srgb`.
  - `metallicRoughnessTexture`: `TextureColorSpace::NonColor`.
  - `normalTexture`: `TextureColorSpace::NonColor`.
  - `occlusionTexture`: `TextureColorSpace::NonColor`.
  - Plain imported image textures default to `Srgb` unless importer settings say `linear` or `non_color`.

## Milestone M1: Canonical Color Foundation

- Goal: Establish shared color types, parse/format, conversion, alpha semantics, texture color-space tags, and contract tests before any consumer migration.
- In-scope behaviors: `Srgba`, `LinearRgba`, `Hsla`, `Hsva`, `Color`, `TextureColorSpace`, CSS-like parse/format, serde shape, sRGB-linear conversion, straight-alpha vs premultiplied conversion.
- Dependencies: current `zircon_runtime_interface::math` vector aliases and serde availability.

### Implementation Slices

- [ ] Convert `zircon_runtime_interface/src/math.rs` into a structural math module that can expose `pub mod color;` without losing existing `Real`, `Vec2`, `Vec3`, `Vec4`, `Quat`, `Mat4`, render aliases, finite helpers, and `Transform`.
- [ ] Add `zircon_runtime_interface/src/math/color/mod.rs` with only child module declarations and `pub use` exports for `Srgba`, `Srgba8`, `LinearRgba`, `PremultipliedLinearRgba`, `Hsla`, `Hsva`, `Color`, `ColorParseError`, `ColorComponentError`, `TextureColorSpace`, and `AlphaRepresentation`.
- [ ] Implement `Srgba` in `srgba.rs` with fields `red`, `green`, `blue`, `alpha`, constants `BLACK`, `WHITE`, `NONE`, `RED`, `GREEN`, `BLUE`, constructors `new`, `rgb`, `rgb_u8`, `rgba_u8`, `from_u8_array`, `to_u8_array`, `to_hex`, and `parse`.
- [ ] Implement exact sRGB transfer functions on `Srgba`: `srgb_to_linear_component(value: f32) -> f32` and `linear_to_srgb_component(value: f32) -> f32` using the standard `0.04045` and `0.0031308` thresholds.
- [ ] Implement `LinearRgba` in `linear_rgba.rs` with fields `red`, `green`, `blue`, `alpha`, constants `BLACK`, `WHITE`, `NONE`, `RED`, `GREEN`, `BLUE`, constructors `new`, `rgb`, `from_vec4`, `to_vec4`, `is_finite`, `premultiply`, `From<Srgba>`, and fallible `PremultipliedLinearRgba::unpremultiply() -> Option<LinearRgba>` that returns `None` when alpha is `0.0` and any premultiplied RGB channel is non-zero.
- [ ] Implement `Hsla` and `Hsva` with hue in degrees and saturation/lightness/value/alpha in `[0.0, 1.0]`; normalize hue by wrapping into `[0.0, 360.0)` and reject non-finite components.
- [ ] Implement the `Color` enum with `to_srgba`, `to_linear_rgba`, `with_alpha`, `alpha`, `is_finite`, and constructors `Color::srgba`, `Color::linear_rgba`, `Color::hsla`, `Color::hsva`, `Color::hex`.
- [ ] Implement CSS-like parsing in `format.rs` for `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb(255, 128, 0)`, `rgb(100%, 50%, 0%)`, `rgba(255, 128, 0, 0.5)`, and `rgba(100%, 50%, 0%, 50%)`.
- [ ] Implement error variants that tests can assert directly: `Empty`, `UnsupportedFormat`, `InvalidHexLength`, `InvalidHexDigit`, `InvalidFunction`, `WrongComponentCount`, `InvalidComponent`, `NonFiniteComponent`, `ComponentOutOfRange`, and `UnknownColorSpace`.
- [ ] Implement serde as the canonical tagged shape from the Serialization Contract section. Do not accept legacy bare strings or arrays.
- [ ] Implement `TextureColorSpace` with serde rename-all snake-case and values `Srgb`, `Linear`, `NonColor`.
- [ ] Add tests in `zircon_runtime_interface/src/tests/color_contracts.rs` for parse examples, formatting, sRGB-linear numeric known values, HSL/HSV conversion roundtrips, serde TOML/JSON roundtrips, error cases, alpha premultiply, and texture color-space serde.
- [ ] Update `zircon_runtime_interface/src/tests/mod.rs` with `mod color_contracts;`.

### Lightweight Checks

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/math.rs" "zircon_runtime_interface/src/math/color/mod.rs" "zircon_runtime_interface/src/math/color/srgba.rs" "zircon_runtime_interface/src/math/color/linear_rgba.rs" "zircon_runtime_interface/src/math/color/hsla.rs" "zircon_runtime_interface/src/math/color/hsva.rs" "zircon_runtime_interface/src/math/color/color.rs" "zircon_runtime_interface/src/math/color/alpha.rs" "zircon_runtime_interface/src/math/color/format.rs" "zircon_runtime_interface/src/math/color/error.rs" "zircon_runtime_interface/src/math/color/serde.rs" "zircon_runtime_interface/src/math/color/texture_color_space.rs" "zircon_runtime_interface/src/tests/color_contracts.rs"`
- `cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-foundation-interface" --message-format short --color never`

### Testing Stage

- Run `cargo test -p zircon_runtime_interface --lib color_contracts --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-foundation-interface" --message-format short --color never`.
- If any consumer fails to compile before migration, do not add compatibility types. Record stale caller paths and proceed to M2/M3 cutover tasks.

### Exit Evidence

- `color_contracts` tests pass.
- `zircon_runtime_interface` check passes or only fails at known consumer compile sites scheduled in M2.

## Milestone M2: UI And Editor Color Contract Cutover

- Goal: Replace UI/editor semantic string colors with canonical `Color` and remove editor-local hex parsing.
- In-scope behaviors: UI value color, resolved style colors, brush colors/tints/gradients, text paint/decorations/cursor colors, visualizer overlays, Slint boundary conversion, painter conversion, editor component showcase state, Inspector semantic color projection.
- Dependencies: M1 color foundation.

### Implementation Slices

- [ ] Update `UiValue::Color(String)` to `UiValue::Color(Color)` in `zircon_runtime_interface/src/ui/component/value.rs` and adjust `kind`, `display_text`, `from_toml_with_kind`, and `to_toml` to use canonical serde/formatting.
- [ ] Update `UiResolvedStyle` color fields in `resolved_style.rs` from `Option<String>` to `Option<Color>`.
- [ ] Update `UiBrushPayload` constructors in `brush.rs` to accept `impl Into<Color>` or explicit `Color`, and change all payload fields from strings to `Color` / `Option<Color>`.
- [ ] Update `UiTextPaint`, `UiTextPaintRun`, and `UiTextPaintDecoration` in `text_shape.rs` so paint color fields use `Color`.
- [ ] Replace `TEXT_SELECTION_COLOR`, `TEXT_CARET_COLOR`, and `TEXT_COMPOSITION_UNDERLINE_COLOR` in `command.rs` with `Srgba::rgba_u8(...)` constants and convert them with `Color::from(...)` at command construction sites.
- [ ] Update `visualizer.rs` overlay color literals to construct `Color` values.
- [ ] Update `UiTextCursorStyle.color` in `zircon_runtime_interface/src/ui/text.rs` to `Option<Color>`.
- [ ] Update all UI contract tests and template parsing tests to use canonical color TOML/JSON objects, not bare `"#ffffff"` strings.
- [ ] Delete local hex parsing from `zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs` and convert colors through shared `Color` APIs at the `[u8; 4]` framebuffer boundary.
- [ ] Update editor painter/theme constants so semantic constants are `Color` or `Srgba`; `[u8; 4]` remains only inside final pixel drawing functions.
- [ ] Update Slint conversion paths in `template_node_conversion.rs` and related host data code so `UiValue::Color(Color)` maps to `slint::Color::from_argb_u8(a, r, g, b)` using `Srgba::to_u8_array()`.
- [ ] Update editor showcase/demo state to emit `UiValue::Color(Color::hex("#ffcc33").expect("valid demo color"))`, not `UiValue::Color("#ffcc33".to_string())`.
- [ ] Add or update editor host tests that previously asserted string colors so they assert canonical color values or final pixel output.
- [ ] Add `SceneInspectorFieldValue::Color(Color)` in `zircon_editor/src/scene/viewport/edit_mode_projection/scene_inspector_field_value.rs`; update `edit_mode_projection/build.rs` to use it for mesh tint and material color fields instead of `Vec4`.
- [ ] Search `zircon_runtime_interface/src/ui`, `zircon_runtime/src/ui`, and `zircon_editor/src` for `Option<String>` color fields, `color: String`, `parse_style_color`, and `#` color literals in Rust code; rewrite semantic color uses to shared `Color`.

### Lightweight Checks

- Run `rustfmt --edition 2021 --check` on every Rust file changed in M2. The minimum expected set is `zircon_runtime_interface/src/ui/component/value.rs`, `zircon_runtime_interface/src/ui/surface/render/resolved_style.rs`, `zircon_runtime_interface/src/ui/surface/render/brush.rs`, `zircon_runtime_interface/src/ui/surface/render/text_shape.rs`, `zircon_runtime_interface/src/ui/surface/render/command.rs`, `zircon_runtime_interface/src/ui/surface/render/visualizer.rs`, `zircon_runtime_interface/src/ui/text.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/text.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/theme.rs`, `zircon_editor/src/ui/slint_host/ui/template_node_conversion.rs`, `zircon_editor/src/ui/slint_host/host_contract/data/template_nodes.rs`, `zircon_editor/src/scene/viewport/edit_mode_projection/build.rs`, and `zircon_editor/src/scene/viewport/edit_mode_projection/scene_inspector_field_value.rs`. Include any additional files changed by the UI/template search slice.
- `cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-ui-interface" --message-format short --color never`.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-editor" --message-format short --color never` if unrelated active editor drift does not block; otherwise record exact external blocker.

### Testing Stage

- Run `cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-ui-interface" --message-format short --color never`.
- Run focused editor tests that cover color field projection and painter output:
  - `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-editor" --message-format short --color never`.
  - `cargo test -p zircon_editor --lib native_runtime_text_painter --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-editor" --message-format short --color never`.
  - `cargo test -p zircon_editor --lib slint_window --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-editor" --message-format short --color never`.
  - `cargo test -p zircon_editor --lib template_runtime --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-editor" --message-format short --color never`.
  - `cargo test -p zircon_editor --lib viewport --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-editor" --message-format short --color never`.
  - If an editor command fails in files outside this color lane, record the exact failing test name, first error file, and active session note that owns it before narrowing the filter.
- Run `git diff --check -- "zircon_runtime_interface/src/ui" "zircon_runtime_interface/src/tests" "zircon_runtime/src/ui" "zircon_editor/src/ui" "zircon_editor/src/scene/viewport/edit_mode_projection"`.

### Exit Evidence

- No semantic UI/editor color fields remain as `String`.
- No editor-local semantic hex parser remains; only shared color parser handles CSS-like syntax.
- UI contract tests pass; editor tests pass or external blockers are recorded with exact failing files outside this lane.

## Milestone M3: Material, Texture, And Asset Serialization Cutover

- Goal: Replace material color arrays and texture color-space gaps with shared color types and hard-cutover serialized asset formats.
- In-scope behaviors: material TOML, texture asset metadata, importer settings for texture color space, built-in material assets, asset tests, project manager artifact tests.
- Dependencies: M1 foundation and any M2 shared UI type compile fixes that touch `zircon_runtime_interface`.

### Implementation Slices

- [ ] Change `MaterialAsset.base_color` to `LinearRgba` in `zircon_runtime/src/asset/assets/material.rs`.
- [ ] Change `MaterialAsset.emissive` to `LinearRgba` with alpha fixed to `1.0`; use the `LinearRgba::rgb(red, green, blue)` constructor from M1 for emissive convenience.
- [ ] Change `TextureAsset` in `texture.rs` to include `color_space: TextureColorSpace` and rename the semantic texture payload field from `rgba` to `rgba8`.
- [ ] Update `import_texture.rs` to set `color_space` from `context.import_settings.get("color_space")`; valid values are `"srgb"`, `"linear"`, and `"non_color"`; default is `Srgb`.
- [ ] Update `import_material.rs` material TOML fixtures/tests to use tagged `LinearRgba` serialized colors.
- [ ] Update built-in materials in `pipeline/manager/builtins/builtin_resources.rs`: default material `base_color` uses `LinearRgba::WHITE`, missing material `base_color` uses `LinearRgba::new(1.0, 0.0, 1.0, 1.0)`, and both `emissive` fields use `LinearRgba::BLACK`.
- [ ] Update all `MaterialAsset` constructors in runtime tests and support code found by `grep "MaterialAsset \{"`.
- [ ] Update all `TextureAsset` constructors and texture-asset `.rgba` call sites found by searching `TextureAsset {`, `texture.rgba`, `payload.rgba`, and `ImportedAsset::Texture` so texture assets use `.rgba8` and `color_space`. Do not rename `CapturedFrame.rgba`, `ViewportFrame.rgba`, `CpuTexturePayload.rgba`, or local function parameters named `rgba` for final pixel buffers.
- [ ] Update asset pipeline manager tests that mutate `material.base_color = [...]` to assign `LinearRgba`.
- [ ] Delete the test `material_asset_parses_legacy_locator_only_references` or rewrite it so only reference locator compatibility remains tested; do not keep legacy color array compatibility in the same test.
- [ ] Add negative tests proving old `base_color = [0.8, 0.8, 0.8, 1.0]` is rejected by `MaterialAsset::from_toml_str`.
- [ ] Add texture importer tests for default `Srgb`, explicit `Linear`, explicit `NonColor`, and invalid color-space setting errors.

### Lightweight Checks

- Run `rustfmt --edition 2021 --check` on every Rust file changed in M3. The minimum expected set is `zircon_runtime/src/asset/assets/material.rs`, `zircon_runtime/src/asset/assets/texture.rs`, `zircon_runtime/src/asset/importer/ingest/import_texture.rs`, `zircon_runtime/src/asset/importer/ingest/import_material.rs`, `zircon_runtime/src/asset/pipeline/manager/builtins/builtin_resources.rs`, and any runtime tests/support files found by the `MaterialAsset` / `TextureAsset` searches.
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-runtime-asset" --message-format short --color never`.

### Testing Stage

- Run `cargo test -p zircon_runtime --lib material_asset --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-runtime-asset" --message-format short --color never`.
- Run `cargo test -p zircon_runtime --lib importer --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-runtime-asset" --message-format short --color never`.
- Run `cargo test -p zircon_runtime --lib asset --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-runtime-asset" --message-format short --color never` after the narrower material/importer filters pass; if this broader asset filter exposes unrelated active-lane failures, record the exact failing test names and keep the narrower color evidence.

### Exit Evidence

- No `MaterialAsset.base_color: [f32; 4]`, `MaterialAsset.emissive: [f32; 3]`, or semantic material color array tests remain.
- Texture assets carry explicit `TextureColorSpace`.
- Old material color arrays fail deserialization by test.

## Milestone M4: glTF Material And Texture Color Import Rules

- Goal: Make glTF import produce color-aware material and texture asset entries, with explicit color-space rules for PBR texture roles.
- In-scope behaviors: glTF material factors, texture role color-space tags, imported material entries, dependencies from model root to material/texture outputs, importer tests.
- Dependencies: M3 asset color types.

### Implementation Slices

- [ ] Update `import_gltf.rs` to iterate `document.materials()` and create one `ImportedAssetEntry` per glTF material using a stable locator label built as `format!("material/{}", material.index().unwrap_or(material_ordinal))`, producing locators such as `res://models/robot.gltf#material/0`. Current `ResourceLocator` labels are stable and `AssetReference::from_locator` derives the stable UUID from the full locator text.
- [ ] Use `AssetReference::from_locator(material_or_texture_locator.clone())` for material and texture references; do not introduce direct raw string references.
- [ ] Map `pbr_metallic_roughness().base_color_factor()` to `MaterialAsset.base_color: LinearRgba::new(r, g, b, a)` and add a code comment that glTF numeric PBR factors are linear multipliers while referenced base color textures are tagged `TextureColorSpace::Srgb`.
- [ ] Map `material.emissive_factor()` to `MaterialAsset.emissive: LinearRgba::new(r, g, b, 1.0)`.
- [ ] Map `alpha_mode` and alpha cutoff from glTF to `AlphaMode::{Opaque, Mask, Blend}`.
- [ ] Map `double_sided` from glTF to `MaterialAsset.double_sided`.
- [ ] For each glTF texture role, create or reference a `TextureAsset` with the correct `TextureColorSpace`: base color/emissive `Srgb`, metallic-roughness/normal/occlusion `NonColor`. If the current importer cannot safely extract embedded texture bytes from `gltf::import` images in this milestone, emit a `ResourceDiagnostic::warning` on the material entry naming the unsupported texture role and still add tests for factor-only material import.
- [ ] Add dependencies to the root model import outcome for emitted material and texture locators using `ImportedAssetEntry::with_dependency` or direct `entry.dependencies.push(locator)`.
- [ ] Keep `ModelPrimitiveAsset` unchanged in M4. It currently has `vertices`, `indices`, and `virtual_geometry` only; primitive-material binding is explicitly deferred to a later model-schema milestone. M4 must still emit material assets and dependencies so the asset graph can track imported materials.
- [ ] Extend `importer_decodes_obj_and_gltf_into_model_assets` or add a new focused test with a glTF fixture containing base color factor, alpha mode, double-sided flag, and texture role declarations.
- [ ] Add tests asserting imported texture color-space tags for each glTF PBR role.

### Lightweight Checks

- `rustfmt --edition 2021 --check "zircon_runtime/src/asset/importer/ingest/import_gltf.rs" "zircon_runtime/src/asset/tests/assets/importer.rs" "zircon_runtime/src/asset/assets/material.rs" "zircon_runtime/src/asset/assets/texture.rs" "zircon_runtime/src/asset/assets/model.rs"`.
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-gltf" --message-format short --color never`.

### Testing Stage

- Run `cargo test -p zircon_runtime --lib gltf --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-gltf" --message-format short --color never`.
- Run `cargo test -p zircon_runtime --lib importer_decodes_obj_and_gltf_into_model_assets --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-gltf" --message-format short --color never` after the broader `gltf` filter so the existing OBJ+glTF smoke test remains covered.

### Exit Evidence

- glTF importer tests prove material factor import, alpha mode import, and texture color-space tagging.
- Any intentionally deferred primitive-material binding is documented with exact current schema blocker.

## Milestone M5: Runtime Graphics Linear Consumption

- Goal: Ensure renderer/material runtime consumes linear colors and respects texture color-space metadata at upload/sampling boundaries.
- In-scope behaviors: material runtime color types, resource streamer conversion, GPU texture format selection, CPU sample helpers, rendering tests.
- Dependencies: M3 material/texture asset types and M4 glTF output if glTF tests depend on rendering.

### Implementation Slices

- [ ] Change `MaterialRuntime` and `MaterialCaptureSeed` semantic color fields to `LinearRgba` in `material_runtime.rs`.
- [ ] Update `resource_streamer_ensure_material.rs` to copy `material.base_color` and `material.emissive` directly as `LinearRgba`.
- [ ] Update all GPU packing and CPU renderer code that expects `Vec4`/`Vec3` material colors to call `to_vec4()` at the final boundary.
- [ ] Update `gpu_texture_resource_from_asset.rs` to select `wgpu::TextureFormat::Rgba8UnormSrgb` for `TextureColorSpace::Srgb` and `wgpu::TextureFormat::Rgba8Unorm` for `Linear` and `NonColor` through a helper `texture_format_for_color_space(color_space: TextureColorSpace) -> wgpu::TextureFormat` in the GPU texture resource module.
- [ ] Update fallback texture creation helpers to assign explicit color-space metadata.
- [ ] Update `sample_texture_asset_rgba` in `resource_streamer_accessors.rs` so sRGB texture samples convert to linear before multiplication with material base colors; keep alpha as straight normalized `[0.0, 1.0]`.
- [ ] Update graphics tests in `zircon_runtime/src/graphics/tests/m4_behavior_layers.rs`, `project_render.rs`, and material capture/Hybrid GI seams that construct or compare colors.
- [ ] Add a focused renderer/resource-streamer test proving a mid-gray sRGB material/texture path differs from a linear path by the expected transfer function.

### Lightweight Checks

- `rustfmt --edition 2021 --check "zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs" "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs" "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs" "zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs" "zircon_runtime/src/graphics/scene/scene_renderer/post_process/fallback_texture/create_view.rs" "zircon_runtime/src/graphics/tests/m4_behavior_layers.rs" "zircon_runtime/src/graphics/tests/project_render.rs"`.
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-graphics" --message-format short --color never`.

### Testing Stage

- Run `cargo test -p zircon_runtime --lib color --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-graphics" --message-format short --color never`.
- Run focused graphics tests that cover material/resource streaming and color-sensitive render output. Start with filters `m4_behavior_layers`, `project_render`, `material`, and any new test names added in this milestone.

### Exit Evidence

- Runtime material code no longer uses `Vec4`/`Vec3` as semantic color storage.
- GPU upload/sampling boundary names `TextureColorSpace` explicitly.
- Tests prove sRGB-linear conversion affects render/material behavior.

## Milestone M6: Docs, Global Search, And Hard-Cutover Validation

- Goal: Document the new color architecture, prove old semantic representations were removed, and record validation evidence.
- In-scope behaviors: docs with machine-readable headers, grep-based old representation audit, scoped and expanded Cargo validation, coordination note archival.
- Dependencies: M1-M5 implementation.

### Implementation Slices

- [ ] Create `docs/zircon_runtime_interface/math/color.md` with YAML frontmatter listing all related color module files, implementation files, plan source, and tests.
- [ ] Create `docs/zircon_runtime/asset/color-and-material-import.md` with YAML frontmatter listing `MaterialAsset`, `TextureAsset`, importers, glTF importer, resource streamer, and tests.
- [ ] Update `docs/zircon_runtime_interface/ui/contract-spine.md` to describe `Color` usage in UI values/styles/brushes/text/visualizer/cursor.
- [ ] Update `docs/assets-and-rendering/render-framework-architecture.md` to describe linear material runtime colors and texture color-space sampling.
- [ ] Update `docs/ui-and-layout/slate-style-ui-surface-frame.md` to describe shared color use for surface frame rendering and editor painter handoff.
- [ ] Update `.codex/sessions/20260508-2057-color-foundation-design.md` with final touched modules, blockers, validation evidence, and completion status.
- [ ] Run hard-cutover searches and resolve all semantic hits:
  - `grep`/`rg` equivalent for `UiValue::Color("`, `Color(String)`, `color: String`, `Option<String>` near color fields, `parse_style_color`, `base_color: \[`, `emissive: \[`, `TextureAsset` payload `.rgba`, and `Vec4` fields labelled `Color`.
  - Remaining hits are allowed only for final framebuffer pixels (`[u8; 4]`), generic vector math not representing color, test names documenting rejected legacy formats, or external reference trees under `dev/`.

### Testing Stage

- Before running build/test, check free space on the target drive with `Get-PSDrive -Name E | Select-Object Name,Free,Used`; if free space is `<= 50 GB`, run `cargo clean --target-dir "E:\cargo-targets\zircon-color-final-interface"`, `cargo clean --target-dir "E:\cargo-targets\zircon-color-final-runtime"`, and `cargo clean --target-dir "E:\cargo-targets\zircon-color-final-editor"` before the final scoped validation commands.
- Run `rustfmt --edition 2021 --check` with the full touched Rust file list from `git diff --name-only -- "*.rs"`; pass the expanded file list explicitly to `rustfmt` rather than leaving the glob as a shell expression.
- Run `git diff --check -- "zircon_runtime_interface" "zircon_runtime" "zircon_editor" "docs" ".codex/sessions"`.
- Run `cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-final-interface" --message-format short --color never`.
- Run `cargo test -p zircon_runtime_interface --lib color_contracts --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-final-interface" --message-format short --color never`.
- Run `cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-final-interface" --message-format short --color never`.
- Run `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-final-runtime" --message-format short --color never`.
- Run `cargo test -p zircon_runtime --lib importer --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-final-runtime" --message-format short --color never`.
- Run `cargo test -p zircon_runtime --lib material --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-final-runtime" --message-format short --color never`.
- Run `cargo test -p zircon_runtime --lib gltf --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-final-runtime" --message-format short --color never`.
- Run `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-color-final-editor" --message-format short --color never` after refreshing `.codex/sessions` for active editor-session blockers. If unrelated editor drift blocks this, record exact failures and run the five M2 editor test filters above with the same target dir.
- Because this touches shared APIs, expand to workspace validation if active unrelated lanes are not blocking. Preferred command: `.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -RepoRoot "E:\Git\ZirconEngine" -TargetDir "E:\cargo-targets\zircon-color-workspace"`. If the validator is unavailable, run `cargo build --workspace --locked --verbose --target-dir "E:\cargo-targets\zircon-color-workspace"` followed by `cargo test --workspace --locked --verbose --target-dir "E:\cargo-targets\zircon-color-workspace"`. If broad workspace validation fails in unrelated active lanes, record exact external failures and do not claim workspace green.

### Exit Evidence

- All scoped color/UI/asset/runtime checks pass.
- Hard-cutover searches show no old semantic color representations in touched production paths.
- Docs contain machine-readable headers and current validation evidence.
- Active coordination note is archived or deleted after completion.

## Out Of Scope

- No reflection/type-registry implementation for color metadata; this is coordinated with the active reflection session and should land later if needed.
- No compatibility deserialization for old material `[f32; 4]` colors or UI bare `"#rrggbb"` color fields.
- No broad render pipeline redesign beyond texture color-space format selection and material color conversion boundaries.
- No palette/theme UX redesign; editor swatches reuse the canonical type but visual design stays unchanged.
- No CSS named colors unless implemented as a small table in M1 without growing scope. Required CSS-like support is hex/rgb/rgba.

## Implementation Warnings

- This repository is heavily dirty with active parallel work. Never revert unrelated changes.
- Do not edit reflection registry files unless a compile failure is directly caused by this color migration.
- Do not preserve old color fields as `legacy_*`, compatibility aliases, or `pub use` shim modules.
- Root files (`math.rs`, `mod.rs`, `lib.rs`) must stay structural; put parsing/conversion behavior in child files.
- If a touched source file approaches 1000 lines or gains a second responsibility, split it before adding behavior.
- When an upper-layer UI/editor/render test fails, diagnose the shared color foundation first before patching the upper layer.

## Plan Self-Review Checklist

- Requirement coverage:
  - Canonical color module: M1.
  - `Srgba`, `LinearRgba`, `Hsla`, `Hsva`: M1.
  - sRGB-linear conversion strategy: M1 and M5.
  - CSS-like hex/rgb parse/format: M1.
  - Unified material/UI/asset serialization: M1-M3.
  - glTF base color, texture sRGB, linear factor rules: M4.
  - Straight vs premultiplied alpha: M1 and M5.
  - Editor swatch/Inspector reuse: M2.
  - Conversion/parse/roundtrip/error tests: M1-M6.
- Placeholder scan: this plan intentionally contains no placeholder tasks; each milestone names concrete files, commands, and exit evidence.
- Type consistency: semantic colors are `Color`, `Srgba`, or `LinearRgba`; `[u8; 4]` is only final framebuffer pixels; `Vec4` is only final math/GPU packing, not semantic color storage.
