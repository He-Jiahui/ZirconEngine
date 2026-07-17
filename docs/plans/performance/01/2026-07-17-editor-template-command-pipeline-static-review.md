---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline_tests/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion/**/*.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/list.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
tests:
  - template_node_pipeline_tests clip/dropdown/menu/transform tests
  - render command text/style tests
  - current-source Windows Cargo pending
  - deterministic cache-generation parity test pending
  - 1/100/10000-node conversion/probe/sort/allocation trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template command pipeline逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`paint_template_nodes`的root、`template_nodes*`、`template_node_pipeline*`、`render_commands*`和`render_command_conversion*`共 **61/61** 个Rust文件、**2,874** 行已逐文件阅读；同时追到`zircon_runtime_interface/src/ui/surface/render/{command.rs,list.rs}` **2/2** 个直接转换owner、**473** 行。合计 **63/63** 文件、**3,347** 行。当前源Cargo、产品paint trace和规模counter未完成，因此仍留在`pending.md`，不进入`review.md`。

## 已有正确边界

Template clip与active damage clip无交集时整组早退；单node在specialized dispatch前完成frame/clip交集判断；不可见、frame-only与opacity-zero node不会生成命令。Text run、shaped cluster、decoration、image atlas与fallback路径有行为测试，command draw按z-index和原插入序保持稳定次序。代码已有collect/sort/draw及command-kind profile scope，没有paint内线程创建、阻塞队列或文件I/O。

## 热点与计划

PERF-MVP-178：`UiRenderCommand::to_paint_elements`每次先构造新的`Vec<UiPaintElement>`。带background、image、text、border的command最多产生4个element，而`base_paint_element`为每个element调用`cache_generation()`；该函数每次把包含style、text layout、glyph/cluster、resource与String的完整command执行`serde_json::to_vec`，所以同一个generation会重复计算最多4次，并为每次计算分配完整JSON byte Vec。Editor与runtime renderer均直接调用该入口，debug/parity/list路径也复用同一转换。

Retained host随后把paint elements再次投影为新的`Vec<HostPaintCommand>`，复制text run/line/cluster String和resource key；image opacity小于1时还复制完整RGBA再逐pixel改alpha。每个draw再分配`Vec<(index, &command)>`并stable sort O(C log C)，即使generation与命令次序未变。Template pipeline对全部model rows取得owned DTO；transform路径在此基础上再clone一次宽node。每个可见且未被较早处理的node依次探测5个primary、dropdown、22个secondary handler，之后仍进入material、MUI-X、surface、image、text、popup等fallback链。PERF-MVP-150/156/174分别继续拥有resource bytes、text layout与shared text，避免把这些owner重复塞入host cache。

局部先把cache generation移到单command转换之前，只算一次，并让确定性JSON serializer直接写入FNV state，保持旧JSON字节hash而不生成临时byte Vec。最终Runtime09在render-extract generation提交typed paint element/role；EditorUI08按presentation与damage generation持有compiled、already-ordered paint segments，stable generation不再做第二次转换、handler probing或sort，单node变化只替换对应segment。Slint的`ItemCache`以property dependency只重算dirty item并在render前filter clip；Bevy以typed `ExtractedUiItem`进入prepare阶段并按image连续batch。这两者共同约束Zircon把类型判定、失效与排序移到generation owner，而不是留在每次host draw。

## 动态验收

先增加cache-generation兼容测试：同一representative command的新hash必须等于旧`stable_hash64(serde_json::to_vec(command))`，1与4 element路径均只计算一次且JSON heap bytes=0。随后在1/100/10,000 nodes、1/4 elements per command、纯typed/fallback混合集合上记录element Vec build、serialized bytes、generation calls、host-command build、DTO/text/RGBA clone bytes、primary/secondary/fallback probes、sort allocation/comparisons、visited/drawn segments和CPU scope。Stable generation上述build/probe/sort/clone为0；单node change只重建对应segment。保持cache generation、z与同z insertion order、clip、transform suppression、text/image/dropdown/menu、theme、GPU/Softbuffer pixels与fallback parity。
