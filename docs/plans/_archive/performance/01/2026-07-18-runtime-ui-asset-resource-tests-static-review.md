---
related_code:
  - zircon_runtime/src/ui/tests/asset_resource_refs.rs
  - zircon_runtime/src/ui/tests/asset_resource_resolver.rs
  - zircon_runtime/src/ui/template/asset/resource_ref
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - twelve resource-ref and thirteen resolver semantic tests reviewed
  - one source-level RED to GREEN single-retain guard added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, resource/cache scale counters and F4 trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI asset resource refs/resolver测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`asset_resource_refs.rs` 393行/12测试与`asset_resource_resolver.rs`原始473行/13测试，共2/2个tracked Rust文件；resolver加入1项源码性能守卫后为489行/14测试。范围覆盖typed refs/validation/kind inference/dependency fingerprint/file validation，以及runtime handle/fallback/scheme map/cache/invalidation/dependency report parity。

## PERF-MVP-309：URI batch逐项全cache retain

原`invalidate_uris`在URI循环内对完整BTreeMap cache执行一次retain；U个change和C个cache refs产生U次map traversal/compaction，并在每次比较中解析mapped locator。新增守卫先确认RED，再先折叠requested URI并执行一次retain，守卫转GREEN；首出现URI顺序、primary/fallback/mapped scheme匹配和removed count不变。第56组局部优化把cache结构遍历由U次降为1次。

仍开放：requested URI用Vec线性dedup为O(U²)，单次retain内每reference仍与U个URI比较并可能重复构造mapped locator；cache hit返回完整resolved clone，placeholder恢复diagnostic index扫描历史，diagnostics无界增长。继续归PERF-MVP-309/EditorUI05的reverse index与generation-bounded diagnostics。

## PERF-MVP-308/311：dependency收集与同步文件探测

resource refs fixture最多6 dependencies；compiler从document/widget/style/token/value多轮递归收集并为path/String分配，fingerprint再次收集/排序。`validate_resource_dependency_files`对每primary/fallback同步filesystem探测；测试只用临时目录，不记录calls/bytes/main-thread latency。共享compiled dependency index与异步I/O仍归PERF-MVP-308/311。

## 验收要求

对1/100/10k values/dependencies/cache refs和1/100/1k invalidation URIs记录value/tree visits、path bytes、cache retain passes/entry probes、locator parses/alloc、diagnostic bytes、filesystem calls及resolve/invalidate p95。每batch cache retain passes<=1；最终目标为reverse URI index使entry probes只与命中refs相关。当前源码refs 12项/resolver 14项Cargo、规模counter与F4 icon/font/resource hot-reload trace完成前，两文件留在`pending.md`。
