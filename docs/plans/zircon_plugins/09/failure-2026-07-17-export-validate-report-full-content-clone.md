---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: export-validate-report-full-content-clone
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/09-export-publishing.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/export_build_plan/export_validate_report.rs
  - zircon_runtime/src/plugin/export_build_plan/export_build_plan.rs
  - zircon_runtime/src/bin/zircon_export_validate/run.rs
tests:
  - 1 MiB/100 MiB generated-content validate-report RSS benchmark
  - compact report schema compatibility test
  - optional content-artifact output parity test
---

# Plugins09：export validate report 深复制完整 generated contents

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：export-build-plan 39/39 Rust 文件逐文件静态审查
- 修复责任计划：`docs/plans/zircon_plugins/09-export-publishing.md`
- 交接原因：report public schema、CLI stdout/file artifact 与 editor consumer 必须一起迁移，不能仅删一个字段。

## 失败现象与复现证据

`ExportValidateReport::from_build_plan` 生成 `ExportValidatePlanSummary` 时深 clone 全部 plan rows；每个
`ExportValidateGeneratedFileSummary` 又 clone `path + purpose + contents`。CLI 随后把 report 序列化成第二份完整
JSON String，可能再 `fs::write` 一次并始终 `println!` 到 stdout。generated contents 随插件/feature 选择和平台
模板增长时，同一文本同时存在于 plan、summary、JSON 与 capture pipe，多倍放大 peak RSS 和进程间输出。

现有测试/生产搜索未发现 consumer 依赖 validate summary 的 `generated_files[*].contents`；但这是已序列化的 public
DTO，删除字段必须按 schema version/consumer compatibility 处理，不能作为局部小修。

## 最低共享层根因

Validate report 混合了“摘要/诊断”与“可物化文件内容”两个数据面，没有 compact metadata projection 或显式
include-contents/artifact 选项，CLI 也同时把相同 JSON 输出到文件和 stdout。

## 架构修复验收

- 默认 report 的 generated file row 仅含 path、purpose、byte length 与稳定 digest；不复制 contents。
- 若调试/CI 需要完整内容，通过显式 flag 输出独立 artifact/stream，report 记录路径与 digest。
- `--report` 与 stdout 契约避免无条件双写大型 payload；编辑器/CI consumer 完成 schema version 迁移。
- 1 MiB/100 MiB synthetic contents 下 peak RSS 与 stdout bytes 接近 compact metadata，和总内容大小解耦。
- diagnostics/fatal/profile/plan summary 的已有字段、排序与 exit code 保持兼容或有明确 hard-cut version。

## 禁止临时方案

- 不得只把 `contents` 换成 `Arc<String>`：serde 仍会复制/输出完整 payload。
- 不得静默删除 public JSON 字段而不迁移 editor/CI consumers。
- 不得压缩后仍默认向 stdout 输出大 blob；摘要与 artifact 必须分层。

## 修复结果与回传

Open state: `待 Plugins09 设计 compact validate report schema 与可选 contents artifact`。
