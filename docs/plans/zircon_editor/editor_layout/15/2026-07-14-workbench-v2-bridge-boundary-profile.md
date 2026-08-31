---
record_kind: milestone_slice
plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
milestone: M1.1 TextField/Dialog atomic-composite visual standardization
slice: workbench-v2-bridge-boundary-profile
status: accepted
date: 2026-07-14
related_code:
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_bridge_startup_profile.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/workbench/reference/template_surface.rs
  - zircon_runtime/src/ui/v2/surface_builder.rs
  - zircon_runtime/src/ui/layout/pass/layout_tree.rs
  - zircon_runtime/src/ui/surface/render/text_prewarm.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/text/font/coverage.rs
  - zircon_runtime/src/text/font/database.rs
  - zircon_runtime/src/text/font/database/tests.rs
  - zircon_runtime/src/text/shaping/fallback_spans.rs
  - zircon_editor/assets/ui/editor/components/workbench/primitives/chrome/workbench_chip.zui
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips/style.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/chip_visual_screenshot.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/icon_button_visual_screenshot.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_icon_button/selection/background.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_icon_button/selection/border.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_icon_button/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field/palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields_tests/paint.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/field_visual_screenshot.rs
  - zircon_runtime/src/text/shaping/cosmic/font_system_cache.rs
  - zircon_runtime_interface/src/ui/tree/node/ui_tree.rs
tests:
  - ui::template_runtime::runtime::build_session::tests::builtin_v2_template_file_cache_is_reused_across_runtime_instances
  - tests::host::retained_callback_dispatch::template_bridge::workbench_projection::startup_template_runtime_loads_componentized_workbench_window_bridge_source
  - tests::host::retained_callback_dispatch::template_bridge::workbench_bridge_startup_profile::profiles_startup_workbench_v2_surface_boundaries (ignored diagnostic)
  - text::font::database::tests::text_font_database_effective_instance_cache_reuses_weight_resolution_and_invalidates
  - text::font::database::tests::text_font_database_defers_discovered_system_coverage_until_the_face_is_used
---

# Layout 15：Workbench V2 Bridge 分段性能诊断

## 目标

将 Blend Space 三档截图验证中的持续高 CPU 停顿定位到最低共享路径。诊断不能移除 Workbench 组件闭包、改用简化 fixture、复用旧截图或增加 Blend Space 专用旁路；它必须从正常的 startup template runtime、Workbench document、shared V2 surface、layout 和 retained host projection 顺序通过。

## 已记录证据

- 最后可运行的兼容 editor test binary 执行 `builtin_v2_template_file_cache_is_reused_across_runtime_instances`：1/1 通过，1.52 秒。该用例加载两个 startup runtime 并确认 V2 file cache 被复用，故启动模板 cache 本身不是长时间 CPU 停顿点。
- 同一 binary 执行 `startup_template_runtime_loads_componentized_workbench_window_bridge_source`：60 秒仍未返回，累计约 58.9 CPU 秒；验证进程以精确 image path/PID 核实后终止。停顿发生在 startup runtime 成功之后、完整 Workbench bridge 返回之前。
- 静态闭包统计：`workbench_window.zui` 传递导入 122 个 `.zui`，含 2,155 个声明节点；组件展开模拟为约 2,035 个 surface 节点，root 加载时合并 809 条 stylesheet rules。该规模用于选择共享 surface/style/layout/projection 的诊断目标，不构成截图验收。

## 本切片实现

- 新增独立且 `#[ignore]` 的 `profiles_startup_workbench_v2_surface_boundaries`，避免继续向 140KB 的 `workbench_projection.rs` 聚合诊断逻辑。
- 探针按实际调用链输出五个累计边界：`startup-runtime`、`document-projection`、`surface-build`、`surface-layout`、`host-projection`。它同时断言被测 surface 仍超过 1,000 个节点并保留 `WorkbenchWindowRoot`，杜绝缩小 fixture 后得到无效性能结论。
- 模块文档 `docs/zircon_editor/ui/workbench/reference/surface.md` 已登记该测试、调用链和“只用于定位、不能作为验收”的约束。
- 当前源码 profiler 已产生实际边界：`startup-runtime` 873ms、`document-projection` 908ms（2,035 nodes / 1,221 bindings）、`surface-build` 1,407ms（2,035 nodes）；其后 `surface.compute_layout` 在 60 秒内没有返回。该结论排除 startup cache、文档投影与 V2 surface build，尚不能直接把根因归为 tree insertion、style 或 render。
- 为将 layout 边界缩小到具体阶段，`layout_tree.rs` 仅在 `ZR_UI_LAYOUT_PROFILE` 存在时输出 responsive-style、每个 root 的 measure/arrange start/complete 与 selection-report 累计耗时。默认行为与日志均保持不变；下一次同一 Workbench profile 将使用该环境变量取得根因证据。
- 使用该变量的实测把停顿收敛到 Measurement：startup 1,155ms、document projection 1,200ms（2,035 nodes / 1,221 bindings）、surface build 1,818ms、responsive style 8ms；root measure 开始后，首个 2-byte Button 触发约 30,215ms，随后每个叶文本仍需约 44–1,335ms。`shape-total` 与 `fallback-spans` 基本同量级，而没有 `buffer-shape` 慢点，说明共享 `FontDatabase` 的逐 grapheme 回退解析是主要热路径，不能把问题归咎于 V2 tree、样式解析或 arrange。

## 当前状态与下一步

`rustfmt --edition 2021 --check` 覆盖新增测试 module 和 module declaration 已通过；`git diff --check` 覆盖本记录、测试和模块文档已通过。受管 editor 矩阵曾在本探针将 `RetainedUiProjection` 误视为扁平 `nodes` 字段、又将 `RetainedUiHostProjection` 误视为 `root` 树时报告 E0609；诊断代码现分别递归统计 `RetainedUiProjection.root` 与读取 `RetainedUiHostProjection.nodes.len()`，不影响生产 bridge。

Runtime Text01 的生产 `FontDatabase::fallback_candidates_for_codepoint` 曾调用被错误标为 `#[cfg(test)]` 的 `FallbackResolver::candidates_for_codepoint`，令受管 `zircon_editor` 矩阵在 `zircon_runtime` 报 E0599。最低层修复已恢复该模块内方法的产品可见性；随后同一 Editor 重建越过该位置，说明 Layout15 未以 UI fallback、测试 bypass 或调用点特例规避 Text01。

Render18 的下一处最低层阻塞曾是 `light_grid_pass.rs` 与 `build_mesh_draws/build.rs` 读取已移入 `RenderFrameExtract::lighting.advanced_lighting` 的旧顶层 `advanced_lighting` 字段，受管 `zircon_editor` 矩阵报两处 E0609。已按硬切换将两个消费方直接收敛到 `lighting.advanced_lighting`，并确认 `zircon_runtime/src` 不再保留 `extract.advanced_lighting` 或 `frame.extract.advanced_lighting` 旧路径；没有添加兼容字段、alias 或渲染/UI 特例。随后的 profiler 已运行到 layout 边界，证明这两处构建漂移不再是当前门禁。

已在最低共享 Runtime Text owner 落下单一路径修复，而非在 Blend Space / Workbench 做旁路：

- `cosmic/font_system_cache.rs` 从首次 shape 起使用进程级 `FontDatabase` 的 `fontdb` snapshot 构建 locale `FontSystem`；不再先调用 `FontSystem::new()` 触发第二次全系统字体发现，再替换为共享数据库。
- `fallback_spans.rs` 在一次文本 shape 内只创建一个 `FontShapingFaceResolver`；若主字体覆盖全部 codepoint，则直接形成单一 primary span，避免每个 grapheme 重复 `match_face`、fallback-family 扫描和诊断聚合。`FontCoverage` 的有序范围查询同时由线性扫描收敛为二分查询；混合覆盖和未知覆盖仍走原有 resolver 路径。`FontDatabase` 进一步以规范化 `FontQuery` 缓存 primary `FontMatch`，字体/回退族新增时清空缓存，避免 Workbench 数千条同样文本样式重复排序同一字体族。
- `text_prewarm.rs` 新增 layout 前的叶文本收集。它对每个实际可测的 leaf label 以及水平文本测量所需的 `"Hg"` metrics sample，使用现有两线程 `TaskPool` 和同一个 `ShapedRunCache` 并行预热；主线程的 `measure_node` 随后只读取同一 cache。竖排文本不会被错误预热为水平 run。
- `UiSurface::compute_layout` 与 dirty rebuild 都在 `begin_frame` 后、layout 前调用该共享入口。已有 render-extract 预热继续保留给组件 painter 产生的文本命令，二者不替代也不分叉 cache。
- 新增 unit guard `prewarm_layout_tree_text_populates_metric_and_leaf_label_runs`：只收集 leaf，预热后同样 label 的 measurement 不得新增 shaped-run miss，锁住 measure/layout 的 shared-cache 契约。
- 新增 unit guard `fallback_spans_keep_primary_coverage_in_one_contiguous_span`，以 coverage gap guard 锁住二分范围查询不产生 false positive，并以 `text_font_database_match_cache_invalidates_when_a_better_face_registers` 锁住 primary-match cache 不跨数据库变更陈旧；三者只覆盖 Runtime Text 的共享层，不引入 Workbench 特判。
- `WorkbenchChip` 在组件展开为通用 `Label` 后此前会失去 painter identity，导致 Blend Space 的 `Perspective` / `Lit` 退化为未装饰文本。primitive 现在显式携带 `component_variant = "chip"`，shared chip identity 接受该语义 variant；新 regression 以展开后的 `WorkbenchExtensionBlendSpacePreviewCamera` control id 锁住该原子级路由，而不是增加 Blend Space 专用 painter 分支。

首次只包含 shared-font-system/layout-prewarm 的复测已经证实最初约 30 秒的 standalone font discovery 不再出现，但在 `responsive-style` 前仍有超过 56 秒的 fallback span 预热，因此不能当作最终通过。上述单 resolver、primary-coverage fast path 与二分 coverage lookup 在该失败之后实施，仍需 fresh binary 验证。

随后以包含这些 Runtime Text 改动的 current editor binary 精确执行该 ignored profile：startup 1,103ms、document projection 1,160ms、surface build 1,765ms；layout engine 的 responsive/measure/arrange/selection stages 在 4,344ms 内完成，而 `UiSurface::compute_layout` 到 render extract 返回为 68,183ms、完整 probe 68.56s（1/1 passed）。因此原有“layout 60 秒未返回”已关闭，但 render-text prewarm 仍为共享余项：日志中每个 fallback span 多为约 14–33ms，数量累积到 render extract。该 probe 之后新增的 primary `FontQuery` cache 尚未以 fresh binary 复测，不把它记录为性能通过。

受管 editor 构建已跨过本次 Runtime Text 源码，随后由 Plugin12/Editor12 正在处理的 `PluginEventConsumerManifest` / `runtime_event_consumers` API 迁移报错而停止；本切片不重叠修改那些已声明路径。同时，统一验证协调器当前短暂处于 `draining`，拒绝新的 runtime matrix job，故没有以直接 Cargo 绕过。协调器恢复后，先运行受管 `zircon_runtime` matrix 和两个精确 unit guard，再以同一 ignored profile（带 `ZR_UI_LAYOUT_PROFILE=1`）复测时间与 cache 行为；Plugin12/Editor12 return 接受后，依次运行 Workbench bridge / Blend Space contracts，并将新三档视觉截图只写入 `docs/tests/editor`。任一结果在这些门禁完成前均不标记为截图验收完成。

2026-07-15 的语义 chip 修复已通过 source contract、native extension event contract、`rustfmt --check` 与 `git diff --check`；它等待 fresh `zircon_editor` binary 执行 identity/Blend Space contract 以及三档 capture。旧 `docs/tests/editor/editor-window-m3-blend-space-workbench-*.png` 仅作为修复前对照，不能替代本次验收。

## 2026-07-15 运行时复核与后续修复

- 含此前 primary `FontQuery` cache 和 layout-time text prewarm 的 fresh editor binary 已执行 profile。其 layout engine 的 measure/arrange/selection 在 3,298ms 内完成，但 `UiSurface::compute_layout` 仍为 83,836ms（完整 profile 84.29s）。这证明预热不是消除 shared shaping 成本，而是把 render extract 的大量工作提前到了 layout 前；该布局前预热已撤回，保留现有 layout 后 render-command prewarm，避免三档截图为同一文本重复付费。
- 进一步的 profiler/source inspection 定位到每个已覆盖文本 span 都会调用 `effective_instance_id`，后者每次重新解析 sfnt variation axes。`FontDatabase` 现按 `(FontFaceId, font_weight)` 共享缓存有效 instance id，受容量限制并在 stored-face mutation 时失效；新增 `text_font_database_effective_instance_cache_reuses_weight_resolution_and_invalidates` 锁住同一字重重用与失效行为。此修复位于 Runtime Text owner，未向 Blend Space/retained painter 增加专项 fast path。
- fresh editor binary 已以 `workbench_chip_matches_the_semantic_variant_after_component_expansion`（0.01s）和 `blend_space_preview_toolbar_projects_shared_chip_variants`（210.80s）验证 chip semantic variant；后者仍运行在撤回预热和 instance cache 之前的 binary，只可作为结构正确性证据，不能作为当前性能或视觉验收。
- 本次尝试受管 `zircon_editor` build（check lane `0a203ed1001a478795869bf98c9ed805`）最初停在本会话范围外、刚创建的 `zircon_runtime_interface/src/runtime_api/session/operation.rs:60`：`matches!(Self::Completed | Self::Failed, self)` 的 pattern/value 顺序错误，报 E0424/E0369。该 interface 修复随后由其 owner 写入；Layout15 未改写或以局部 shim 绕过。
- 旧 binary 的三档 screenshot capture 在写入前主动停止，因此 `docs/tests/editor/editor-window-m3-blend-space-workbench-*.png` 时间戳未变化，且没有在 `target/` 产生截图。待外部 interface 编译错误消除后，先重新受管构建，再运行 Runtime Text cache guard/profile，最后只刷新 `docs/tests/editor` 的 640×520、900×620、1260×780 三张图并逐张检查。

### 当前源码验证门禁（2026-07-15 05:16）

- interface error 消除后，受管 `zircon_editor` test lane 再次编译到 `zircon_runtime`，但被 Plugins12 仍持有的动态 API operation export visibility 漂移拦截：`dynamic_api/session/operation.rs` 的 `submit_operation`、`poll_operation`、`harvest_operation` 仅为 `pub(in crate::dynamic_api)`，而 `session.rs` 需要 re-export 到 sibling `exports.rs`，报 E0364/E0603。该路径属于 Plugins12 open handoff `plugin-editor-runtime-mirror-consumer-wiring`，并非 Layout15；本会话不提升可见性、不开 compatibility wrapper，也不将它伪装为 Text/UI 修复。
- 为保证修复后不再只得到一个 80 秒总数，ignored Workbench profile 现把 `UiSurface::last_rebuild_report` 的 `layout`、`arranged`、`hit_grid`、`render_extract` 四项 ms 明确写入同一条 diagnostic output。它不改变产品行为，也不缩小 2,035-node real Workbench fixture。待 Plugins12 修复释放 current-source build 后，此 probe 将先确认共享 hot path 再决定下一项 Runtime Text work。

### 原子视觉迭代（2026-07-15 05:39）

- 人工对照 `docs/ui-and-layout/ai-workbench-style/ai-blend-space-layout.png` 与旧的 Blend Space 1260×780 图，确认共享 Chip 的被动 1px 边框使其像文本输入框，而非 Starship/Slate 的 quiet/simple rounded control。此结论仅针对原子控件，不把旧整窗图当作本次视觉通过证据。
- `template_chips/style.rs` 现在把普通 Chip 的 outline 与自身 surface 合并；hover、focused、pressed/open、disabled 仍保留明确反馈。`selected`/`checked` 另走 shared `surface_selected` + `accent_soft` border，而不是冒充 keyboard focus ring；这些色彩均来自 `EditorDesignTokens` 的 retained-host projection，未写本地 RGB、字体族或 Blend Space 专用分支。
- `chip_component_visual_paints_pill_chevron_selected_focus_pressed_and_disabled` 将 atlas 中的 viewport `Lit` 改为真实 selected/checked 语义，并新增 shared-style unit guard，锁住 selected chip 的低强调 surface/border 与 focused ring 的区分。`rustfmt --check`、`git diff --check` 已通过；current-source 编译仍由 Plugins12 dynamic API export visibility job 占用，故尚未声称 focused Cargo 或截图通过。
- 下一次 current-source binary 可用时，先执行该 atom 的 exact focused test/ignored capture，刷新 `docs/tests/editor/editor-components-chips-900x360.png`；然后重走 Runtime Text cache guard、真实 Workbench profile 和 Blend Space 的 640×520、900×620、1260×780 capture。所有新 PNG 仍只允许写入 `docs/tests/editor`。

### 当前源码 Editor gate 跟进（2026-07-15 05:55）

- Plugins12 的 `zircon_first_party_editor_catalog` current-source validation 已以 exit 0 完成，证明前一轮 dynamic API operation export visibility 门禁已不再阻断上游 `zircon_editor` 编译。
- 随后受管 `zircon_editor` test lane 实际编译到 editor lib test，发现三处 E0451：`editing/transaction_engine/{recovery.rs,scope.rs}` 使用 `FixtureContext { ..Default::default() }` 构造测试上下文，但 Editor03 最近为 runtime gateway 增加的 `gateway` 字段仍是 fixture 子模块私有。最低修复是该 test fixture 的字段向 transaction-engine sibling tests 开放 `pub(super)`；没有修改生产 gateway、transaction routing、Layout15 painter 或新增 bypass。该精确变更已通过 coordinator patch `#21` 应用到 Editor03 owner 路径，`rustfmt --check` 和 `git diff --check` 针对 fixture/recovery/scope 通过。
- 该完整 test lane 因最初 launcher supervisor 中断而由 coordinator 释放，不能计作测试通过；下次受管 `zircon_editor` lane 必须重新运行。此后再次因 Render18 使用同一兼容 pool 报 `cargo_reuse_pool_busy`，本会话按协调器规则等待而未创建额外 target。当前编译 gate、Chip capture、Runtime Text cache guard/profile 和 Blend Space 三档 capture 都保持待验证。

### 实际截图验收与 Runtime Text 最低层跟进（2026-07-15 07:33）

- 以包含 Chip quiet/default、selected 和 semantic-identity 修复的 current-source `zircon_editor` binary 精确执行：`chip_component_visual_paints_pill_chevron_selected_focus_pressed_and_disabled` 1/1（32.74s）、ignored `capture_chip_component_visual_artifact` 1/1（32.11s）、Blend Space Chip source contract 1/1（89.75s）和 ignored 三档 capture 1/1（460.32s）。所有 capture 都只写入 `docs/tests/editor`：`editor-components-chips-900x360.png`、`editor-window-m3-blend-space-workbench-{640x520,900x620,1260x780}.png`；target scan 没有同名 PNG。
- 已逐张人工检查新图：Chip 普通态没有被动输入框 outline，hover/selected/focus/pressed/disabled 的层级可辨；640 宽度折叠为窄 tier 而无破版，900 与 1260 保留可读的 Sample Grid、时间线和右侧预览/详情区。它们仍是 Layout15 的局部原子和 Blend Space 自适应验收，并不把当前完整 Workbench 的其余视觉债务标为完成。
- 同一 binary 的真实 2,035-node profile 1/1 通过：startup 1,519ms、document projection 1,588ms、surface build 2,667ms、layout 37,872.262ms、arranged 92.990ms、hit grid 49.315ms、render extract 15,071.745ms，完整 56.67s。带 `ZR_UI_LAYOUT_PROFILE=1` 的复测显示最早 Button 的 fallback/shape 仍会积累约 27s，普通 label shape 约 14–33ms；根因仍在 Runtime Text 的共享系统字体 coverage，而非 Blend Space 特例。
- 因此将 `FontDatabase` 的已发现 system face cmap coverage 改为 `OnceLock` 按 face 首次实际 lookup 初始化；已注册的文件/asset/test face 保持立即 coverage，未知 coverage 继续保持原有 permissive 语义。新 Windows guard 锁定 discovery 不预扫所有 cmap、首次 `'中'` lookup 后初始化；这与已有 primary match / effective instance / fallback span cache 处于同一 Runtime Text owner，未向 editor painter 或 Workbench 写特例。`rustfmt --check` 和 `git diff --check` 已通过；首次受管 `zircon_runtime` full package lane 在 333.6s 后以 exit 1 结束，诊断已归因到 Editor03 动态 API 当前源码漂移：V1 API table test 未 re-export `zircon_runtime_get_api_v1`/`ZrRuntimeApiV1`（E0425），session construction 又漏写 `mut linked_extensions`（E0596）。它们分别由协调器 patches #29/#25 应用到该 owner。修复后启动的第二轮同池 matrix（job `ef03368ca8c04ef9ac6f4458f8bb4c0f`）持续编译约 11 分 49 秒后被协调器标为 `orphaned`，没有 terminal exit/fingerprint，仍不得计为通过。
- 释放后第三轮 Windows managed matrix（job `3e3149d127f14e19848f5805d73ef3bc`）取得了 terminal exit 1，且证明前述 Text/Font 变更已经进入当前源编译；但被 Editor03 正在进行的 `dynamic_api::session` 文件拆分整体阻断：40 个 E0432/E0364/E0365/E0603/E0425，涉及 session 子模块重导出、`DEFAULT_VIEWPORT`/`with_session`、`RuntimeDynamicSession` 可见性及冻结 V1 API table。同一 owner session 仍为 `resolving_failure`，已存在 operation sibling-visibility fixed handoff，但当前更广的 session 边界尚未稳定。本切片不修改这些 owner 路径，不新增 re-export shim 或 UI/Text bypass；待 owner 完成收口后重新取得 terminal matrix 与 lazy-coverage guard 结果。
- 之前的完整 `zircon_editor` package lane在 30 分钟以上无新 CPU 进展、全部 test worker 等待时经 PID/进程树取证后终止并由 coordinator release；它同样不计为通过。已运行的 exact Chip、Blend Space、Runtime Text cache guards 和 screenshots 仍可作为各自范围的独立通过证据。

### 窄档底部抽屉编辑区预留（2026-07-15）

- 重新检查 640×520 Blend Space 截图时发现，空的底部 Console/抽屉仍占用了约四分之一的垂直编辑空间；这不是可接受的“窄档无破版”，而是 Layout15 C1 所要求的「抽屉在空间紧张时收为 rail/affordance」尚未落实。
- shared `drawer_layout` 现在按现有 physical-width tier 判定：`Ultra`/`Narrow`（≤640px）保持可点击的 42px tab strip，却把 drawer body 的高度交还给 active document。Regular/Wide 保留原有依据纵向可用空间的 compact logic；实现只读取 shell size、scale factor、tokenized header metric 与 anchors，不加入截图分辨率坐标或 Blend Space 特例。
- 先新增 `narrow_width_collapses_a_visible_bottom_drawer_to_its_tab_strip` 回归 guard，随后实现共享 composer 路径；`rustfmt --check` 与该文件的 `git diff --check` 通过。此时的 current-source 仍被 Editor03 进行中的 `dynamic_api::session` 拆分阻断，故该 unit guard、fresh production capture 与截图人工验收均保持待验证，不能用旧 640 图冒充修复后证据。

### Primary Button 的 Starship 状态层级（2026-07-15）

- 源码复核 `FStarshipCoreStyle::SetupButtonStyles` 后确认：UE 的 `PrimaryButton` normal/hover/pressed 是独立的 primary fill 层级，而当前 `filled` Zircon button 错误地复用了普通灰色 `surface_base`，使 Compile 与 Browse 在组件图集中没有清晰主次。
- shared `WorkbenchButtonPalette` 现把 primary rest/hover 映射为中央 `EditorDesignTokens` 投影的 `accent`/`focus_ring`；pressed 继续使用既有低强调 `surface_selected`，边框、圆角、12px 水平内边距与 1px 按下下沉仍走同一 host metrics。没有复制 UE 蓝色 hex、增加本地色板，或把按钮逻辑塞进 Blend Space。
- 先添加 palette、normal、hover 三个 selector regression，再把 Button atlas 的 primary 取样直接绑定到 `EditorPaletteTokens::WORKBENCH_ACCENT`，确保真实 retained painter 不能退回灰色。`rustfmt --check` 与 `git diff --check` 已通过；current-source managed build 仍由 Editor03 运行中 Runtime lane 占用，故 focused test、ignored capture 及人工图像审查仍待执行，现有旧按钮图不得作为本切片验收。

### 当前源码窄档与 Primary Button 截图验收（2026-07-15 09:56）

- Editor03 的 V2-only Runtime check 释放共享池后，本会话受管 `zircon_editor` check lane `087272f38cc84ebd8338044806c54157` 以 exit 0 完成（5m53s）。完整 package test lane `e9224849cae24daaa14a9606981f7a23` 进入同一二进制后持续 30 分钟以上无 CPU/working-set 进展；取证后终止，coordinator 已 release。它没有 terminal test result，不能计为 package test 通过。
- 同一 fresh test binary 的精确 guards 均通过：`narrow_width_collapses_a_visible_bottom_drawer_to_its_tab_strip` 1/1（0.01s）、`primary_button_uses_the_starship_primary_surface_role` 1/1（0.04s）、`primary_button_hover_uses_the_brighter_primary_surface_role` 1/1（<0.01s）及真实 atlas `button_component_visual_paints_text_icon_pressed_disabled_and_tabs` 1/1（10.93s）。这分别锁住 width-tier drawer collapse、primary rest/hover token route 与 painter 端实际主色 surface。
- ignored Button capture 1/1（10.54s）已刷新 `docs/tests/editor/editor-components-buttons-900x360.png`：32,465 bytes，SHA-256 `B95491007A4879B254E58BE78BBEF09813A9DB7FB47A2FD939A65BE1C724DDDD`。人工检查确认 Compile 成为紧凑 teal 主操作，Browse 仍是低强调 secondary，图标/菜单、pressed/disabled 与 quiet tab 未被主色误染。
- fresh Blend Space capture 已刷新 `docs/tests/editor/editor-window-m3-blend-space-workbench-{640x520,900x620,1260x780}.png`，分别为 73,931 / 91,677 / 206,682 bytes，SHA-256 分别为 `F854347AFDAAFFF0253E0181A8390ABA6AC2B124E0EE4F63C00FE2C71ECCF5EB`、`73AC795EBBF85F78DFB32B36CB56A3D97F2985CEB755C459DC57D53C301FFFB8`、`76F42B66B48E6A41CD3B453DDA5644DC8CF1043CBCAD53570E4A1E2C2ABFDF3B`。逐图检查：640px 只保留 Bottom Drawer tab strip，空 Console body 不再占用 document 高度；900px 保持 timeline；1260px 保持 preview、sample weights、validation log 和 detail columns。capture supervisor 在全部文件落盘后未回传 terminal exit，故该三档的依据是 fresh artifacts、精确 drawer guard 和人工审图，而不是将 capture command 误记为 1/1。

### 三档 Blend Space capture current-binary 复验（2026-07-15 12:20）

- 为把上述“文件已落盘但 supervisor 无终态”的证据替换为可重复的测试结果，直接执行 shared `841a` current-source editor test binary 的 ignored `capture_blend_space_workspace_visual_artifacts`：1/1 通过（448.56s）。这没有启动 Cargo、未占用或改变外部会话的受管 job；它只重放已经由同一 source tree 构建的 visual artifact test。
- 三张图再次只写入 `docs/tests/editor`：640×520 为 73,446 bytes，SHA-256 `4CC19D1934337AA476D88B29A49B7CCED53968EE8C2DEED96E8C44D71D55D337`；900×620 为 91,191 bytes，SHA-256 `170D3A595580E68F9EF6C64F79DF9EF21FDAA6A2661280024A1FF110C19D2F93`；1260×780 为 206,197 bytes，SHA-256 `260ABB2DC83F71DE909591D8D1A149CD80E6C90BA7A4B6ABAFF71EE87CB8DC4C`。仓库 `target/` 的三文件名扫描均为 0。
- 人工复核确认：640px 保留顶端主命令、窄档 left navigation 和可重开的 Console tab strip，文档区取得剩余高度；900px 继续显示 sample grid 和 preview timeline；1260px 同时保留 preview、weights、validation 与 inspector。截图验证的是共享相对约束/width tier，不增加截图像素定位。完整 editor package 以及 Runtime Text package/profile 的受管终态仍是独立 pending gate。
- Button screenshot capture 的 repo `target` scan 为 0 matching PNG；Blend Space capture 的路径由 artifact test 固定写为 `docs/tests/editor`。本次可接受的是 Layout15 的 primary-button 原子层与窄档 bottom-drawer 相对布局；Runtime Text lazy-coverage package test/profile、完整 package test 和其它 Workbench 视觉债务仍保持 open。

### Alert 焦点/按下边框原子验收（2026-07-15 10:17）

- 精确执行 current `zircon_editor` test binary 的 `alert_toast_component_visual_paints_tones_actions_focus_and_disabled`：1/1 通过（10.39s）。其中直接对比 warning alert 的 pressed 和 focused border，锁住 pressed 使用 active border、focused 保留 tone border，避免键盘焦点伪装成鼠标按下。
- ignored `capture_alert_toast_component_visual_artifact`：1/1 通过（9.54s），刷新 `docs/tests/editor/editor-components-alert-toasts-900x360.png`（46,367 bytes，SHA-256 `A1D2891B59F7DC3D057EFE75B71FC3D0E38C88651FB1D5AABDC23C63270D3DC3`）。人工审图确认 info/success/warning/error 四种 tone、toast action、pressed/focused warning 与 hovered/disabled toast 层级均可辨。
- 对该文件名执行 repository `target` scan 为 0 matching PNG；截图唯一落点仍为 `docs/tests/editor`。这闭合 S15.4gf/S15.6fg 的 focused Cargo 与视觉证据，不扩张为 Runtime Material 或全局状态优先级的验收；后续 Runtime Text lazy-coverage 和 package/profile gate 仍保持 open。

### Shared focused-only state priority 截图复验（2026-07-15 10:21）

- 精确执行 `state_priority_visual_paints_focus_without_promoting_hot_or_selected_surfaces`：1/1 通过（9.42s）。fixture 和像素断言同时覆盖 focused-only list row、popup row、chip、prominent command 与 chrome，分别保持 focus outline 或 normal fill；hovered/selected/pressed 对照仍保留自己的 surface。
- ignored `capture_state_priority_component_visual_artifact`：1/1 通过（10.45s），刷新 `docs/tests/editor/editor-components-state-priority-900x360.png`（41,819 bytes，SHA-256 `96554D331B892D61D3B5FB8725AB1B10AB45A2E2F4AF859368976A792E6055BB`）。人工审图确认焦点行仅有 outline、popup/row 的 hover 与 selected fill 分离、focused chip 与 pressed chip 分离，focused command 未升格为 hovered command，focused chrome 保持正常面。
- repository `target` scan 为 0 matching PNG。这重新验证 S15.4fw/S15.6ex、S15.4fv/S15.6ew、S15.4fu/S15.6ev、S15.4fq/S15.6er 与 S15.4fm/S15.6en 的共同截图证据；其它 runtime Material、notification、MUI X state-priority 子项仍各自保持 open，不能由此截图替代。

### Runtime Text lazy coverage focused guards（2026-07-15 10:26）

- 直接复用 current `zircon_runtime` test binary 精确执行 `text_font_database_defers_discovered_system_coverage_until_the_face_is_used`：1/1 通过（2.13s）。该 guard 锁住 discovered system face 的 cmap coverage 不会在 discovery 时为整个 Windows catalog 预扫，首次实际 lookup 才初始化。
- 同一 binary 精确执行 `fallback_spans_keep_primary_coverage_in_one_contiguous_span`（1/1，0.07s）、`text_font_database_effective_instance_cache_reuses_weight_resolution_and_invalidates`（1/1，0.01s）及 `text_font_database_match_cache_invalidates_when_a_better_face_registers`（1/1，0.03s）。它们分别锁住 primary-coverage fast path、weight-instance reuse/失效与新增更优 face 后 primary match cache 失效。
- 这四项是 Runtime Text 最低共享层的 focused evidence，不替代受管 `zircon_runtime` package/profile matrix。该兼容 pool 此刻由 Shader06 active job 使用，Layout15 不抢占或另建 target；package/profile 结果继续保持 pending。

### Runtime Text managed package lane release（2026-07-15 10:48）

- Windows managed `zircon_runtime -SkipBuild` package lane `d32d4719ad11455cb56405f678430496` 启动后，外部 supervisor 在约 13 分钟后退出；coordinator 将 job 标记为 `orphaned`，没有 exit code 或 Cargo terminal output，且进程树为空。
- 已按实际结果执行 coordinator `cargo release`；job 现在是 `released`，`released_at=2026-07-15T02:48:04.361865Z`，无 live PIDs，target 仍是获准的共享 `D:\cargo-targets\zircon-engine\pool\841a...`。没有删除 target、没有新建 repo `target/`，也没有改动其它会话的 source。
- 因缺少 terminal package result，这一尝试不能计作 Runtime Text package/profile 测试通过。focused four-guard evidence 仍有效；下一轮只可在 coordinator 分配可用 lane 后重跑并获得 terminal exit。

### ConfirmDialog responsive rail review correction（2026-07-15 19:12，待 managed verification）

- 独立只读审查在初版 source/static slice 中确认两个 Important：154×88 的短窄 ConfirmDialog 可能让纵向 Cancel/Confirm 组侵入标题区域；而 `AlertDialog` 被归类为 `ConfirmDialog`，会意外接受新的响应式 action/body rail。没有发现 Critical。
- 共享 `template_dialogs` 现以标题底部加 tokenized content gap 作为 action rail 的最低位置。测量宽度不够时仍优先纵向堆叠；若默认 stack gap 会令上按钮越过该 floor，则只压缩两个堆叠按钮之间的 gap，保留各自测量宽度，正文因没有一整行余量而被抑制。新 guard 覆盖 154×120 的正常 stack 以及 154×88 的 compact-gap stack，并只使用 dialog metrics 和可用宽度，不含截图尺寸坐标或固定最小宽度。
- `AlertDialog` 现在是独立 kind，但继续使用原有 severity mark/border/title color 与原有 `body_top`/`action_bottom` 算法、原 command sequence 和 measured label layout；该分离防止响应式 ConfirmDialog 修复改变 Alert 的可见行为。`rustfmt --edition 2021 --check` 与 scoped `git diff --check` 均通过。当前 Runtime Text03 `VerticalTextLayoutScope: TextShapeRunProvider` E0277 仍阻断 managed `zircon_editor` build，因此这不是 screenshot 或 package acceptance；待 lower provider contract 修复后只从 fresh managed binary 执行 Dialog guards 和 atlas capture。
- 复审修正后的报告为 Critical 0 / Important 0 / Minor 1：在 154×88 的数学极限中，维持标题、两条完整测量宽度 action 与默认 bottom inset 会让第二行低出 surface 约 0.2 logical px。ConfirmDialog command clip 现显式取 inherited clip 与 dialog rect 的 intersection，故保留 label width 和无标题重叠，同时不会把该像素泄漏到 dialog 外；Alert 保持原 inherited clip，未改变其路径。此裁剪修正后再次 `rustfmt --check` 与 scoped `git diff --check` 均通过，仍待 fresh managed compile/test/capture。
- 最终独立只读复审为 Critical 0 / Important 0 / Minor 0；`artifact audit` 也报告 `unmanaged: []`。这些是 source/static/review evidence，不替代被运行中的 Runtime Text owner job 阻塞的 managed editor compile、Dialog exact guards、atlas capture 或完整 package gate。
- Runtime Text owner 后续受管 repair `f1931ecfaed64f9ea15f44ea69322bf8` 已以 exit 0/released 结束；本会话立即请求 `zircon_editor -SkipTest` managed compile。coordinator 正确拒绝 request，原因是 foreign orphan job `08424cf7870d433faf5fc86420df5faa` 仍记录一个 live descendant PID 43688 并持有兼容 shared target。没有创建 alternate target、没有 finish/release/terminate foreign job、也没有将这次 acquire rejection 记作 Dialog compile result；待 coordinator 回收该 descendant 后重试。
- 对同一 managed request 的第二次重试得到相同 `cargo_process_tree_alive` 拒绝。只读取 PID 43688 的 Windows process metadata 后确认，它是 `D:\Tools\app\Wps\WPS Office\...\wpscloudsvr.exe`（parent PID 59880、19:16 启动），不是 Cargo/rustc；因此这是 stale orphan process identity 的 PID reuse，须由 foreign owner/coordinator 处理。本会话没有杀 WPS、终止 foreign process 或修改 foreign job；在 coordinator 收束此错误归属前不再进行第三次相同 acquire。

### Toolbar IconButton quiet chrome（2026-07-15 11:00）

- 对照 `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Styling/StarshipCoreStyle.cpp::ToolbarButton`：normal 使用 `FSlateNoResource`，hover/pressed 才使用 4px rounded selection。Zircon 原 selector 却把 `WorkbenchToolbar*` normal state 画成常驻 surface + 1px border，使顶栏图标像一排输入控件。
- shared `workbench_icon_button` selector 现移除该 control-id 特例：Toolbar normal/disabled 无 surface/border，hover/pressed/open/selected 使用既有 state surface 但无 outline，focused-only 仍保留 focus-ring border；Panel/Rail 的既有上下文规则不变。无死 helper、无 local RGB、无 Workbench/Blend Space 专用分支。
- selector TDD guard 与真实 icon atlas 已直接执行同一共享 `841a` current-source editor binary：`toolbar_icon_button_uses_starship_quiet_simple_button_chrome` 1/1（<0.01s）和 `icon_button_component_visual_paints_context_sizes_and_pressed_offset` 1/1（6.24s）通过。它们分别锁住 normal quiet、hover/pressed rounded surface 无 border、focused-only border，以及 normal toolbar sample 回落到 panel、pressed 与 disabled 分离。
- ignored `capture_icon_button_component_visual_artifact` 1/1（6.16s）刷新 `docs/tests/editor/editor-components-icon-buttons-900x360.png`（24,818 bytes，SHA-256 `C8CDD46E32E0469DA6E495770DBC73B2CE92F9927955AA76928BBB4BB7C3DBCB`）。人工审图确认 Toolbar normal 没有输入框式 tile，Panel/Rail 的既有上下文对比保持，pressed 才显示 rounded selection。仓库 `target/` 同名文件扫描为 0；截图唯一落点是 `docs/tests/editor`。
- 该 binary 来自仍由外部 session 持有的 managed `zircon_editor` test job；本会话没有直接运行 Cargo、也没有把该外部 job 的未终态视作 package test 通过。IconButton 原子层的 exact test/capture 已完成，Runtime Text package/profile 与完整 editor package terminal result 仍为本切片的独立 pending gate。
- 作为原子组合一致性复核，重新执行 `capture_workbench_component_slate_atlas_visual_artifact`：1/1（21.31s）更新 `docs/tests/editor/editor-components-workbench-slate-atlas-900x620.png`（98,626 bytes，SHA-256 `0F6B0AA92323FB2ADEBDE153610EA6903FE8EA1BD425B05F32AC7AA6C2AA3A20`）。此图的 Primary 已与 Button atlas 一致地呈 teal，避免旧截图把已收敛的 shared primary selector 误呈为灰色；仓库 `target/` 同名文件扫描为 0。

### TextField Starship input-outline 原子修复（2026-07-15，待 current-source 验证）

- 对照 `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Styling/StarshipCoreStyle.cpp` 的 `NormalEditableTextBox`：normal、hover 与 focus 都保留同一 recessed `Input` fill；状态仅切换 `InputOutline`、`Hover` 与 `Primary` outline。Zircon 原来把 focus 映射到中性边框，未提供键盘焦点语义。
- shared `workbench_text_field` selector 现在把 normal/toolbar/hover/focus 统一投影到 `surface_inset`：hover/drag/drop 使用 tokenized `surface_hover` outline，focused/open/pressed 使用 `focus_ring` outline。error、disabled、文本、padding、stepper 分隔线仍走既有共享 token/metric 路径，没有引入本地 RGB、截图坐标或 Workbench 专用路径。
- 新的 selector guards 直接锁住「focus 不抬升 input surface、但必须为 primary outline」以及「hover 不得伪装为 keyboard focus」；既有 `template_fields_tests` 的 exported test-token fixtures 和 painter guard 也已同步到同一 focus role，避免测试常量保留旧的 neutral-border 语义。Field atlas 同时增加 normal editable field 与 SearchBox focused outline 的精确 token 取样。`rustfmt --edition 2021 --check` 和 scoped `git diff --check` 已通过。
- 受管 `zircon_editor` check `3ffa39fe7f654198a1614e8730bf6136` 已以 exit 0 完成（7m04s）；由它生成的 current test binary 随后精确通过两个 selector guard（各 1/1）、`focused_workbench_field_uses_primary_focused_border`（1/1，5.71s）和 `workbench_field_uses_starship_recessed_surface_primary_focus_and_muted_placeholder`（1/1）。此前两次完整 package test lane 被 external supervisor 标为 orphaned，均已按无 terminal exit 的实际状态 release，不能计作 package acceptance。
- ignored `capture_field_component_visual_artifact` 以该 current binary 通过 1/1（7.84s），只刷新 `docs/tests/editor/editor-components-fields-900x360.png`：34,590 bytes，SHA-256 `1C05CB159118299D30B9B995F0651FF7389EF382B1291D043161722D20C15E41`；仓库 `target/` 同名文件扫描为 0。人工审图及 PNG 像素读取均确认 focused SearchBox `(300,186)` 和 normal focused field `(728,130)` 为 shared `WORKBENCH_FOCUS_RING`，而旧 rounded-corner sample `(250,186)` / `(678,130)` 分别不是 outline。图集现在清楚展示 recessed 普通输入、focused SearchBox、hover stepper、teal focus 及 disabled input。
- 由旧 corner sample 造成的 `field_component_visual_paints_input_search_stepper_focus_and_disabled` failure 已在 source test 中修正为上述直线 outline sample，并再次通过 `rustfmt --edition 2021 --check` 与 scoped `git diff --check`。其间 Runtime Text hard-cut owner 已以 managed `zircon_runtime` build exit 0 恢复此前 16 个 `core::framework::render` → `text` import 错误；随后的 corrected `zircon_editor` check jobs `b716888f781942b9b1c9831b15e739e6` 与 `b94b60b8a53343c6a7eaf6fe7927a11e` 均因外部 supervisor 无 terminal result 而 orphaned/released，未生成可执行 fresh test binary。没有修改该 Runtime Text owner、没有 re-export shim；因此本项 visual capture 已接受，但修正后 atlas guard 和 package terminal acceptance 仍需在下一次有终态的 current-source rebuild 中重跑。

### TextField corrected current-source visual acceptance（2026-07-15 15:01）

- 修正后的 current-source editor test binary 由受管 `zircon_editor` package lane `5d748aeab86547fc8849b40b97da3928` 产生。直接执行该 binary 的 exact guards 全部通过：`text_field_focused_state_uses_starship_primary_outline_without_raising_input_surface` 1/1（<0.01s）、`text_field_hover_uses_hover_outline_without_impersonating_keyboard_focus` 1/1（<0.01s）、`focused_workbench_field_uses_primary_focused_border` 1/1（5.88s）、`workbench_field_uses_starship_recessed_surface_primary_focus_and_muted_placeholder` 1/1（0.01s），以及已修正采样坐标的 `field_component_visual_paints_input_search_stepper_focus_and_disabled` 1/1（7.05s）。
- ignored `capture_field_component_visual_artifact` 同一 binary 通过 1/1（8.84s），仅刷新 `docs/tests/editor/editor-components-fields-900x360.png`：34,590 bytes，SHA-256 `1C05CB159118299D30B9B995F0651FF7389EF382B1291D043161722D20C15E41`；仓库 `target/` 同名文件扫描为 0，artifact audit 无 unmanaged target。人工审图确认 recessed normal surface、focused SearchBox、hover stepper 与 focused input 的层级清晰；焦点仅改变 1px teal outline，不抬升输入框 fill。
- 因此 TextField 原子视觉/selector/真实 painter 截图证据现已接受。受管 `zircon_editor` package lane `5d748aeab86547fc8849b40b97da3928` 在 test binary 启动后保持约 26 分钟、CPU/working set 基本不变（进程 `23500`），经 PID、父子关系和归属 session 核验后已终止并由 coordinator release；它没有 terminal test result，不能计作 package acceptance。Runtime Text full package/profile 也是独立 pending gate。
- 独立只读代码审查结论为 Critical 0、Important 0、Minor 1：reference surface 文档曾将普通 field 错写为 teal focus outline；已将其改正为 focused field。审查同时确认 token 路由仍经 `current_host_palette()`、没有新增生产 RGB bypass、截图路径固定为 `docs/tests/editor`；本次文字修正后再次执行 scoped `rustfmt --check` 与 `git diff --check`。

### ConfirmDialog 内容/action rail 相对布局（2026-07-15，受管验证待运行）

- 最新 Workbench atlas 的重叠精确归因于 `WorkbenchDialogAtlas`，而不是 Alert/Toast：220×104 ConfirmDialog 的固定 `body_top` 与固定 bottom action row 共用垂直带，令 `Apply material changes?` 与 Cancel/Confirm 文本相交。对照 UE `SNotificationList::ConstructInternals`，标题、说明和 buttons 必须是分离的 vertical slots，buttons 作为独立右对齐行置于内容之后。
- shared `template_dialogs` 现先根据两个运行时测量后的 action 宽度计算 rail：宽度足够时保留右对齐 horizontal Cancel/Confirm；不足时按同一 content width stack 两行，并把每个 action clamp 到可用内容宽度。action painter 返回最上行的 y；正文布局从 title 后的 host-token gap 开始、在该 y 之前保留 content/action gap，空间不足则不画正文而不发生重叠。没有改 Alert/Toast、没有局部 RGB、截图坐标、强制最小 Dialog 宽度或 Workbench 专用分支。
- 新回归覆盖正文低于 title 且在 rail 前结束，以及窄宽两 action 垂直 stack、同 x/right edge、无互相覆盖和不超出 content width。`rustfmt --edition 2021 --check` 与 scoped `git diff --check` 已通过。Windows managed `zircon_editor -SkipTest` 尝试在获取 lane 前被 coordinator 以 `cargo_reuse_pool_busy` 拒绝（外部 job `b5c9181f1ff847c5bc29fc41f6a34e0a`）；没有启动 Cargo、没有新建 target，真实 unit/capture/人工审图仍待 pool 释放后重跑。
- 随后该 pool 空闲时启动的 managed check `e8c826cacbca418d9c32c5432fec4078` 具有已核验的本会话 supervisor/Cargo/rustc 进程树，并在约 3 分钟后被 coordinator 标为 `orphaned`，无 exit code、无 live PID；已按实际状态 release，target 仍为既有 managed shared pool。它不能计为编译通过，也不使用其残留产物推断 unit/capture 通过；下一次只能取得 terminal managed check/test 结果后继续截图验收。
- 下一次 terminal managed check `b3b558ec561f4c33b7cf7cb97e54cf96` 真实以 exit 1 完成并已 release（1m43s）。它在 `zircon_runtime` 先报两处 E0277：`ui/text/layout_engine/rich_inline_vertical.rs` 将 hard-cut 后的 `SharedTextLayoutSession::vertical_scope(VerticalMode::Mixed)` 传给 `rich_vertical_columns_with_provider` 与 `measured_grapheme_widths_with_provider`，但 `text/layout_session.rs::VerticalTextLayoutScope` 尚未实现 `TextShapeRunProvider`。这不是 Dialog、selector 或 retained painter 的失败。
- 按 support-first 检查，最低层候选为 `VerticalTextLayoutScope` 的 shared trait forwarding、`VerticalTextShapeRunProvider` adapter，以及 rich-inline caller 的 scoped provider ownership；已证明的最低共享断点是前者缺失的 trait contract。Text03 是 `text/layout_session.rs`/VerticalRl rich-inline layout owner；现有 Text03 open handoff 只覆盖 ellipsis/paragraph regressions，尚未覆盖本 E0277。Layout15 不添加 editor bypass、旧 namespace shim 或调用点特例。由于 coordinator 正确拒绝 Layout15 在 `docs/plans/zircon_runtime/text/03` 直接写 canonical failure artifact（`outside_registered_child`），本条作为 origin evidence 留档，等待 Text03 owner 创建并返回其 child-plan handoff 后重新运行同一 managed editor check。

### ConfirmDialog current-source acceptance（2026-07-16）

- Text03 修复返回后，Windows managed job `edd25ded210548dbabfea57f6fcf2087` 成功编译 current-source `zircon_runtime` 与 `zircon_editor`，生成 `D:\cargo-targets\zircon-engine\pool\841a...\debug\deps\zircon_editor-7cbf6e3f9c684171.exe`（2026-07-16 04:58:03 +08:00）。同一 fresh binary 精确通过五项 Dialog guards：正文/action rail 分离、窄宽 action stack、短窄 compact stack gap、Alert legacy identity/body offset，以及短高 Dialog 在丢弃正文前压缩 content/action gap，合计 5/5。
- ignored `capture_workbench_component_slate_atlas_visual_artifact` 以同一 binary 通过 1/1（11.97s），只刷新 `docs/tests/editor/editor-components-workbench-slate-atlas-900x620.png`：99,148 bytes，SHA-256 `2334AB05CDF5FC6744870B79F4A71A79D9164FBCA7D4E6475186164D19A6AA99`。人工审图确认 220×104 ConfirmDialog 的 `Apply material changes?`、Cancel 与 Confirm 均可读、互不重叠并位于 surface 内；本次 capture 未写入 target。`E:\cargo-targets\verify\...` 中仍存在 2026-07-14 的外部 validation-copy 旧图，不是本次产物，也未由 Layout15 删除或吸收。
- managed package runner 在 fresh binary 启动后进入既有无 CPU 进展状态；经本 job 的 PID/父子关系核验后仅终止其 owned process tree，并以实际 `exit 124` finish/release。故本记录接受 current-source compile、五项精确 contract 与视觉截图，不把完整 `zircon_editor` package 宣称为通过。
- 首次 upward compile 暴露的 Render18 `volumetric_ambient_radiance` canonical re-export 已由 Render18 owner 确认。修复证据记录在 `fixed-2026-07-16-volumetric-ambient-radiance-module-export.md`，Failure 已 return，`froxel/mod.rs` 与原 Failure 路径的 lease/ownership 均已释放；该 runtime patch 明确排除在 Layout15 milestone manifest 外，由 Render18 AF-M3 保留。

### StandardDialog typography current-source acceptance（2026-07-16）

- Windows managed job `f10ce55f61804e7aaa6e66d53b2f6d79` 生成包含最终 84px fixture 的 fresh `zircon_editor-7cbf6e3f9c684171.exe`（2026-07-16 09:58:25 +08:00，182,060,544 bytes）。同一 binary 精确通过 8 项 focused contracts：host metrics projection、UE StandardDialog 10/8/10 role、正常 body/action rail、84px compact body/action gap、Alert legacy identity/body offset、窄宽 action stack、短窄 compact stack，以及专用 Dialog painter 像素 guard，合计 8/8。
- 同一 binary 的 ignored captures 通过 2/2：`capture_dialog_component_visual_artifact` 1/1（6.62s），`capture_workbench_component_slate_atlas_visual_artifact` 1/1（13.43s）。输出只写入 `docs/tests/editor`：`editor-components-dialogs-900x360.png` 为 33,044 bytes、SHA-256 `29929C5A47ED94A1315D16C8BAF9F2283128E26BAC3A31941624A12A8CD0EFAA`；`editor-components-workbench-slate-atlas-900x620.png` 为 95,681 bytes、SHA-256 `BAAF115498884D16B1233F71D67E277809D9E58D1496563DF28AD45613E80184`。仓库 `target` 与本次 managed pool 同名扫描均为 0。
- 人工审图确认 Standard/Confirm/States/disabled 样例的 title、body、action 层级清晰；聚合 atlas 的 `Unsaved asset`、`Apply material changes?`、Cancel 与 Confirm 保持 token-relative 间距且互不重叠。标题/action 继续走用户 typography 的 `font_body`，正文走 `font_small`，没有 local font family、RGB 或截图像素定位。
- broad package worker 在 focused/capture 完成后 30 秒内 CPU 仅 `175.28 -> 175.31`、working set 保持 71,499,776 bytes、87 threads；仅终止本 job 的 test/Cargo tree，并以真实 `exit 124` finish/release。故本记录接受 current-source compile、8 项 focused contract 与 2 张视觉图，不声明完整 `zircon_editor` package 通过。
- 独立只读 reviewer 对 exact candidate 的结论为 Critical 0 / Important 0 / Minor 0，确认 TextField inset/hover/focus token 路由、ConfirmDialog 水平/堆叠 rail、Alert legacy identity、StandardDialog `font_body`/`font_small` role、Runtime Text 路径、三张 PNG 哈希与 Render18 排除边界一致；scoped tracked/untracked diff checks 通过，verdict 为 READY。该 review 不替代上述 fresh binary 证据，也不把 broad package `exit 124` 改写为通过。

### Coordinator closeout audit（2026-07-16 11:26）

- Layout15 exact closeout manifest 已移除 `fixed-2026-07-16-volumetric-ambient-radiance-module-export.md`；`froxel/mod.rs`、该 fixed record 和所有 Render18 路径均不由本里程碑吸收或提交。共享 Git index 当前为 0 staged paths。
- 旧 package job `e9224849cae24daaa14a9606981f7a23` 已核验为 `released`：`finished_at=2026-07-15T09:40:44.929228+08:00`、`released_at=2026-07-15T09:40:49.270881+08:00`、`live_process_pids=[]`；没有对 Shader06 或当前外部 Cargo 进程执行终止/清理。
- Closeout checker 的 `open_failure_remaining` 仍来自 Tooling01 的 `cargo-pid-reuse-identity-guard`。其修复和来源 upward gate 已实质满足（当前 daemon schema 42，Layout15 fresh managed compile、TextField 5/5、Dialog 8/8、ignored captures 2/2），但 canonical handoff 仍为 `open`；Layout15 不冒用旧 Tooling01 owner Session 执行 Failure return，等待修复责任计划受管回传。
- 父计划已在 coordinator 明确 `maintenance_allowed` 后补入 schema-1 `zircon-workflow` 与 pending 的 `M1.1` slice；`milestone prepare M1.1` 已激活 run `c35dc033c2f2403e9688acf510ae5d6e`。拓扑缺失不再是门禁，剩余 gate 为 exact manifest/review 绑定、Tooling01 Failure return，以及与本 Rust/截图切片相符的 managed validation evidence；不会用无关的 coordinator/web 模板冒充编辑器验证。
- `paint_text/draw/layout/tests.rs` 的 Runtime Text hard-cut fixture 由 `editorui03-shaped-glyph-fixture-fix-20260714` 声明，但当前哈希被历史操作误归属到 Layout15；该路径不在 TextField/Dialog manifest，且不会被本里程碑吸收。候选路径的 lease/current-hash attribution 只会在上述 owner 门禁解除后、紧邻 coordinator closeout 时重新建立。

Plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
Milestone: M1.1
Status: accepted
Files: ["zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field/palette.rs","zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field/surface.rs","zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs/actions/commands.rs","zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs/commands.rs","zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs/content.rs","zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs/identity.rs","zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs/layout.rs","zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs/metrics.rs","zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs/style/colors/border.rs","zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs/style/colors/text.rs","zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs/surface.rs","docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md","docs/zircon_editor/ui/workbench/reference/surface.md","docs/tests/editor/editor-components-fields-900x360.png","docs/tests/editor/editor-components-dialogs-900x360.png","docs/tests/editor/editor-components-workbench-slate-atlas-900x620.png","zircon_editor/src/tests/host/retained_menu_pointer/field_visual_screenshot.rs","zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_text_field/tests.rs","zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs/tests.rs","zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields_tests/paint.rs","zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields_tests/style.rs"]

## Scope delivered

- TextField 统一使用 Starship recessed input surface，并以 hover outline 与 keyboard-focus primary outline 区分交互状态；field atlas 通过真实 painter 采样锁住该层级。
- ConfirmDialog action rail 根据运行时文本测量在水平行与纵向栈之间自适应，正文、标题和 action 保持 token-relative 间距；Alert 保留 legacy identity，StandardDialog 使用 `font_body/font_small/font_body` 角色。
- M1.1 只交付上述 TextField/Dialog 原子与复合切片、对应模块文档和三张 `docs/tests/editor/` 截图；父 M1、S15.4 全项、Runtime Text package/profile 与 Layout15 全计划继续保持 pending。

## Fresh testing evidence

- Windows managed job `f10ce55f61804e7aaa6e66d53b2f6d79` 生成 current-source `zircon_editor` test binary；TextField exact guards 5/5、Dialog exact guards 8/8、ignored captures 2/2 通过。
- `editor-components-fields-900x360.png` SHA-256 `1C05CB159118299D30B9B995F0651FF7389EF382B1291D043161722D20C15E41`；`editor-components-dialogs-900x360.png` SHA-256 `29929C5A47ED94A1315D16C8BAF9F2283128E26BAC3A31941624A12A8CD0EFAA`；`editor-components-workbench-slate-atlas-900x620.png` SHA-256 `BAAF115498884D16B1233F71D67E277809D9E58D1496563DF28AD45613E80184`。
- 三张图只写入 `docs/tests/editor/`，repository/managed target 同名扫描为 0。broad package worker 的真实结果为 exit 124，未计作 package acceptance。

## Review

- 独立只读 review 对 exact candidate 给出 Critical/Important/Minor = 0/0/0，verdict `READY`；确认 Runtime Text 接口、Unreal/Starship 状态层级、响应式 Dialog rail、截图哈希和模块边界一致。
- M1.1 exact manifest 共 22 个路径（含本记录和父计划 workflow maintenance）；Render18/Froxel/volumetric 路径命中 0，不吸收外部 Failure 修复或 foreign staged scope。
