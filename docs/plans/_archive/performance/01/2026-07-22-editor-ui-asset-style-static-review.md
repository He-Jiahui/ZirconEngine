---
related_code:
  - zircon_editor/src/ui/asset_editor/style
  - zircon_editor/src/ui/asset_editor/session/theme_state.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
tests:
  - UI asset style production inventory 12/12 statically read
  - editor performance source contracts 25/25
  - focused rustfmt and diff checks
  - current-source Windows Cargo and product-scale tests pending
doc_type: implementation-evidence
status: incremental_static_complete_dynamic_pending
---

# UI Asset Style 静态性能审查（2026-07-22）

`zircon_editor/src/ui/asset_editor/style`当前 **12/12** Rust owner已逐文件阅读，覆盖inspector fields/semantics、matched rules、declarations、theme summary/compare/cascade，以及theme authoring action/merge/promotion。该模块是Editor UI MVP的样式与主题编辑主链，动态门闭合前继续只记`pending.md`。

## 主要瓶颈

- pane presentation会分别构建theme source summary/details/compare/cascade/helper/refactor，多次扫描相同tokens/rules/imports并物化BTreeMap、declaration block和String labels；stable document没有generation-owned typed artifact。
- matched-rule inspection先DFS整树找selected node，再对全部imported/local selector逐次parse、path match和sort；authoring query仍缺node/style/control索引，回链PERF-MVP-305/307与Editor07。
- active cascade和imported rule helper原对每条imported rule再次线性扫描全部local stylesheets/rules，规模退化为O(I×L)；单项helper/refactor lookup还会深clone选中action。
- batch adopt/replay保持单次document clone和一次apply，没有发现per-item compile/save；但mutation后的full pane projection、theme compare/cascade labels与undo replay bytes仍无规模预算。

## 本轮直接止损

- `theme_state.rs`单项helper/refactor action改为消费临时Vec的`into_iter().nth(index)`，删除目标action字符串二次clone。
- `inspector_semantics.rs`的只读path查询改为borrowed segment walker，常量字段读取不再分配`Vec<String>`；semantic kind直接返回`&str`。
- `can_prune_duplicate_local_theme_overrides`直接检查typed refactor actions，不再先格式化全量labels；`theme_rule_helper_actions`复用同一次`refactor_count`，删除第二轮action扫描。
- imported/active-cascade rule action先建立一次borrowed `(stylesheet.id, selector) -> declaration`索引，并用`or_insert`保留旧逻辑的首个匹配规则语义。

## 动态验收

需补1/100/10k nodes/rules/tokens/imports的tree/rule/action visits、selector parse、path/label/declaration clone bytes、RSS与edit/helper/presentation p95；stable generation的summary/compare/cascade/helper/matched-rule build应为0，单rule/token mutation只失效依赖行。还需current-source受管Cargo、save/undo/redo/import collision parity、F4产品trace与UI像素验收。
