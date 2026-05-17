# Render Product M4B Postprocess Pass Graph

## Scope
- M4B postprocess pass graph productization from `docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md`.
- Affected layers: neutral `zircon_runtime::core::framework::render::post_process` contracts, `RenderFrameExtract` postprocess payloads, submit-time effective graph derivation, concrete renderer graph-resource validation, renderer execution records, and render stats.

## Baseline
- M4A Core2d/Core3d phases and direct submit closure were accepted before this gate.
- M4B initially had review gaps: history-resolve evidence could be reported when concrete renderer history was unavailable, product postprocess executors were too close to metadata no-ops, and stats could read a graph that did not match final renderer execution authority.
- The checkout remains heavily dirty with unrelated render, asset, UI, hub, platform, reflection, and ECS sessions. This record covers narrowed M4B runtime/render gates, not full workspace acceptance.

## Test Inventory
- Neutral graph cases: `zircon_runtime/src/core/framework/tests.rs` covers normal stack construction, disabled-effect elision, missing scene color, invalid history input, duplicate output resources, missing effect dependencies, and dependency cycles.
- Renderer executor cases: `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs` covers concrete product postprocess executor registration and validation semantics.
- Runtime bridge cases: `zircon_runtime/src/graphics/tests/render_framework_bridge.rs` covers bloom/color-grading/final-composite product node stats, history-resolve evidence gating across repeated submissions, and normal render graph pass-order isolation.
- Boundary cases covered: no compatible history, compatible viewport metadata without actual renderer history texture, disabled effect nodes, missing required resources, duplicate outputs, missing dependencies, cycles, and culled graph passes.
- Negative cases covered: invalid graph resource dependencies reject validation, and history-resolve execution evidence is absent until the renderer has a real compatible history texture.

## Tooling Evidence
- Tool name: Windows Cargo through PowerShell.
- Why these tools were used: M4B acceptance concerns Rust compile/test behavior in neutral postprocess contracts, render graph validation, concrete executor registration, submit normalization, and renderer stats.
- WSL-specific tools were not used for this narrowed gate because no platform-specific crash, memory, or thread-safety failure was observed; Linux CI-parity remains a later full-workspace gate.
- Exact commands:
- `cargo test -p zircon_runtime --locked render_product_post_process`
- `cargo test -p zircon_runtime --locked render_graph`
- `cargo check -p zircon_runtime --lib --locked`

## Results
- M4B postprocess product gate passed on 2026-05-17: 9 focused `render_product_post_process` tests passed, 0 failed.
- M4B render graph gate passed on 2026-05-17: 18 focused `render_graph` tests passed, 0 failed.
- Scoped runtime library check passed on 2026-05-17: `cargo check -p zircon_runtime --lib --locked` finished successfully.
- Validation scope was narrowed to `zircon_runtime`; no workspace-wide root validation or plugin workspace validation was run for this closeout.

## Review Evidence
- Spec review result: passed with no findings.
- Quality review initially found a renderer graph-authority blocker: final stats and history-resolve evidence had to come from the graph actually recorded by the renderer, not from a submit-time graph that could still include unavailable history.
- Follow-up remediation preserved the submitted effective graph as the source, derives a frame-local no-history graph only when actual renderer history is unavailable, validates concrete product node resources, and records renderer-owned postprocess execution evidence.
- Final review result: remaining blocker resolved.

## Acceptance Decision
- Accepted for narrowed M4B postprocess pass graph productization based on the focused neutral graph, render graph, runtime bridge, review, and scoped runtime library evidence listed above.
- Remaining risks: full workspace build/test, plugin workspace validation, WSL/Linux CI parity, graph-native pixel implementation for individual postprocess nodes, M5A PBR/light renderer integration, sprite, AA, VG/HGI deep profile integration, and Solari remain open later milestones.
