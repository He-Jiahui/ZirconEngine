# Render Main Chain Sparse Texture Reservation

## Scope
- M8 sparse texture reservation evidence for the WGPU render main chain.
- Affected layers: neutral RHI texture descriptors, WGPU capability rejection, RenderGraph resource lifetimes and transient allocation planning, submit-time `RenderStats`, runtime `DiagnosticStore` projection, and module documentation.
- Explicitly out of scope: sparse page tables, tile upload, residency eviction, sparse WGPU objects, terrain/tree streaming, and any concrete virtual-texture renderer.

## Baseline
- The WGPU render main-chain plan already treats `SparseTexture` as an explicit-opt-in advanced slot, not as an implemented renderer.
- Current backend truth keeps `RenderBackendCaps.supports_sparse_texture` false for WGPU/headless paths, so sparse texture creation must be rejected unless a future backend/provider opts in.
- The current shared checkout has multiple unrelated active Cargo lanes for editor, hub, asset, and workspace validation. This acceptance record covers the isolated render-main-chain target directory and does not claim full workspace acceptance.

## Test Inventory
- Descriptor and capability cases:
- `zircon_runtime/src/rhi/tests/descriptors.rs` verifies `TextureDesc::with_sparse_residency()` stores `TextureResidency::SparseReserved`.
- `zircon_runtime/src/rhi/tests/capabilities.rs` verifies the sparse capability flag can be represented in neutral backend caps.
- `zircon_runtime/src/rhi_wgpu/tests.rs` and device tests cover WGPU capability summary and sparse-reservation rejection while support is false.
- RenderGraph boundary cases:
- `zircon_runtime/src/render_graph/tests/resources.rs::graph_preserves_sparse_texture_reservations_without_dense_transient_slot` verifies sparse texture lifetimes preserve the descriptor, increment `sparse_texture_lifetime_count`, do not consume dense transient texture slots, and increment `sparse_texture_slot_count`.
- Submit and diagnostics bridge cases:
- `zircon_runtime/src/graphics/tests/render_framework_bridge.rs` verifies compiled graph stats flow into `RenderStats`.
- `zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins` verifies `render.graph.sparse_texture_lifetime_count` and `render.graph.sparse_texture_slot_count` reach `DiagnosticStore`.
- Documentation coverage:
- `docs/zircon_runtime/rhi/descriptors.md`, `docs/zircon_runtime/render_graph/builder.md`, `docs/zircon_runtime/core/diagnostics.md`, `docs/zircon_runtime/graphics/render-product-submit.md`, and `docs/assets-and-rendering/render-framework-architecture.md` document that sparse reservation stats are graph/resource evidence only.

## Tooling Evidence
- Tool name: Windows-native Cargo and rustfmt through PowerShell.
- Target directory: `D:\cargo-targets\zircon-render-main-chain-sparse-0604`.
- WSL/debugger tools were not selected because the observed risk is Rust type/test coverage for a data-flow contract, not a crash, memory error, race, or platform-specific runtime fault.
- Exact commands:
- `rustfmt --edition 2021 --check zircon_runtime/src/rhi/descriptors.rs zircon_runtime/src/rhi/capabilities.rs zircon_runtime/src/rhi_wgpu/capabilities.rs zircon_runtime/src/rhi_wgpu/device.rs zircon_runtime/src/render_graph/graph.rs zircon_runtime/src/render_graph/tests/resources.rs zircon_runtime/src/core/framework/render/backend_types.rs zircon_runtime/src/core/diagnostics/render_stats_store/graph.rs zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs zircon_runtime/src/graphics/tests/render_framework_bridge.rs zircon_runtime/src/tests/runtime_diagnostics/mod.rs`
- `cargo test -p zircon_runtime --lib sparse_texture --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-main-chain-sparse-0604 --message-format short --color never`
- Planned after sparse test completion: `cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-main-chain-sparse-0604 --message-format short --color never`
- Planned after focused tests: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-main-chain-sparse-0604 --message-format short --color never`

## Results
- `rustfmt --edition 2021 --check ...`: passed.
- `cargo test -p zircon_runtime --lib sparse_texture ...`: running in the isolated target directory at the time this acceptance note was created; no pass/fail claim is made yet.
- Diagnostics bridge and scoped `cargo check` commands remain pending until the sparse test lane produces an exit code.

## Acceptance Decision
- Pending. The code and documentation are staged for the M8 sparse reservation bridge, but this slice is not accepted until the focused sparse test, diagnostics bridge test, and scoped runtime type-check produce fresh evidence.
- Remaining risks: broad workspace validation is intentionally not claimed because other active sessions are running editor, hub, asset, plugin, and workspace Cargo lanes in the same shared checkout.
