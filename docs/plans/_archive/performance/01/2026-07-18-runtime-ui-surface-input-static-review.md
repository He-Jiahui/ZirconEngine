---
related_code:
  - zircon_runtime/src/ui/surface/input
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/slint/internal/core/input.rs
  - dev/Fyrox/fyrox-ui/src/text_box.rs
tests:
  - eight source-level RED to GREEN performance guards passed
  - rustfmt check and scoped git diff check passed
  - current-source Windows runtime lib test running through shared Cargo coordinator
  - route/effect/editable/IME scale counters pending
  - F4 product input and IME pixel trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI surface input逐文件性能静态审查（2026-07-18）

## 范围与覆盖

本批逐文件完整阅读`zircon_runtime/src/ui/surface/input/**`全部57/57生产文件：顶层routing/dispatch/keyboard/pointer/text/timer等33/33，`editable_text` helper 3/3，effect helper 11/11，state 7/7，`text_keyboard` helper 3/3。至此`surface`当前批128/128，整个`ui`累计315/783。

静态风险扫描命中90处clone、9处collect、23处format、96处to_string、0处sort、0处lock/thread spawn，并确认1处输入handler同步刷新完整render extract。高clone文件集中在route policy、pointer reply、text pointer、editable mutation、pointer与effect/host request；这些命中已逐项阅读，不把测试fixture或必要最终所有权仅凭模式计为问题。

## PERF-MVP-293：默认输入诊断复制完整路由

route policy、authority与steps在每个事件上持续生成多份owned route/DTO/String；keyboard/text结束还clone完整event才能同时借用event并修改result。pointer/navigation specialized trace原先先构造generic trace再整体覆盖。本轮RED→GREEN把policy fields与trace projection拆开，specialized path不再做被覆盖工作。

EditorUI01需让dispatch持有单一shared route artifact，release默认仅保留轻量policy/target/handled summary；完整stage、route steps与format notes由显式诊断capture按entry+byte+age预算采集。UE Slate `FEventRouter`按routing policy消费现有path，详细input broadcast放在debug guard内；Zircon不应让调试投影成为release输入固定成本。

## PERF-MVP-294：effect payload与结果分类多份所有权

reply、applied、host/component/rejected会分别持有或复制effect及其String/clipboard/drag payload。text-result merge原先另建local→merged index Vec，再为每个分类结果线性find，最坏O(E²)。本轮RED→GREEN改为以append offset常量rebase有效local index，删除临时表和线性查找。

EditorUI01联动runtime interface应发布单一effect arena或Arc artifact，所有分类结果只持stable index+status，host边界消费或共享同一payload。必须覆盖invalid local index、rejected effect、mixed host/component effect与ABI/serde兼容，不能只优化空payload。

## PERF-MVP-295：editable状态重建与八次属性事务

每次键盘/指针编辑从TOML metadata重建并复制完整text/composition state，随后value、caret、selection与composition最多8项各自调用property mutation，每项独立binding、dirty与diagnostic；change/submit event又复制正文。constraint/filter与semantic key此前还为短token分配normalized String。

本轮RED→GREEN让navigation/semantic keyboard与constraint token使用borrowed ASCII compare，并让filter借用TOML String。EditorUI03仍需持久化editable state，以共享正文+ranges表达caret/selection/composition，并用一次`TextStatePatch`只提交changed fields。Fyrox TextBox把formatted text、caret和selection作为持久状态按实际变更invalidate，可作为避免每event从声明metadata恢复全状态的参考，不照搬其布局算法。

## PERF-MVP-296：IME输入同步全树extract

有IME owner时，每次keyboard/text/preedit/commit都会在输入handler同步`refresh_render_extract_for_current_tree()`，再线性扫全部render commands找target layout；surrounding text复制完整committed正文，fallback cursor/composition从字符串头扫描。原lookup还clone整份layout/style，本轮RED→GREEN已改为借用当前extract中的对象。

EditorUI03需联动EditorUI08/Text09发布node→text-layout handle和text generation，输入只读取同generation索引，不得同步触发全树extract。长文本surrounding/composition用共享source+ranges，在platform请求边界才物化；normal dirty pipeline负责增量更新布局。

## PERF-MVP-297：高频state使用String key、线性容器与无界生命周期

analog control每event重复normalize并以String key维护map；popup/tooltip/typeahead/drag owner存在多处线性查找或缺少统一entry/byte/age预算，timer result复制event与diagnostics。pointer capture原先所有event都clone整map，owner clear又先collect id Vec；本轮RED→GREEN把map snapshot限制到Up/Cancel并以retain原地清理owner。

EditorUI01联动Runtime12应使用typed/interned control id、indexed popup/timer/drag owner state和明确生命周期预算。mouse move/analog可按frame合并latest value/delta，但press/release/cancel与capture边沿必须lossless。Slint `MouseInputState`持有当前item stack/grab并在遍历时以abort result停止路由，可参考“单一活动路径+早停”，不能继续用多份诊断路径替代runtime state。

## 责任计划与验收

EditorUI01收到route diagnostics、effect ownership与input-state lifecycle handoff；EditorUI03收到editable transaction与IME full-extract handoff，并联动EditorUI08、Text09、Runtime11/12。以route depth 1/16/64、effects 1/10/1k、UI nodes 1/100/10k、text 1/10k/100k chars、125/500/1000 Hz pointer/analog及连续1M事件记录clone/alloc bytes、route/stage visits、effect owners、property transactions、extract calls、command scans、state entries/bytes/age与CPU p50/p95/p99。current-source Cargo、F4 workbench输入/IME产品trace、像素与规模counter完成前继续留`pending.md`。
