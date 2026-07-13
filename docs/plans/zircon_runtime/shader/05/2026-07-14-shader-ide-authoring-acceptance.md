# Shader 05 IDE authoring acceptance

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | generated `self::material` IDE anchor hint | `completed` | 2026-07-14 | `.zshader` importer now emits one info diagnostic when a surface source uses a real `zr_mat_*` identifier without `#include <self::material>`. The scan ignores line comments, nested block comments, and commented-out anchors; existing authored anchors suppress the hint, and runtime assembly is unchanged. Focused current-source tests passed 3/3. |
| M2 | current-source IDE environment product run | `completed` | 2026-07-14 | Managed Windows build of `zircon_shader_ide_env` passed. The Vampire project generated 4 shader scopes, 24 module-map entries, 4 generated material stubs, and 4 default previews. Naga parsed 24/24 stubs and validated 4/4 previews. The first run wrote 33 managed files; an unchanged second run wrote 0 and removed 0 stale files. Product evidence is under `E:/ZirconBuilds/shader05-ide-env-acceptance-20260714/product`, outside both the repository and Cargo target directories. |
| M3 | template segment table and source diagnostic remap regression | `completed` | 2026-07-14 | Current-source focused suites passed: `shader_ide_env` 9/9, `shader_template_` 21/21, and `shader_module` 7/7. The template suite includes Naga parse-error remapping from assembled lines to the authored module id and local line. |
| M3 | SH05-M3-T template/IDE authoring testing stage | passed | 2026-07-14 | Status anchor: `shader_plan05_ide_env_anchor_hint_and_segment_remap_product_acceptance_passed`. `rustfmt --check` and scoped `git diff --check` passed. The new scanner and tests use a folder-backed child owner, keeping the importer orchestration file below the repository's large-file warning threshold. The real CLI accepted `--help`, `--project-root`, `--out-dir`, `--pretty`, and `--variants`, with all runs exiting 0. |

## Implementation scope

- `zircon_runtime/src/asset/importer/ingest/import_shader_package.rs`
- `zircon_runtime/src/asset/importer/ingest/import_shader_package/generated_material_anchor_hint.rs`
- `docs/zircon_runtime/asset/importer/ingest/import-shader-package.md`

The implementation deliberately keeps the hint in importer diagnostics. It does not
change the module registry, generated material ABI, shader variant key, or assembled
WGSL bytes.

## Validation detail

The package-wide validation wrapper was interrupted after producing a current-source
test executable, so it is not counted as a package-wide pass. All test counts above
come from direct execution of that freshly linked test executable. The standalone CLI
binary was then built through the managed Windows Cargo pool and exercised twice on a
real project.

Runtime frame diagnostics remain a separate follow-up: the current
`ShaderVariantMissReport` records miss counters and dimensions but does not yet retain
the full remapped pipeline compilation failure payload. This record closes the
template/import/IDE remap slice only and does not mark the overall Shader goal complete.
