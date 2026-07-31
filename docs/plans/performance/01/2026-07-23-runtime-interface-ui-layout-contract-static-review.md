---
related_code:
  - zircon_runtime_interface/src/ui/layout
  - zircon_runtime/src/ui/layout
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout
tests:
  - zircon_runtime_interface/src/tests/layout_engine_contracts.rs
  - zircon_runtime_interface/src/tests/ui_layout.rs
  - current-source Windows layout tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface UI layout 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/ui/layout/**`当前源 **11/11** 个 Rust 文件、**1,694** 行已逐文件阅读，并反查runtime layout pass、Taffy bridge、slot lookup、virtual window、surface incremental report及Editor diagnostics消费者。interface现有8条engine合同与4条layout合同覆盖backend/fallback、report counts、serde defaults、constraints、geometry、slot、style与virtualization。本轮未修改源码。

## 性能结论

- constraints、geometry、metrics、scroll state与fixed-extent virtual window均为Copy值类型和常数数学，无堆分配；`compute_virtual_list_window`以offset/extent直接算可见范围，是PERF-MVP-262必须保留的O(1)正向基线。
- runtime对每个container记录engine selection时都调用`UiLayoutEngineCapability::taffy_flex_grid_wrap_block()`和`zircon()`；两者分别新建supported-family Vec。随后selection report保存每container一行，incremental merge clone全部未访问旧selection、append changed rows，再用BTreeMap重算fallback counts。该per-node heap/report全量重建补强 **PERF-MVP-263**，不重复编号。
- `UiLayoutStyle`的grid rows/columns为owned Vec，style clone会复制完整tracks；应由compiled style/layout generation共享，继续归 **PERF-MVP-261/274/312**，不能在Taffy compute或diagnostics重复物化。
- `UiSlot`合同本身是小型owned row，但runtime `slot_for_container_child`仍从全局slots Vec线性查parent+child+kind，并被ordering、measure、arrange等多阶段重复调用；继续归 **PERF-MVP-260**。
- interface virtual window只暴露first/last；runtime arrange仍先为全部children建立positions并处理offscreen subtree，继续归 **PERF-MVP-262**。接口不得扩展为每帧全量row DTO来掩盖该问题。
- debug packet与selection report拥有per-node Vec及style-source String，只允许显式诊断capture或changed generation构造；普通F4 frame不得无条件保存全容器明细。

## 既有任务补充设计与验收

1. PERF-MVP-263把backend capability改为静态slice/bitmask或const match，selection hot path零heap；report默认仅聚合counts/首例，完整per-node rows由有界debug gate生成。incremental report按node index原位patch或generation publish，不clone untouched rows。
2. PERF-MVP-261保持一个surface-owned persistent Taffy tree、entity/node mapping和children scratch；style/children/measure只对changed nodes upsert。Bevy `UiSurface`长期持`TaffyTree`、`entity_to_taffy`和scratch Vec，作为本地参考。
3. PERF-MVP-260让slot归属child edge或共享`(parent,child)->slot`索引；一次生成ordered child/layout input供measure/engine/arrange复用。
4. PERF-MVP-262只访问visible+overscan+edge delta；1/1k/100k rows滚动一步，offscreen visits=0，工作不随total rows增长。
5. nodes/containers/slots/tracks 1/100/10k、stable/1% dirty记录capability Vec allocations、selection clones/report rows、slot probes、Taffy create/upsert/compute、style track clone bytes和layout p95；stable generation以上计数为0。保持12条interface合同、fallback语义、RTL/DPI/pixel snapping与arranged geometry一致。

current-source Cargo、规模counter与F4 resize/scroll/layout产品trace未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
