---
related_code:
  - zircon_runtime/src/core/runtime/descriptors
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
tests:
  - twelve production Rust files reviewed
  - one source-level RED to GREEN activation-sort ownership guard added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, deep-DAG counters and startup trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core descriptors逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/core/runtime/descriptors/**` 12/12个生产Rust文件，当前588行/1个inline源码守卫。范围覆盖module/service/plugin descriptor、registry name解析缓存、factory/object owner及module activation拓扑排序。

## PERF-MVP-325：activation sort临时名称复制已止损

`sort_module_activation_order`原先把每个module name clone进HashMap，DFS进入每层又clone String进stack；正常无环排序因此在最终输出之外仍多两组名称owner。新增RED→GREEN守卫让name index借用descriptor `&str`、DFS stack保存usize，仅最终order与真实cycle diagnostic物化String，正常深链临时name clone=0。

剩余算法仍是递归DFS，10k/100k深链有native stack风险；cycle path用stack线性定位，batch调用方还先clone所有完整ModuleDescriptor再排序。最终Runtime02冻结DAG时应迭代Kahn/显式frame排序并缓存order/generation，注册增量只更新受影响边；错误路径按需生成cycle名称。

RegistryName已缓存module/service offsets与ServiceKind，避免resolve时重复split/parse；它仍以owned String作为各descriptor/entry/list的重复owner，统一intern/arena责任继续归PERF-MVP-322。

## 验收要求

对modules/edges各1/100/10k/100k、depth 1/100/10k/100k与cycle首中尾记录name/descriptor clone bytes、hash probes、DFS frames/native stack、sort calls和startup p95/RSS：正常sort临时name clone=0，单generation sort≤1，迭代复杂度O(M+E)且100k深链不栈溢出；duplicate/missing/init-level/cycle path/order parity、current-source Cargo/F0 trace通过前，12文件留在`pending.md`。
