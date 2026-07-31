---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot
tests:
  - current Virtual Geometry debug snapshot slice 5 of 5 Rust files reviewed, 1016 lines
  - page-size and resident-slot index source regression added
  - page inspection nested-scan gate changed from RED to GREEN
  - scoped rustfmt and diff check passed
  - current-source Cargo, F2 diagnostics trace and RenderDoc capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics Virtual Geometry debug snapshot静态审查（2026-07-18）

## 当前源覆盖

`build_virtual_geometry_debug_snapshot.rs`及其`execution/node_cull/page/support`子文件当前5/5个Rust文件、1,016行已逐文件静态阅读。该批覆盖page inspection、CPU execution projection、selected cluster/visbuffer/hardware records、node traversal/page requests及最终大snapshot装配；新增1条源码回归。

## 直接止损

resident、pending和evictable page inspection原先对每个page id重新线性扫描`extract.pages`取size，evictable又先线性扫resident inspection，规模为`(R+Q+E)×P + E×R`。本轮先以保持“duplicate id取首项”的`BTreeMap::entry().or_insert()`建立page id→size与resident id→slot/size索引，再按原plan顺序生成DTO；输出顺序、missing size=0和fallback slot语义不变。源码门禁先得到两项RED，修改后均GREEN；rustfmt与diff检查通过。

后续`build_runtime_frame.rs`完整审查又删除了visbuffer overlay对snapshot marks的整Vec clone，改为borrowed slice；该局部止损不改变本计划的按需snapshot/retained overlay终态。

## 剩余根因

只要Virtual Geometry extract存在，`build_runtime_frame`就无条件构造完整debug snapshot，未检查任何debug订阅或camera shared-product ownership。正常帧会clone page plan、visibility feedback、instances、dependencies、resident payload、CPU references、hierarchy children，并构造resident/request/evictable/selected/visbuffer/hardware/submission等多组Vec；node cull又在CPU重放完整hierarchy traversal，即使实际render path已有prepared/GPU report。

execution projection对每个draw segment线性找cluster再线性找instance。selected cluster更严重：每个segment ordinal扫描全部clusters，predicate内又为候选entity收集cluster ids、sort/dedup并position；大scene接近selected×clusters×entity-sort。node traversal对每个node id线性扫hierarchy nodes，stats和record路径还另有一套execution/traversal扫描。page索引只修复确定的局部二次方，不是终态。

新增`PERF-MVP-416`要求Render03发布generation-owned `PreparedVirtualGeometryFrameReport`：normal diagnostics-off不构造snapshot/CPU traversal/详情Vec；debug订阅开启时复用实际prepared execution、cull、page和feedback owner，compiled asset提供page/cluster ordinal/instance/node dense indices，详情按页/可视窗口惰性投影。Runtime07按camera slot/shared-product generation发布Arc snapshot，Editor07使用if-newer订阅和虚拟化，不按UI刷新率深clone全量DTO；Render17量化observer overhead。

本地UE仅在`bVisualizeNanite`为真时调用`Nanite::AddVisualizationPasses`，函数内部还要求`VisualizationData.IsActive()`与`EngineShowFlags.VisualizeNanite`；采用的是“可视化明确门控、未启用不创建诊断pass/资源”的原则，不复制其Nanite实现或C++接口。

## 验收状态

静态、源码RED→GREEN、rustfmt与diff门禁已完成。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，新增测试没有current-source结果；本机RenderDoc CLI不可用且无capture。diagnostics off/on、1/8 cameras、1k/1M pages/clusters/nodes的build/clone/visit/alloc/RSS与GPU对拍未完成，继续留在`pending.md`，不进入`review.md`。
