---
handoff_kind: failure
status: open
created_at: 2026-07-13
summary_slug: plugin-viewport-overlay-provider-runtime-wiring
origin_plan: docs/plans/zircon_plugins/05-navigation.md
fixing_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
origin_child_dir: docs/plans/zircon_plugins/05
fixing_child_dir: docs/plans/zircon_editor/editor/05
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_plugins/navigation/editor/src/overlay.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_navigation_editor -SkipBuild -LibTests -TestFilter registered_navigation_provider_extracts_shared_frame_and_clears_after_pie
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests -TestFilter viewport_overlay_provider_registration_routes_toggle_and_capability_lifecycle_to_packets
---

# Editor 05：插件 viewport overlay provider 宿主接线缺失

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/05-navigation.md`
- 来源执行切片：M6-T2 NavMesh viewport overlay
- 修复责任计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 交接原因：最低共享原因是 viewport 仅消费内建 scene gizmos，没有插件 provider registry/factory 与每帧 extract 合并点。

## 失败现象与复现证据

Navigation 已声明稳定 provider id，并实现 toggle controller 与 `NavigationViewportGizmoSink`。Editor host 已硬切为 `SceneModeRegistration` 与独立 overlay provider factory 的可执行安装契约，不再接受仅保存字符串的 descriptor-only tool mode；Navigation 仍需在其责任计划内注册真实 provider 并提供端到端显示/清除证据。toggle operation 同时受 Editor 03 factory 缺口影响。离线构造测试不能证明 viewport 实际显示或清除 overlay。

## 最低共享层根因

Editor viewport/tool-mode 扩展模型缺少 provider 实例注册、capability 生命周期、每帧 extract 调用与 `RenderOverlayExtract.scene_gizmos` 合并契约。

## 架构修复验收

- Editor 05 提供共享 viewport overlay provider registry/factory；tool-mode descriptor 的 provider id 必须解析到唯一实例。
- provider 按 capability、view/tool-mode enabled state 和 plugin lifecycle 启停；每帧 extract 合并到共享 gizmo channel，关闭时清除旧 extract。
- Navigation 注册真实 provider；host 测试从 menu/command toggle 到 render packet，断言 area、agent path 与 avoidance extract 出现和消失。
- 上述两个现存测试分别只覆盖 Navigation provider 的直接 extract、模拟 Weather provider 的宿主 packet。closeout 前须在可依赖 `zircon_editor` 的 Navigation editor 插件测试中补充并受管执行真实 Navigation frame → host menu/command toggle → shared extract → render packet → disable/PIE stop 清除的集成测试；不得把两个独立的通过结果合并冒充这条生产链的通过结果。

## 禁止临时方案

- 禁止在 Navigation 中直接修改 viewport state、仅保存 provider 字符串、全局开启所有 gizmos或使用 test-only sink 冒充 host wiring。
- 禁止 aliases、compatibility shims、silent fallback、duplicated truth、test-only bypasses 或 call-site exceptions。
- 禁止削弱测试或 M6 验收标准以隐藏失败。

## 修复结果与回传

Open state: `host 与 Navigation provider/frame 源码已接入，端到端受管验收仍待执行`; host 已实现 provider registration、按 capability gate toggle，并把 provider/mode gizmo 合并到 render 与 pointer 共用的 immutable interaction extract。扩展 scene mode 入口也已 hard-cut 为 executable registration，descriptor-only provider string 不再能冒充运行时 mode。Navigation 现行源码已注册可实例化 provider，并发布包含 NavMesh 与 PIE 数据的 canonical `NavigationOverlayFrame`；源码修复和早期编译阻断的原始证据见 [Plugins05 navigation overlay frame handoff](../../../zircon_plugins/05/failure-2026-07-30-navigation-overlay-frame-publication.md)。其中的结构检查不能代替产品测试：现有受管 Cargo 作业均未执行 Navigation 测试，且未取得 host toggle -> shared extract -> render packet 的通过证据。Editor05 不创建全局 overlay cache 或插件专用 bypass；依赖 Plugins05 动态验收与回传后，再完成本 handoff 的宿主端到端验收、审查和 closeout。

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-07-31 | viewport overlay provider host audit | open | Editor05 registry 已具备 duplicate validation、factory install、capability disable cleanup、toggle 及 `RenderOverlayExtract.scene_gizmos` 合并；Navigation `register_navigation_overlay` 仍只声明 descriptor/provider id，仓库没有 Navigation `register_viewport_overlay_provider` consumer。真实 frame/provider 缺口已链接至 Plugins05；端到端 managed product gate 保持 open。 |
| 2026-08-01 | executable host registry + shared pick/render extract | implementation_complete / managed_validation_pending | provider install/toggle/capability changes invalidate the shared interaction cache；provider 与 mode gizmo 在 cache rebuild 内合并，render/pointer 同消费一份 `Arc`。descriptor-only scene-mode API 已删除；Navigation pseudo mode 同步移除，真实 provider/frame 仍由 Plugins05 failure 负责。 |
| 2026-09-24 | cross-plan link and current-source acceptance audit | open / managed_validation_pending | 已修正指向 Plugins05 子目录的相对链接；`NavigationOverlayFrame`、runtime publication、editor `register_viewport_overlay_provider` 在现行源码存在。Plugins05 记录的两次 Cargo 编译在 Navigation 测试执行前分别被 Runtime 与 Editor 的外部诊断阻断；不复用静态检查或编译失败作为动态验收。独立审查指出原 `tests` 字段只有插件单包，现补列已存在的精确插件/宿主测试并显式保留尚缺真实跨宿主 packet 集成测试的 closeout 门禁；原 E 盘固定 target 仅作历史记录，后续目标目录由协调器分配。保留 2026-07-31/08-01 原始审计行，不将缺少产品验收的源码接入认定为已关闭。 |

## 2026-09-25 successor current-source seal

- Coordinator successor session: `failure-roll-01a084c8-editor05-viewport-overlay-r1`. The stale `failure-roll-01a084c8-editor05-scene-mode-test-budget-r2` session was archived through the coordinator before this independent viewport-overlay scope was claimed; no source or validation receipt was reused from that predecessor.
- Current-source marker: `EDITOR05_VIEWPORT_OVERLAY_CURRENT_SOURCE_PASS=7/7`. The probe found the host registry/factory, capability lifecycle and extract cleanup, shared render-packet merge, Navigation provider registration, host toggle/packet coverage, and Navigation extract coverage. `NAV_DESCRIPTOR_ONLY_SCAN=PASS` (no descriptor-only provider-id fallback in the selected Navigation sources).
- Existing exact static contracts were executed directly and passed: `tools/tests/test_editor05_viewport_interaction_extract_contract.py` (`Ran 5 tests ... OK`) and `tools/tests/test_plugins_05_navigation_overlay.py` (`Ran 1 test ... OK`). These are source-level checks only and are not Cargo/product acceptance.
- Formatting/working-tree evidence: `rustfmt --edition 2021 --check` over the claimed manifest returned exit 1 solely for the pre-existing import ordering in `zircon_editor/src/scene/viewport/render_packet.rs`; no foreign source was edited to make that check green. Scoped `git diff --check` returned exit 0 (only normal line-ending warnings). The checkout currently carries foreign/pre-existing dirty overlays in `zircon_editor/src/core/editor_extension.rs`, `zircon_editor/src/core/extension/store/batch.rs`, `zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers.rs`, `zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers/batch_append_tests.rs` (untracked), `zircon_editor/src/scene/viewport/render_packet.rs`, `zircon_editor/src/scene/viewport/render_packet/reused_overlay_storage_tests.rs`, and `zircon_editor/src/tests/editor_event/runtime/extensions_registration/overlay_lifecycle.rs`; this handoff made no Rust source edits and the final manifest confirms no source drift after snapshot `3860`.
- The source portion of the claimed manifest was sealed with the following current SHA-256 values before managed submission; coordinator snapshot `3856` is authoritative for the full 18-path record (including these plan documents):
  - `zircon_editor/src/core/editor_authoring_extension.rs` — `f6f3fd845bf375bbaefa0ffbffdf1078c7edb39319750e19a6a588c946375f3e`
  - `zircon_editor/src/core/editor_extension.rs` — `a6639a8ded53746ec1ac4b318fc24a0b26d1eeb04ee113dff4d4f9c3a6d62d2f`
  - `zircon_editor/src/core/editor_extension/viewport_overlay_provider.rs` — `7f7b3688578b919c019c8ae6449ab1541034224f913f1b3c0b2f30d329efff28`
  - `zircon_editor/src/core/editor_extension/viewport_overlay_provider/optimization_tests.rs` — `1f7ffebc4d14fbee16cb4731c2d2f80c0d4f43867b4cafb958f6407d48f20f76`
  - `zircon_editor/src/core/extension/store/batch.rs` — `866c2d0f40ce08f8867df4c4f0dd60ac08bb793fe6d8b648524c433ae611737f`
  - `zircon_editor/src/scene/viewport/render_packet.rs` — `a19a0e5fc2680fa70374588d67d579b47d6af59a2e4640840bf2dae22e49018d`
  - `zircon_editor/src/scene/viewport/render_packet/reused_overlay_storage_tests.rs` — `c1d02a982a3991089a6b307ef5d11061d871ca948a1cb48a05baf51513f390b8`
  - `zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers.rs` — `87609e1c3ae087ad8385a1aac46c9243b44239d80da4c7f1be8ad600a3c1a40e`
  - `zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers/batch_append_tests.rs` — `dbbbe41536504924132b757267d04dd5d01eebd892981576968e61b2538e68f9`
  - `zircon_editor/src/tests/editor_event/runtime/extensions_registration/overlay_lifecycle.rs` — `7573164758cad8c3253b25a8767eb14a065c02e648bea006306f14a4f71107fa`
  - `zircon_plugins/navigation/editor/src/overlay.rs` — `16f3c5310c5c151f6ab0b75080c3792d30554b553a345c8a97aecdf6425171fa`
  - `zircon_plugins/navigation/editor/src/viewport_overlay_provider.rs` — `4cc66f9c12f8e9585bb263af0c6404f6d3d03c4970bf09531e0b56feeeefae00`
  - `zircon_plugins/navigation/editor/src/tests/viewport_overlay_provider.rs` — `be2cf260d590bc0d22b69f10efb939e00aa7e6d988cb74d61f630f2c25794653`
  - `zircon_plugins/navigation/editor/src/runtime_mirror.rs` — `85b34ffa6bd12cdf3572a1ea639594bd2991fb3b09063b649c262d4c2290b116`
  - `zircon_plugins/navigation/editor/src/plugin/registration/mod.rs` — `2c6a6b744e2ceaf397cd530d9ffcd4d574eaa87df7c48070d022b0858aa1b492`
  - `zircon_plugins/navigation/editor/src/tests.rs` — `55967f48b4dfe610ca156444d21d6d3a7cff30bff1b2897d8a1255eff2733d34`
- Dynamic acceptance remains pending. The two frontmatter filters must execute under Windows managed Cargo with the coordinator-assigned target and include the real Navigation frame → host menu/command toggle → shared extract → render packet → disable/PIE-stop clear chain. Full Editor/Plugins product gates, independent C/I/M review, canonical `fixed-*` return, and closeout remain open; the dirty external `E:/Git/zr_vm` dependency must not be treated as passing evidence.

## 2026-09-25 static validation ticket receipt

- Coordinator accepted static source-contract ticket `12a90bf56eef4f1a9cc61ffe7ced834b` (request `3a18f55b46254ba9ae3bc01303407c0b`) against the exact 18-path manifest sealed by authoritative snapshot `3856`. Its inline Python command checks the six host/Navigation source contracts and emits `EDITOR05_VIEWPORT_OVERLAY_CURRENT_SOURCE_PASS=7/7`; `coverage.fullCoverage=false`, `staticParseOnly=true`, and `cargo/rust` are disabled.
- Admission returned `status=queued` with `executionKind=pending`; no Cargo process, managed test filter, or test body has run. The dependency report includes the open Editor07 viewport-selection, Runtime04 visibility/picking, Plugins05 Navigation frame-publication, and other upstream/downstream plan gates. This receipt records queue admission only and is not dynamic acceptance.

## 2026-09-25 independent static review

- Reviewer `review-editor03-gizmo-private` re-read authoritative snapshot `3856` and wording-only snapshot `3859`, confirming the 18-path manifest, source hashes, `EDITOR05_VIEWPORT_OVERLAY_CURRENT_SOURCE_PASS=7/7`, `NAV_DESCRIPTOR_ONLY_SCAN=PASS`, both direct static script results, the pre-existing rustfmt import-order caveat, scoped diff-check, dirty-owner provenance, queued ticket receipt, and explicit dynamic Navigation frame-to-clear gate.
- Independent review result: `Critical=0`, `Important=0`, `Moderate=0`. This approves only the static handoff record; the failure remains `open` / `managed_validation_pending` and no `fixed-*` return or closeout is authorized until the real managed Cargo/product chain passes.
