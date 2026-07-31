---
related_code:
  - zircon_runtime/src/ui/tests/asset_contract_spine.rs
  - zircon_runtime/src/ui/tests/asset_dependency_index.rs
  - zircon_runtime/src/ui/template/asset/dependency_index.rs
  - zircon_runtime/src/ui/template/asset/watch_invalidation.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - two contract-spine and nine dependency-index semantic tests reviewed
  - one source-level RED to GREEN borrowed-cascade guard added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, graph scale counters and F4 hot-reload trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI asset contract spine与dependency index测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`asset_contract_spine.rs` 137行/2测试与`asset_dependency_index.rs`原始333行/9测试，共2/2个tracked Rust文件；后者加入1个源码性能守卫后为348行/10测试。覆盖M1 typed contract/migration、正反依赖边、替换/删除/rename/cycle、browser query及watch cascade parity。

## PERF-MVP-310：contract fixture不证明迁移成本

contract spine只用单节点source与legacy fixture锁定typed section保留/默认省略。它不记录schema migrator的Value判型、header/typed parse次数或中间TOML bytes；single-parse schema migration继续归PERF-MVP-310/EditorUI05，无新增重复条目。

## PERF-MVP-309：cascade identity复制与batch重复遍历

原`cascade_invalidation_targets`对每个命中dependent分别为seen、result和queue复制3份String。新增源码守卫先确认RED，再让seen/queue借用index内`&str`并只为最终result分配一次，守卫转GREEN；cycle seed、dedup和BFS输出顺序不变。`apply_watch_changes`仍对batch内每个change各自运行完整cascade，重叠子图重复访问，继续由PERF-MVP-309/EditorUI05的watch generation与dependency DAG预算处理。

## 验收要求

对1/100/10k assets/edges、1/100/1k watch changes及chain/fanout/cycle图记录edge visits、BFS queue peak、identity/output allocation bytes、重复visited edges和invalidation p95。单cascade内部非输出String分配=0；batch重叠图每个edge/generation访问有明确上限。migration另记录parse次数和intermediate bytes。当前源码contract 2项/dependency 10项Cargo、规模counter与F4 hot-reload trace完成前，两文件留在`pending.md`。
