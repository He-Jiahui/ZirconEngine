# Render Product Default Profile Acceptance

## Scope
- M10A default/headless product acceptance from `docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md`.
- Affected layers: `RenderProfileBundle::default_render`, `RenderProfileBundle::headless`, runtime submit via `WgpuRenderFramework`, default Core3d world extract, runtime UI graph placement, default post-process/AA stats, and Core2d sprite submit stats.
- Explicitly out of scope: full workspace validation, editor surface damage, Solari visual implementation, and plugin workspace all-target validation.

## Fixture Description
- Default Core3d fixture: `World::new().to_render_frame_extract()` at `320x240`, submitted with a runtime UI extract containing one quad command and one image command.
- Default Core2d fixture: an orthographic `RenderFrameExtract` with one transparent sprite using a missing texture handle so the concrete sprite fallback path is observable.
- Headless fixture: `RenderProfileBundle::headless()` only; it must validate with default/empty backend capabilities and must not enable `DefaultRender`, mesh, UI render, VG, HGI, or Solari product features.

## Runtime Assertions
- DefaultRender enables mesh/material/sprite/UI/postprocess/AA and excludes VG/HGI/Solari.
- Core3d submit records material and light stats, executes postprocess and AA, executes one runtime UI graph pass, and leaves advanced/Solari disabled.
- Core2d submit records one sprite, one texture fallback, and three sprite graph passes.
- Headless profile has an empty feature list and no backend capability requirement.

## Tooling Evidence
- Tool name: Windows-native Cargo through PowerShell.
- Target directory: `E:\Git\ZirconEngine\target\codex-render-m9b-solari`.
- Exact command:
- `cargo test -p zircon_runtime --locked render_product_submit --jobs 1 --message-format short --color never`
- `cargo test -p zircon_runtime --locked render_product --jobs 1 --message-format short --color never`
- `cargo test -p zircon_app --locked render_profile --jobs 1 --message-format short --color never`
- `cargo test -p zircon_app --locked --no-default-features --features "plugin-ui,first-party-advanced-render-runtime-plugins" render_profile --jobs 1 --message-format short --color never`
- `cargo check -p zircon_editor --lib --locked --message-format short --color never`
- `cargo check --manifest-path zircon_plugins\Cargo.toml --workspace --locked --all-targets --message-format short --color never`

## Results
- `render_product_submit` passed 9 focused tests, including `render_product_submit_default_profile_accepts_default_3d_ui_and_2d_sprite_paths` and `render_product_submit_headless_profile_has_no_render_product_activation`.
- `render_product` passed 57 focused runtime product tests.
- Default-feature `zircon_app render_profile` passed 2 tests.
- Advanced-plugin `zircon_app render_profile` passed 5 app profile tests, including default/headless render profile storage cases.
- `zircon_editor --lib` check passed with existing sprite-atlas unused-code warnings only.
- Plugin workspace all-target check passed with existing VG/HGI/editor unused-code warnings only.

## Acceptance Decision
- Accepted for focused M10A DefaultRender and Headless profile evidence.
- Remaining risks: full workspace CI-equivalent build/test and the optional validation-matrix script still need a cleaner worktree window before final promotion.
