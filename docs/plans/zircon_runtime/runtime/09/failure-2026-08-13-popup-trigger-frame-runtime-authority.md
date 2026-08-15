---
handoff_kind: failure
status: open
created_at: 2026-08-13
summary_slug: popup-trigger-frame-runtime-authority
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/zircon_runtime/runtime/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/widget.rs
  - zircon_runtime/src/ui/surface/control_index.rs
  - zircon_runtime/src/ui/surface/popup_stack.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/render/popup_position.rs
  - zircon_runtime/src/ui/surface/render/popup_menu.rs
tests:
  - cargo test -p zircon_runtime --locked --lib popup_trigger_identity
  - cargo test -p zircon_runtime --locked --lib popup_stack
  - cargo test -p zircon_runtime --locked --lib render_popup_menu
  - cargo test -p zircon_runtime --locked --lib focus_navigation
---

# Runtime09 failure handoff: popup trigger frame runtime authority

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M4 命令、popup、focus 与触发一致性
- 来源执行 Session：`editor-ui12-m4-popup-focus-v1-20260813`
- 修复责任计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 交接原因：Runtime09 拥有 generic popup stack、arranged-tree render extraction、control index 与 focus/input routing；UI12 只能声明 open/close、placement policy 和 trigger identity，不能继续在 Editor callback 中计算绝对 popup 几何。

## 失败现象与复现证据

UI12 计划 4.4 要求 generic popup 由 Runtime 依据实时 trigger frame 定位，并由 stack 记录触发控件以恢复 focus。当前源码只有数值 anchor 路径，没有可供 workbench menu 使用的 trigger identity 到实时 arranged frame 的合同：

1. `UiTemplateNodeMetadata` 只有通用 `attributes`、`control_id` 与 widget contract；全仓索引搜索没有 `popup_trigger`、`trigger_control`、`trigger_id`、`anchor_owner` 或 `popup_owner` 的生产实现。
2. `seed_popup_stack_from_tree_metadata` 与 `sync_popup_stack_for_node` 都把 popup 节点自身作为 `open_popup(..., owner, ...)` 的 owner；这不能表达触发控件，也不能为关闭后的 focus restore 提供真实 target。
3. `popup_menu_render_commands` 只接收 popup 节点的 metadata、frame 与 clip；`popup_anchor_frame` 只读取 popup 自身的 `popup_anchor_x/y/width/height`，缺失时回退 popup 自身 frame。
4. render extraction 同时持有完整 `UiTree`、`UiArrangedTree` 和 node index，但调用 popup render 时没有解析或传入 trigger frame。现有 `UiSurfaceControlIndex::node_id` 已能按唯一 `control_id` 找 node，可作为解析入口，但尚未连接 arranged frame、stack owner 或依赖失效。
5. workbench window menu 当前由 Editor `window_menu_state.rs` 计算 trigger frame、popup width/height、root clamp 和绝对 menu frame，并写回 `position` 与 `popup_anchor_*`。若 UI12 直接删除这些写入，Runtime 会回退 popup 自身的 collapsed/placeholder frame；若只保留数值 anchor，则仍存在 Editor 第二套 generic 几何 authority。

以上是 current-source 静态架构失败，不以未运行的 Cargo 测试冒充动态 red。M4 在该合同回传前不得删除 Editor 旧路径后宣称 trigger-frame hard cut 完成。

## 最低共享层根因

Runtime 已有 flip/clamp 算法、popup stack、control index 和 arranged frame，但四者没有通过一个可序列化的 popup trigger contract 连接起来。结果是 popup 节点同时充当 stack owner、布局 fallback 与数值 anchor 容器，Editor callback 被迫拥有实时触发几何。

## 架构修复验收

- runtime-interface 提供最小、可序列化且 editor-agnostic 的 placement/trigger/policy contract。控件触发型 popup 至少能声明 trigger `control_id`；不得承载 Editor command、窗口状态或 Editor 专用枚举。
- Runtime 在当前 arranged tree 中解析 trigger identity，并把触发控件的实时 arranged frame 传给 popup placement。控件移动、缩放、DPI/layout tier 变化后，已打开 popup 必须随下一次有效 extraction 更新，不能依赖 open-time 数值快照。
- popup stack 的 owner/restore target 对控件触发型 popup 指向解析后的 trigger node，而不是 popup node；nested popup 关闭、outside click、Escape 和 focus trap/restore 保持 Runtime 单一所有权。
- 缺失、重复、disabled、collapsed 或不可见 trigger 必须采用明确且可测试的拒绝/关闭策略，不能静默回退到 popup 自身 frame 后产生错误位置。
- render full extraction 与 partial/incremental extraction 保持一致：trigger frame 或 trigger identity 变化必须使依赖 popup 的 render commands 失效，不能留下上一帧几何。
- 保留指针坐标型 context menu 的领域语义：pointer anchor 可以作为 Runtime 输入策略，但不得迫使所有 popup 伪造 trigger control，也不得重新引入 Editor clamp。
- focused tests 至少覆盖实时 trigger frame、trigger 移动后的已打开 popup、边缘 flip/clamp、无效 trigger、nested stack owner、Escape/outside click 和 focus restore；随后运行 Runtime09 UI surface 回归与 UI12 workbench menu focused tests。
- 修复完成后写 `fixed-*` return，并附 current-source manifest、managed Windows job/run、实际测试数与独立 review 结果。

## 禁止临时方案

- 不得让 UI12 保留或重命名 `popup_anchor_metrics`、绝对 `position`、open-time `popup_anchor_*` 快照作为 generic workbench menu 的长期实现。
- 不得在 `.zui` 复制 trigger 的固定 x/y、增加 viewport 尺寸特判，或让 popup render 静默使用 popup 自身 frame 掩盖 trigger 解析失败。
- 不得把 Editor command/action identity 放进 runtime-interface popup contract。
- 不得绕过 Runtime stack 单独在 Editor 处理 Escape、outside click、nested close 或 focus restore。

## 修复结果与回传

Open state：`source_reconciled_managed_validation_pending`；UI12 继续执行与该基础合同无依赖的 M4 切片，等待 Runtime09 fixed return 后再完成 workbench popup hard cut。

### 2026-08-14 current-source reconciliation

The original failure snapshot no longer describes current Runtime source. `UiWidgetContract::popup_anchor` owns the serializable editor-agnostic `UiPopupAnchor::Control { control_id }` contract; it stores only stable UI identity, not Editor command state or geometry. `UiSurfaceControlIndex` resolves only a unique live control, `UiSurface::popup_anchor_owner` rejects missing, duplicate, disabled, collapsed, and invisible triggers, and the popup stack records that resolved trigger as its owner for close/focus restoration.

`resolve_popup_anchor_frame(...)` resolves the trigger's current arranged frame during extraction and suppresses an open control-anchored popup when resolution fails. `popup_trigger_requires_full_render_extract(...)` and stack reconciliation propagate trigger/ancestor/duplicate-id changes into a full extraction, while unrelated dirty nodes remain local. Pointer/numeric placement stays on the `UiPopupAnchor::None` path; `popup_anchor_frame(...)` only reads `popup_anchor_*` attributes for that non-control policy.

Current focused source coverage includes `popup_trigger_identity_resolves_open_popup_from_live_control_anchor_frame`, `render_extract_rejects_missing_duplicate_and_disabled_control_anchors`, the trigger and ancestor dirty-extraction regressions, `declarative_control_popup_effect_routes_to_resolved_trigger`, and close/rebind/invalid-trigger routing regressions. This is source reconciliation only: the Runtime09 focused managed Cargo tests have not executed in this repair window, so the handoff remains open and no fixed return is claimed.

### 2026-08-14 UI12 compile-anchor and formatting reconciliation

`UiPainterFamily` and `UiPainterResolvedState` are imported from the style hard-cut in
`zircon_runtime_interface::ui::style`; `UiDirtyFlags` is imported in both the surface and v2 style
owners. `UiSurface` imports `UiRuntimeTreeFocusExt`, whose descendant query is defined on `UiTree`,
and pointer dispatch exposes `dispatch_surface_route` at the runtime-internal `pub(crate)` boundary.
The navigation helper call and definition both use `(tree, current, kind)`, so the former callback
argument mismatch no longer exists in the current source.

`rustfmt +1.94.1 --check` passed for `ui/v2/style.rs`,
`ui/surface/render/popup_rows.rs`, and `zircon_runtime_interface`'s UI contract spine. Checking
the `surface.rs` root additionally visits unrelated `default_interactions` children with existing
import-order drift; those foreign files were not formatted or modified. No managed Runtime09 or
UI12 Cargo test has executed in this repair window, so this handoff remains `open` and is not a
fixed return.

### 2026-08-14 projected hit-grid managed validation attempt

UI12 released the Cargo lane after the Tooling finalizer completed. Runtime09 then submitted four
Windows validation tickets for the projected-grid module tests, frame/instance authority,
parent-input descendant propagation, and missing lookup reindex. All four tickets were bound to
the same exact seven-path source manifest
`57575239bcce7bd3ad119c6fe6f74f80e152e459494c81793b3132607af5bca4`; the externally changed
`zircon_runtime/src/ui/surface/render/cache.rs` was not part of that manifest.

The tickets and copy jobs were:

- `69750716903a4254bea2401665adb421` / `4e6b2d53fa8d4276bf295198ec2a7256`;
- `0f9c7e3efa1348a58710d58280a9360c` / `bc4600f60d5446cb932a883a7bec57ab`;
- `f4373d46564747e6a4528c0873b74f62` / `874d33160c5e4b858bbaec7603a9cd85`;
- `184ea8155d5e4136a1accf95d5c19283` / `7a4bee882de4409f8038fcfb56521c4f`.

Each copy failed in `closure_planning` with
`validation_copy_compile_time_resource_missing` before Cargo or rustc started. Replaying the
materializer's own Rust include lexer read-only identified the exact shared blocker:
`zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/host_adapter.rs`
still includes the deleted
`zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/tests.rs`. Plugins01 has already
hard-cut that aggregate test owner into `abi_decode`, `bridge_scope`, `context_handles`,
`ecs_registration`, and `registration_policy` children. This is a Runtime15/Plugins01 stale review
guard, not a popup source failure. The managed result is therefore `0` Runtime tests executed and
no Cargo exit code. The seven popup paths remain unchanged, the handoff stays open, and no Editor
product-path validation is claimed until the shared closure blocker is repaired and the same
source-bound tickets are resubmitted.

### 2026-08-15 popup routing source integration

The previously unintegrated popup routing source is now in `82ffe00f173dc8ae48734db9878f59d9f533903a`:
`popup_stack.rs`, `surface/event_routing.rs`, and `tree/node/focus.rs`. The commit keeps control
anchor owner resolution and modal focus routing in Runtime. Pointer routing now calls the public
`UiPointerDispatcher::dispatch` route API; the internal `dispatch_surface_route` helper remains in
its dispatcher owner and is no longer a stale surface caller requirement.

`rustfmt --check` and `git diff --check` passed for exactly those three source files. This is a
copy-stable source integration only. No Runtime09 or UI12 Cargo job ran for this revision, so the
handoff remains `open` and this section is not a fixed return.
