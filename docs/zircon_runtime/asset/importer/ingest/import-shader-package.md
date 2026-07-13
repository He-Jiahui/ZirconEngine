---
related_code:
  - zircon_runtime/src/asset/importer/ingest/import_shader_package.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader_package/generated_material_anchor_hint.rs
  - zircon_runtime/src/core/framework/render/shader/module_import.rs
  - zircon_runtime/src/graphics/shader/template/assemble.rs
implementation_files:
  - zircon_runtime/src/asset/importer/ingest/import_shader_package.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader_package/generated_material_anchor_hint.rs
plan_sources:
  - user: 2026-07-14 continue the shader architecture plan and close verified gaps
  - docs/plans/zircon_runtime/shader/05-ide-and-authoring-dx.md
tests:
  - zircon_runtime/src/asset/importer/ingest/import_shader_package.rs
  - zircon_runtime/src/graphics/shader/template/tests/surface_modules.rs
  - zircon_runtime/src/graphics/shader/ide_env_generation/tests.rs
doc_type: module-detail
---

# Shader Package Import

## Purpose

`import_shader_package.rs` converts a compound `.zshader` package into the runtime
`ShaderAsset` contract. It owns schema-v2 parsing, deterministic module identity,
WGSL source collection, import diagnostics, generated material metadata, and source
entries used by the asset pipeline.

## Import Flow

1. Resolve the package directory and primary `.zshader` descriptor.
2. Parse `ZShaderDocumentV2` and derive the default `import_path` from the project
   namespace plus the package path.
3. Read the declared WGSL files, validate include declarations, and reject reserved
   module identities or include-module ABI violations.
4. Build material property layout, option table, texture slots, and the generated
   `self::material` WGSL module.
5. Return the `ShaderAsset`, import dependencies, source entries, and diagnostics as
   one `AssetImportOutcome`.

Explicit `import_path` values remain valid only when they intentionally differ from
the derived identity. Repeating the derived value emits a warning so projects can
remove redundant declarations without changing module identity.

## Generated Material Anchor Hint

Surface WGSL receives the generated material module automatically during template
assembly. Therefore `#include <self::material>` is not a runtime dependency and
adding or removing it must leave assembled WGSL byte-identical.

The include is still useful to IDE tooling because `module_map.json` maps it to the
per-shader generated material stub. When a surface source references an identifier
with the `zr_mat_` prefix but omits the include, the importer now emits one
`ResourceDiagnosticSeverity::Info` diagnostic asking the author to add the anchor.
This is advisory only: it neither blocks import nor changes generated bindings,
variant keys, or runtime assembly.

The identifier scan ignores line comments and nested block comments. This prevents
examples or commented-out material calls from producing a false authoring hint.
Include and non-surface shader kinds do not receive this diagnostic.

The advisory scan and its focused tests live in the folder-backed
`generated_material_anchor_hint.rs` child owner. The package importer stays responsible
for orchestration and does not accumulate another parsing behavior family.

## Invariants

- `self::material` is engine-generated and cannot be claimed by project or plugin
  assets.
- Authored includes must either be built-in/generated tokens or have matching
  `.zshader` import declarations.
- Include modules cannot declare entry points, bind groups, or reserved engine
  symbols.
- The IDE anchor diagnostic is emitted at most once per imported shader package.
- Runtime assembly remains the authority for generated material injection.

## Test Coverage

The importer-local tests cover missing-anchor reporting, info severity, suppression
when the anchor exists, and suppression for comment-only symbol text. Template tests
separately prove that explicit and implicit `self::material` injection produce the
same WGSL bytes. IDE-environment tests verify the generated module is present in the
module map and material stub tree.
