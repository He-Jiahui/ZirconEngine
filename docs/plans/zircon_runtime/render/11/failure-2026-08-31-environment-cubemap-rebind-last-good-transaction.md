---
handoff_kind: failure
status: open
created_at: 2026-08-31
summary_slug: environment-cubemap-rebind-last-good-transaction
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_runtime/render/11-environment-lighting.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_runtime/render/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_write_scene_uniform/write_scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_environment_capture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_frame/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/environment_capture_source_submission.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/environment_capture_wgpu_recorder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_runtime/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_profile_test_support.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback/tests.rs
tests:
  - managed Rust transaction tests for rebind commit, rollback, unchanged-size upload, and old-environment restoration
  - managed WGPU fault injection before resource-upload admission and graphics submission
  - current-source environment replacement PNG and RenderDoc replay under docs/tests/runtime/shader
---

# Render11: Environment cubemap rebind must preserve last-good

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行切片：Shader06 environment/IBL publication and C3/C4 atomicity review
- 修复责任计划：`docs/plans/zircon_runtime/render/11-environment-lighting.md`
- 交接原因：最低共享根因位于 Render11 的普通 source-cubemap GPU resource/bind-group publication owner；相关文件是共享工作树中的外部在途修改，Shader06 不得拆分覆盖。

## 失败现象与复现证据

`SceneEnvironmentCubemap::ensure_uploaded` 已把 upload key 分为 `committed` 与 `pending`，但在 source/PMREM/IEM 尺寸或 mip 布局变化的 `requires_rebind` 分支中，它在 graphics submission 接受之前就立即替换全部 texture/view/size 字段。`write_scene_uniform` 随后立即用这些候选 view 替换 `scene_bind_group`。

`render_scene` 只在 `submit_graphics_command_buffers_with_frame_diagnostics_and_surface` 成功后调用 `commit_pending_upload`。在此之前的 resource-upload admission、submission transaction、writeback、surface blit 或 graphics submit 失败都会直接返回。下一次 `write_scene_uniform` 仅调用 `discard_pending_upload()`，而该函数当前只清除 pending key，不恢复旧 texture/view/size 或 scene bind group。因此控制面仍记录旧 generation，采样面却可以指向从未被接受 submission 初始化的新资源。

当前取证 hash：

- `environment_cubemap.rs`: `CE22A7DEA7D2A8007CA18226C1A93E3B039753F6FF38FC6846EC921A82ADD981`
- `write_scene_uniform.rs`: `2B5FE2C872FD12C0B51BA7D1E31C588D01FE1BAD97530DCB1D39375D101F425D`
- `render_scene.rs`: `FA7D2C301D39DCC8ADEAFFD827E108DD137294F14EDB3F30ACA9888129E15902`
- `scene_renderer_environment_capture.rs`: `13E3B18C316D2454993DF5ECA6D6EA031E9015D52DD66DE2D8D90023982F4341`

## 最低共享层根因

上传身份已是两阶段事务，但对应的 physical resources 和 bind group 仍是立即发布。这是同一 environment generation 被分成 key 与 GPU object 两个生命周期 owner，不是 PMREM 算法、WGSL 采样或 cache-key 错误。

同尺寸更新不需要新代资源：其 copy 仍位于当前 graphics encoder，未被接受的 submission 不会覆写已提交 texture。缺陷限定在 `requires_rebind` 的 texture/view/size/bind-group 替换路径。

## 架构修复验收

- 以一个 generation-qualified transaction 同时拥有 upload key、source/PMREM/IEM texture 与 view、布局尺寸、scene bind group 和 staging reservation。
- `requires_rebind` 时旧代继续作为 last-good；当前 encoder/draw 可以引用候选 bind group，但不得把候选写入持久 committed 字段。
- resource-upload admission 或 graphics submit 前任一失败都丢弃候选，旧 texture/view/bind group 仍可采样，committed key 不变。
- graphics submission 接受后原子提交候选 generation，然后才允许旧代进入现有 fence/retirement owner；不得以 GPU completion wait 代替 submission admission。
- 同尺寸更新保持现有原位 copy 路径，不因事务化每次重建 texture 或 bind group。
- 单元回归覆盖 rebind commit、所有提交前 rollback、失败后返回旧 environment、下一帧仍请求新 environment 时重试，以及同尺寸路径不新建资源。
- managed WGPU fault injection 验证失败帧之后仍采样 last-good；current-source PNG/RenderDoc 证明替换成功后 source/PMREM/SH9 来自同一 generation。

## 禁止临时方案

- 不得只回滚 pending key，或只恢复 texture 而留下指向候选 view 的 bind group。
- 不得在每个早退分支手工复制回滚代码；回滚必须由单一 transaction/guard owner 覆盖未来新失败路径。
- 不得通过 `device.poll`/wait、同步 readback、恢复 CPU texel upload 或禁用 environment replacement 规避失败。
- 不得将普通 environment 资源塞入 realtime IBL 双槽或 reflection-probe array；三者保持独立资源所有权。
- 不得为了补救 rollback 而让稳定同尺寸路径每帧创建 GPU object。

## 修复结果与回传

Open state: `待修复`; no runtime, image, timing, memory, or power pass is claimed.

## 2026-09-07 current-source transaction repair

Current state: source implementation prepared; managed compilation and acceptance pending.
The original evidence and lifecycle remain open.

- Owner: `failure-roll-01a07160-render11`. Exact-path ownership transfers:
  `1f1136f6206a45e2b171276ee0a4f645` and `7b73a00ce28d414b9daec343b2b8be65`.
- `CubemapResources` now retains a candidate texture/view/layout bundle without
  replacing committed resources or clearing the committed upload key. The shared
  system black cube also requires a private rebind on the first 1x1 upload.
- `SceneEnvironmentFrame` owns cleanup across direct rendering, compiled graph
  rendering, and environment capture. Its destructor discards candidate resources
  and bindings on every pre-submission exit. The accepted graphics submission
  publishes resources, key, bindings, and any rebind SH9 buffer through one
  infallible commit method; WGPU command references retain submitted resources.
- Rebinds isolate SH9 from early resource-upload admission. Capture prepares the
  main scene binding as well as its per-face binding, preventing a successful
  capture from leaving the main scene on views from the previous generation.
- Same-size updates retain the existing texture and binding path. Added WGPU
  regressions sample source, PMREM, and SH9 after failures before upload admission
  and before graphics submission, then retry the new generation. A separate
  regression covers same-size rollback/retry and binding reuse.
- Source snapshot: `2871`, request `97beda7f3f564ef084ac5beef526639a`, 15 exact
  source/test paths. Local Rust 2021 rustfmt check and scoped `git diff --check`
  passed. These checks do not prove compilation or dynamic acceptance.

Managed command submitted once:

```text
cargo test --locked -p zircon_runtime --no-default-features --features graphics --lib graphics::scene::scene_renderer::core:: -- --nocapture --test-threads=1
```

Submission request: `failure-roll-01a07160-render11-cubemap-rebind-wgpu-20260907-r1`.
Admission was rejected with `validation_ticket_external_worktree_dirty` for
`E:\Git\zr_vm`; no validation ticket was created. The user excludes that external
repository from this cleanup. Keep this item pending without repeated admission
requests until that external prerequisite changes.

Required remaining evidence: accepted managed compilation and WGPU regressions,
current-source PNG/RenderDoc replacement evidence, independent review with zero
Critical/Important/Moderate findings, canonical `failure return`, and coordinator
closeout/WeCom receipt. No repair commit or `fixed-*` is claimed for this lifecycle.

Read-only independent source review was requested from the existing task
`优化 ZirconEngine 验证编译` (`01a07160-5337-7570-a507-ed6decf2d32b`), with snapshot
`2871` and the complete record snapshot `2872`. Review is pending and does not
replace the rejected managed Rust/WGPU admission. All 15 source hashes were
rechecked against snapshot `2871` after dispatch, with zero drift.

## 2026-09-07 independent review and SH9 admission repair

The independent review of snapshot `2871` returned Critical 0, Important 1,
Moderate 0. The reviewer verified all 15 hashes before and after the read-only
review. Its Important finding extends the original rebind diagnosis: capture
could admit a same-size resource upload, reject graphics admission, and leave
that upload queued. A later flush could update committed SH9 while the dropped
graphics encoder left the source and PMREM textures unchanged.

- Capture now settles its accepted resource-upload ticket if graphics submission
  fails. Settlement errors retain the original graphics error in
  `GraphicsError::FrameSubmissionSettlement`.
- All three entrypoints encode SH9 through one retained COPY_SRC/COPY_DST staging
  buffer. Only the graphics encoder copies SH9 into the frame's sampling buffer,
  so early upload admission cannot publish SH9 without the cubemap copies.
  Same-size updates retain the existing textures, bind group, sampling buffer,
  and warmed staging buffer.
- The same-size WGPU regression now covers accepted uploads followed by a dropped
  encoder and an independent graphics flush. A new regression fills the real
  RHI submission capacity, runs the production capture scene-batch path, verifies
  graphics backpressure and the accepted upload's `Cancelled` status, and then
  samples the unchanged last-good source/PMREM/SH9.
- Source snapshot `2875`, request `be46f2737e1b4dc19dd88f3b36bb9c78`, contains the
  same 15 source/test paths; six changed from snapshot `2871`. Attribution request
  `52a94d47fc104bfdb7f71c0c84a82fe3` binds the review repair to the same Render11
  Session. Scoped Rust 2021 rustfmt and `git diff --check` passed.

The new Rust tests have not run. The external `zr_vm` admission prerequisite is
unchanged, so no repeated Cargo request was sent. Independent re-review and all
previous managed WGPU/product evidence remain required. This lifecycle is still
open, with no repair commit, canonical fixed return, or WeCom closeout claim.

## 2026-09-07 second review fixture correction

Review of snapshot `2875` returned Critical 0, Important 0, Moderate 1. The
production SH9 and upload-ticket repair passed read-only review, but the capture
backpressure regression requested an invalid face size of 1 and would fail
before reaching submission. The fixture now uses the existing
`SOURCE_CUBEMAP_MIN_FACE_SIZE` constant (currently 64); the small old/next source
textures and production request validation are unchanged.

Source snapshot `2879`, request `a11a798fd16643bb842e0da16e662049`, contains the
same 15 paths with only the test file changed from `2875`. Rust 2021 rustfmt and
scoped diff checks passed. This correction still requires independent review and
the previously blocked managed WGPU acceptance; no dynamic result is claimed.

## 2026-09-07 third source review accepted

The same independent review task completed its read-only review of source
snapshot `2879`: Critical 0, Important 0, Moderate 0. It confirmed the corrected
capture request reaches a valid face-size boundary and retained its earlier
production review findings for the other 14 unchanged source files. The reviewer
verified all 15 source SHA256 hashes both before and after review, plus all 16
paths in full record snapshot `2880`; ownership remains the Render11 Session.

This is a zero-finding source review only. Managed compilation, WGPU regressions,
and the original product evidence remain pending behind the recorded external
admission prerequisite. No canonical fixed return, failure closeout, commit, or
WeCom notification is claimed for this lifecycle.

## 2026-09-07 Capture Work Item Import Repair

Frameworks01's production compilation evidence exposed a remaining import through
the private `graphics::runtime::render_framework` module. The capture entrypoint
now imports `EnvironmentCaptureWorkItem` from the graphics-scoped runtime export;
its implementation and the reviewed transaction behavior are unchanged.

The consumer retains Render11 ownership. Source snapshot `2905`, request
`129e556d27d24ccb979535e5b1681bd2`, freezes its current hash
`65aea9b06b4a918b2d751df4142980ca4a255c8ce38d9de94d8e0fa6227e68cd`.
Attribution `77340db76cc843edb1a5f9653f29c880` binds this revision to the same
Session. The runtime export and the second capture consumer remain Frameworks01
owned in source snapshot `2904`.

Scoped Rust 2021 rustfmt and diff checks passed. This import revision requires
independent review; all previously recorded managed WGPU and product evidence
remain pending. No compilation pass or closeout is claimed.

## 2026-09-08 Graphics Compilation And Test Support

The complete 15-file transaction source from snapshot `2879`, with the capture
import revision from `2905`, was included in the sealed graphics support input:

- Input root: `E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime-graphics-owner-support-20260908`.
- Input manifest: `b88423534de93715bc6f36a6db86d2426850966c6c6d8f8c2056c4c026f9142f`;
  10,955 files, preserved before and after execution.
- Managed production check job `3e61161de7f8480e906067c0a37784dd` passed with exit 0
  and no compiler errors. The Windows validator used `zircon_runtime`, only the
  `graphics` feature, no default features, static linkage, `--locked`, and
  `-SkipTest -CheckOnly`. This is production compilation, with zero tests run.
- The original SSAO library-test batch on the same input, managed job
  `1131b404d33449d899409926a2a7831a`, failed compilation with 180 errors and ran
  zero tests. Its durable diagnostics include the Render11 test errors below.
  `results/graphics-production-check.json` and
  `results/ssao-naga-graphics-library.json` retain the respective receipts.

The stable fixing Session resumed without replacing its prior identity. Four
existing files received test-only corrections in source snapshot `3026`, request
`b8d16f6269af44efbcc04a440b905ab9`:

| Path below `environment/` | Current SHA-256 |
|---|---|
| `environment_capture_source_submission.rs` | `11531c19ed7333121525012664444b1f62b6c985227b0c5cb87eb6cca52f32e8` |
| `environment_capture_wgpu_recorder.rs` | `f2f8af89aff2ebff9e9daad6898797bbf2d18231494a1eaf52086b545620c98c` |
| `realtime_ibl_runtime/tests.rs` | `cb231a09417b68efb0a096a0fd4819c8f049939ac6518079798d4a0e044f062b` |
| `realtime_ibl_wgpu_recorder/tests.rs` | `f602f98a20f25325025418d95f4cdc643da2a2cb717e3b2394e6260efeade5e8` |

The full path prefix is
`zircon_runtime/src/graphics/scene/scene_renderer/environment/`. Missing test
imports now name their current production types and helpers. Graph-plan test
helpers use the canonical environment module. The two manual GPU profiles borrow
the live backend and read its immutable `device_profile().adapter()` receipt;
name, backend family, vendor ID, device ID and adapter class remain in the output.
The backend family uses the neutral RHI enum's Debug representation, and the
existing output contract test was migrated with the fixture. No rendering,
scheduling, shader algorithm, profile workload or admission policy changed.

Exact-path ownership transfers from the archived Shader06 Session were
`9083153b1d6a49ad9f028816b85e77b7` and
`1e689241ff0544d1b4fe3f969ac68f45`. Three pre-edit files were byte-identical to HEAD.
The source-submission file already contained the resident-handle propagation
documented under "Last resident-publication correlation" in
`docs/plans/zircon_runtime/shader/06/2026-08-25-pbr-ibl-preoptimization-architecture-audit.md`.
Its documented hash `084fc0cc9602a16dc3554e44f1275c2b04ebd23deec7667d600879a887a64048`
matched the pre-edit file exactly. That pre-existing production slice retains its
Shader06 provenance and separate acceptance obligations; this repair adds only
the missing test import. Closeout must account for that dependency separately,
not present its existing production changes as a new Render11 fix.

Scoped Rust 2021 rustfmt and `git diff --check` passed for all four files. No
dynamic result is claimed for snapshot `3026`; other library-test compiler
blockers remain. Incremental independent review, transaction WGPU tests and the
original PNG/RenderDoc evidence are still required. Operational managed Cargo
receipts must also be bound to the fixing Session's formal validation contract
before canonical return or closeout. The lifecycle remains open.

Incremental read-only review was queued to the existing user-selected task
`优化协调器验证效率` (`01a07063-6f03-7803-a12d-13ea015ca645`) with message
`01a07ed4-ef63-7f92-9dfc-3c7f6ccf7201`. It covers source `3026` and the earlier
capture import increment `2905`, with separate source-review and acceptance
conclusions required. This is a dispatch receipt, not a review result.

### Manual Profile Support Compiler Repair

Source snapshot `3032`, request `58546ccea67f4591ab1ac7e018bb37be`, corrects the
`Option<&u8>::filter` closure signature in
`zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_profile_test_support.rs`.
The closure keeps the existing ASCII-drive-letter check, absolute-path rules,
C-drive rejection and namespace/UNC rejection. No output policy changed.

The pre-edit file matched HEAD and the archived owner's attribution at
`ef668fcdb11766b93eb3aeb93318c27f76c19d550d16a0dc4d17a3f428580b38`.
Audited transfer `cb7069a0a4284e1092ad917367756178` and attribution
`34f1a92d4c304aa9aae48a5b5470f227` retain the stable fixing Session. Current hash:
`cc564c9cc9ceb23749743ba9b7a2a700f4bd091e802d77630f235002d2d6c51a`.
Rustfmt and scoped whitespace checks passed. This one-line increment still needs
independent review and actual managed execution of the existing path regression.

### Managed Test Compilation Recheck

The exact five Render11 support files from `3026` and `3032`, together with the
separately owned Frameworks01 test corrections, were compiled in managed Windows
job `b0612b75cf6b4a4dada9e3d89b53dbc6`. Input:
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime-graphics-test-support-3033-20260908`,
10,955 files, manifest
`1adec2abe9c4763f4b8e02163a57b8d752481555cf35fc893e43383772015761`.
The same graphics-only, no-default-features, static, locked library-test command
now reports 126 compiler errors instead of 180, with no diagnostic in any of the
13 edited files. Cargo still returned 101 and zero tests ran, so transaction
behavior and the profile path regression are not accepted yet. The complete
input manifest was verified after the command. The durable receipt and grouped
remaining diagnostics are in that input's `results` directory, also linked from
the Frameworks01 failure. Its queue/sync/check times were 6.156/22.364/210.044
seconds. Further independent test-owner work remains before another full batch.

Review supplement `01a07eed-55b6-77c2-8c6c-d2a9852a4a0e` was queued once to the
same user-selected task for source `3032` and the new managed evidence; earlier
source `3026`/`2905` review requests remain pending separately.

Source `3037`, request `f9b066e67aea4c6e8c1d0acc2182775b`, fixes six compiler
diagnostics in `environment/ibl_bake_runtime_writeback/tests.rs`. The SH9, PMREM
and IEM fixtures call resource materialization on `resources`, then return the
declared `(resources, graph)` pair after dispatch. Their previous code applied
the tuple to the method receiver and returned only `resources`. The graph,
dispatches, readback assertions and resource-pool contract are unchanged.
The pre-edit file matched HEAD at
`419465bd86790e7e8a1e67fd75c00f39abacfba9b319883ba9eecba04fef6493`.
Archived-owner transfer `269d38c73c6f468f9ffa1ef4550e7c28` and attribution
`db0fbfdd9d69437f944c150fba0ea752` bind the test consumer repair to this fixing
Session. Current hash `d2054e620df58780099a4916160a92743cbfe014001149655a6542e5f683f130`;
Rust 2021 scoped rustfmt and whitespace checks passed. Source `3037` still needs
managed compilation, dynamic execution and incremental independent review.

### Source 3037 Compilation Result

Managed Windows job `b18ea987bdfc49cd8c5c0799dbc9a46d` compiled source `3037`
with Frameworks01 `3036`, preserving each fixing owner. The 10,955-file input
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime-graphics-test-support-3037-20260908`
has manifest `c0d346a32524a0f477d3e98178f52834f25db67babf834d5466c83db27231813`.
The existing graphics-only static locked library-test command now reports 115
compiler errors, down from 126; all six writeback-fixture errors and five
Frameworks01 errors disappeared. Cargo remains 101 / wrapper 1, with zero tests.
Queue/sync/check times were 6.636/28.213/283.327 seconds. All 354 dependencies
and the complete post-run manifest were verified. Durable receipts and grouped
remaining diagnostics are in that input's `results` directory. The writeback
behavior, last-good transaction gates, PNG/RenderDoc evidence and independent
source review are still pending; no canonical return or accepted closeout is
claimed from this compiler result.

## Independent Profile Review And Source 3051

The existing user-selected task reviewed sources `2905`, `3026`, `3032`,
`3037` with Critical 0, Important 0, Moderate 1. All source hashes matched
before and after review, with no reviewer-owned conflict. Report:
`.codex/tmp/graphics-support-review-resume-20260908-result.txt`.
The profile migration had changed actual values from `dx12` / `DiscreteGpu`
to RHI Debug values `Dx12` / `Discrete`, despite preserving the field names.
The earlier output-policy statement therefore did not establish preservation
of the profile's text values.

Source `3051`, request `b7b94246873a42e992432b3fef257a3b`, maps the canonical
RHI profile values to the existing manual-profile labels in the shared test
support module. Both callers use that presentation mapping. The new regression
compares every backend and adapter-class variant against the native text
contract and asserts the literal DirectX/discrete output. The 256-iteration
workloads, profile storage policy and canonical device-profile source remain.
Frozen hashes under `environment/`:

- `realtime_ibl_profile_test_support.rs`:
  `dbcfa1c95d841a0db400c6af12efc1811e73abcfd3fbf4f1334d83c13847d81b`.
- `realtime_ibl_runtime/tests.rs`:
  `82092b0ee4988390f684e0d32dae2a0aa43f750e9ad751bdba7bef561ac7e0e8`.
- `realtime_ibl_wgpu_recorder/tests.rs`:
  `45689c6d975fd1a9ef8d39b926bf633dda194e964b32c8155e541b8ddc4abc30`.

The edit used the preceding live lease. After expiry, lease
`5a6c217af3e04ce088fdf50986eb887c` and attribution
`bd854cf73c9f4063ab6bdac9f7825fbe` refreshed the same frozen bytes.
Rust 2021 scoped rustfmt and whitespace checks pass. Managed Windows job
`3a8034c59d4546a39836f6bde23c5ce6`, input
`runtime-graphics-test-support-3051-20260908` under the approved benchmark root,
digest `bc81d18f821d3f151fc4919286259e9b1e02e953d5a220aca06d81cd4451ab9b`,
compiled all three paths without diagnostics. The full library-test compile
still reported 95 errors, zero tests, Cargo 101 / wrapper 1. Its exact receipt
and parent/owner provenance are linked from the Frameworks01 failure. A fresh
independent review, actual profile/transaction tests and original product
acceptance remain required; this lifecycle remains open.

### Source 3051 Review Result

The same user-selected task completed the incremental review with Critical 0,
Important 0, Moderate 0. Report:
`.codex/tmp/graphics-3052-review-resume-20260908-result.txt`.
All three source hashes and record `3054` matched before and after review;
no reviewer attribution or lease conflict was found. The review verified every
backend/adapter presentation mapping against the native labels and retained
the 256 workloads, 21-pass/ticket assertions and cold/warm counters. This
resolves the preceding fresh-review requirement for source `3051` only.
Actual profile/transaction execution, original product acceptance and formal
fixing-Session evidence binding remain pending. No canonical return, commit or
new WeCom notification is claimed.

### 2026-09-19 successor static source-contract receipt

Fixing Session `failure-roll-01a084c8-render11-environment-cubemap-r1` sealed
the current lower-layer Render11 source contract in ticket
`1aaeb2c17f3c43aaa12bdb81fdc3f8f7`. Coordinator copy job
`6e47fd0a77f644c8afd7d7f258c66729` and run
`1aaeb2c17f3c43aaa12bdb81fdc3f8f7` exited 0 with
`RENDER11_ENVIRONMENT_CUBEMAP_SOURCE_CONTRACT_PARTIAL_PASS`, checking 21
owned paths. The Shader06-owned
`realtime_ibl_wgpu_recorder/tests.rs` path was explicitly excluded and remains
an audited-transfer dependency; no ownership or acceptance is inferred for it.
The ticket's pre-receipt source manifest was
`9bebec859637113e6170e5b9c1fc2126001fa93d70d6de2b8f2ca4c17480e2ee`.

This paragraph intentionally changes the failure-record bytes after that
static snapshot. It is a durable receipt only: managed Windows graphics Cargo
and WGPU fault-injection/product evidence, external `E:/Git/zr_vm` admission,
independent Critical/Important/Moderate zero-finding review, canonical
`failure return`, and coordinator closeout remain pending. A later dynamic
ticket must reseal the current document and retain this receipt.
