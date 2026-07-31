---
related_code:
  - zircon_editor/src/ui/asset_editor/source
  - zircon_editor/src/ui/asset_editor/palette
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
tests:
  - UI asset source inventory 3/3 statically read
  - UI asset palette inventory 6/6 statically read
  - editor performance source contracts 27/27
  - focused rustfmt and diff checks
  - current-source Windows Cargo and product-scale tests pending
doc_type: implementation-evidence
status: incremental_static_complete_dynamic_pending
---

# UI Asset Source/Palette 静态性能审查（2026-07-22）

`zircon_editor/src/ui/asset_editor/{source,palette}`当前 **9/9** Rust owner已逐文件阅读。source负责source buffer、selection/outline roundtrip；palette负责native/component/reference catalog、placement、slot validation与实例化。

## 主要瓶颈

- `build_source_outline`先收集全部source lines，再对每个document node从头执行block或tree-header/range扫描，复杂度趋近O(nodes×lines)；`source_outline_node_id_for_line`每次按行反选又完整重建outline。Editor07必须发布source-generation outline index，单次parse形成node→range与line interval查询。
- palette catalog在presentation路径仍物化全部native/component/import entries与labels；drag move已复用selected entry，但stable document/registry/import generation仍缺catalog cache。
- `unique_node_id`/`unique_control_id`为每个suffix候选再次调用全树contains/iter，冲突密集时重复扫描；应消费PERF-MVP-305的generation-owned node/control index。
- source buffer为保存语义持有current/saved两份完整文本；大文件typing、undo/replay与debounce需要Editor07记录bytes/RSS，不能用本轮局部修复掩盖。

## 本轮直接止损

- reference conversion参数校验直接查询`component.params`，删除clone全部allowed key的临时`BTreeSet`。
- component/native slot占用计数改为`BTreeMap<&str, usize>`，借用child mount名；required/multiple/default mount语义不变。

## 动态验收

需补1/100/10k nodes、1/100/10k source lines、1/100/10k palette/import/slot entries的outline scans/index builds、tree/catalog visits、key/mount/label clone bytes、typing/selection/drag/drop p95与RSS；stable generation outline/catalog build=0，line lookup近O(logN)，unique-id查询近O(1)。还需current-source受管Cargo、source roundtrip/slot/reference/undo parity与F4产品trace。
