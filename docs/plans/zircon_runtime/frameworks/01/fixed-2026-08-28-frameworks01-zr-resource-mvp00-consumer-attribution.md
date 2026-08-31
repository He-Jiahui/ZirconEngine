---
handoff_kind: fixed
status: fixed
created_at: 2026-08-26
summary_slug: frameworks01-zr-resource-mvp00-consumer-attribution
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/mvp/00-current-source-baseline-recovery.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/mvp/00
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/project/tests/template_creation.rs
  - zircon_editor/src/tests/editing/asset_workspace.rs
  - zircon_editor/src/tests/editor_message/refresh.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/module_dependencies.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs
  - zircon_runtime/src/scene/world/project_io/document.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/tests/runtime_shader_pbr_realtime_ibl_export.rs
tests:
  - .\tools\zircon-session.ps1 ownership matrix --prefix zircon_editor/src -Json
  - .\tools\zircon-session.ps1 ownership matrix --prefix zircon_runtime/src/scene/world/project_io -Json
resolved_at: 2026-08-28
---

# MVP00: resolve false Frameworks01 Resource consumer write attribution gate

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：M1 `zr_resource` current-source atomic scope rotation
- 修复责任计划：`docs/plans/mvp/00-current-source-baseline-recovery.md`
- 交接原因：Frameworks01 originally treated every sealed Resource migration input as a candidate
  write and therefore interpreted MVP00's executable read-only consumer attributions as blockers.

## 失败现象与复现证据

The 2026-08-28 schema-3 Resource snapshot contained 778 atomic inputs and 555 Rust consumers. Nine
dirty read inputs resolved to active Session
`mvp00-current-source-convergence-r2-01a00797-20260818`, so ownership transfer returns
`source_owner_executable` even though no live lease overlaps the union. Current hashes are:

- `template_creation.rs`: `997dd4353a362855ea321a591440ada8d4e02280a44c6945cb7b21ba8e080d78`
- `asset_workspace.rs`: `3392901f5a4fb494712cbc8774e33bbcb439e663efc4e08edbd42e36868333e5`
- `refresh.rs`: `bb6e7cde25466522d99fb7c36fb1be0bbe2cab73ba3eb568ca618c1f28205c33`
- `document_roundtrip.rs`: `0cc0dd787b37c7e6fec93933fe5ca55f37629e9cf64240f930ae9074ee6ef911`
- `module_dependencies.rs`: `5d3ce0c1881aa7ddc8bfbab9d78cb7158afb79cdadfa1899a058741c635fbe97`
- `wgpu_render_framework.rs`: `ec838d95122053c32d2a5ce11526942a05f647036575a10709e43a90edffe746`
- `document.rs`: `938a049909b7c7be886727fa66982e864e0b692ff974582df6db87f3440ec37d`
- `scene_asset.rs`: `e3305645731840b122ee2b4f41636a74796ea2e1f27716482f89fd327654aa92`
- `runtime_shader_pbr_realtime_ibl_export.rs`:
  `aaba139c31a6ff4bfeb129cc4497e1377327f4393f6c40a08951826a09a4c802`

All nine attribution hashes differ from current bytes. Coordinator baseline-516 transfer preview
request `9f0936c8240d4d09ab1949755ed97404` returned fingerprint
`f6e384c869183278ecdf0ed817c9601ee53addef5f952a048d1a732d060c249e`; every path is ineligible
only because the source owner remained executable. The source manifest is retained at
`D:/zircon-frameworks01-r12-resource-current-20260828/hard-cut-source-r1.json`.

That preview was accurate for the requested transfer but the transfer request itself was not an
architecturally valid prerequisite. None of these nine paths occurs in the final schema-3 write
manifest, and the deterministic hard-cut patch changes none of their bytes.

## 最低共享层根因

The lowest proven failure was Frameworks01 admission modeling, not coordinator Session lifecycle or
Resource consumer semantics. The schema-3 atomic input manifest is a sealed read set. Ownership
transfer is required only for the separately generated move/patch write set. Conflating those sets
created a false dependency on MVP00 terminalization and would have transferred foreign code that
the migration neither edits nor owns.

## 架构修复验收

- Seal read inputs independently from the canonical migration write paths.
- Prove that all nine MVP00 paths are absent from the write manifest and emitted patch.
- Acquire attribution/leases only for the exact write set, while preserving the nine MVP00 blobs.
- Apply the no-compat `zr_resource` cut without modifying or transferring these read-only inputs.

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses,
  or call-site exceptions.
- Do not rewrite these mixed blobs merely to change attribution, and do not weaken the complete
  atomic migration gate.

## 修复结果与回传

- 根因：Frameworks01 conflated the sealed schema-3 read set with the migration write set and falsely required transfer of nine unchanged MVP00 consumers.
- 架构修复：Schema-3 now emits a canonical 156-path write manifest from move sources, destinations, required facades, guards, manifests, and eight explicit consumer patches; read-only consumers remain sealed inputs without ownership transfer.
- 验证：All nine MVP00 paths are absent from write manifest 12377714bc021a44fce725ef76f205fc08b8de1e8b4670837a420b380611bb88 and patch 4ee6898de02a4e799750fe911ae8edc7abf9dc78969d07f903ff063899900f67; 156 leases acquired with zero conflicts; 155 emitted changes plus one exact pre-applied root path matched all hashes.
- 回传：False cross-plan attribution gate removed without modifying or transferring any MVP00 blob; Frameworks01 physical zr_resource hard cut is applied, while managed Cargo/product acceptance remains open.
