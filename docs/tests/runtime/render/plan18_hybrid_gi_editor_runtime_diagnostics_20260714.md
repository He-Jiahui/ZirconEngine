# Plan 18 HybridGI Editor Runtime Diagnostics Evidence

- Date: 2026-07-14
- Product: `zircon_app --bin zircon_editor --no-default-features --features target-editor-host`
- Product SHA-256: `B828F854EB342D661637E16A6944D59437937A74AD29F5B25C743899C911251F`
- Build result: passed in 47m12s on Windows with the coordinator-owned E-drive target
- Focused contracts: `17 passed`; exact HybridGI scene-velocity executor regression: `1 passed`

## Custom actual

- PNG: `plan18_hybrid_gi_editor_runtime_diagnostics_actual_20260714.png`
- Size: `1688x980`
- SHA-256: `35A8FF93D8C67E3EEBC6A59F9C251EE9FAB279BC00406753C0D6FD600511844E`
- Visible resolved state: `profile=custom`, `mode=dynamic-only`, `quality=medium`
- Visible budgets: `trace=32`, `cards=64`, `voxels=16`
- Visible fallback: `none`
- Product result: the WGPU viewport is nonblank and presents the project scene; redirected stderr is empty.

## Indoor Static missing-bake fallback

- PNG: `plan18_hybrid_gi_editor_runtime_diagnostics_fallback_20260714.png`
- Size: `1688x980`
- SHA-256: `9619D6E3B8B011CC5EB53D71505D5F19606B02C5FCDFEDDF5ECACEBB4F6BB7CF`
- Visible resolved state: `profile=indoor-static`, `mode=dynamic-only`, `quality=high`
- Visible budgets: `trace=64`, `cards=256`, `voxels=64`
- Visible fallback: `baked-lighting-unavailable`
- Product result: the WGPU viewport is nonblank and presents the project scene; redirected stderr is empty.

The compact diagnostics pane prioritizes the resolved HybridGI profile, budgets, fallback and active-probe count. The Render viewport/frame line remains below the visible fold in this product binary, but the captured viewport contains the submitted scene output rather than the earlier blank zero-frame state. A follow-up display-order test keeps the Render line in the first visible status group for the next product build.

This evidence closes only the Plan 18 HybridGI Editor actual/fallback product subgate. Broad/full workspace validation and later HGI milestones remain open.
