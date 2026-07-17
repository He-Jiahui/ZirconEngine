---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - componentized extension workspace pixel tests
  - asset content projector and scrollbar geometry tests
  - current-source Windows Cargo pending
  - 1/100/10000-node paint visited/allocation/damage trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor paint workbench renderer逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`paint_workbench_renderer.rs` + `paint_workbench_renderer/**`共 **102/102** 个Rust文件、**5,127** 行已逐文件阅读，包含componentized scene、dock/floating/pane、menu、native pane、scrollbar、Welcome、tests与root glue。共核对38处`row_data`和6处显式Vec/BTree结构收集。当前源Cargo、产品paint trace和规模counter未完成，因此仍留在`pending.md`。

## 已有正确边界

不可见dock/pane/floating/splitter会早退；viewport image直接借用RGBA；move state未变的damage入口已在上层idle；recent projects最多paint 7行；geometry helpers为常数时间；componentized extension tests覆盖active subtree、legacy sibling、inactive host和search field像素。Renderer已经有host-painter scopes，便于按层采样。没有paint内文件I/O、线程创建或阻塞队列。

## 热点与计划

PERF-MVP-177：componentized window每帧对`workbench_window_nodes`按top/status clip至少完整paint两遍；extension workspace region先`row_data` clone整表并建node/parent map，subtree又clone整表、深copy ids并为每个node沿parent chain新建visited `BTreeSet`，最坏接近O(N×depth logN)，最后transform paint再扫完整表。稳定generation也重复全部结构投影和字符串分配。

同一模式出现在基础pane。Activity asset projector先全扫两遍再paint；Browser projector为grid/table/header/preview各自从头扫，scrollbar重复同样查找，hover又线性找第N行。Welcome为outer/recent/main和8个form/top frame以及header/list分别调用`welcome_node_frame`，一帧约13次全表scan。Hierarchy虽有clip，却对全部scene rows执行`row_data`后才剔除；长menu也为全部离屏row clone DTO并计算文本列。Floating windows、rail、diagnostics overlay仍逐row深clone或collect第二个Vec。`SharedString=String`使这些clone复制文本；theme/text成本分别归PERF-MVP-161/174/156。

局部先用统一行stride计算Hierarchy/menu visible range，并在单次pass中解析Welcome/asset所需frame与extent，消除同帧重复scan。最终EditorUI08在presentation generation提交immutable paint projection：stable node identity、parent/child/subtree membership、control-to-frame、clip/section paint ranges与extension active root；damage stream只访问相交segment。EditorUI01的virtual-row authority提交visible range + overscan，renderer不能另建第二套row truth。Slint以per-item dependency-tracked cache只在property dirty时更新，并在render traversal先`filter_item`/clip再render，证明结构投影和可见性判断应在paint work前收敛。

## 动态验收

在1/100/10,000个workbench nodes、10层extension subtree、10,000 hierarchy/menu/asset rows与13个Welcome controls上记录row DTO clone、string bytes、parent steps、map/set build、visited/painted nodes、text layouts、alloc、damage area与CPU scope。稳定generation的topology/map/set/string build=0；componentized node不因top/status/extension形成三次全表访问；列表visited≤visible+overscan；Welcome control lookup不随无关node线性增长；diagnostics不collect第二份primitive Vec。保持active/inactive subtree、clip/z、scroll/hover、popup、fallback、text focus、floating和pixel parity。
