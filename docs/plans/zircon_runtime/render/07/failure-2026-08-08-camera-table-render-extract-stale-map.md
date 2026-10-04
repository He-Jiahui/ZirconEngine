---
handoff_kind: failure
status: open
created_at: 2026-08-08
summary_slug: camera-table-render-extract-stale-map
origin_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
fixing_plan: docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
origin_child_dir: docs/plans/zircon_runtime/runtime/09
fixing_child_dir: docs/plans/zircon_runtime/render/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/tests/render_extract/camera_order.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib --locked -- scene::tests::render_extract::camera_order:: --test-threads=1
  - tools/build/build-editor.ps1
---

# Render07: Camera render extraction still reads the removed fixed map

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 来源执行记录：`docs/plans/zircon_runtime/runtime/09/2026-08-07-runtime-ui-incremental-refresh.md`
- 来源执行切片：M7 product editor bundle build
- 修复责任计划：`docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md`
- 交接原因：失败位于 Render07 的 scene render table-component migration，低于 Editor/UI 构建层。

## 失败现象与复现证据

`tools/build/build-editor.ps1` 编译 `zircon_runtime` 时在 `zircon_runtime/src/scene/world/render.rs:535` 和 `:550` 报错：`World` 已无 `cameras` 字段，但 camera descriptor 构建仍读取旧 map。

## 最低共享层根因

Mesh、sprite、particle 和 post-process 读取已迁移到 typed component storage；camera descriptor 的单实体读取与全量枚举遗漏了同一 hard cutover。

## 架构修复验收

- 单 camera 读取使用 `World::get::<CameraComponent>`。
- camera 枚举直接遍历 registered typed table，以回调提供的稳定 `EntityId` 构建 descriptor。
- 不扫描全部实体，不恢复 `World.cameras`，保持 render order/target/entity 排序语义。
- 原始 Editor bundle production build 通过。

## 禁止临时方案

- 不恢复旧 camera map、镜像字段或同步 shim。
- 不退化为每帧全实体扫描。
- 不添加测试专用分支或静默 fallback。

## 修复结果与回传

2026-08-10 current-source result:

- camera override/active selection 通过 `contains_component::<CameraComponent>` 与 `World::get::<CameraComponent>` 读取 typed storage；首个 fallback camera 直接遍历 camera table 并按稳定 entity id 取最小值，不恢复旧 map。
- `scene_camera_descriptors_with_override` 预分配 registered camera component row count，直接遍历 `for_each_table_component::<CameraComponent>`，通过 entity registry 恢复 stable id，并保持 `(render_order, target_key, entity)` 排序。
- 当前 owner 内无 `self.cameras` 读取；`rustfmt --edition 2021 --check zircon_runtime/src/scene/world/render.rs` 与 scoped source contract 通过。
- 本轮未执行原始 `tools/build/build-editor.ps1` 或受管 Cargo，不能从静态门推导 Editor bundle 或真实 camera render 已通过。

Open state: `camera_table_source_repair_complete_pending_managed_editor_bundle_and_render_extract_gate`; no pass is claimed.

### 2026-09-19 rolling source-contract recheck

Successor Session `failure-roll-01a084c8-render07-camera-table-r1` claimed this
failure record and `zircon_runtime/src/scene/world/render.rs` at baseline epoch
`611`. The exact current source preserves the lower-layer camera-table repair:

- `build_render_camera` selects an override/active camera through typed
  `CameraComponent` storage and reads it with `World::get`;
- `first_scene_camera_entity` traverses only the registered camera table and
  retains the minimum stable entity id;
- `scene_camera_descriptors_with_override` traverses the registered camera
  table, applies the selected descriptor override, and keeps the documented
  `(render_order, target_key, entity)` ordering;
- the focused `camera_order.rs` contract tests cover the typed-storage,
  stable-fallback, frozen-descriptor, and render-extract paths.

The local source-contract checks and `rustfmt --edition 2021 --check` pass for
the current source. This is lower-layer evidence only. The original managed
`tools/build/build-editor.ps1` production bundle, Cargo render-extract tests, and
camera render acceptance have not run in this cycle, so this lifecycle remains
`open` and no fixed return is claimed.

### 2026-09-19 static materialization correction

The first current-source ticket `152175baf16e43a0afdd221f4d510467` reached the
managed wrapper but terminated before its contract assertions: standalone
`rustfmt` resolved `mod lights` relative to the copied single-file root and
reported the absent `world/lights.rs`. The repository's actual child is
`world/render/lights.rs`; this was a validation-copy/module-context issue, not
a camera-table assertion result. No pass is claimed from that run.

Corrected ticket `6249ef62b62545abbfbcecac999b4ddb` (request
`failure-roll-01a084c8-render07-camera-table-20260919-r3`) keeps the same
three-file source manifest and uses `rustfmt --config skip_children=true` for
the static single-file check. The original failure remains retained; managed
Editor/Cargo/render acceptance, independent review, fixed return and closeout
remain pending until the corrected ticket reaches a terminal receipt.

The corrected ticket reached terminal `passed`: coordinator job
`bab891d589f84db9a434c20b2915f609` / run
`6249ef62b62545abbfbcecac999b4ddb` exited 0 and emitted
`RENDER07_CAMERA_TABLE_SOURCE_CONTRACT_PASS`; cleanup was recorded complete.
This receipt only proves the corrected static materialization and source
contract. Managed Editor/Cargo/render acceptance, independent review, fixed
return, and closeout remain pending.

### 2026-09-20 independent static review

- Reviewer Session `review-render07-camera-table-r1` independently inspected `zircon_runtime/src/scene/world/render.rs` and the `scene/tests/render_extract/camera_order.rs` contract tests. Findings are `Critical=0 / Important=0 / Moderate=0`.
- `build_render_camera` selects overrides and active cameras through typed `CameraComponent` storage; `first_scene_camera_entity` traverses only the registered camera table and retains the minimum stable entity id. `scene_camera_descriptors_with_override` applies the selected descriptor before sorting by `(render_order, target_key, entity)`.
- The render-extract tests cover frozen descriptor reuse, typed-storage selection, stable fallback, custom-target layers, and explicit-camera isolation. No legacy `self.cameras` map or entity-wide scan remains in the owned path.
- This is static/read-only review evidence only. Managed Editor bundle/Cargo, Render04/Runtime15 upward gates, and canonical return/closeout remain pending; no dynamic pass is claimed.

### 2026-09-27 current typed-table regression correction

- Successor fixing Session `failure-roll-01a0df1a-render07-camera-table-r1` received
  this record and the exact `camera_order.rs` test from archived r1 through transfer
  fingerprint `b632dc80f45c43fc47d6aa7de1391ddd225cb4337960f6f617a88e4282b8da97`.
  The active production file `zircon_runtime/src/scene/world/render.rs` remains with its
  actual owner and was not edited or transferred by this repair.
- The previous `scene_camera_extraction_stays_on_typed_camera_storage` static assertion
  required `location_for_internal` in `first_scene_camera_entity`. That lookup no longer
  exists in the current production path: `ArchetypeIndex::for_each_table_component` passes
  each stable `EntityId` directly to its callback. `first_scene_camera_entity` now takes
  the minimum callback ID. The old assertion would fail even with correct production
  behavior; its original bytes and source history remain available in the pre-edit
  snapshot `4712`. The corrected source guard requires typed camera-table traversal
  and no retired location lookup or all-entity/map scan; the behavioral regression
  below checks stable selection independently of the production expression.
- A dynamic regression moves the smallest stable camera through a typed table removal
  and reinsertion, asserting selection of the other camera while the first is absent
  and restoration of the smallest stable ID after reinsertion. A removed active-camera
  component forces both extracts through fallback selection. Existing camera ordering,
  target, frozen descriptor and explicit
  camera tests remain intact. The active lower command names the current mounted
  `scene::tests::render_extract::camera_order::` module; actual nonzero execution is
  required. The original `tools/build/build-editor.ps1` product build remains a separate gate.
- Prior corrected ticket `6249ef62b62545abbfbcecac999b4ddb` proves only its old static
  source contract; it cannot validate this changed test file or the current production
  input. No Cargo test or Editor bundle was run here. Normal coordinator inventory must
  freeze all compile inputs. All build products and compiler caches must physically stay
  under drive-root `D:\cargo-targets`, `E:\cargo-targets` or `F:\cargo-targets`.
  Failure remains `open` pending managed lower, original product and upward gates,
  independent final review, canonical fixed return and closeout.
