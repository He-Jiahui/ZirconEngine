---
related_code:
  - zircon_editor/src/ui/asset_editor/tree
  - zircon_editor/src/ui/asset_editor/session/palette_state.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
reference_sources:
  - dev/Fyrox/editor/src/asset/item.rs
  - dev/Fyrox/editor/src/plugins/absm/canvas.rs
  - dev/godot/editor/scene/canvas_item_editor_plugin.cpp
tests:
  - UI asset tree inventory 8/8 statically read
  - editor palette-drag performance source contracts 6/6
  - editor performance source contracts 30/30
  - focused rustfmt and diff checks
  - current-source Windows Cargo and product-scale tests pending
doc_type: implementation-evidence
status: incremental_static_complete_dynamic_pending
---

# UI Asset Tree 静态性能审查（2026-07-22）

`zircon_editor/src/ui/asset_editor/tree`当前 **8/8** Rust owner已逐文件阅读，覆盖tree edit、palette drop resolution以及Flow/Grid/Overlay slot目标生成。

## 主要瓶颈

- `update_palette_drag_target`每次pointer move完整执行`build_preview_projection`，再反向线性扫描全部canvas nodes；稳定document/preview generation没有可复用的hit index。
- Grid候选随rows×columns增长，组件/native候选与slot overlays反复物化String、map和Vec。修复前每个候选还会clone整份`UiAssetDocument`并模拟插入，形成`O(candidates × document)`复制放大。
- tree edit的parent/child/node与unique node/control id继续依赖全树DFS；普通命令频率较低，但大document上的连续palette insert/wrap/extract会重复扫描。
- slot semantic匹配会多次normalize同一名称，presentation又从selected target重建overlay；这些应并入document/preview generation的typed target artifact，而不是另建永久局部cache。

## 本轮直接止损

- component/native drag resolution先构建轻量plans，只对当前选中plan执行一次既有模拟验证；6/9/15个候选不再各自clone整份document。
- component选中项复用刚生成的slot overlays，不再二次解析component definition、available slots和完整target Vec。
- imported component只查询一次widget import map；slot occupancy改为`BTreeMap<&str, usize>`借用mount名；target slot map直接move进入plan。

## 参考引擎约束

Fyrox编辑器用稳定`Handle<UiNode>`路由`DragOver/Drop`，释放阶段直接消费UI hit test；Godot Canvas编辑器保留typed drag state并在mouse motion只更新当前drag与请求redraw。Zircon应同样让document/preview generation持有稳定hit/target authority，move只做索引命中、解析selected target和局部presentation patch。

## 动态验收

需补1/100/10k canvas nodes、1/10/100 Grid axes、1/10/1k component slots与125/500/1000 Hz pointer move的preview builds、canvas visits、candidate/overlay builds、document clone bytes、String/map allocations、main-thread p95与RSS。最终stable generation每move preview/document clone=0，hit近O(1)或O(logN)，candidate/overlay物化受可见或chooser预算约束；drop、manual candidate cycle、required/multiple slot、Grid/Flow/Overlay边界、undo/redo与serialized source保持等价。还需current-source受管Cargo与F4产品trace。
