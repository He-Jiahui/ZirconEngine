---
related_code:
  - zircon_editor/src/ui/asset_editor/preview
  - zircon_editor/src/ui/asset_editor/session/presentation_state.rs
  - zircon_editor/src/ui/asset_editor/session/palette_state.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
reference_sources:
  - dev/slint/internal/core/model/repeater.rs
  - dev/Fyrox/editor/src/plugins/absm/canvas.rs
tests:
  - UI asset preview inventory 8/8 statically read
  - editor preview performance source contracts 8/8
  - editor performance source contracts 42/42
  - focused rustfmt and diff checks
  - current-source Windows Cargo and product-scale tests pending
doc_type: implementation-evidence
status: incremental_static_complete_dynamic_pending
---

# UI Asset Preview 静态性能审查（2026-07-22）

`zircon_editor/src/ui/asset_editor/preview`当前 **8/8** Rust owner已逐文件阅读，覆盖preview host/projection、mock fields/entries/suggestions、expression parse/evaluation与binding/state graph。

## 主要瓶颈

- `build_preview_projection`在presentation与palette pointer move重复执行；修复前每条render command为control id全扫document nodes，构造component label时再次扫描，复杂度趋近O(commands×nodes)。本轮降为单次调用内O(nodes)索引，但stable generation仍重建所有owned rows/canvas DTO。
- `build_preview_mock_fields`每次重建全部subject/property/nested entries、suggestions、schema与state graph；state graph全扫document props/bindings，并为expression dependencies重复parse/evaluate/format。
- mock entries为effective/nested values复制完整TOML subtree，suggestion/schema又重复flatten与排序；深collection/object会放大Value clone、path String和RSS。
- expression parser每次物化char/segment/function argument Vec，reference/control-id解析仍依赖document DFS；应在document/mock generation编译typed expression/dependency artifact。
- preview overrides应用需要owned preview document，目前按完整document clone后逐override DFS；真正重编译/host rebuild cadence由session统一解决，不能在preview helper建立第二权威。

## 本轮直接止损

- preview projection单次构建borrowed control-id→node index，command映射和component label共用；click index复用同类索引，删除逐command全树查找。
- mock subject fallback/列表直接检查已迭代node，删除iter_nodes内再次`document.node`的O(N²)模式；override reconciliation每node只查一次document，再校验全部keys。
- reference/function expression删除验证式二次parse；单项mock suggestion使用`into_iter().nth`；recursive tables借用entries排序。
- preview resize借用root ids；entry sort key改为`(priority, &str)`零String分配；bool parse改为ASCII无分配比较。

## 动态验收

需补1/100/10k document/render nodes、props/bindings/overrides/expressions与depth 1/16/64，stable presentation及125/500/1000 Hz drag的projection/mock/state-graph/expression builds、tree/control/path visits、Value/String clone bytes、rows、RSS和main-thread p95。最终stable generation projection/mock/schema/graph build=0，pointer move不重建owned projection，control/node/reference lookup近O(1)，changed override只更新dependency closure；preview geometry、selection、suggestion、expression、binding graph、resize、undo/redo与source roundtrip等价。还需current-source受管Cargo与F4产品trace/像素证据。
