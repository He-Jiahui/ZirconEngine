# Render Product Advanced Profile Acceptance

## Scope
- M10A advanced/Solari product acceptance from `docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md`.
- Affected layers: `RenderProfileBundle::advanced_render`, `RenderProfileBundle::solari_experimental`, VG/HGI provider fixtures, Solari provider availability, first-party app plugin planning, runtime submit stats, and Solari gated degradation status.
- Explicitly out of scope: a concrete Solari lighting pass, raytraced scene execution, full plugin workspace all-target validation, and full workspace CI-equivalent gates.

## Fixture Description
- Advanced fixture: `pluginized_wgpu_render_framework_with_advanced_providers()` with `advanced_quality_profile("m10a-advanced-acceptance")` and authored VG/HGI extract data from `advanced_product_extract()`.
- Solari fixture: `pluginized_wgpu_render_framework_with_solari_provider(SolariRuntimeStatus::Unavailable)` with full Solari backend capability flags, `solari=true`, experimental gate enabled, and VG/HGI disabled to prove Solari reports independently.
- App fixture: `EntryConfig::new(EntryProfile::Runtime).with_render_profile(RenderProfileBundle::solari_experimental())`, using linked first-party advanced render plugin registrations.

## Runtime Assertions
- AdvancedRender enables VG and HGI while excluding Solari.
- Advanced submit executes five VG graph passes and four HGI graph passes, reports authored VG payload source, and records both advanced provider reports as `Ready`.
- SolariExperimental enables Solari and records provider id `test.solari` with `SolariRuntimeStatus::Unavailable`, while VG/HGI graph counts remain zero when their quality flags are disabled.
- App first-party plugin planning selects `virtual_geometry`, `hybrid_gi`, and `solari`, contributes one Solari runtime provider, and includes `SolariPluginModule` in diagnostics.

## Tooling Evidence
- Tool name: Windows-native Cargo through PowerShell.
- Target directory: `E:\Git\ZirconEngine\target\codex-render-m9b-solari`.
- Exact commands:
- `cargo test -p zircon_runtime --locked render_product_submit --jobs 1 --message-format short --color never`
- `cargo test -p zircon_runtime --locked render_product --jobs 1 --message-format short --color never`
- `cargo test -p zircon_app --locked --no-default-features --features "plugin-ui,first-party-advanced-render-runtime-plugins" render_profile --jobs 1 --message-format short --color never`
- `cargo test -p zircon_app --locked render_profile --jobs 1 --message-format short --color never`
- `cargo check -p zircon_editor --lib --locked --message-format short --color never`
- `cargo check --manifest-path zircon_plugins\Cargo.toml --workspace --locked --all-targets --message-format short --color never`
- Earlier M9B Solari focused gate: `cargo test -p zircon_runtime --locked render_product_solari --jobs 1 --message-format short --color never`

## Results
- `render_product_submit` passed 9 focused tests, including `render_product_submit_advanced_profile_accepts_provider_backed_vg_hgi_path` and `render_product_submit_solari_experimental_reports_gated_provider_status`.
- `render_product` passed 57 focused runtime product tests.
- `render_profile` app gate passed 5 focused tests, including `render_profile_runtime_plugins_link_solari_provider_when_solari_experimental_is_selected`.
- Default-feature `zircon_app render_profile` passed 2 tests.
- `zircon_editor --lib` check passed with existing sprite-atlas unused-code warnings only.
- Plugin workspace all-target check passed with existing VG/HGI/editor unused-code warnings only.
- The earlier M9B Solari focused gate passed 5 tests covering not-requested, provider-missing, experimental-disabled, unavailable-provider, and missing-capability cases.

## Acceptance Decision
- Accepted for focused M10A AdvancedRender and SolariExperimental profile evidence.
- Remaining risks: no Solari visual implementation exists yet by design; full workspace CI-equivalent build/test and the optional validation-matrix script remain separate gates for a cleaner worktree window.
