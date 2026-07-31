---
related_code:
  - zircon_runtime/src/core/framework/project/export_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest/profile.rs
  - tools/check_conventions.py
implementation_files:
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/zircon_runtime/frameworks/03/2026-07-17-export-profile-runtime-profile-explicit-hardcut.md
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
---

# Frameworks06 G7 ExportProfile 当前 Owner 文档硬切 Batch 15

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-18
Session: `frameworks06-g7-export-profile-current-owner-doc-hardcut-batch15-20260718`

## 完成项目

- 将 Runtime01 技术选型记录中的退役 `plugin/export_profile.rs` 机器路径硬切到唯一现存 `core/framework/project/export_profile.rs`。
- 删除易漂移的历史行号与全仓引用计数，按当前源码明确 `RuntimeProfileId` 必须显式构造、缺失 identity 只产生 fatal plan 诊断。
- 对齐当前依赖事实：zstd 归缓存压缩、zip/`ZipWriter` 归导出归档、tar 未引入；并将 Runtime01 机器状态从过时的 `completed` 修正为正文已声明的 `in_progress`。
- 不恢复 name/target fallback、旧 owner、alias、shim、兼容重导出或第二套 ExportProfile。

## Fresh Testing Evidence

- 修改前 fresh G7：所选文档 `1` 个 missing-path violation。
- 修改后 fresh `python tools/check_conventions.py --only docs --json`：所选文档 `0` violations；共享 current-source 全局快照为 `474` violations / `126` documents，G7 继续保持 RED。
- 所选文档内退役 `plugin/export_profile.rs` 机器路径为 `0`；current owner、explicit constructor 与 fatal-plan 投影路径均存在。
- exact-scope `git diff --check` 通过，staged_total 为 `0`。

## Review

独立只读复审首轮为 `Critical 0 / Important 2 / Minor 0`：一项指出原归档描述遗漏现存 zip/`ZipWriter`，另一项指出 Runtime01 frontmatter `completed` 与正文 `in_progress` 冲突。两项均已按 current Cargo/source/status 事实修正；最终复审为 **Critical 0 / Important 0 / Minor 0**。

## 里程碑判定

本批 focused G7 与独立复审已通过。Frameworks06 M1、全局 G7 和计划 06 均保持 `in_progress`；本批不构成独立里程碑提交。
