# Build Asset Staging Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `tools/zircon_build.py --targets editor,runtime` produce a staged `ZirconEngine` folder that contains the editor/runtime built-in assets needed by exported binaries.

**Architecture:** Stage `zircon_editor/assets` and `zircon_runtime/assets` into one runtime payload root at `<out>/ZirconEngine/assets`, with collision checks for files that share the same relative path. Move built-in asset lookup away from hard-coded crate roots by adding a small runtime asset path resolver that prefers `ZIRCON_ASSET_ROOT`, executable-adjacent `assets`, current-directory `assets`, then crate dev fallback.

**Tech Stack:** Python 3.11 `pathlib`/`shutil`/`filecmp`, Rust `std::env`/`PathBuf`, existing Cargo workspace, existing UI asset loader.

---

## File Structure

- Modify `tools/zircon_build.py`: add asset root staging and file-collision checks, call staging for editor/runtime targets.
- Create `zircon_runtime/src/asset/runtime_asset_path.rs`: runtime-owned helper for executable/dev asset path resolution.
- Modify `zircon_runtime/src/asset/mod.rs`: export the helper.
- Modify `zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs`: use the helper for runtime UI fixtures.
- Modify `zircon_editor/src/ui/template_runtime/builtin/template_documents.rs`: use the helper for built-in editor template roots.
- Modify `zircon_editor/src/ui/template_runtime/runtime/build_session.rs`: resolve `res://` imports through the helper.
- Modify `zircon_editor/src/ui/asset_editor/node_projection.rs`: resolve editor UI asset paths through the helper.
- Modify `zircon_editor/src/ui/layouts/views/view_projection.rs`: resolve view template paths through the helper.
- Modify `docs/cli-and-tooling/zircon-build-tool.md`: document staged `assets/` layout and checks.
- Modify `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md`: document exported built-in UI asset lookup.
- Update `.codex/sessions/20260504-1302-build-asset-staging.md`: record implementation state and validation evidence.

## Milestone 1: Asset Staging And Lookup

### Implementation Slices

- [ ] Add `stage_engine_assets(config)` in `tools/zircon_build.py`.
- [ ] Copy `zircon_editor/assets` and `zircon_runtime/assets` into `config.engine_root / "assets"`.
- [ ] Merge directories recursively and fail when two source roots provide the same relative file with different bytes.
- [ ] Keep identical duplicate files idempotent.
- [ ] Print each staged source root so users can see that assets are copied.
- [ ] Add runtime asset lookup helper in `zircon_runtime/src/asset/runtime_asset_path.rs`.
- [ ] Replace editor/runtime built-in asset `CARGO_MANIFEST_DIR/assets` path construction with the helper.
- [ ] Keep project `res://` resolution in `EditorUiHost::resolve_ui_asset_path` unchanged.
- [ ] Update docs and session note.

### Testing Stage

- [ ] Run `python -m py_compile tools/zircon_build.py`.
- [ ] Run `python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug`.
- [ ] Confirm `E:\zircon-build\ZirconEngine\assets\ui\editor\host\editor_main_frame.ui.toml` exists.
- [ ] Confirm `E:\zircon-build\ZirconEngine\assets\ui\runtime\fixtures\hud_overlay.ui.toml` exists.
- [ ] Run scoped Cargo check/tests for touched runtime/editor UI paths.
- [ ] Smoke-run `E:\zircon-build\ZirconEngine\zircon_editor.exe` long enough to confirm it stays alive.
- [ ] Run `git diff --check` for touched files.

## Acceptance Evidence

- The build command copies binaries and built-in assets into `E:\zircon-build\ZirconEngine`.
- Exported editor/runtime built-in UI template loads no longer require running from the repository checkout.
- Existing project-root asset resolution remains separate from built-in engine asset resolution.

## Self-Review

- Spec coverage: staging, lookup order, docs, validation, and non-project `res://` scope are covered by Milestone 1.
- Placeholder scan: no `TBD`, `TODO`, or unspecified test instructions remain.
- Type consistency: helper name and call sites all use runtime-owned built-in asset path semantics.
