---
related_code:
  - zircon_editor/src/core/settings/mod.rs
  - zircon_editor/src/core/settings/defaults.rs
  - zircon_editor/src/core/settings/io.rs
  - zircon_editor/src/ui/v2_design_tokens.rs
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/extension_host.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs
implementation_files:
  - docs/editor-and-tooling/retained-host-text-preferences.md
  - docs/zircon_editor/ui/retained_host/host_contract/paint_text.md
  - docs/zircon_editor/ui/retained_host/host_contract/paint_theme.md
  - docs/zircon_runtime/script/vm/host_interface.md
  - docs/zircon_plugins/zr_vm_language/host_interface.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- <exact batch paths>
---

# Frameworks06 G7 当前 Owner 文档路径硬切 Batch 10

Status: completed_focused_g7_passed_global_g7_red_rechecked
Date: 2026-07-18
Session: `frameworks06-g7-current-owner-doc-path-hardcut-batch10-20260718`

## 完成项目

- retained-host 文字/主题偏好已继续硬切到当前 settings/design-token owner：`core/settings/defaults.rs` 负责启动选择，`core/settings/io.rs` 负责持久化，`zircon_runtime_interface::ui::design_tokens` 持有模型，`ui/v2_design_tokens.rs` 负责 Editor V2 安装缓存；已删除的 `ui/preferences/**` 不再作为当前架构路径。
- 将 Runtime VM host-interface 文档中的真实 ZrVM backend owner 切到语言插件的 `real_backend/extension_host.rs`。
- 将 ZrVM language 文档中的两条退役 Runtime backend 路径切到插件 crate 的 `extension_host.rs` 与 `host_modules.rs`，没有恢复 Runtime 兼容模块、别名或重导出。
- 同步 live 文档中的当前 owner 描述和可重放 rustfmt/扫描命令，所选文档不再把已经删除的 root 文件描述为现行架构。

## 验证

- 修改前 fresh G7：`445` violations / `96` documents。
- 修改后 fresh `python tools/check_conventions.py --only docs --json`：所选 5 份文档 `0` violations；全库收敛到 `433` violations / `91` documents。
- closeout current-source 重放：所选 5 份文档仍为 `0` violations；并发迁移使全局变为 `464` violations / `118` documents，因此全局 G7 继续明确为 RED，未复用历史总数冒充当前基线。
- 2026-07-29 current-owner 重放：全库为 `678` violations，其中 Frameworks 计划自身仅本记录的 4 条已删除 `ui/preferences/**` 路径；本次硬切到 settings/design-token owner 后，Frameworks 计划命中归零，全局其余 `674` 条继续由各自编号计划承担。
- 三种退役机器路径在本批次 5 份文档中为 `0`；所有新增 owner 路径存在。
- exact-scope `git diff --check` 通过，仅报告仓库既有的 Windows LF/CRLF 提示。
- 独立只读复审：**Critical 0 / Important 0 / Minor 0**；复核 12 条 frontmatter 违规差值、5 份 focused 文档归零、6 个 current owner 存在，以及无 compatibility/alias/shim/旧路径重导出。

## 里程碑判定

本 G7 批次 focused 完成。Frameworks06 M1 和计划 06 继续保持 `in_progress`：2026-07-29 current-owner 重放后全库 G7 仍为 RED（`674` violations / `165` documents），必须按真实 owner 和活动会话范围继续收敛，不能由本批次冒充全局完成。
