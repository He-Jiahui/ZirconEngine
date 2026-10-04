---
related_code:
  - zircon_runtime/src/ui/icon_atlas/mod.rs
  - zircon_runtime/src/ui/icon_atlas/atlas.rs
  - zircon_runtime/src/ui/icon_atlas/svg.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_editor/assets/ui/editor/icons/run.icon.toml
  - zircon_editor/assets/ui/editor/icons/save.icon.toml
  - zircon_editor/assets/ui/editor/icons/search.icon.toml
  - zircon_runtime/src/ui/tests/icon_atlas.rs
  - zircon_runtime/src/ui/tests/mod.rs
implementation_files:
  - zircon_runtime/src/ui/icon_atlas/mod.rs
  - zircon_runtime/src/ui/icon_atlas/atlas.rs
  - zircon_runtime/src/ui/icon_atlas/svg.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_editor/assets/ui/editor/icons/run.icon.toml
  - zircon_editor/assets/ui/editor/icons/save.icon.toml
  - zircon_editor/assets/ui/editor/icons/search.icon.toml
  - zircon_runtime/src/ui/tests/icon_atlas.rs
  - zircon_runtime/src/ui/tests/mod.rs
plan_sources:
  - user: 2026-06-12 implement editor UI architecture plan code
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
tests:
  - zircon_runtime/src/ui/tests/icon_atlas.rs
  - python -c "import tomllib, pathlib; paths=sorted(pathlib.Path('zircon_editor/assets/ui/editor/icons').glob('*.icon.toml')); [tomllib.loads(p.read_text(encoding='utf-8')) for p in paths]; print('parsed', len(paths), 'icon toml files:', ', '.join(p.name for p in paths))" (2026-06-12: passed)
  - rustfmt --edition 2021 zircon_runtime\src\ui\icon_atlas\mod.rs zircon_runtime\src\ui\icon_atlas\atlas.rs zircon_runtime\src\ui\icon_atlas\svg.rs zircon_runtime\src\ui\mod.rs zircon_runtime\src\ui\tests\icon_atlas.rs zircon_runtime\src\ui\tests\mod.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-icon-atlas-0612-coremin --message-format short --color never (2026-06-12: passed, existing warnings only)
  - cargo test -p zircon_runtime --lib icon_atlas --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-icon-atlas-0612-coremin --message-format short --color never -- --test-threads=1 --nocapture (2026-06-12: timed out after 908 seconds during lib-test compilation/linking, no Rust diagnostics returned)
doc_type: module-detail
---

# UI Icon Atlas

`ui::icon_atlas` is the CPU-side data layer for the 05.M4 icon channel. It intentionally stops before GPU texture upload, renderer bind groups, or scene UI shader changes. That boundary lets the UI asset system plan icon slots and validate SVG authoring without colliding with active render work.

## Inputs

The primary input is `UiIconRasterRequest`:

- `icon_id`: the lookup id used by UI nodes and render resources.
- `asset`: a parsed `UiIconAsset`.
- `dpi_scale`: the scale used to convert the asset's logical `default_size` into pixel dimensions.

Inline SVG icons use `UiIconSourceKind::Svg` with `source.text`. External SVG or bitmap icon assets can enter the atlas plan without inline parsing; their bytes are resolved by later asset/GPU consumers.

## SVG Subset

`parse_ui_svg_icon(...)` accepts a deliberately small SVG subset:

- root `<svg>` tag with optional `width`, `height`, and `viewBox`;
- `<path>` elements with `d`, optional `fill`, and optional `stroke`.

The parser extracts metadata for later raster/tessellation work. It does not tessellate paths, evaluate transforms, load external resources, or perform paint compositing. Unsupported or malformed inputs fail early with `UiSvgIconParseError`, which keeps authoring errors visible before render integration.

## Atlas Plan

`UiIconAtlasBuilder::build_plan(...)` deduplicates requests by `icon_id`, sorts slots deterministically, computes per-icon pixel size from `default_size * dpi_scale`, and assigns square grid cells with configurable padding and minimum atlas side.

Each `UiIconAtlasSlot` records:

- `icon_id` and `semantic_id`;
- pixel `rect` inside the atlas;
- normalized `uv` coordinates;
- requested `pixel_size`;
- parsed inline SVG metadata when available.

The plan is renderer-neutral. Later renderer code can consume the same slot table to rasterize SVG paths, blit bitmap sources, upload atlas regions, or map `UiRenderResourceKind::Icon` to atlas coordinates.

## Default Editor Icon Pack

The first editor icon pack lives under `zircon_editor/assets/ui/editor/icons/` and uses the same `.icon.toml` authoring format as imported user icons:

- `run.icon.toml` -> `editor.icons.run`
- `save.icon.toml` -> `editor.icons.save`
- `search.icon.toml` -> `editor.icons.search`

These icons are small inline SVG path documents with stable semantic ids and a logical default size of 20 px. Runtime coverage parses the real TOML files through `UiIconAsset::from_toml_str(...)`, feeds them into `UiIconAtlasBuilder`, and asserts that all default icons enter the atlas plan with parsed SVG metadata. That keeps editor chrome icon authoring on the same asset path as project icons instead of embedding ad-hoc strings in components.

## Current Boundary

This module now covers M4.S1 CPU planning plus the M4.S2 default editor icon pack seed. It still does not rasterize SVG paths, upload a GPU atlas, compare rendered output against a native painter, or replace all authored inline component icons with asset references. Those remain renderer and component-library integration tasks.
