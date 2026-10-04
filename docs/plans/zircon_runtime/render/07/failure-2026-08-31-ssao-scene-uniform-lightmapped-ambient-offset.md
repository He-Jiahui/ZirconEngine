---
handoff_kind: failure
status: open
created_at: 2026-08-31
summary_slug: ssao-scene-uniform-lightmapped-ambient-offset
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_runtime/render/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/ssao_spatial_denoise.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/ssao_bilateral_upsample.wgsl
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/tests/runtime_ssao_scene_uniform_contract.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests/core_contracts/ao_input_qualification.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/ambient_occlusion_tests.rs
tests:
  - managed Naga validation for ssao_spatial_denoise.wgsl and ssao_bilateral_upsample.wgsl
  - managed graphics integration target runtime_ssao_scene_uniform_contract
  - managed WGPU half-resolution GTAO spatial-denoise and bilateral-upsample product lane
---

# Render07: SSAO SceneUniform lightmapped ambient offset drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行切片：P1-18 authored ambient/lightmapped ambient SceneUniform ABI audit
- 修复责任计划：`docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md`
- 交接原因：最低共享根因位于 Render07 当前在途的 half-resolution GTAO spatial-denoise / bilateral-upsample shader owner；两份 shader 和其 descriptor 均处于 Render07 的未提交变更集，Shader06 不得拆开覆盖。

## 失败现象与复现证据

CPU `SceneUniform` 的当前合同为 `ambient_color @ 192`、`lightmapped_ambient_color @ 208`、`previous_view_proj_unjittered @ 224`，总大小 496 bytes。canonical `zr_scene_runtime.wgsl` 与当前 `ssao.wgsl` 已保持该字段顺序。

Render07 新增的两份生产 shader 仍在 `ambient_color` 后直接声明 `previous_view_proj_unjittered`：

- `ssao_spatial_denoise.wgsl` SHA-256 `467DD842AAB28CD531E38ED28193A47AEE99C60559CE53E097F6A4A109E515D7`
- `ssao_bilateral_upsample.wgsl` SHA-256 `C772B43A110322614E97385B32C07CBEF9885AB1D34D715AD7A6C9308F7412DE`

两份 shader 均由 `screen_space_ambient_occlusion.rs` 的 builtin `include_str!` 直接装配，并读取 `inverse_view_proj`、`camera_world_position`、`camera_view_direction`。`inverse_view_proj` 位于新增字段之前，仍正确；`previous_view_proj_unjittered` 及之后字段的 WGSL 偏移比 CPU 上传合同少 16 bytes，因此 camera position/direction 会读取错误槽位。当前 source review 不能把 half-resolution AO 标为 ABI-correct 或动态可验收。

## 最低共享层根因

P1-18 扩展共享 `SceneUniform` 时，Render07 同期新增的两个未跟踪 shader 没有消费 canonical SceneUniform 字段序列，也没有通过结构守卫覆盖所有“声明超过 `ambient_color` 的 SceneUniform 镜像”。这是共享 scene ABI 漂移，不是 AO 采样算法或 bind-group schema 问题。

## 架构修复验收

- 两份 Render07 shader 都必须在 `ambient_color` 后声明 `lightmapped_ambient_color: vec4<f32>`，字段顺序与 `zr_scene_runtime.wgsl` 一致；不得改变 AO 算法、资源 binding 或 dispatch。
- 增加一个覆盖 production WGSL 镜像的 SceneUniform prefix/order 守卫：凡镜像声明并读取 `previous_view_proj_unjittered` 或其后字段，必须包含 `lightmapped_ambient_color` 且顺序一致；只读取首字段 `view_proj` 的合法前缀镜像不应被迫复制完整 ABI。
- 两份 shader 的 managed Naga validation 通过，half-resolution GTAO spatial-denoise / bilateral-upsample WGPU 产品 lane 无 validation error，camera motion 与 orthographic/perspective reconstruction 结果保持正确。
- Shader06 P1-18 的广域 SceneUniform ABI gate 重新执行后才能从 partial 更新为 accepted。

## 禁止临时方案

- 不得在 CPU 上传端恢复旧 480-byte layout，或为 SSAO 单独生成旧布局 buffer。
- 不得用匿名 padding、手工 byte offset、调用点重写或 shader fallback 掩盖字段漂移。
- 不得把两份 shader 从 descriptor 拆除来规避验证；修复必须保留 Render07 已声明的 half-resolution AO 拓扑。
- 不得削弱 Naga/WGPU、camera reconstruction 或 SceneUniform 顺序验收。

## 修复结果与回传

Open state: existing source repair retained; managed acceptance pending.

## 2026-09-08 current-source acceptance takeover

Stable fixing Session `failure-roll-01a07160-render07` is registered to the
original Render07 plan and remains `resolving_failure`. Registration request
`e641298c66814297926e55d8dd80fe95` was accepted before the client timed out;
its exact request result later confirmed completion. No duplicate registration
or new lifecycle was created.

Both production shaders already contain `lightmapped_ambient_color` immediately
after `ambient_color`. Their current hashes are respectively
`289b7eb4a7ae7fefb3cd6a136933484ee3dd9fb9d57acc83d605af50650f12a0`
and `b030caeb0817c7a2a065ac4e2f17c75e1106bf77deb598dd7b8fd13683e99713`.
The existing guard `tools/tests/test_render07_scene_uniform_shader_mirrors.py`,
hash `b85276fb575306d3f7f855dffa1dbec73e0173504dfad2e430f1b3ae148cfe35`,
checks canonical field prefixes across production graphics WGSL and confirms
that the SSAO descriptor still includes both shaders. All three files match
HEAD after explicit CRLF normalization; no new shader or guard edit was made.

Read-only source snapshot 3013 freezes the exact bytes. Their cancelled owner
`root-render07-ssao-scene-uniform-offset-closeout-r2-20260831` was transferred
through preview fingerprint
`5dccd35c4458d65ee6a18155e33498ceff15c92b4d1d7d675da6d7ccd90e7626`
and apply request `704436e83b11458897781767c7bf8194`. Original source changes
and prior ticket ownership remain intact.

The command `python -B -m unittest
tools.tests.test_render07_scene_uniform_shader_mirrors -v` was submitted for
Windows Python 3.14.4 with dependency roots `tools/tests` and
`zircon_runtime/src/graphics`. Initial requests r1 and r2 were rejected before
ticket creation for missing dependency roots and stale ownership respectively.
After those prerequisites were corrected, logical request
`failure-roll-20260908-render07-ssao-prefix-3013-r3`, transport request
`27f763819b3948f88d7dfe87529d950d`, created ticket
`532114f4dcf345fd99d59596c9fcaa0c` in `queued` state with no blockers.
Source manifest hash:
`fd920c9db1492e53ca9b3ffa1c2fae4d16efd2d0bc9f2a2e70094041a2afbde4`.
The initial queue receipt was not a pass and no repeated submission was made.

At the next result-processing boundary, this exact ticket was terminal `passed`.
Managed copy job `3df4554d763148eb8241c8fe951e1338`, run
`532114f4dcf345fd99d59596c9fcaa0c`, executed both named tests: canonical
production SceneUniform prefixes and descriptor inclusion. The immutable
terminal event 9644 retains stderr `Ran 2 tests in 1.322s`, `OK`, exit 0,
completed at `2026-09-08T01:16:23.467516+00:00`. Event 9645 confirms cleanup
completed. The ticket's source manifest hash and command still match the
submission above. This accepts only the ABI structure guard.

The existing Rust gate
`ssao_evaluate_spatial_and_upsample_shaders_are_valid_wgsl` parses and validates
all three SSAO shaders with Naga. Managed job
`1131b404d33449d899409926a2a7831a` requested that exact library test against
input `b88423534de93715bc6f36a6db86d2426850966c6c6d8f8c2056c4c026f9142f`.
The production graphics library passed separately, but library-test compilation
reported 180 errors and ran zero tests. Those compiler diagnostics and ownership
are retained in the Frameworks01 failure. No original test was removed or ignored.

An independent integration target now validates the actual production shader
includes using Naga parsing and all validation flags/capabilities. It compares
SceneUniform field names, types, offsets and prefix span with the canonical
Naga structure, including the CPU ABI anchors 192/208/224 and full size 496.
Source snapshot 3021, request `520fc70f03fe494194fb21051ee8393c`, freezes the
new `runtime_ssao_scene_uniform_contract.rs`, hash
`7f533262479684daf1a1b4d41ccc1743bf220d5ef4ea974131ca0a34df731b06`,
and the two unchanged repaired shaders.

The new file has a live-lease attribution under the same Render07 Session:
lease `a1657c09dc7648be8ffe879d18d3b646`, attribution
`79cf13b4523f4a5589d7e5a69e2c1942`. Registration request
`cdacb04861724720afb166999e63fa42` attempted to preserve all prior declared
paths and append this test. It returned `session_write_scope_immutable`;
the original Session declaration and pending ticket identities remain intact.
An audited declaration extension remains a closeout prerequisite; no replacement
Session or direct coordinator database change was used.

Input `render07-ssao-naga-3021-20260908` under the approved benchmark root seals
10,956 files, digest
`177ba8fac88c1060ae387422787e7459939b127c3452df375473068336046d51`.
Its parent is the passing graphics production input; only the new integration
test differs. Managed Windows job `1281688749874d0d83f95481d1376052` ran
`tools/dev/dev-fast-build.ps1 -Action test -Package zircon_runtime
-FeatureOverride graphics -TestTarget runtime_ssao_scene_uniform_contract
-TestFilter runtime_ssao_scene_uniform_prefix_matches_naga_layout
-LinkMode static -Linker auto`, with this exact source path and digest.
The embedded validator preserved Cargo `--locked` and reported exit 0 with
1 test passed, covering all three production shaders. Queue 7.177 seconds,
sync 20.726 seconds, check 88.133 seconds, compile/link 257.543 seconds and
test stage 3.974 seconds. The complete source manifest was reverified after
execution; logs and durable receipt are in `results/ssao-naga-abi.log` and
`results/ssao-naga-abi.json` under that input.

This accepts Naga syntax, validation and ABI layout for the recorded input.
The required WGPU half-resolution/camera-reconstruction product path and
Shader06 broad gate remain pending. Operational `validate-matrix` receipts
are not relabeled as fixing-Session Cargo runs. Incremental independent review,
coordinator closeout evidence binding, canonical return, commit and
SHA-deduplicated WeCom notification remain pending.

The incremental review of source `3021` and record `3025` was queued once to
`优化协调器验证效率` (`01a07063-6f03-7803-a12d-13ea015ca645`) in message
`01a07ec5-9d07-7010-8feb-f2c2ce5742f7`. The request preserves the scope audit,
WGPU and formal closeout evidence gaps. No review result has been accepted yet.

## Source 3049 AO Acceptance Fixture Migration

Source `3049`, request `aa0409475f1a403bb166704704a14b3b`, repairs four
compiler diagnostics in the existing AO acceptance fixtures. The qualification
tests obtain texture descriptors from the existing typed resource-lifetime
helper and retain all GBuffer format/sample-count and half-resolution extent
assertions. The execution-record fixture identifies its actual transient
texture resource with `TransientTexture`. No production shader or AO contract
is changed by this increment.

Both files matched HEAD before editing; audited transfer
`a788380ede8a4c1ebef2634e39e62f8e` extends the same Render07 Session scope.
Frozen hashes: `ao_input_qualification.rs` =
`caea6a1b01b4d6f6370faf24774a7f5b0d2db91eee5a81b046fec27133f2df3c`;
`ambient_occlusion_tests.rs` =
`e7288b024bc9b5e244bbecdc8ec581bc7897d141a720f157d7a31e03bf16cbba`.
Scoped rustfmt and whitespace checks pass. Managed Windows job
`3a8034c59d4546a39836f6bde23c5ce6` removed all four diagnostics in the sealed
input `runtime-graphics-test-support-3051-20260908`, digest
`bc81d18f821d3f151fc4919286259e9b1e02e953d5a220aca06d81cd4451ab9b`.
The remaining library-test compile has 95 errors and executed zero tests.
The original Naga pass remains limited to its earlier exact input; this new
increment still needs dynamic acceptance and independent review. The existing
integration-file scope audit and formal closeout binding remain unresolved.

### Source 3049 Review Result

The user-selected task completed the `3049` incremental review with Critical 0,
Important 0, Moderate 0. Report:
`.codex/tmp/graphics-3052-review-resume-20260908-result.txt`.
Both source hashes and record `3055` matched before and after review, without
a reviewer attribution or lease conflict. The typed texture descriptors and
`TransientTexture` identity preserve format, sample-count, half-resolution,
dispatch and upsample checks. This resolves the fresh-review requirement for
`3049` only. The earlier `3021` Naga integration scope was not examined and
still needs its own review. The WGPU/camera product gate, integration-file
scope audit and formal closeout binding remain open.

### Source 3021 Review Result

The existing task `优化协调器验证效率` has now reviewed source `3021`, with
Critical 0, Important 0, Moderate 0. Report:
`.codex/tmp/text-framework-3080-review-20260908-result.txt`.
The two shader hashes and integration-test hash matched their snapshots,
ObjectStore and attribution before and after review. The Naga test consumes
the three actual shader includes, validates them, and checks field names,
types, order, prefix span, offsets 192/208/224 and the canonical 496-byte
layout. This accepts the source review previously pending for `3021`;
the recorded one-test Naga and two-test ABI receipts retain their original
scope. WGPU half-resolution/camera behavior, the Session declaration audit,
Shader06 broad acceptance and formal closeout binding remain pending.
