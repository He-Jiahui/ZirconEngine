# Typed Shader Definitions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed bool/int/uint shader definition values to Zircon shader assets, `.zshader` authoring, variant keys, readiness diagnostics, and docs while preserving legacy string flag compatibility.

**Architecture:** `zircon_runtime::core::framework::render::shader` owns the neutral `RenderShaderDefinitionValue` DTO and typed `RenderShaderVariantKey` surface. `zircon_runtime::asset::assets::shader` stores that DTO directly, parses legacy and typed `.zshader` rows into one value list, and reports definition readiness without touching graphics shader cache or renderer preparation.

**Tech Stack:** Rust, Serde/TOML, Zircon `zircon_runtime` asset and render-framework modules, Bevy shader precedent from `dev/bevy`, scoped Cargo validation.

---

## Current Baseline

The approved spec is `docs/superpowers/specs/2026-05-25-typed-shader-definitions-design.md`.

Fresh source inspection shows the dirty workspace already contains several typed shader-definition edits, including `RenderShaderDefinitionValue`, typed `RenderShaderVariantKey.defines`, `ZShaderDocument::shader_definition_values()`, and typed readiness rows. Treat that as current workspace state. Do not revert or overwrite it blindly. Execution should audit those files against this plan, fill missing pieces, update docs, and validate.

Stay on `main`. Do not create a branch or worktree. Do not touch texture upload, mesh/glTF, graphics shader cache, material renderer preparation, editor UI, Hub, sound, ECS, app-entry, plugin workspace, root `Cargo.lock`, or unrelated formatting.

## File Structure

- Modify: `zircon_runtime/src/core/framework/render/shader/definition_value.rs`
  - Owns the neutral typed shader definition DTO, constructors, value accessors, normalization helper, legacy string deserialization, and string-to-bool-true conversion.
- Modify: `zircon_runtime/src/core/framework/render/shader/variant_key.rs`
  - Makes variant keys carry `Vec<RenderShaderDefinitionValue>` instead of `Vec<String>`.
- Modify: `zircon_runtime/src/core/framework/render/shader/mod.rs`
  - Structural module declaration and re-export only.
- Modify: `zircon_runtime/src/core/framework/render/mod.rs`
  - Structural re-export only.
- Modify: `zircon_runtime/src/asset/assets/shader/shader_asset.rs`
  - Stores typed shader definitions and projects them into `RenderShaderVariantKey`.
- Modify: `zircon_runtime/src/asset/assets/shader/zshader.rs`
  - Preserves legacy `shader_defs: Vec<String>`, adds typed `shader_def_values`, parses them into neutral DTOs, and reports invalid kind/value combinations.
- Modify: `zircon_runtime/src/asset/importer/ingest/import_shader_package.rs`
  - Uses `ZShaderDocument::shader_definition_values()` and turns parse failures into `AssetImportError::Parse`.
- Modify: `zircon_runtime/src/asset/assets/shader/readiness.rs`
  - Reports typed definition rows and duplicate/empty-name diagnostics.
- Modify: shader constructor call sites under `zircon_runtime/src/**` and relevant plugin tests only if compilation exposes constructor drift.
- Modify: `zircon_runtime/src/asset/tests/assets/shader_readiness.rs`
- Modify: `zircon_runtime/src/asset/tests/assets/render_product.rs`
- Modify: `zircon_runtime/src/asset/tests/project/zmeta.rs`
- Modify docs: `docs/zircon_runtime/core/framework/render/shader.md`, `docs/zircon_runtime/asset/render-assets.md`, `docs/zircon_runtime/asset/zmeta-shader-material.md`

## Milestone 1: Neutral DTO And Asset Projection

Goal: Establish the typed shader-definition value model in the neutral render framework and make shader variant keys carry typed values.

In-scope behaviors:
- Bool, signed integer, and unsigned integer shader definition values.
- `From<&str>` and `From<String>` produce bool-true flags.
- Legacy string deserialization works for persisted `ShaderAsset.shader_defs = ["FLAG"]` rows.
- Typed TOML rows deserialize for persisted `ShaderAsset.shader_defs = [{ kind = "uint", name = "BINDING", value = 2 }]`-style data.
- `RenderShaderVariantKey.defines` is typed.

Dependencies:
- Existing `zircon_runtime::core::framework::render::shader` module boundary.
- Existing `ShaderAsset::variant_keys()` projection.

Implementation slices:

- [ ] Add or audit `zircon_runtime/src/core/framework/render/shader/definition_value.rs` with this final DTO shape:

```rust
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RenderShaderDefinitionValue {
    #[serde(rename = "bool")]
    Bool { name: String, value: bool },
    #[serde(rename = "int")]
    Int { name: String, value: i32 },
    #[serde(rename = "uint")]
    UInt { name: String, value: u32 },
}

impl RenderShaderDefinitionValue {
    pub fn bool(name: impl Into<String>, value: bool) -> Self {
        Self::Bool { name: name.into(), value }
    }

    pub fn int(name: impl Into<String>, value: i32) -> Self {
        Self::Int { name: name.into(), value }
    }

    pub fn uint(name: impl Into<String>, value: u32) -> Self {
        Self::UInt { name: name.into(), value }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Bool { name, .. } | Self::Int { name, .. } | Self::UInt { name, .. } => name,
        }
    }

    pub fn normalized_name(&self) -> String {
        self.name().trim().to_string()
    }

    pub fn value_as_string(&self) -> String {
        match self {
            Self::Bool { value, .. } => value.to_string(),
            Self::Int { value, .. } => value.to_string(),
            Self::UInt { value, .. } => value.to_string(),
        }
    }
}

impl From<&str> for RenderShaderDefinitionValue {
    fn from(name: &str) -> Self {
        Self::bool(name, true)
    }
}

impl From<String> for RenderShaderDefinitionValue {
    fn from(name: String) -> Self {
        Self::bool(name, true)
    }
}

impl<'de> Deserialize<'de> for RenderShaderDefinitionValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case")]
        enum TaggedDefinitionValue {
            #[serde(rename = "bool")]
            Bool { name: String, value: bool },
            #[serde(rename = "int")]
            Int { name: String, value: i32 },
            #[serde(rename = "uint")]
            UInt { name: String, value: u32 },
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum DefinitionValueRepr {
            LegacyFlag(String),
            Tagged(TaggedDefinitionValue),
        }

        Ok(match DefinitionValueRepr::deserialize(deserializer)? {
            DefinitionValueRepr::LegacyFlag(name) => Self::from(name),
            DefinitionValueRepr::Tagged(TaggedDefinitionValue::Bool { name, value }) => Self::bool(name, value),
            DefinitionValueRepr::Tagged(TaggedDefinitionValue::Int { name, value }) => Self::int(name, value),
            DefinitionValueRepr::Tagged(TaggedDefinitionValue::UInt { name, value }) => Self::uint(name, value),
        })
    }
}
```

- [ ] Keep `zircon_runtime/src/core/framework/render/shader/mod.rs` structural:

```rust
mod definition_value;
mod dependency;
mod entry_point;
mod pipeline_layout;
mod stage;
mod variant_key;

pub use definition_value::RenderShaderDefinitionValue;
pub use dependency::RenderShaderDependency;
pub use entry_point::RenderShaderEntryPointDescriptor;
pub use pipeline_layout::{
    RenderShaderBindGroupLayoutDescriptor, RenderShaderBindingDescriptor,
    RenderShaderBindingResourceType, RenderShaderPipelineLayoutDescriptor,
};
pub use stage::RenderShaderStage;
pub use variant_key::RenderShaderVariantKey;
```

- [ ] Update or audit `zircon_runtime/src/core/framework/render/shader/variant_key.rs`:

```rust
use serde::{Deserialize, Serialize};

use super::RenderShaderDefinitionValue;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderShaderVariantKey {
    pub entry_point: Option<String>,
    pub stage: Option<String>,
    pub defines: Vec<RenderShaderDefinitionValue>,
}
```

- [ ] Ensure `zircon_runtime/src/core/framework/render/mod.rs` re-exports `RenderShaderDefinitionValue` beside the other shader DTOs.

- [ ] Update or audit `zircon_runtime/src/asset/assets/shader/shader_asset.rs` so `ShaderAsset.shader_defs` is `Vec<RenderShaderDefinitionValue>` and `variant_keys()` clones that typed list into each key.

- [ ] Update programmatic `ShaderAsset` constructors under `zircon_runtime/src/**` from string vectors to typed vectors. Use `RenderShaderDefinitionValue::from("FLAG")`, `RenderShaderDefinitionValue::bool("NAME", value)`, `RenderShaderDefinitionValue::int("NAME", value)`, or `RenderShaderDefinitionValue::uint("NAME", value)`.

Unit-test code to add or audit before the testing stage:

- [ ] In `zircon_runtime/src/asset/tests/assets/render_product.rs`, include `render_product_assets_shader_defs_accept_legacy_flags_and_typed_values` with this behavior:

```rust
let shader: ShaderAsset = toml::from_str(r#"
uri = "res://shaders/typed.shader"
source = "@fragment fn fs_main() {}"
wgsl_source = "@fragment fn fs_main() {}"

[[shader_defs]]
kind = "bool"
name = "ENABLE_FOG"
value = false

[[shader_defs]]
kind = "uint"
name = "BINDING_INDEX"
value = 2

[[shader_defs]]
kind = "int"
name = "DEBUG_MODE"
value = -1
"#).unwrap();

let legacy_shader: ShaderAsset = toml::from_str(r#"
uri = "res://shaders/legacy.shader"
source = "@fragment fn fs_main() {}"
wgsl_source = "@fragment fn fs_main() {}"
shader_defs = ["USE_UNLIT"]
"#).unwrap();

assert_eq!(
    shader.shader_defs,
    vec![
        RenderShaderDefinitionValue::bool("ENABLE_FOG", false),
        RenderShaderDefinitionValue::uint("BINDING_INDEX", 2),
        RenderShaderDefinitionValue::int("DEBUG_MODE", -1),
    ]
);
assert_eq!(
    legacy_shader.shader_defs,
    vec![RenderShaderDefinitionValue::from("USE_UNLIT")]
);
```

Lightweight checks:
- No Cargo commands during this implementation slice unless a syntax/type blocker prevents planning the next slice.

Testing stage:
- Covered in Milestone 4.

Exit evidence:
- Milestone 4 commands pass or failures are diagnosed and corrected from the lowest touched layer.

## Milestone 2: `.zshader` Authoring And Importer Parsing

Goal: Preserve legacy `.zshader shader_defs = ["FLAG"]` authoring while adding explicit typed `[[shader_def_values]]` rows and importer diagnostics.

In-scope behaviors:
- Legacy string flags import as bool true.
- Typed bool, int, and uint rows import into the same typed `ShaderAsset.shader_defs` list.
- Typed rows preserve authoring order after legacy flags.
- Invalid kind/value combinations fail import with structured parse text.

Dependencies:
- Milestone 1 DTO and typed asset field.

Implementation slices:

- [ ] Update or audit `zircon_runtime/src/asset/assets/shader/zshader.rs` with these fields and helper methods:

```rust
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ZShaderDocument {
    #[serde(default = "default_zshader_version")]
    pub version: u32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_path: Option<String>,
    #[serde(default)]
    pub wgsl_files: Vec<String>,
    #[serde(default)]
    pub entry_points: Vec<ZShaderEntryPointDocument>,
    #[serde(default)]
    pub imports: Vec<ZShaderImportDocument>,
    #[serde(default)]
    pub shader_defs: Vec<String>,
    #[serde(default)]
    pub shader_def_values: Vec<ZShaderDefinitionValueDocument>,
    #[serde(default)]
    pub properties: Vec<ShaderMaterialPropertyAsset>,
    #[serde(default)]
    pub texture_slots: Vec<ZShaderTextureSlotDocument>,
    #[serde(default)]
    pub editor: toml::Table,
}

impl ZShaderDocument {
    pub fn shader_definition_values(&self) -> Result<Vec<RenderShaderDefinitionValue>, String> {
        let mut definitions = self
            .shader_defs
            .iter()
            .cloned()
            .map(RenderShaderDefinitionValue::from)
            .collect::<Vec<_>>();
        for definition in &self.shader_def_values {
            definitions.push(definition.to_render_definition()?);
        }
        Ok(definitions)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZShaderDefinitionValueDocument {
    pub name: String,
    pub kind: String,
    pub value: toml::Value,
}
```

- [ ] Ensure `ZShaderDefinitionValueDocument::to_render_definition()` accepts `bool|boolean`, `int|i32|integer`, and `uint|u32`, returning these exact diagnostic fragments for tests and authoring UI:
  - `unsupported kind `<kind>``
  - `value is not a boolean`
  - `value is not an i32 integer`
  - `value is not a u32 integer`

- [ ] Update or audit `zircon_runtime/src/asset/importer/ingest/import_shader_package.rs` so it calls `document.shader_definition_values()` immediately after parsing the `.zshader` document and maps failures to:

```rust
AssetImportError::Parse(format!("parse zshader shader_def_values: {error}"))
```

- [ ] Ensure the constructed `ShaderAsset` uses that parsed typed `shader_defs` vector.

- [ ] Keep raw single-file WGSL/GLSL/SPIR-V importers in `import_shader.rs` using `shader_defs: Vec::new()`.

- [ ] Re-export `ZShaderDefinitionValueDocument` through `zircon_runtime/src/asset/assets/shader/mod.rs`, `zircon_runtime/src/asset/assets/mod.rs`, and `zircon_runtime/src/asset/mod.rs` only as structural `pub use` changes.

Unit-test code to add or audit before the testing stage:

- [ ] In `zircon_runtime/src/asset/tests/project/zmeta.rs`, add or audit `zshader_typed_shader_definition_rows_validate_kind_and_value` to parse one legacy flag plus bool/int/uint typed rows and assert errors for unknown kind, non-bool bool value, and negative uint.
- [ ] Extend `project_manager_imports_compound_zshader_package_with_subassets` with typed `[[shader_def_values]]` rows and assert `shader.shader_defs` plus `shader.variant_keys()[0].defines` contain bool true flags and typed bool false/int/uint values.
- [ ] Keep the documented fixture assertion that `shader_defs = ["USE_UNLIT"]` still maps to `RenderShaderDefinitionValue::from("USE_UNLIT")`.

Lightweight checks:
- No Cargo commands during this implementation slice unless a syntax/type blocker prevents planning the next slice.

Testing stage:
- Covered in Milestone 4.

Exit evidence:
- Milestone 4 commands pass or failures are diagnosed and corrected from the lowest touched layer.

## Milestone 3: Shader Readiness And Docs

Goal: Make standalone shader readiness explain typed definitions and keep code-facing docs synchronized.

In-scope behaviors:
- Readiness rows include the typed value.
- Empty normalized names are invalid.
- Duplicate normalized names are invalid even if values differ.
- Bool false rows are valid unless their name is empty or duplicated.
- Docs identify typed shader defs as asset/framework data, not graphics cache behavior.

Dependencies:
- Milestone 1 typed DTO and Milestone 2 `.zshader` parsing.

Implementation slices:

- [ ] Update or audit `zircon_runtime/src/asset/assets/shader/readiness.rs` so `ShaderDefinitionReadiness` is:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderDefinitionReadiness {
    pub raw_name: String,
    pub normalized_name: String,
    pub value: RenderShaderDefinitionValue,
    pub diagnostic: Option<String>,
}
```

- [ ] Ensure `shader_definition_readiness()` uses `definition.normalized_name()`, copies `definition.name()` into `raw_name`, clones the typed value, and reports duplicate names with the existing text `shader definition `<name>` is duplicated`.

- [ ] Update or audit `zircon_runtime/src/asset/tests/assets/shader_readiness.rs` so `shader_readiness_reports_shader_def_diagnostics` uses bool true, empty bool true, uint, and duplicate bool false definitions, then asserts `value_as_string()` values `true`, `1`, and `false`.

- [ ] Update `docs/zircon_runtime/core/framework/render/shader.md`:
  - Add `zircon_runtime/src/core/framework/render/shader/definition_value.rs` to `related_code` and `implementation_files`.
  - Add `docs/superpowers/specs/2026-05-25-typed-shader-definitions-design.md` and this plan to `plan_sources`.
  - Add focused typed shader-def test names and commands to `tests` after validation runs.
  - Replace string-only shader-def wording with typed bool/int/uint wording and state that renderer cache invalidation remains out of scope.

- [ ] Update `docs/zircon_runtime/asset/render-assets.md`:
  - Add this spec and plan to the header.
  - Describe legacy `.zshader shader_defs` as bool-true compatibility input.
  - Describe `[[shader_def_values]]` bool/int/uint rows and readiness duplicate/empty-name diagnostics.
  - Add validation evidence after Milestone 4 runs.

- [ ] Update `docs/zircon_runtime/asset/zmeta-shader-material.md`:
  - Add this spec and plan to the header.
  - Document the `.zshader` authoring schema for `shader_defs` and `shader_def_values`.
  - State invalid kind/value rows fail import with parse diagnostics.

Lightweight checks:
- Defer Rust formatting and Cargo commands to Milestone 4 unless a syntax/type blocker prevents entering the testing stage.

Testing stage:
- Covered in Milestone 4.

Exit evidence:
- Milestone 4 commands pass or failures are diagnosed and corrected from the lowest touched layer.

## Milestone 4: Testing Stage And Acceptance

Goal: Validate typed shader definitions with scoped runtime checks and hygiene without claiming workspace-wide success.

In-scope behaviors:
- Compile/type correctness for `zircon_runtime` lib tests.
- Focused shader and project fixture tests.
- Rust formatting for touched files.
- Diff whitespace hygiene for touched files.
- Clear acceptance notes and remaining-risk statement.

Dependencies:
- Milestones 1-3 completed.
- Shared Cargo/rustc queue is not actively saturated by other sessions, or validation is run with a separate external target directory and low parallelism.

Implementation slices:

- [ ] Run rustfmt on the exact touched Rust files, not the whole repository. Expected success: rustfmt exits 0.

```powershell
rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/shader/definition_value.rs zircon_runtime/src/core/framework/render/shader/variant_key.rs zircon_runtime/src/core/framework/render/shader/mod.rs zircon_runtime/src/asset/assets/shader/shader_asset.rs zircon_runtime/src/asset/assets/shader/zshader.rs zircon_runtime/src/asset/assets/shader/readiness.rs zircon_runtime/src/asset/importer/ingest/import_shader_package.rs zircon_runtime/src/asset/tests/assets/shader_readiness.rs zircon_runtime/src/asset/tests/assets/render_product.rs zircon_runtime/src/asset/tests/project/zmeta.rs
```

- [ ] Run focused shader tests. Expected success: all matching tests pass; existing unrelated warnings may remain.

```powershell
cargo test -p zircon_runtime --lib shader --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1
```

- [ ] Run the compound `.zshader` focused project test. Expected success: one matching test passes.

```powershell
cargo test -p zircon_runtime --lib project_manager_imports_compound_zshader_package_with_subassets --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1
```

- [ ] Run the runtime lib test compile/check gate. Expected success: check exits 0; existing unrelated warnings may remain.

```powershell
cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs
```

- [ ] Run diff whitespace hygiene on only touched files. Expected success: no trailing-whitespace or conflict-marker errors for touched files; CRLF warnings alone are not source failures.

```powershell
git diff --check -- docs/superpowers/specs/2026-05-25-typed-shader-definitions-design.md docs/superpowers/plans/2026-05-25-typed-shader-definitions.md docs/zircon_runtime/core/framework/render/shader.md docs/zircon_runtime/asset/render-assets.md docs/zircon_runtime/asset/zmeta-shader-material.md zircon_runtime/src/core/framework/render/shader/definition_value.rs zircon_runtime/src/core/framework/render/shader/variant_key.rs zircon_runtime/src/core/framework/render/shader/mod.rs zircon_runtime/src/core/framework/render/mod.rs zircon_runtime/src/asset/assets/shader/shader_asset.rs zircon_runtime/src/asset/assets/shader/zshader.rs zircon_runtime/src/asset/assets/shader/readiness.rs zircon_runtime/src/asset/importer/ingest/import_shader_package.rs zircon_runtime/src/asset/assets/shader/mod.rs zircon_runtime/src/asset/assets/mod.rs zircon_runtime/src/asset/mod.rs zircon_runtime/src/asset/tests/assets/shader_readiness.rs zircon_runtime/src/asset/tests/assets/render_product.rs zircon_runtime/src/asset/tests/project/zmeta.rs
```

Debug/correction loop:
- If `RenderShaderDefinitionValue` fails to deserialize legacy strings, fix the custom `Deserialize` implementation before touching asset/importer code.
- If typed `RenderShaderVariantKey` causes constructor failures, update the call site to use `RenderShaderDefinitionValue` helpers; do not add a parallel string field.
- If `.zshader` import tests fail, fix `ZShaderDocument::shader_definition_values()` or `ZShaderDefinitionValueDocument::to_render_definition()` before changing project manager behavior.
- If readiness tests fail, fix `shader_definition_readiness()` before changing higher-level tests.
- If broad `cargo check` fails in unrelated active ECS/UI/Hub/sound/lockfile areas, record the failure and do not patch those lanes from this task unless the diagnostic is in the typed shader-def touched files.

Exit evidence:
- Record exact pass/fail results in the final response and in any touched docs that list validation evidence.
- Do not claim full workspace success unless a fresh full workspace validator is run and passes after this slice.

## Acceptance Criteria

- `RenderShaderDefinitionValue` is the single typed shader-def value model used by render shader variant keys and shader assets.
- Legacy `.zshader shader_defs = ["FLAG"]` and legacy serialized `ShaderAsset.shader_defs = ["FLAG"]` remain valid as bool-true definitions.
- New `.zshader [[shader_def_values]]` rows support bool, int, and uint values with deterministic parse errors for invalid rows.
- `ShaderReadinessReport` reports typed definition rows, rejects empty and duplicate normalized names, and accepts bool false rows when otherwise valid.
- Docs describe the new authoring and runtime contract and clearly state that WGSL import composition and graphics shader cache invalidation remain future work.
- Scoped runtime tests and hygiene commands from Milestone 4 pass, or any failure is recorded with a lower-layer diagnosis and no unrelated owner lanes are modified.

## Self-Review Checklist

- Spec coverage: Milestone 1 covers neutral DTO and variant keys; Milestone 2 covers `.zshader` compatibility and parsing; Milestone 3 covers readiness/docs; Milestone 4 covers validation and acceptance evidence.
- Type consistency: use `RenderShaderDefinitionValue`, `ZShaderDefinitionValueDocument`, `shader_definition_values()`, `RenderShaderVariantKey.defines`, and `ShaderDefinitionReadiness.value` exactly as named.
- Scope discipline: no texture, mesh/glTF, graphics cache, material renderer, editor UI, plugin, lockfile, or broad-format work is included.
