---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: virtual-geometry-stable-instance-key-collapse
origin_plan: docs/plans/zircon_runtime/render/04-visibility-culling.md
fixing_plan: docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
origin_child_dir: docs/plans/zircon_runtime/render/04
fixing_child_dir: docs/plans/zircon_runtime/render/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/visibility/declarations/visibility_virtual_geometry_draw_segment.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/execution.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/virtual_geometry_indirect.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_output_sources/virtual_geometry_snapshot_rebuild.rs
tests:
  - automatic virtual-geometry extraction retains original primitive ordinals after non-VG filtering
  - same-entity virtual-geometry visibility selection by stable instance key
  - same-entity virtual-geometry pending draws expand only their own stable-key segments
  - same-entity virtual-geometry execution statistics preserve distinct stable-key segments
  - legacy virtual-geometry key falls back only to the entity's primitive zero key
  - same-entity virtual-geometry execution-snapshot reconstruction by stable instance key
---

# Render03: virtual-geometry segment expansion collapses sibling primitives

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/04-visibility-culling.md`
- 来源执行切片：Render04 multi-primitive visibility failure forward repair second review
- 修复责任计划：`docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md`
- 交接原因：virtual-geometry extract, segment identity and GPU-driven indirect expansion own the missing render-instance identity; Render04 must not add an entity-keyed call-site filter.

## 失败现象与复现证据

Scene extraction emits a unique `stable_instance_key` for each primitive. Virtual-geometry cluster,
instance and visibility segment contracts retain only `EntityId`; `virtual_geometry_indirect.rs`
groups execution segments by entity and expands every matching segment into every pending draw for
that entity. Two primitives owned by one entity therefore cross-submit segments and duplicate
indirect work.

## 最低共享层根因

The Render03 virtual-geometry DTO boundary models authoring ownership but not render-instance
identity, unlike the mesh GPU-scene boundary. Entity-keyed grouping is therefore structurally
unable to distinguish sibling primitives.

## 架构修复验收

- Carry `stable_instance_key` from virtual-geometry extract through visibility draw segments and execution segments.
- Group and expand indirect segments by stable render-instance key; retain `EntityId` only as authoring ownership metadata.
- Add a same-entity/two-primitive regression where each primitive receives only its own segment and indirect draw count.

## 禁止临时方案

- Do not filter or deduplicate entity-keyed segment lists at the Render04 call site.
- Do not use primitive ordering as an implicit identity or duplicate a second mapping table.

## 修复结果与回传

Source repair is integrated forward: automatic extraction now emits the mesh-compatible key for
each model primitive; visibility plans and draw segments preserve that key; execution snapshots,
debug reconstruction, indirect expansion, and prepared-queue execution statistics use it as their
grouping identity. Legacy authored extracts without the field use the existing `(entity, primitive
0)` compatibility key.

## 二次审查

- 2026-08-01: 完成 P2 覆盖补齐后复审。原始 primitive ordinal 在 non-VG filter 后保持不变；生产 pending-draw expansion
  直接覆盖 same-entity sibling 与 legacy primitive-0 隔离。未发现可操作的 correctness、性能或模块边界问题。
- 2026-08-13: 按 failure-first 规则再次核对五个 related-code owner：extract DTO、visibility segment、debug snapshot、
  indirect expansion 与 plugin snapshot rebuild 均以 `stable_instance_key`/`stable_instance_key_or_legacy()` 传递和分组，
  未发现 entity-keyed sibling collapse 残留；源码修复保持完成，动态 acceptance 仍由 coordinator 管理。

Status remains `resolving_failure` until coordinator-managed Windows validation and the required
render screenshot/RDC evidence are accepted; no pass is claimed here.

## 2026-09-19 受管静态合同回执

- 当前快照：`6b4bc86089cb4464f850136c8079e90cb6513ebc`，baseline epoch `611`。
- 票据：`74faa92926074515a4d1ae44752b28b5`；受管 job `31d943e7452f44c0ad6ebbab71939284`；run `74faa92926074515a4d1ae44752b28b5`；终态 `passed`，exit code `0`（2026-09-19）。
- source manifest：`6ea5b3ae0cf9e2732597464dd40ef8db95992c7008721774edfad56818997158`；输出 `RENDER03_VG_STABLE_INSTANCE_KEY_SOURCE_CONTRACT_PASS`，`CHECKED_PATHS=6`，确认 execution owner 使用 `stable_instance_key_or_legacy`/`instance_index_for_draw_segment`，indirect 与 plugin owner 保持 stable-key 分组。
- 票据 `3e702e32b9a2442dbf20f752580f5da9`、`49bc7c0996474219bc36c24e4b991910`、`43e138c92cb144da8ec8b9e3339a4709` 的 checker-only anchor 失败均保留为原始证据；本回执只复用当前源码静态通过，不把它们改写为动态通过。
- 仍待：受管 Windows `zircon_runtime`/virtual-geometry Cargo `--locked` 回归、Render03/04 upward GPU-driven gate、WGPU/indirect screenshot 与 RenderDoc 证据、1/100/1k/10k 规模证据、独立 Critical/Important/Moderate 零问题审查、canonical `fixed-*`/`failure return`/closeout/真实 SHA/企微回执；`E:\Git\zr_vm` dirty admission blocker 仍有效，failure 保持 open。

### 2026-09-20 independent review

- Reviewer Session `review-render03-vg-stable-key-r1` inspected the source-sealed failure record and
  all five related-code owners. Current hashes are `scene_extract.rs`
  `3378b7c127f5c23e3c69cba6f7f015e897b9b95343ca2ab7ac528ee6c8ff6736`,
  `visibility_virtual_geometry_draw_segment.rs`
  `8d0f1fad2561e79eaa22945697e3abb1239a062077addd674125f041260c61b8`,
  `execution.rs` `f552eee90c30e151b557567fa84c67251a2b75d3065c4feb6dbffa671ad4bc31`,
  `virtual_geometry_indirect.rs`
  `48805de94b49723694410ac2b14d11db76aa0bf9e3eacbd1bc272ae6baced948`, and
  `virtual_geometry_snapshot_rebuild.rs`
  `27168a5d78b6af860d3918adb1fa186819b279d89b8aa4bcc45ea677e65ba736`.
  The reviewer held the failure-document lease for the audit.
- Review result: `Critical=0`, `Important=0`, `Moderate=0`. Visibility draw segments carry
  `stable_instance_key`; execution snapshots index clusters by `(entity, stable key, cluster/page/
  LOD)` and retain `stable_instance_key_or_legacy()` for authored legacy data. Indirect expansion
  groups by stable key and uses each pending draw’s key, while plugin snapshot reconstruction indexes
  and deduplicates by stable key plus cluster identity. Entity remains authoring metadata rather than
  the sibling-primitive grouping key.
- The source probe emitted `RENDER03_VG_STABLE_KEY_REVIEW_CONTRACT_PASS`; scoped `git diff --check`
  passed. Rustfmt reports only existing import-order drift in shared owners (no review edits were
  made). Managed Runtime/virtual-geometry Cargo, Render03/04 GPU-driven acceptance, 1/100/1k/10k
  evidence, WGPU/RenderDoc artifacts, fixed return, and closeout remain pending; static evidence is
  not promoted to those gates.
