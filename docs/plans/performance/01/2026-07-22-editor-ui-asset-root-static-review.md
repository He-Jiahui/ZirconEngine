---
related_code:
  - zircon_editor/src/ui/asset_editor/*.rs
  - zircon_editor/src/ui/asset_editor/session/palette_state.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
reference_sources:
  - dev/godot/core/object/undo_redo.cpp
  - dev/godot/editor/editor_undo_redo_manager.cpp
tests:
  - UI asset root inventory 11/11 statically read
  - editor UI asset root performance source contracts 2/2
  - editor palette drag performance source contracts 7/7
  - aggregate editor performance source contracts 45/45
  - focused rustfmt and diff checks
  - current-source Windows Cargo and product-scale tests pending
doc_type: implementation-evidence
status: incremental_static_complete_dynamic_pending
---

# UI Asset Root 静态性能审查（2026-07-22）

`zircon_editor/src/ui/asset_editor/*.rs`当前 **11/11** Rust owner已逐文件阅读，覆盖command/contract、document diff、node projection、palette target chooser、presentation、widget promotion、workspace replay、undo stack与value path。

## 主要瓶颈

- `UiAssetDocumentDiff`持有完整target document；每个history entry又可同时保存before/after完整document replay和typed replay commands，undo/redo还在把entry移入对侧stack前clone整个transition。大document与长会话的clone bytes、RSS和延迟会按文档大小乘历史长度增长。
- undo/redo stack与replay journal没有entry、bytes、age或压缩预算；`replay_records`还会一次clone完整历史。普通局部编辑应由typed apply/revert delta成为唯一权威，完整document snapshot只保留显式checkpoint/恢复用途。
- `UiAssetEditorPanePresentation`持有约250个owned字段；`NodeProjection`只缓存document/surface，仍在全局mutex内执行render extract、完整node metadata和DTO projection。稳定generation的pane读取仍可能重复锁、遍历和复制。
- `value_path`每次编辑重新物化chars/segments，widget promotion执行多轮DFS与document clone。后者是低频命令，先保留行为；前者应随typed path artifact与generation cache统一治理。
- 本地Godot参考以method/value do/undo delta组成action并按history路由，没有为普通属性变化永久保留每步完整document；该模型支持Editor03收敛到typed inverse delta和稀疏checkpoint。

## 本轮直接止损

- `UiAssetHistoryReplay::apply_to_document`在没有document commands时直接应用完整replay或返回no-op，避免source/selection-only及replay-only路径先clone当前document。
- palette target chooser消费上一份chooser并原位返回sticky/mismatch状态，删除pointer move时完整chooser/resolution clone。
- node projection借用root ids，并将已拥有的render-extract command text移入临时信息，删除无意义复制。

## 动态验收

Editor03按PERF-MVP-563补1/1k/100k history entries、document 1KiB/1MiB/100MiB、普通属性/结构/selection/source/replay-only edit，记录document owners、clone bytes、stack/journal entries/bytes/oldest age、apply/revert/main-thread p95与RSS。普通typed edit完整document snapshot=0，undo/redo transition clone=0，历史内存有entry+bytes+age硬预算，compact status O(1)，detail显式分页；checkpoint恢复、失败回滚、source roundtrip与redo截断等价。

EditorUI08继续以generation共享最终node/pane projection：稳定generation build=0、全局projection lock不随consumer数增长，changed node接近delta；补1/100/10k nodes、1/10 panes、stable/1% mutation下的node visits、DTO/String clone bytes、lock wait/hold、RSS与F4/RenderDoc像素证据。当前源码合同与rustfmt只能证明直接止损，受管Cargo及产品规模门未完成，因此不得进入`review.md`。
