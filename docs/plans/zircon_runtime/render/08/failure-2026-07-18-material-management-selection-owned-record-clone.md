---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: material-management-selection-owned-record-clone
origin_plan: docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
fixing_plan: docs/plans/zircon_runtime/render/08-material-shader-permutation.md
origin_child_dir: docs/plans/zircon_runtime/render/03
fixing_child_dir: docs/plans/zircon_runtime/render/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/render/material/management/selection.rs
tests:
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter selection_owns_records_after_source_changes -VerboseOutput
  - cargo +1.94.1 test -p zircon_runtime --lib graphics::tests::render_product_advanced::gpu_driven_product::render_product_gpu_scene_multi_draw_64_instances_matches_cpu_fallback --locked --jobs 1 --color never -- --exact --nocapture --test-threads=1
---

# Render08: material management selection 必须克隆 owned record

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md`
- 来源执行切片：GS-M4 64-instance multi-draw product evidence gate
- 修复责任计划：`docs/plans/zircon_runtime/render/08-material-shader-permutation.md`
- 交接原因：最低共享错误位于 Render08 的 framework material management selection，不属于 Render03 GPUScene、indirect submission 或产品证据 scope。

## 失败现象与复现证据

- Render03 managed GPU reservation `fd933159ee9d4f5a993721b85c576450`、job `677e6af32cfb4a7b8ab90159712cb6a5`、run `e77e3dad12c840008de054cbaa2e9792` 在进入 exact 产品测试前 terminal/released exit 101，无 live PID。
- Rust 1.94.1 编译 `zircon_runtime` lib test 时在 `selection.rs:51/52/53/59` 报告 4 个 `E0308`：`Vec<&RenderMaterialManagementRecord>` 被传给要求 `&[RenderMaterialManagementRecord]` 的 summary/status/issue 构造器，并被写入要求 owned record 的结果字段。
- broken source 在 `records_by_id.get(material_id)` 返回 `&&RenderMaterialManagementRecord` 后执行 `record.clone()`，只克隆引用而不是 record。
- 编译结束后共享工作树已出现 `selected_records.push((**record).clone())` 候选改动，但该路径当时无 coordinator lease/attribution，且没有 fresh managed compile、review 或 commit，因此不能把失败结果改写成通过。
- RenderDoc 和 ignored exporter 均未执行；`plan03_gpu_scene_multi_draw_64_instances_wgpu_20260718.png` 与配对 RDC 均未生成。

## 最低共享层根因

Material selection 的索引有意借用输入 records，但 selection 结果的公共合同拥有完整 records。broken implementation 对 `HashMap<ResourceId, &RenderMaterialManagementRecord>::get` 返回的二次引用使用浅层 `clone`，让借用类型泄漏进 owned 输出构造链。

## 架构修复验收

- Render08 owner 以显式 owned record clone 保持 request order、duplicate-id collapse、missing-id 和 summary/status/issue 一致性，并增加或确认 selection focused tests。
- 在 immutable current source 上通过 material management focused gate，以及本记录 frontmatter 中的 Render03 原始 exact compile/product reproduction。
- Render03 owner 获得 fixed return 后重新运行 DX12 WGPU parity + ignored PNG exporter + RenderDoc capture，PNG/RDC 两个 exact artifact 同时存在才可关闭 GS-M4 evidence slice。

## 禁止临时方案

- 不得把 selection 的 owned `records` 公共合同改为借用引用来绕过编译错误。
- 不得跳过 summary/status/issue 构建、弱化 Render03 产品断言或以旧 test binary/capture 代替 current-source upward gate。
- 不得由 Render03 会话吸收 material management 源路径或未归属的共享工作树改动。

## 修复结果与回传

Open state: `current-source ownership repair present; managed validation pending`.

- The selection index still borrows its input records, but `RenderMaterialManagementSelection::from_records` explicitly clones `(**record)` into the public owned result before deriving summary, status, and issue indexes.
- Request ordering, duplicate-id collapse, and missing-id behavior remain in the same selection owner; no borrowed-record compatibility path was introduced.
- `selection_owns_records_after_source_changes` now mutates the source row after selection and asserts that the selected public record retains its original value, directly guarding the owned-clone boundary.
- The 2026-08-27 managed focused replay was rejected before Cargo with `unmanaged_artifacts_detected`: cleanup reservation `D:\ZirconBuilds\tooling15-wave140-runtime-20260827-071332` still existed. Official cleanup request `3d5ae58970f54b4793bcf33756955e8a` preserved the identity-bound reservation and returned the path in `failed`/`remaining`; no Render08 Cargo job or test result was created.
- The required managed material-management and Render03 product gates, including PNG/RDC evidence, remain outstanding. This handoff therefore stays `open` and does not return `fixed`.

## 2026-09-11 failure rolling repair

- The owned-record repair remains present in the current `HEAD`: `RenderMaterialManagementSelection::from_records` clones `(**record)` before deriving the public summary, status, and issue indexes, and `selection_owns_records_after_source_changes` is present in the same production test owner.
- The exact source file currently also contains an unrelated unowned capacity/test-mount overlay (`selection/optimization_batch_iy_runtime636_tests.rs`). This rolling session does not absorb or rewrite that neighboring work; no current-source validation claim is made while the overlay is unresolved.
- A primary Render08 session was registered with leases and baseline attribution for this failure document and `selection.rs`. Managed focused validation is deferred until the immutable source can be sealed; the coordinator's external validation lane is also currently blocked by dirty `E:\Git\zr_vm`.
- Corrected snapshot `3424` froze the two owned paths. Request `render08-material-selection-owned-clone-20260911-r1` used `cargo +1.94.1 test -p zircon_runtime --lib selection_owns_records_after_source_changes --locked --color never -- --exact --nocapture --test-threads=1` and was rejected at admission with `validation_ticket_external_worktree_dirty` for `E:\Git\zr_vm`; no ticket, Cargo job, or dynamic result was created.
- No Cargo/test, failure return, review, or closeout is claimed from this admission-only record. The lifecycle remains `open`, and the primary session is `waiting_validation` until the external repository can be sealed cleanly.

### 2026-09-19 rolling source-contract snapshot (Render08 owner retry)

- Retry Session `failure-roll-01a084c8-render08-material-selection-r2` transferred only this failure record and `zircon_runtime/src/core/framework/render/material/management/selection.rs`; the neighboring `selection/optimization_batch_iy_runtime636_tests.rs` overlay remains foreign and untouched.
- Current source still clones `(**record)` into the public owned selection before deriving summary/status/issue indexes, and the source-change regression remains present. The current file parses, but recursive `rustfmt --check` reports an import-order diff in the foreign nested overlay; this is recorded as non-green rather than hidden. The owned source hash is `502f514a9d28784f60f4a2c0d14918abf538a5f3d8f998651f512dde74d1eb8d`.
- The 2026-09-11 focused Cargo admission was rejected before ticket creation by `validation_ticket_external_worktree_dirty`; the 2026-08-27 unmanaged-artifact rejection and old compile failures are not dynamic passes. A fresh managed focused test and the Render03 product/upward gates remain required.
- State remains `source_repair_recorded / static_source_contract / managed_validation_pending`; no fixed/return or closeout is justified until exact behavior, PNG/RDC evidence, and independent Critical/Important/Moderate review complete.

### 2026-09-19 failed static ticket evidence (preserved)

- Retry ticket `e60c360a460a4caaa0755b5b7426cbe5` (request `failure-roll-01a084c8-render08-material-selection-20260919-r2`) was admitted and materialized, but ended `failed` with coordinator failure category and exit code 1.
- The source-contract assertions were not reached: the sealed manifest contained only `selection.rs`, while `rustfmt --emit stdout` recursively required the unsealed module `zircon_runtime/src/core/framework/render/material/management/selection/id_capacity_tests.rs`. The managed copy therefore reported `os error 3` and `rustfmt parse failed`.
- This is validation-harness/source-manifest incompleteness, not evidence against the owned-record clone. The failed result is excluded from reuse; a corrected ticket must seal the direct module dependency while retaining the foreign nested overlay boundary.

### 2026-09-19 corrected static validation

- Corrected ticket `cf7fbd8c579446328aaa68e13397e498` (request `failure-roll-01a084c8-render08-material-selection-20260919-r3`) sealed the same two owned inputs and ran the root-only contract with `rustfmt +1.94.1 --edition 2024 --config skip_children=true --emit stdout`; this intentionally avoids traversing the separately attributed nested test overlay.
- Coordinator job `385c555fe77543f49ae96c0101e235fe` / run `cf7fbd8c579446328aaa68e13397e498` finished exit 0 with stdout `RENDER08_MATERIAL_OWNED_SELECTION_SOURCE_CONTRACT_PARSE_PASS`; cleanup completed. This is static source-contract evidence only; managed Render08 Cargo, Render03 product/PNG/RDC, review, and fixed return remain pending.

### 2026-09-20 independent review

- Reviewer Session `review-render08-material-selection-r2` inspected the source-sealed plan,
  failure record, and `zircon_runtime/src/core/framework/render/material/management/selection.rs`
  at owned source hash `502f514a9d28784f60f4a2c0d14918abf538a5f3d8f998651f512dde74d1eb8d`.
  The reviewer held the failure-document lease while checking the exact Render08 scope. The
  separately attributed `selection/optimization_batch_iy_runtime636_tests.rs` overlay was read
  only for boundary awareness and was not absorbed or edited.
- Review result: `Critical=0`, `Important=0`, `Moderate=0`. `records_by_id` intentionally stores
  borrowed input records, while `get` yields a second reference; `selected_records.push((**record).clone())`
  therefore clones the `RenderMaterialManagementRecord` value into the public owned result before
  deriving summary, status, and issue indexes. Request order, first-record duplicate collapse,
  missing-id partitioning, and the source-change ownership regression remain coherent.
- The focused `selection_owns_records_after_source_changes` test mutates the input after selection
  and checks that the selected name remains `Original`, directly exercising the ownership boundary.
  The corrected managed static ticket `cf7fbd8c579446328aaa68e13397e498` / job
  `385c555fe77543f49ae96c0101e235fe` remains valid for the sealed root module and emitted
  `RENDER08_MATERIAL_OWNED_SELECTION_SOURCE_CONTRACT_PARSE_PASS`; the earlier incomplete-manifest
  failure is not reused.
- Scoped `rustfmt --check --edition 2024 --config skip_children=true` for the owned root module and
  `git diff --check` both passed. This review made no source edits. Managed Render08 Cargo,
  Render03 upward product reproduction, PNG/RDC evidence, performance/ownership measurements,
  fixed return, and closeout remain pending and are not claimed by this receipt.

### 2026-09-25 current-source rolling reconciliation (Render08 owner)

- Session `failure-roll-01a084c8-render08-material-selection-r3` owns this refresh. Snapshot `3816` seals `zircon_runtime/src/core/framework/render/material/management/selection.rs` at SHA-256 `502f514a9d28784f60f4a2c0d14918abf538a5f3d8f998651f512dde74d1eb8`.
- The separately attributed untracked `selection/optimization_batch_iy_runtime636_tests.rs` overlay remains foreign and is excluded from the snapshot and write scope. No source edits were made in this reconciliation. `rustfmt +1.94.1 --edition 2024 --config skip_children=true --check` and scoped `git diff --check` pass for the owned root module (only the normal LF→CRLF notice is emitted).
- Exact source contract probe `RENDER08_MATERIAL_OWNED_SELECTION_CURRENT_SOURCE_PASS` confirms `records_by_id` remains a borrowed index while `selected_records.push((**record).clone())` creates owned records before summary/status/issue derivation, and `selection_owns_records_after_source_changes` remains present. This does not replace the required managed focused Cargo test.
- Managed Windows Render08 Cargo, the Render03 DX12/WGPU parity product test, PNG/RDC RenderDoc evidence, performance/ownership measurements, fixed return, closeout, and WeCom remain pending; the failure stays `open`/`resolving_failure`.

### 2026-09-25 independent static review receipt

- Reviewer `review_editor03_gizmo_private` checked snapshot `3816`, the owned root hash, and the excluded foreign nested overlay. Result: Critical/Important/Moderate = `0/0/0`.
- The review confirms borrowed indexing followed by owned record cloning and source-change regression coverage; static marker, formatter, and diff evidence remain non-Cargo only.
