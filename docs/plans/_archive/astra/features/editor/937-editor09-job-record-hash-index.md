---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09/2026-08-26-job-record-hash-index.md
related_records:
  - docs/plans/astra/features/editor/926-editor09-background-job-hotpaths.md
  - docs/plans/astra/features/editor/936-editor09-submission-preflight-borrowed-batch.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/jobs/system/state.rs
tests:
  - zircon_editor/src/core/jobs/system/state/job_record_hash_tests.rs
---

# Editor937 Editor09 Job Record Hash Index

## 完成列表

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| JobId lookup | `records` 与 `terminal_orders` 使用 `HashMap<JobId, ...>` 提供预期常数时间查找；terminal/eviction 顺序继续由有序 `BTreeSet` 拥有。 | 行为测试覆盖依赖解析、terminal 顺序与有序 eviction 集合；source contract 锁定 HashMap/BTreeSet 双索引边界。 |
| 性能门禁 | 1,024 条记录、4,096 次稳定命中不再走有序树查找；命中路径不新增分配。 | ignored marker `EDITOR09_JOB_RECORD_HASH_INDEX_BENCH_V1` 要求 P95 至少比旧 BTreeMap 路径低 30%；managed Editor09 Release 回执仍待定。 |

## 本地验证

- Editor09 Python 合同批量保持 `68/68`。
- Rustfmt、行为/source contract 已随 Editor09 包测试纳入；未修改 tooling。
