---
handoff_kind: fixed
status: fixed
closeout_status: accepted
created_at: 2026-07-17
resolved_at: 2026-07-29
summary_slug: export-validate-report-full-content-clone
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/09-export-publishing.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/09
plan_link_mode: child_record_only
related_code:
  - Cargo.lock
  - Cargo.toml
  - docs/plans/zircon_plugins/09-export-publishing.md
  - docs/zircon_runtime/plugin/export_build_plan.md
  - tools/zircon_export/cli.py
  - tools/zircon_export/pipeline_report_source_template_validate_schema.py
  - tools/zircon_export/pipeline_report_validate_stage_schema.py
  - tools/zircon_export/source_template.py
  - tools/zircon_export/source_template_contents_artifact.py
  - tools/zircon_export/source_template_generated_project.py
  - tools/zircon_export/tests/export_test_support.py
  - tools/zircon_export/tests/fixtures/source_template_contents_artifact_v1.json
  - tools/zircon_export/tests/test_compile_host_source_template.py
  - tools/zircon_export/tests/test_pipeline_report_source_template.py
  - tools/zircon_export/tests/test_pipeline_report_source_template_schema.py
  - tools/zircon_export/tests/test_source_template_contents_artifact.py
  - tools/zircon_export/tests/test_source_template_generated_files_gate.py
  - tools/zircon_export/tests/test_source_template_stage_validate_gates.py
  - tools/zircon_export/validate_stage.py
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/bin/zircon_export_validate/args.rs
  - zircon_runtime/src/bin/zircon_export_validate/error.rs
  - zircon_runtime/src/bin/zircon_export_validate/run.rs
  - zircon_runtime/src/plugin/export_build_plan/export_validate_report.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_runtime/tests/plugins09_export_validate_report.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --test plugins09_export_validate_report --bin zircon_export_validate --locked --jobs 1 --color never -- --nocapture --test-threads=1
  - python -m unittest tools.zircon_export.tests.test_compile_host_source_template tools.zircon_export.tests.test_pipeline_report_source_template tools.zircon_export.tests.test_pipeline_report_source_template_schema tools.zircon_export.tests.test_source_template_contents_artifact tools.zircon_export.tests.test_source_template_generated_files_gate tools.zircon_export.tests.test_source_template_stage_validate_gates
---

# Plugins09：export validate report 完整内容深复制收口

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：export-build-plan 39/39 Rust 文件逐文件静态审查
- 修复责任计划：`docs/plans/zircon_plugins/09-export-publishing.md`
- 交接原因：report public schema、CLI stdout/file artifact 与 Python SourceTemplate consumer 必须原子硬切，不能只删除一个字段。
- 生命周期键：`export-validate-report-full-content-clone`

## 失败现象与复现证据

旧 `ExportValidateReport::from_build_plan` 将每个 generated file 的 `path + purpose + contents` 深复制进 summary；CLI 随后再次序列化完整 JSON，并在写 report 时仍默认输出 stdout。同一 payload 因此同时存在于 build plan、summary、JSON String 与 capture pipe，generated contents 增长会直接放大内存与输出。

2026-07-23 的 job `fb78c1d786194147b9ee801dd1825a7a` / run `5e89dc785f6147b7a61c4deaf6e14878` 只覆盖旧 4-test 源码，早于最终 schema、artifact path、输出句柄与 Python consumer 修复；该 GREEN 已明确降级为历史诊断，未用于本次验收。

## 最低共享层根因

Validate report 把摘要/诊断数据面与可物化 generated contents 数据面混在同一 public DTO 中，CLI 又缺少显式 contents artifact 与 stdout policy。Python consumer 直接读取旧 report contents，使 Runtime schema、CLI 输出和消费端不能独立演进。

## 架构修复验收

- 默认 report 硬切到 schema v2；generated file row 只包含 `path`、`purpose`、`byte_length` 与稳定 SHA-256 `content_digest`，不再携带或兼容读取 `contents`。
- 完整内容只通过显式 `--contents-artifact` 输出 schema v1 artifact；report 记录 artifact 的绝对路径、字节数与 SHA-256 digest。
- `--report` 默认抑制 stdout，只有无 report 或显式 `--stdout` 才输出；report/artifact 在 truncate 前冻结实际打开句柄身份，拒绝 hard-link、symlink 与父目录别名竞态。
- Python SourceTemplate consumer 严格验证 report v2、artifact v1、长度/digest/行集合与未知 root，不保留旧 report contents fallback。
- 1 MiB/100 MiB regression 证明默认 report 不包含 generated contents，序列化大小小于 payload 的十分之一；本门未把结构/输出规模回归虚报为直接 peak-RSS 采样，Performance01 更大规模性能矩阵可继续独立记录进程指标而无需重开本 clone 生命周期。
- fatal diagnostics、profile、plan summary、排序与 exit code 保持现有 hard-cut 合同；没有 alias、shim 或双 schema fallback。

## 修复结果与回传

- 根因：默认 validate report 把摘要/诊断与 generated contents 混在同一 DTO，CLI 又无条件形成完整 JSON/stdout 副本，Python consumer 因旧 `contents` 字段与 Runtime schema 耦合。
- 架构修复：report schema v2 只发布 compact metadata，完整内容硬切到显式 schema v1 artifact；CLI 冻结输出句柄身份并实施 stdout policy，Python consumer 严格校验双 schema 且不保留旧 fallback。
- 验证：Python consumer/CLI 87/87、scoped rustfmt/diff-check、独立复审 `C0/I0/M0/Minor0` 与 current-source Rust 12/12 全部通过；snapshot `1226` 运行前后 29/29 零漂移。
- 静态证据：snapshot `1203` 的 exact29 实现快照独立复审为 `Critical 0 / Important 0 / Moderate 0 / Minor 0`；Python consumer/CLI 相关套件 87/87、scoped rustfmt 与 diff-check 通过。
- current-source 证据：snapshot `1226` 在运行前请求 `a7f3dd4be78e4aa694c0829e2694d715` 与运行后请求 `bef0674d848f49b988b09fb28e9bd383` 均为 29/29 零漂移。
- 受管 Rust 门：reservation `8a0cb98da01547e8b6a0e9bbef1cef75` → job `a6b68b4c89bc4bfc83b3eea8a1733131` / run `33142540200d4c1a9f7eea70aa8839eb` natural released exit0/no PIDs；build 37m08s，bin 10/10、integration 2/2，总计 12 passed / 0 failed。集成门明确覆盖 compact metadata 与 1 MiB/100 MiB 默认内容剥离。
- 回传：canonical fixed 记录已迁入来源 Performance01；Plugins09 只保留 return 摘要。PF-M1 其他 catalog generation/profile fingerprint 与整体性能矩阵仍按父计划独立推进。

## 禁止临时方案

- 不得恢复 `generated_files[*].contents`、Arc 包装、旧 schema fallback 或隐式 artifact。
- 不得在 `--report` 时恢复无条件 stdout 双写。
- 不得以字符串路径比较代替已打开文件身份，也不得在校验后重新打开并失去竞态保护。
