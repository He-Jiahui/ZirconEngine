---
handoff_kind: failure
status: open
created_at: 2026-07-16
summary_slug: forward-depth-shadow-compare-inversion
origin_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
fixing_plan: docs/plans/zircon_runtime/render/05-lighting-shadows.md
origin_child_dir: docs/plans/zircon_runtime/render/18
fixing_child_dir: docs/plans/zircon_runtime/render/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --jobs 1 --no-run --message-format short --color never
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --jobs 1 render_shadow_atlas_compare_function_matches_forward_depth_contract -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --jobs 1 render_volumetric_shadow_equality_depth_remains_visible_with_less_equal_compare -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --jobs 1 -- --test-threads=1
---

# Render05：Forward-depth shadow comparison inversion

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 来源执行切片：Render18 AF-M3 D3D12 replay / volumetric light-scatter ShadowAtlas consumer.
- 修复责任计划：`docs/plans/zircon_runtime/render/05-lighting-shadows.md`
- 交接原因：ShadowAtlas depth-clear and comparison-sampler ownership is Plan 05; Render18 only consumes the atlas.

## 失败现象与复现证据

D3D12 replay recorded a nonempty 4096 D32 atlas with min depth `0.538` and max depth `1.0`.
Shadow rendering uses forward depth (`LessEqual`) with a clear value of `1.0`, while the atlas comparison sampler is bound as `GreaterEqual`. A receiver evaluates `light_ndc.z - bias`; therefore a clear aperture sampled as `1.0` is incorrectly treated as occluded/dark.

The focused regression now requires `SHADOW_ATLAS_COMPARE_FUNCTION` to be `LessEqual`; against the pre-fix sampler value it must fail before implementation is changed.

## 最低共享层根因

`shadow/atlas/resources.rs` declared the comparison sampler as `GreaterEqual`, which inverts the forward-depth `LessEqual` / clear-`1.0` atlas convention shared by every receiver, including Render18 light scatter.

## 架构修复验收

- The focused Plan 05 shadow sampler contract test passes with `LessEqual` for the forward-depth/clear-`1.0` atlas.
- The managed original Render18 AF-M3 `advanced_lighting` gate is rerun after the focused test.
- No Render18 froxel, plugin, or Shader06 source path is changed; no threshold is weakened.

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- Do not change Render18 froxel/plugin or Shader06 source paths to compensate for the shared atlas contract.

## 修复结果与回传

Open state: `source repair and static fixture review complete; managed validation pending`; r3 current-source reconciliation is static only and does not claim fixed/return/closeout.

- The shared forward-depth contract is `SHADOW_ATLAS_COMPARE_FUNCTION = LessEqual` with a clear value of `1.0`; the volumetric consumer imports that single constant instead of restating sampler polarity.
- The Render18 froxel fixture now writes an occluder depth and exercises a receiver exactly at that depth, proving the equality case remains visible under `LessEqual` while preserving the unshadowed comparison region.
- The scoped Rustfmt check passes and the focused diff check reports only repository CRLF warnings. The required managed Plan05 and Render18 gates, plus real product evidence, remain outstanding, so this handoff remains `open`.

### 2026-08-23 managed current-source validation

- Coordinator job `69a871b625ac4bc6b3f113fef45452c7` ran the declared focused `zircon_runtime` lib-test filter and released normally with exit code 1 after Cargo returned 101.
- Compilation stopped before the focused test executed. The first current-source errors are unrelated UI test debt: `ui/tests/asset_surface_index/binding_ownership_performance.rs:129` cannot resolve `TARGET_BINDING_COUNT`, followed by `ui/tests/text_pipeline/measure_cache.rs:653-654` failing to resolve `measure_line_width`.
- No diagnostic was attributed to `shadow/atlas/resources.rs`, but a compile-blocked test is not a pass. The Plan05 focused gate and Render18 upward gate therefore remain pending; this record stays `open` and the unrelated UI owners are not absorbed here.

### 2026-09-06 immutable validation admission

- Session `failure-roll-01a07160-render05` submitted request `failure-roll-01a07160-render05-focused-20260906` for `cargo test -p zircon_runtime --lib render_shadow_atlas_compare_function_matches_forward_depth_contract --locked`, with source SHA-256 `8be2b2762f25ec24f035080bab2100debf673b3444d9300b9ddc8ebb5727109d` and Rust `1.94.1` on Windows.
- Admission rejected the request with `validation_ticket_external_worktree_dirty` for `E:\Git\zr_vm`. No validation ticket or Cargo run was created, and the focused test did not execute. The external repository owner must commit its pending changes before that clean revision can be sealed.
- Both declared gates remain pending. This failure stays `open`; no duplicate request, fixed return, commit, or success notification is justified by the admission response. Independent failures can proceed while this dependency is unresolved.

### 2026-09-11 failure rolling repair

- Current-source review confirms the shared sampler constant is `SHADOW_ATLAS_COMPARE_FUNCTION = wgpu::CompareFunction::LessEqual`, and the focused regression `render_shadow_atlas_compare_function_matches_forward_depth_contract` remains present. Rustfmt and scoped diff checks pass; no Render18 or Shader06 path was changed.
- Snapshot 3415 freezes this failure record and `zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs` (source SHA-256 `8be2b2762f25ec24f035080bab2100debf673b3444d9300b9ddc8ebb5727109d`). Managed request `render05-forward-depth-shadow-20260911-r1` used `cargo +1.94.1 test -p zircon_runtime --locked --lib render_shadow_atlas_compare_function_matches_forward_depth_contract -- --exact --test-threads=1 --nocapture`.
- Admission again returned `validation_ticket_external_worktree_dirty` for `E:\Git\zr_vm`; no ticket was created and Cargo did not execute. This session is `waiting_validation`; after the external owner supplies a clean revision, rerun the exact focused gate, then the required Render18 `advanced_lighting` upward gate. No return, closeout, or pass is claimed.

### 2026-09-19 rolling source-contract snapshot (Render05 owner retry)

- Retry Session `failure-roll-01a084c8-render05-forward-depth-shadow-r2` transferred only this failure record and `zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs`; Render18 fixture and Shader06 paths remain outside this ownership scope.
- Current source still exposes the single `SHADOW_ATLAS_COMPARE_FUNCTION = wgpu::CompareFunction::LessEqual` forward-depth contract and the focused equality regression. The current `rustfmt --check` reports one import-order diff in this shared file (so it is not recorded as green); the source snapshot for `resources.rs` is `97d60ca2674bbd764a447546378a135a690b1f43aebc7107ad356ddede3801e1`.
- The prior managed focused Cargo admissions (2026-08-23 and 2026-09-11) were rejected before ticket creation by `validation_ticket_external_worktree_dirty` for `E:\Git\zr_vm`; no dynamic test result is reused. The new ticket must therefore remain focused on source-contract evidence until that external dependency is clean.
- State remains `source_repair_recorded / static_source_contract / managed_validation_pending`; no fixed/return or closeout is justified until the exact Plan05 test, Render18 `advanced_lighting` upward gate, and independent Critical/Important/Moderate review complete.

### 2026-09-19 successor static source-contract receipt

Fixing Session `failure-roll-01a084c8-render05-forward-depth-shadow-r2` sealed
ticket `d4145b717f6e49fca4452487ee12829f`. Coordinator copy job
`4c7f214d515c4e28ab859ce8d1a8f8d1` and run
`d4145b717f6e49fca4452487ee12829f` exited 0 with
`RENDER05_FORWARD_DEPTH_SHADOW_SOURCE_CONTRACT_PARSE_PASS`. The ticket covered
the failure record and the shared `shadow/atlas/resources.rs` owner only; the
pre-receipt source manifest was
`20905a27603457caae77695480996252737d52348cc660c969a925c598f85525`.

This is parse/static evidence only. The exact Plan05 Cargo test, Render18
`advanced_lighting` upward gate, external `E:/Git/zr_vm` admission,
independent Critical/Important/Moderate zero-finding review, canonical
`failure return`, and closeout remain pending. The appended receipt changes
the document hash after the ticket snapshot; a dynamic successor must reseal
the current bytes.

### 2026-09-20 independent static review

- Reviewer Session `review-render05-forward-depth-r2` independently inspected the owned `shadow/atlas/resources.rs` path and the read-only Render18 upward fixture. Findings are `Critical=0 / Important=0 / Moderate=0`.
- `SHADOW_ATLAS_COMPARE_FUNCTION` is the single shared comparison truth and is `wgpu::CompareFunction::LessEqual`; `create_compare_sampler` consumes it, and no `GreaterEqual` shadow-atlas comparison remains in the scene-renderer path.
- The focused unit regression `render_shadow_atlas_compare_function_matches_forward_depth_contract` asserts the forward-depth contract. The upward fixture imports the same constant and `render_volumetric_shadow_equality_depth_remains_visible_with_less_equal_compare` exercises occluder-depth equality against the unshadowed shaft. No Render18 or Shader06 source was modified or absorbed.
- This review is static/read-only evidence only. Managed Plan05 Cargo, Render18 `advanced_lighting`, GPU/RenderDoc artifacts, canonical return, and closeout remain pending; no dynamic pass is claimed.

### 2026-09-25 current-source r3 reconciliation

- Successor Session `failure-roll-01a084c8-render05-forward-depth-shadow-r3` owns this failure record only. The source owner remains the existing Render05 source chain; no Render18 or Shader06 path is claimed by this Session.
- Current production source still contains one shared `SHADOW_ATLAS_COMPARE_FUNCTION = wgpu::CompareFunction::LessEqual`; `create_compare_sampler` consumes that constant, and the focused equality regression plus the Render18 upward fixture resolve to the same symbol. The current source hash is `97d60ca2674bbd764a447546378a135a690b1f43aebc7107ad356ddede3801e1`.
- `git status` reports an existing foreign import-order-only diff in `zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs`; this Session does not absorb or edit that source. Therefore rustfmt is not recorded as green, and the exact Cargo commands in frontmatter remain pending until the same attributed source snapshot can be admitted.
- The current command list is executable and exact: the focused Plan05 test is `render_shadow_atlas_compare_function_matches_forward_depth_contract`, the Render18 upward equality test is `render_volumetric_shadow_equality_depth_remains_visible_with_less_equal_compare`, followed by the Runtime lib gate. Existing external `E:\Git\zr_vm` dirty-state admission failures remain evidence of a blocker, not a dynamic pass.

### 2026-09-25 independent r3 static review

- Reviewer `review_render05_forward_depth` re-read the current source contract and this handoff. Findings are `Critical=0 / Important=0 / Moderate=0`.
- The current `resources.rs` SHA-256 is `97d60ca2674bbd764a447546378a135a690b1f43aebc7107ad356ddede3801e1`, matching the r3 reconciliation. `SHADOW_ATLAS_COMPARE_FUNCTION` remains `LessEqual`; the focused Plan05 test and Render18 equality filter both resolve to the declared symbols.
- The review is static/read-only evidence. The source path remains a foreign dirty file owned by archived r2 with no live lease, and the Render18 upward fixture is currently modified without attribution. Managed Cargo admission and dynamic execution therefore remain pending; this record stays `open` with no fixed return or closeout.

### 2026-09-26 successor intake (failure-roll-01a084c8-render05-forward-depth-shadow-r4)

- The stale r3 lifecycle was cancelled through the coordinator after its
  heartbeat expired with no active lease. Successor
  `failure-roll-01a084c8-render05-forward-depth-shadow-r4` now owns only this
  failure record. Ownership transfer fingerprint is
  `d20c8391da60fd8a1165961e85b039fb2945c854d42d3811b6ee693c7ede31b7`, and
  pre-review snapshot `3920` sealed the record at SHA
  `9d317cab27847d04d108d05a0ced8caf3a8d1ab5a4c694f69492ab2fff5fe040`.
- The current shared source contract remains
  `SHADOW_ATLAS_COMPARE_FUNCTION = wgpu::CompareFunction::LessEqual` at
  SHA-256 `97d60ca2674bbd764a447546378a135a690b1f43aebc7107ad356ddede3801e1`.
  The source file has a foreign import-order-only dirty diff and is not
  claimed, edited, or leased by r4; the Render18 upward fixture likewise
  remains outside this scope. Earlier static ticket
  `d4145b717f6e49fca4452487ee12829f` and the r3 review are historical
  evidence only and do not satisfy a current dynamic gate.
- External `E:/Git/zr_vm` dirty admission, the exact Plan05 Cargo test,
  Render18 `advanced_lighting` equality gate, GPU/RenderDoc evidence,
  independent review of this refreshed record, canonical `fixed-*` return,
  closeout, and WeCom notification remain pending. The failure stays open;
  no dynamic pass is claimed.

### 2026-09-26 independent successor review receipt

- Read-only reviewer `/root/review_render05_forward_depth` rechecked the
  successor at snapshot `3921` (SHA
  `8c3d0b070f44615b04152a05fb75cf2797b8177a0b2a1fd62ff579335006402e`). The
  stale r3 cancellation/no-lease state, r4 doc-only scope, and transfer
  fingerprint `d20c8391da60fd8a1165961e85b039fb2945c854d42d3811b6ee693c7ede31b7`
  are consistent.
- The shared `resources.rs` hash remains
  `97d60ca2674bbd764a447546378a135a690b1f43aebc7107ad356ddede3801e1`; its
  import-order-only dirty diff is foreign/unowned. The Render18 upward fixture
  remains outside this lifecycle and dirty/foreign. The historical ticket
  `d4145b717f6e49fca4452487ee12829f` is explicitly static-parse-only and is
  not reused. C/I/M: **Critical=0 / Important=0 / Moderate=0**.
- Exact Plan05 Cargo, Render18 `advanced_lighting`, GPU/RenderDoc evidence,
  external clean admission, canonical return, closeout, and WeCom remain
  pending. No dynamic pass or fixed return is claimed; the failure stays open.
