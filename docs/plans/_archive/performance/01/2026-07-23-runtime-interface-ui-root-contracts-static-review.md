---
related_code:
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/picking.rs
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime_interface/src/ui/text.rs
  - zircon_runtime_interface/src/ui/widget.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
reference_sources:
  - dev/slint/internal/backends/qt/qt_accessible.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/SlateStyle.h
tests:
  - zircon_runtime_interface/src/tests/accessibility_contracts.rs
  - zircon_runtime_interface/src/tests/editor_design_tokens.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
  - zircon_runtime_interface/src/tests/ui_painter_style_contracts.rs
  - zircon_runtime_interface/src/tests/ui_theme_contracts.rs
  - current-source Windows UI root contract tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface UI root contracts 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/ui`当前clean root contracts **7/7** 个 Rust 文件、**2,092** 行已逐文件阅读：`accessibility.rs`、`design_tokens.rs`、`mod.rs`、`picking.rs`、`style.rs`、`text.rs`、`widget.rs`。已反查runtime accessibility/theme/input/render与editor preferences/native painter消费者及29条interface合同。外部dirty `focus.rs`、`navigation.rs`和foreign untracked `focus_tests.rs`不在本批，不吸收其内容；本轮未修改Rust源码。

## 性能结论

- `UiAccessibilityTreeSnapshot`拥有全部node/path/name/description/state/actions/children/diagnostics，`node()`每次线性扫描nodes。产品assistive action仍为单target同步构建完整snapshot再查找，AccessKit text position也重复线性查node；精确回链 **PERF-MVP-256/257**，不重复编号。
- `EditorDesignTokens::workbench_dark()`、`UiThemeDocument::dark()`与`to_theme_document()`会重建id/font/variant String及typography/spacing/elevation Vec；`UiThemePalette::dark()`还四次生成整组default surface数组。当前产品默认主要在preferences/theme-registry构造边界，未确认每帧调用；paint接线必须由theme generation长期拥有document/token/family identity，回链 **PERF-MVP-157/251/264**。density token fixed match与painter selector均无分配，后者是Copy/const正向基线。
- `UiTextEdit`同时拥有完整`before`和`after` editable state；`UiWidgetEvent::TextEditChange`再Box该宽事件，ValueChange同时保存previous/current，SelectionChanged保存完整`Vec<UiValue>`。普通编辑正文与多字段mutation放大继续归 **PERF-MVP-265/295**；event/result只应持shared state generation、range和changed-field receipt。
- `UiWidgetBehavior::infer_from_component`在每次Auto解析时对约40个component alias做固定字符串匹配，产品accessibility extract、focus、popup、render与default interaction都会调用。它不是规模N扫描，但被每node/每event重复放大；归 **PERF-MVP-283** 的compiled behavior mask，stable generation不得继续字符串识别。
- `picking.rs`的policy/capture全为Copy值合同；`style.rs`的resolved-state优先级也是const分支，无集合、锁或I/O。该正向基线应保留，不为微小固定分支引入cache。
- Slint Qt backend按value/label/description/focus分别安装`PropertyTracker`，变化后发布对应accessibility event，并提供不会触发整树构建的raw child查询；Zircon采用“property-level dirty + generation node index + OS按需更新”原则。Unreal Slate长期style registry只用于theme owner对照，不复制其类型分表。

## 动态验收

1. accessibility nodes 1/100/1k/10k、depth 1/16/64、changed 0/1/10%，连续10k Focus/Activate/SetValue：记录snapshot builds、node/ancestor/child visits、String/Vec bytes、lookup probes与AccessKit updated nodes；stable generation build=0，action lookup近O(1)。
2. theme presets/typography variants/tokens 1/100/10k、stable 1M lookup：constructor、String/Vec bytes、family/token clone、resolve calls与p95；stable generation constructor/clone/behavior String probes=0，reload仅发布一次新generation。
3. text 1/10k/100k chars与selection 1/100/10k：记录before/after正文owners、widget event bytes、Box/Vec allocation、patch commits与dirty stages；caret-only正文clone=0，每逻辑动作transaction≤1。
4. 当前29条interface合同通过，并增加大树single-target accessibility、stable theme generation、100k widget event与compiled behavior counter；运行current-source Windows Cargo及F4 accessibility/theme/text产品trace。

current-source Cargo、规模counter与F4产品trace未完成，因此本批继续保留在 `pending.md`，不进入 `review.md`。
