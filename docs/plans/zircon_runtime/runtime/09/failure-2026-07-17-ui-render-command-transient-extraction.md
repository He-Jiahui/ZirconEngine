---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: ui-render-command-transient-extraction
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/ui/surface/render/sliders.rs
  - zircon_runtime_interface/src/ui/surface/render
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders
---

# Runtime09 failure handoff: UI render command transient extraction

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F4 template command pipeline 63文件逐文件静态审查
- 修复责任计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 共同责任：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 交接原因：Runtime09拥有`UiRenderCommand -> UiPaintElement`共享转换与render-extract generation；Editor consumer不得各自修补同一cache-generation和typed element权威。

## 失败现象与复现证据

PERF-MVP-178静态审查确认`UiRenderCommand::to_paint_elements`每次分配element Vec。一个带background、image、text与border的command最多产出4个element，而每次`base_paint_element`都重新调用`cache_generation()`；当前实现把包含style、text layout、glyph/cluster与resource payload的完整command执行`serde_json::to_vec`，因此同一command最多生成4份临时JSON bytes并重复计算相同hash。Runtime scene renderer、editor retained host、debug/parity/list路径均消费这一入口。

PERF-MVP-196补充确认runtime UI slider与editor retained slider都把外部tick/steps/layout值直接cast为`usize`后逐项push quad，没有硬预算或pixel密度上限。Runtime09须在`zircon_runtime_interface::ui::surface::render`发布唯一`MAX_UI_SLIDER_TICK_COUNT`合同，两条consumer在解析与最终loop双重clamp；共享预算不能复制为两个私有常量。

## 最低共享层根因

Cache generation属于完整render command，却在per-element constructor内部计算；转换API又只返回owned Vec，未暴露可由runtime/editor共同复用的generation-owned typed element range。因此consumer只能每次重新展开、clone并排序。

## 架构修复验收

Runtime09先把generation计算移到一次command转换前，并让确定性serializer直接写入hash state，保持现有JSON字节hash兼容且不分配临时byte Vec。随后render extract发布typed paint element/role和generation-owned element range，供runtime renderer与EditorUI08共同消费；不得让两个consumer各自建立不同的command-kind或cache-generation权威。

- representative command的新generation等于旧`stable_hash64(serde_json::to_vec(command))`。
- 一个command无论产出1或4个elements，generation计算次数均为1，JSON heap bytes为0。
- 1/100/10,000 commands记录element Vec build、generation calls、serialized bytes、payload clone与CPU scope；stable extract generation转换build为0。
- 保持paint order、z-index、cache invalidation、background/image/text/border拆分、text decorations、resource key、debug/parity和runtime/editor pixels一致。
- 任意有限/非有限/极大slider tick输入在runtime/editor均产生不超过共享预算且不超过track可分辨columns的commands；0/1不画，2/5/常用值语义与pixels等价。

## 禁止临时方案

- 不得只在Editor host复制一套cache generation或typed element cache。
- 不得移除cache generation、改用地址hash或改变序列化兼容语义来换取速度。
- 不得以无界command/element map隐藏稳定帧重建。

## 修复结果与回传

完成后在本目录写入`fixed-*`或return记录，附current-source Windows Cargo、focused contract tests、规模counter与Runtime/Editor consumer parity证据；在此之前本交接保持open。
