---
handoff_kind: failure
status: open
created_at: 2026-08-15
summary_slug: camera-resolution-scale-symbol-drift
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/06-temporal-pipeline.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/06
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/camera/camera_snapshot.rs
  - zircon_runtime/src/core/framework/render/camera/tests.rs
  - zircon_runtime/src/core/framework/render/view_family/resolution.rs
  - zircon_runtime/src/core/framework/render/view_family/tests.rs
tests:
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipTest
---

# Render06: Camera resolution-scale symbol drift blocks the MVP product build

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：M0 current product baseline recovery before WPR, RenderDoc, and energy capture
- 修复责任计划：`docs/plans/zircon_runtime/render/06-temporal-pipeline.md`
- 交接原因：失败位于 Render06 正在迁移的 camera-to-view-family resolution policy boundary；该文件仍由 `render06-view-family-pipeline-20260815` 持有。

## 失败现象与复现证据

2026-08-15 15:28 CST 启动的受管 `zircon_runtime` build-only job `ebc7ae5809ac4713a0ece1b1503283e8` 在约 4 分 50 秒后以 exit 1 结束。保留在 D 盘共享目标池的 Cargo 指纹诊断显示：

- `E0425`：`zircon_runtime/src/core/framework/render/camera.rs:84:17` 找不到 `DEFAULT_RENDER_RESOLUTION_SCALE`。
- 同次构建的另一处 Render01 `BufferViewMut::fill` 错误已由其所有者在构建结束后修正，不属于本交接。
- 该构建产生 209 条 warning；本记录不把 warning 数量解释为性能结论。

## 最低共享层根因

`ViewportCameraSnapshot::render_view_family_pipeline` 已切换到 `RenderResolutionPolicy::with_scales`，但第二个 scale 仍引用迁移前符号 `DEFAULT_RENDER_RESOLUTION_SCALE`；当前模块只声明 `DEFAULT_DYNAMIC_RESOLUTION_SCALE`。在 Render06 确认 primary/display/history resolution 语义前，来源计划不得用本地别名或任意常量替换来掩盖契约漂移。

## 架构修复验收

- Render06 明确 `with_scales(primary_scale, secondary_scale)` 两个参数各自的像素空间语义，并让 camera 默认值引用唯一、现存的 owner constant。
- `camera_dynamic_resolution_adapts_into_the_view_family_resolution_policy` 及 Render06 的 view-family/temporal-history focused tests 通过。
- 原始受管 `zircon_runtime` build-only 通过，再运行 `tools/build/build-editor.ps1` 产出当前源码的 D/E/F 盘编辑器 bundle。
- 来源性能计划只在可运行 bundle 上继续 WPR、RenderDoc 和功耗采集，不使用 2026-08-10 的旧二进制。

## 禁止临时方案

- 不新增 `DEFAULT_RENDER_RESOLUTION_SCALE` 别名、兼容 shim、静默 fallback 或 call-site 特例。
- 不把第二个 scale 猜成 primary scale 的副本；必须由 Render06 的 resolution-space 契约和测试确定。
- 不降低产品构建或动态性能采集验收门。

## 修复结果与回传

Open state: `待修复`; no product-build or dynamic-performance pass is claimed.

### 2026-09-11 failure rolling repair continuation

The existing stable Session `failure-roll-01a084c8-render06-camera-resolution-scale`
was resumed after its heartbeat expired; no replacement owner was created.  Current
source inspection shows the stale `DEFAULT_RENDER_RESOLUTION_SCALE` reference is
gone.  `ViewportCameraSnapshot::render_view_family_pipeline` now uses the single
view-family owner `MAX_RENDER_RESOLUTION_FRACTION` for the secondary fraction, while
the dynamic setting supplies the primary fraction.  The camera regression preserves
the 1920x1080 display/temporal-history contract and the 2/3 primary extent, and the
view-family suite still covers independent primary/secondary history identity.

Local source-contract validation passed as
`RENDER06_CAMERA_RESOLUTION_STATIC_PASS`; it checked the absence of the retired
symbol, the owner constant and `with_scales` wiring, and the named camera and
view-family tests.  This is static evidence only.  The original build-only failure
and its unrelated Render01 compiler error are not reused as acceptance evidence.

The current six-path snapshot is clean and attributed to this Session.  Fresh
managed validation is still required: the focused camera/view-family lib tests, the
original `zircon_runtime` build-only gate, and the current-source editor bundle
build.  No product bundle, WPR, RenderDoc, performance, fixed return, or closeout
is claimed until those gates execute and pass.

Managed continuation details:

- Static ticket `551e1aaba19b45bb97221ee60238731b` was accepted for the six-path
  manifest (`ae3013c026621b07731582aecad5eff23a29af4771ba0f79db67d64c0e2c0886`)
  and remains queued; the local contract command emitted
  `RENDER06_CAMERA_RESOLUTION_STATIC_PASS`.
- The direct exact camera regression submission and the direct locked
  `zircon_runtime` build submission were both rejected at admission with
  `validation_ticket_external_worktree_dirty` for the unchanged foreign worktree
  `E:\\Git\\zr_vm`; neither started Cargo.  The view-family upward test and editor
  bundle gate remain pending the same clean-window prerequisite, and no dynamic
  GREEN or product artifact is claimed.

### 2026-09-18 static ticket terminal result

The managed static ticket `551e1aaba19b45bb97221ee60238731b` reached terminal
`passed` with job `f76e4821b7da48e6bfcd4d0bc63f50f1`, run
`551e1aaba19b45bb97221ee60238731b`, and exit code `0`. It emitted
`RENDER06_CAMERA_RESOLUTION_STATIC_PASS` with no coordinator blockers. This
supersedes the earlier `remains queued` wording; focused Cargo, build-only,

The static receipt is retained as source-contract evidence only. The exact
current-source manifest used by that ticket was
`ae3013c026621b07731582aecad5eff23a29af4771ba0f79db67d64c0e2c0886` and
covered the six camera/view-family paths recorded above. The focused camera and
view-family Cargo tests, the original `zircon_runtime` build-only gate, and the
current-source editor bundle remain unexecuted or blocked by the unchanged
external `E:\\Git\\zr_vm` dirty-worktree admission condition. No dynamic,
product, WPR, RenderDoc, fixed return, closeout, or WeCom success is inferred.

## 2026-09-25 current-source rolling reconciliation (camera resolution r4)

Successor Session `failure-roll-01a084c8-render06-camera-resolution-scale-r4` claimed the
failure record, Render06 plan, and the five camera/view-family source paths. No Rust source
was edited in this continuation. The current source probe completed with marker
`RENDER06_CAMERA_RESOLUTION_CURRENT_SOURCE_PASS 7 of 7`: the retired
`DEFAULT_RENDER_RESOLUTION_SCALE` symbol is absent, the primary fraction uses
`DEFAULT_DYNAMIC_RESOLUTION_SCALE`, the secondary fraction uses the canonical
`MAX_RENDER_RESOLUTION_FRACTION`, `with_scales` wiring is present, and the named camera,
temporal-history, and viewport-identity regressions remain in the current tests.

The claimed current hashes are:

| Path | SHA-256 |
|---|---|
| `zircon_runtime/src/core/framework/render/camera.rs` | `4d54cc2f750f858fdfab2d71a83bb5c0b7de463518668d209542344c7d84ff26` |
| `zircon_runtime/src/core/framework/render/camera/camera_snapshot.rs` | `5a7513b3ded3b18d70747fb0b022427c76ce77172ffef869e684feefe9573d0f` |
| `zircon_runtime/src/core/framework/render/camera/tests.rs` | `3c9444240216c7372164e1bc83a920d1564984740348967e6d855be10d9b52c8` |
| `zircon_runtime/src/core/framework/render/view_family/resolution.rs` | `ce068073486055d6f7d8841d391adb23fdeb8983c87f04c331bed049020ba61e` |
| `zircon_runtime/src/core/framework/render/view_family/tests.rs` | `8f8a58f99f5bb4c2af7094f0d29882ac8e8c81ccd9b27d7711221966dda12f4a` |

The failure document was already dirty when r4 started because of the prior static-result
receipt corrections; the five source paths and Render06 plan were clean and remain
unmodified by r4. Scoped `git diff --check` is clean. The authoritative current-source
manifest is coordinator snapshot `3842` (superseding the scope-correction intermediate
snapshot `3840` and reviewed source snapshot `3841`).

This is static handoff evidence only. Fresh managed Windows focused camera/view-family Cargo,
the original `zircon_runtime` build-only gate, current-source editor bundle, WPR/RenderDoc and
product performance evidence, independent review, canonical fixed return, closeout SHA, and
WeCom result remain pending. The external `E:\\Git\\zr_vm` dirty-worktree admission blocker
and historical jobs are not reused as current acceptance.

Independent review receipt (2026-09-25): the reviewer rechecked coordinator snapshot `3841`
after the related-code scope correction and confirmed all five manifest paths are listed,
all five source hashes match, the `RENDER06_CAMERA_RESOLUTION_CURRENT_SOURCE_PASS 7 of 7`
marker and static anchors are present, source provenance and scoped diff-check are clean, and
all focused Cargo/build/editor/WPR/RenderDoc/performance gates remain explicitly pending.
Review result is Critical `0`, Important `0`, Moderate `0`. This receipt is carried in
post-review coordinator snapshot `3842`.
