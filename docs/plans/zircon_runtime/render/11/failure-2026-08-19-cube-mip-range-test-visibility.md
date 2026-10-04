---
handoff_kind: failure
status: open
created_at: 2026-08-19
summary_slug: cube-mip-range-test-visibility
origin_plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
fixing_plan: docs/plans/zircon_runtime/render/11-environment-lighting.md
origin_child_dir: docs/plans/zircon_runtime/text/03
fixing_child_dir: docs/plans/zircon_runtime/render/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_time_slice.rs
  - zircon_runtime/tests/runtime_environment_wgpu_cubemap_sampling_contract.rs
  - zircon_runtime/src/core/framework/render/environment/skybox/procedural_sky/resolved_sun.rs
tests:
  - .\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter text_oversized_run_keeps_one_logical_shaped_line -VerboseOutput
---

# Render11: CubeMipRange test visibility

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md`
- 来源执行切片：Text03 长文本逻辑行回归门。
- 修复责任计划：`docs/plans/zircon_runtime/render/11-environment-lighting.md`
- 交接原因：失败发生在实时 IBL 图计划的测试编译边界；Text03 仅触发完整 runtime 的 `cfg(test)` 门，未定义或使用该符号。

## 失败现象与复现证据

2026-08-19 的受管 Windows test lane 完成了 `cargo build -p zircon_runtime --locked`，随后在库测试编译前失败：

```text
error[E0422]: cannot find struct, variant or union type `CubeMipRange` in this scope
  --> zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan/tests.rs:135:23
```

受影响的 Text03 命令为：

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter text_oversized_run_keeps_one_logical_shaped_line -VerboseOutput
```

构建阶段通过；`cfg(test)` 在运行任何 Text03 测试前以 exit 101 终止。

## 最低共享层根因

`realtime_ibl_graph_plan/tests.rs` 构造 `CubeMipRange`，但当前测试作用域不再能解析该类型；同一环境模块可见的近似类型为 `CubeFaceRange`。渲染时间切片的类型可见性或测试导入在硬切换后未与图计划测试同步。

## 架构修复验收

- Render11 使用时间切片的规范 `CubeMipRange` 路径或正确的模块级可见性，保持 mip 范围与 face 范围为不同概念。
- Render11 的相关图计划测试可编译并通过。
- 原始 Text03 命令完成并运行 `text_oversized_run_keeps_one_logical_shaped_line`。
- Text03 随后重跑长文本 profiling 和真实 WGPU 产品帧缓冲导出。

## 禁止临时方案

- 不要以 `CubeFaceRange` 替换 mip 范围来消除编译错误。
- 不要加入别名、兼容重导出、测试专用旁路或放宽 Text03 验收。
- 不要跳过完整库测试或将构建通过视为 Text03 测试通过。

## 修复结果与回传

Current-source implementation state: `CubeMipRange` is imported from
`realtime_ibl_time_slice`, where it is visible as `pub(in crate::graphics)`; both exact Rust
owners are clean and pass Rust 1.94.1 formatting. No managed pass is claimed.

The fresh immutable validation copy `84d09eb71f1942b0835e4bbb8abdfdfb` failed during
`closure_planning`, before Cargo ran. Durable evidence identifies a separate stale compile-time
resource: `zircon_runtime/tests/runtime_environment_wgpu_cubemap_sampling_contract.rs` includes the
deleted flat path `zircon_runtime/src/core/framework/render/environment/skybox.rs`, while the
current owner is the folder-backed `environment/skybox` module. The including test file is a dirty
current-source blob attributed to Session `01a019a5-b15f-7461-a1b0-ce4b6aa8e710`, with no live
lease; Render11 did not absorb or rewrite it. This failure remains open until that owner repairs the
stale include and the original Text03 managed command executes on a fresh immutable copy.

## 2026-09-06 current-source path repair

Render11 reclaimed the archived test blob through coordinator ownership transfer
`16d5343524de4920be7eb69b7f5fe573`, then changed the stale compile-time include and its
consumer. `RESOLVED_PROCEDURAL_SUN_SOURCE` reads the folder-backed
`core/framework/render/environment/skybox/procedural_sky/resolved_sun.rs`, and the CPU rotation
assertion uses the existing function-body extractor instead of the deleted `ProceduralSkyParams`
implementation boundary. The horizon regression already present in this test file remains intact.
Current test SHA-256 is
`e99dc5496ceff66b016b337194b3cc976e31385bd55128f25cae58b3823dc29c`; exact source snapshot `2856`
was frozen by request `817fab657db84791bd4c608ba39dab86`.

Rust 2021 formatting passed locally and was submitted as managed ticket
`a8261eecfc2247d0939475b5695ad05a` (request
`failure-roll-01a07160-render11-sun-source-format-20260906-r1`, queued). The actual focused
compile-and-test command was submitted as ticket `8e3f1638cd1f412f90c73ab5368f5175` (request
`failure-roll-01a07160-render11-sun-rotation-contract-20260906-r1`, queued). Its declared test is
`runtime_environment_cpu_sun_rotation_is_inverse_of_shader_lookup_rotation`. Both receipts have
zero admission blockers; neither receipt is an execution pass. The source-contract ticket pins its
dependencies to the recorded base HEAD; it does not validate the separate dirty procedural-sky
radiance changes or replace the full Cargo gates.

The original Render11 graph test, Text03 long-text command, profiling, and product-frame acceptance
remain pending. No fixed artifact, commit, or WeCom notification is claimed.

## 2026-09-07 managed compile evidence and scoped integration

Both queued tickets above have completed on their sealed inputs. Format ticket
`a8261eecfc2247d0939475b5695ad05a` passed in job `8b906245294543debf20008109777395`.
Compile-and-test ticket `8e3f1638cd1f412f90c73ab5368f5175` passed in job
`6f1ee72f33234212a9f200003d9e125a`, with exit 0 and `1 passed; 0 failed; 24 filtered out`.
The test binary was built under the coordinator-assigned `CARGO_TARGET_DIR`; no external
`zr_vm` source was used. These results supersede only the queued status above.

The stale include prevented dependent immutable copies from materializing, so the validated
single-file snapshot was submitted through scoped candidate `edf59d7f08f54fada512a5856738c30d`.
Coordinator finalize request `392fc564632c468e872911fc1ce9c3a1` produced commit
`585b031793088c28feef357488211f290927ec50`, titled `fix(failure): cube-mip-range-test-visibility`,
at `2026-09-07T00:26:21+08:00` (`1 file changed, 22 insertions, 7 deletions`).
Candidate state remains `integrated_validation_pending`; this does not close the lifecycle.

Coordinator WeCom attempt `8a55f191ccfe457c8e34ede806beecd5` for that SHA succeeded with exit 0
and provider errcode 0. The separate candidate notification also succeeded. Neither was retried.
The full Render11 graph and Text03 upward gates, independent closeout review, and canonical
`failure return` remain outstanding.

## 2026-09-25 current-source rolling reconciliation (CubeMipRange r5)

Successor Session `failure-roll-01a084c8-render11-cube-mip-r5` claimed the failure record,
Render11 plan, and the four exact source consumers. No Rust source was edited in this
continuation. The current source probe completed with marker
`RENDER11_CUBE_MIP_CURRENT_SOURCE_PASS 5 of 5`: the graph-plan test resolves the canonical
`CubeMipRange` from `realtime_ibl_time_slice`, the range type remains distinct from face
ranges, the cubemap contract no longer includes the deleted flat `environment/skybox.rs`
path, and the folder-backed `resolved_sun.rs` owner plus
`RESOLVED_PROCEDURAL_SUN_SOURCE` marker are present.

The claimed current hashes are:

| Path | SHA-256 |
|---|---|
| `zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan/tests.rs` | `f61c5a97cd66479093edc5af74b374908a4c0b9a32e289bebdf4c92f467a99b3` |
| `zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_time_slice.rs` | `ba3abecd969310ebd09f1efa36e43033af4f2d6b9be36d2077c1576dd7e78791` |
| `zircon_runtime/tests/runtime_environment_wgpu_cubemap_sampling_contract.rs` | `e99dc5496ceff66b016b337194b3cc976e31385bd55128f25cae58b3823dc29c` |
| `zircon_runtime/src/core/framework/render/environment/skybox/procedural_sky/resolved_sun.rs` | `1957f30130ffe8fddfe141b07b5cca693a59b3abd22fbbe6afb6076ad9d3de4c` |

Only this failure document was already dirty when the Session started, carrying the prior
metadata/test-command and historical receipt corrections; the four source paths were clean
and remain unmodified by r5. Scoped `git diff --check` is clean. The authoritative
current-source manifest is coordinator snapshot `3837`.

This is static handoff evidence only. A fresh managed Windows Render11 graph test, the
original Text03 exact long-text command, full `zircon_runtime` Cargo, profiling and WGPU
product-frame evidence, independent review, canonical fixed return, closeout SHA, and WeCom
result remain pending. Historical jobs, the integrated commit, and supervisor-only outcomes
are not reused as current acceptance.

Independent review receipt (2026-09-25): the reviewer rechecked coordinator snapshot `3837`
and confirmed all four source hashes, the `RENDER11_CUBE_MIP_CURRENT_SOURCE_PASS 5 of 5`
marker, canonical `CubeMipRange` import, deleted flat skybox-path absence, folder-backed
`resolved_sun.rs` marker, clean source provenance, and scoped `git diff --check`. Review result
is Critical `0`, Important `0`, Moderate `0`. This receipt is carried in post-review
coordinator snapshot `3838`; snapshot `3837` remains the reviewed source state.
