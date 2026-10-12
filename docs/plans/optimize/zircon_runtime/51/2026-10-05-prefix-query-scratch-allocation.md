---
date: 2026-10-05
implementation_status: source_applied
validation_status: native_batch_pending
performance_status: original_acceptance_open
---

# 前缀查询临时集合移除

| 范围 | 状态 | 证据 |
| --- | --- | --- |
| 前缀查询与原 secondary posting 回归 | `source_applied_native_validation_pending` | 独立源码审查、当前路径增量补丁、实际写入与换行核对；格式检查及 diff 检查通过 |

精确路径索引中的 UUID 桶互不重叠。生产插入、删除、更新、重命名和持久化加载都通过原索引准入与重建路径维护这一约束。前缀查询直接遍历匹配桶，先计算准确候选数并预分配结果 Vec，去掉额外 HashSet 的分配及哈希插入。原有最后筛选、AssetUri 排序和无过滤查询分支保持。

保留原测试与已迁移的测试目录；新增前缀查询对照 authoritative rows 的回归，覆盖空前缀、不同 scheme、子资源后缀、相似路径及无结果。没有把返回 Vec 的分配计为零，也没有声明延迟、RSS 或性能验收已通过。

本轮证据位于 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261004/runtime51-final-review-v1/`：`review.json`、`binding.json`、`current-main.patch`、`actual-canonical-patch-apply.json` 和 `newline-reconciliation.json`。补丁 SHA256 为 `722aebcdd78b25b8fec297d91c3ed04875b90c1617f46c8dbea1075afdb45646`。Git 写入 CRLF，两个实际文件的 LF-normalized 哈希均精确匹配独审预期；首次字节差异记录保留，没有重复应用。

普通回归及原 G39、10K／100K／1M listing profile 将与 Core、Editor 一起按批次运行。CPU、RSS、I/O、allocation、P50／P95／P99 及原性能门仍待验收。
