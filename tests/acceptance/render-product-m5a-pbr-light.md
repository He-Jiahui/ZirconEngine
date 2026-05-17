# Render Product M5A PBR Light

## Scope
- M5A PBR material and light product baseline from `docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md`.
- Affected layers: neutral render material/light DTOs, runtime `RenderFrameExtract` light payloads, graphics resource streaming, mesh pipeline keys, submit context light plumbing, renderer stats, and lazy SSAO pipeline creation needed to keep non-SSAO PBR submit tests isolated.
- Explicitly out of scope: `.zmaterial` schema/importer work, material editor UI, automatic shader reflection, authored ambient/rect scene components, complete PBR shader lighting, sprite, anti-aliasing, VG/HGI deep profile integration, and Solari.

## Baseline
- M3A asset material contracts and M4A/M4B render product submit/postprocess gates were accepted before this gate.
- The checkout remains heavily dirty with unrelated render, asset, UI, hub, platform, reflection, and ECS sessions. This record covers narrowed M5A runtime/render gates, not full workspace acceptance.
- Active coordination keeps shader/material asset implementation and editor authoring owned by `.codex/sessions/20260516-1455-zmaterial-material-editor-design.md`; M5A only consumes existing `MaterialAsset` / `StandardMaterialDescriptor` data.

## Test Inventory
- Material runtime key cases: `zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs` covers StandardMaterial scalar/color projection, alpha-mask cutoff, double-sided keying, authored PBR texture-slot key bits, missing material fallback, missing shader fallback, missing texture fallback, wrong-kind references, missing runtime WGSL blocking diagnostics, dependency readiness cache invalidation, and unsupported container texture upload fallback.
- Submit/stat cases: `zircon_runtime/src/graphics/tests/render_product_submit.rs` covers material fallback stats, ambient/rect light counts, and that the fixture remains free of Virtual Geometry and Hybrid GI executed graph passes.
- Neutral light cases: `zircon_runtime/src/core/framework/tests.rs` covers ambient/rect DTO degradation contracts and snapshot/extract roundtrip preservation.
- World extraction cases: `zircon_runtime/src/scene/tests/world_basics.rs` covers directional/point/spot world extraction and proves ambient/rect slots are empty but represented as degraded neutral contracts until scene components exist.

## Tooling Evidence
- Tool name: WSL Cargo through `wsl.exe bash -lc`.
- Why this tool was used: Windows-native Cargo validation is currently blocked before Zircon source by dirty `Cargo.lock` dependency skew in the WGPU DX12 dependency graph, while WSL/Linux can validate the affected runtime logic directly.
- Exact commands:
- `wsl.exe bash -lc 'cd /mnt/e/Git/ZirconEngine && cargo test -p zircon_runtime --locked render_product_pbr --no-run'`
- `wsl.exe bash -lc 'cd /mnt/e/Git/ZirconEngine && cargo test -p zircon_runtime --locked render_product_pbr'`
- `wsl.exe bash -lc 'cd /mnt/e/Git/ZirconEngine && cargo test -p zircon_runtime --locked render_product_assets'`
- `wsl.exe bash -lc 'cd /mnt/e/Git/ZirconEngine && cargo check -p zircon_runtime --lib --locked'`

## Results
- M5A compile gate passed in WSL: `cargo test -p zircon_runtime --locked render_product_pbr --no-run` finished successfully.
- M5A PBR runtime gate passed in WSL: `cargo test -p zircon_runtime --locked render_product_pbr` passed 6 focused tests.
- M3A/M5A asset readiness regression gate passed in WSL: `cargo test -p zircon_runtime --locked render_product_assets` passed 8 focused tests.
- Scoped runtime library check passed in WSL: `cargo check -p zircon_runtime --lib --locked` finished successfully.
- Validation scope was narrowed to `zircon_runtime`; no workspace-wide root validation or plugin workspace validation was run for this closeout.

## Blocked Native Gate
- Windows-native command attempted: `cargo test -p zircon_runtime --locked render_product_pbr --no-run`.
- Observed blocker: `wgpu-hal-29.0.3` DX12 compile fails because one dependency path uses `windows v0.61.3` / `windows-core v0.61.2` while `wgpu-hal v29.0.3` uses `windows v0.62.2` / `windows-core v0.62.2`, producing mismatched `ID3D12Device` types.
- Root-cause evidence: dirty `Cargo.lock` currently resolves `gpu-allocator v0.28.0` through `windows v0.61.3`, while `wgpu-hal v29.0.3` resolves through `windows v0.62.2`.
- Acceptance impact: M5A is accepted only for scoped WSL/Linux runtime validation. Do not claim Windows-native, full-workspace, or plugin-workspace green from this gate.

## Acceptance Decision
- Accepted for narrowed M5A runtime PBR material/light baseline based on the focused WSL compile, runtime PBR, asset readiness, and scoped runtime library evidence listed above.
- Remaining risks: Windows-native lockfile dependency skew, full workspace build/test, plugin workspace validation, `.zmaterial` importer/editor authoring, ambient/rect scene components, concrete ambient/rect shading, complete PBR shader lighting, sprite, anti-aliasing, VG/HGI deep profile integration, and Solari remain open later milestones.
