# Typed Shader Definitions Design

## Goal

Add typed shader definition values to the Zircon shader asset and neutral render shader contract so shader variants can carry Bevy-style bool, signed integer, and unsigned integer definitions instead of only raw string flags.

This is the next asset/framework-owned M6 shader slice after `ShaderReadinessReport`. It stays below renderer shader cache work: it changes serialized/value DTOs, shader asset projection, readiness diagnostics, and docs, but it does not compose WGSL imports, allocate WGPU objects, requeue pipelines, or change graphics resource preparation.

## Current Context

- `ShaderAsset.shader_defs` is currently `Vec<String>`.
- `ZShaderDocument.shader_defs` is currently `Vec<String>`, so `.zshader` supports only string flags such as `shader_defs = ["USE_UNLIT"]`.
- `RenderShaderVariantKey.defines` is currently `Vec<String>` and `ShaderAsset::variant_keys()` clones `ShaderAsset.shader_defs` into every entry-point variant key.
- `ShaderReadinessReport` already validates shader definition hygiene by trimming strings, rejecting empty names, and rejecting duplicate normalized names.
- The active texture upload readiness session owns texture/upload modules, the active Bevy rendering parity session owns material/render gates, and active ECS/UI/Hub/app/sound sessions own unrelated validation queues. This slice must avoid those areas.

## Reference Evidence

### Bevy

Bevy is the primary reference for this slice because Zircon's asset-stack roadmap is explicitly Bevy-style and the current gap is directly visible in Bevy shader code.

- `dev/bevy/crates/bevy_shader/src/shader_cache.rs:84-92` defines `ShaderDefVal::{Bool(String, bool), Int(String, i32), UInt(String, u32)}` as compile-time shader values.
- `dev/bevy/crates/bevy_shader/src/shader_cache.rs:94-104` converts `&str` and `String` into `ShaderDefVal::Bool(name, true)`, preserving ergonomic flag-style definitions.
- `dev/bevy/crates/bevy_shader/src/shader_cache.rs:188-196` documents shader defs as shader-cache key inputs.
- `dev/bevy/crates/bevy_shader/src/shader_cache.rs:276-290` maps bool/int/uint shader defs into `naga_oil::compose::ShaderDefValue`, proving typed values matter before concrete renderer pipeline creation.
- `dev/bevy/crates/bevy_shader/src/shader.rs:33-55` stores shader defs on the shader asset alongside source/import data, matching Zircon's asset-owned landing zone.
- `dev/bevy/crates/bevy_sprite_render/src/mesh2d/mesh.rs:598-607` mixes boolean flags such as `TONEMAP_IN_SHADER` with unsigned integer defs for binding indices, showing why string-only flags are not enough for real pipeline specialization.

### Zircon Current Shape

Zircon already has the right owner boundaries for the narrow slice.

- `zircon_runtime/src/core/framework/render/shader/variant_key.rs` owns the neutral variant key that renderer and material layers can consume without importing asset internals.
- `zircon_runtime/src/asset/assets/shader/shader_asset.rs` owns shader asset projection into `RenderShaderVariantKey`.
- `zircon_runtime/src/asset/assets/shader/zshader.rs` owns the `.zshader` authoring document and shader material schema DTOs.
- `zircon_runtime/src/asset/assets/shader/readiness.rs` owns standalone shader payload readiness diagnostics and already reports shader-def rows.

## Chosen Design

Introduce a typed shader definition DTO in the neutral render shader contract and reuse it from the shader asset layer.

```rust
pub enum RenderShaderDefinitionValue {
    Bool { name: String, value: bool },
    Int { name: String, value: i32 },
    UInt { name: String, value: u32 },
}
```

The asset layer stores the neutral DTO directly rather than defining a second shader-def value type. The goal is one value model from `.zshader` import through `ShaderAsset` to `RenderShaderVariantKey` and readiness diagnostics.

`RenderShaderVariantKey` changes from string-only `defines: Vec<String>` to typed `defines: Vec<RenderShaderDefinitionValue>`. Callers that currently construct flag definitions can use a helper constructor or `From<String>`/`From<&str>` so old flag-style code remains concise.

Existing `.zshader` source compatibility is preserved:

```toml
shader_defs = ["USE_UNLIT", "ALPHA_CLIP"]
```

Those rows import as `Bool { name, value: true }`, matching Bevy's `From<&str> for ShaderDefVal` behavior. New typed rows should use a separate optional field to avoid ambiguous mixed TOML array typing:

```toml
[[shader_def_values]]
name = "TONEMAPPING_LUT_TEXTURE_BINDING_INDEX"
kind = "uint"
value = 2

[[shader_def_values]]
name = "ENABLE_FOG"
kind = "bool"
value = true

[[shader_def_values]]
name = "DEBUG_MODE"
kind = "int"
value = -1
```

The final `ShaderAsset.shader_defs` stores typed values from both fields in authoring order: legacy `shader_defs` first, then `shader_def_values`. This keeps existing files stable while giving newer assets typed values.

## DTO And API Shape

The implementation should keep the public types small, serializable, hashable, and usable as future renderer cache-key material.

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RenderShaderDefinitionValue {
    Bool { name: String, value: bool },
    Int { name: String, value: i32 },
    UInt { name: String, value: u32 },
}

impl RenderShaderDefinitionValue {
    pub fn bool(name: impl Into<String>, value: bool) -> Self;
    pub fn int(name: impl Into<String>, value: i32) -> Self;
    pub fn uint(name: impl Into<String>, value: u32) -> Self;
    pub fn name(&self) -> &str;
    pub fn value_as_string(&self) -> String;
}
```

`From<&str>` and `From<String>` should create `Bool { value: true }`. This keeps current tests and constructors readable after their field types change.

`ShaderAsset` should become:

```rust
pub struct ShaderAsset {
    pub shader_defs: Vec<RenderShaderDefinitionValue>,
    // existing fields unchanged
}
```

`ZShaderDocument` should preserve old and new source fields:

```rust
pub struct ZShaderDocument {
    #[serde(default)]
    pub shader_defs: Vec<String>,
    #[serde(default)]
    pub shader_def_values: Vec<ZShaderDefinitionValueDocument>,
}

pub struct ZShaderDefinitionValueDocument {
    pub name: String,
    pub kind: String,
    pub value: toml::Value,
}
```

The importer should parse `shader_def_values` into typed values and add parse diagnostics for invalid kind/value combinations. It should not silently coerce booleans to integers or negative integers to unsigned values.

## Readiness Semantics

`ShaderReadinessReport` should continue to expose `shader_defs: Vec<ShaderDefinitionReadiness>`, but each row should include the typed value so callers can display and compare the effective variant inputs.

```rust
pub struct ShaderDefinitionReadiness {
    pub raw_name: String,
    pub normalized_name: String,
    pub value: RenderShaderDefinitionValue,
    pub diagnostic: Option<String>,
}
```

Readiness validation remains deterministic and asset-local:

- Trim names for duplicate/empty-name checks.
- Empty normalized names are invalid.
- Duplicate normalized names are invalid even if values differ, because two rows with the same shader definition name produce ambiguous specialization state.
- Bool false definitions are valid and must not be dropped by readiness. They are explicit cache/signature inputs.
- Int and uint values are valid only after source parsing has produced the typed DTO. Range errors belong to `.zshader` parse/import diagnostics.

`ShaderReadinessReport::is_ready()` should keep the existing rule: any shader definition diagnostic makes the shader not ready.

## Ownership And Boundaries

In scope:

- `zircon_runtime/src/core/framework/render/shader/variant_key.rs`
- `zircon_runtime/src/core/framework/render/shader/mod.rs`
- `zircon_runtime/src/asset/assets/shader/shader_asset.rs`
- `zircon_runtime/src/asset/assets/shader/zshader.rs`
- `zircon_runtime/src/asset/assets/shader/readiness.rs`
- focused shader asset/project tests and docs

Out of scope:

- WGSL import composition or `naga_oil` integration
- graphics shader module preparation, WGPU module creation, and pipeline cache invalidation
- material renderer preparation, PBR surface expansion, bind-group reflection, and fallback policy changes
- texture upload readiness, mesh/glTF importer paths, editor UI surfacing, plugin workspace changes, root lockfile changes, and broad formatting

## Compatibility Rules

- Existing `.zshader` files with `shader_defs = ["NAME"]` remain valid and import as bool-true definitions.
- Existing programmatic `ShaderAsset` constructors need mechanical updates from `Vec<String>` to typed values. Use `"NAME".into()` where possible.
- Existing docs that say string shader defs are copied into `RenderShaderVariantKey` should be updated to say flag strings become bool-true typed values.
- Do not keep a second long-term `defines: Vec<String>` field beside the typed field. The old string path is an authoring compatibility input, not a parallel runtime contract.

## Tests

Focused test coverage should include:

- `RenderShaderDefinitionValue::from("FLAG")` creates a bool-true definition and preserves `value_as_string()` behavior.
- `.zshader` legacy `shader_defs = ["USE_UNLIT"]` imports into typed bool-true shader defs and variant keys.
- `.zshader` `[[shader_def_values]]` imports bool, int, and uint rows into `ShaderAsset.shader_defs` and `RenderShaderVariantKey.defines`.
- Invalid typed rows produce deterministic diagnostics: unknown kind, non-bool bool value, non-integer int/uint value, and negative uint value.
- `ShaderReadinessReport` reports typed values, rejects empty names, rejects duplicate normalized names across legacy and typed rows, and does not reject a bool-false value by itself.
- Existing compound `.zshader` project fixture keeps its current `shader_defs` behavior after the type migration.

Milestone testing should stay scoped unless a broader integration stage is explicitly selected:

```powershell
cargo test -p zircon_runtime --lib shader --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1
cargo test -p zircon_runtime --lib project_manager_imports_compound_zshader_package_with_subassets --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1
cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs
```

Run `rustfmt --edition 2021 --check` for touched Rust files and `git diff --check -- <touched files>` in the final hygiene stage. Do not claim workspace-wide validation unless the full validator is run after this slice and passes.

## Documentation

Update existing docs rather than creating a duplicate module page:

- `docs/zircon_runtime/core/framework/render/shader.md`: document `RenderShaderDefinitionValue`, variant-key typing, and the boundary before renderer shader cache work.
- `docs/zircon_runtime/asset/render-assets.md`: document `.zshader` legacy flag compatibility, typed definition rows, readiness diagnostics, and validation evidence.
- `docs/zircon_runtime/asset/zmeta-shader-material.md`: document the `.zshader` authoring schema for `shader_defs` and `shader_def_values`.

Each changed module doc must keep its machine-readable header current with related code, implementation files, plan sources, and focused tests.

## Deliberate Divergence

- Bevy stores `ShaderDefVal` in `bevy_shader`. Zircon should store the equivalent value in `zircon_runtime::core::framework::render::shader` because that is the neutral render contract already shared by assets and graphics.
- Bevy allows order/redundancy to affect cache hits. Zircon readiness should reject duplicate normalized names early because this asset layer already has deterministic authoring diagnostics and no renderer cache compatibility burden yet.
- Bevy feeds typed definitions into `naga_oil` composition. Zircon records typed values now but does not compose WGSL imports or validate Naga modules in this slice.

## Open Gaps After This Slice

- WGSL import composition and import-path resolver behavior.
- Shader include graph diagnostics with source spans.
- Renderer shader-cache invalidation keyed by typed shader definitions.
- Pipeline layout reflection from WGSL and bind-group compatibility validation.
- Editor UI surfacing of typed shader definition rows.
