# Frameworks 02 M2 Current-Source Acceptance

Plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
Milestone: M2
Status: in_progress
Files: ["docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md", "docs/plans/zircon_runtime/frameworks/02/2026-08-02-m2-current-source-acceptance.md", "zircon_app/src/plugins/builder.rs", "zircon_app/src/plugins/groups.rs", "zircon_app/src/plugins/groups/resolution.rs", "zircon_app/src/plugins/tests.rs", "zircon_runtime/src/asset/module.rs", "zircon_runtime/src/asset/module/lifecycle.rs", "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/readiness.rs", "zircon_runtime/src/asset/tests/module_lifecycle.rs", "zircon_runtime/src/builtin/runtime_modules/assembly.rs", "zircon_runtime/src/builtin/runtime_modules/assembly/extension_inputs.rs", "zircon_runtime/src/builtin/runtime_modules/assembly/profile_modules.rs", "zircon_runtime/src/builtin/runtime_modules/assembly/profile_selection.rs", "zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs.rs", "zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs/tests.rs", "zircon_runtime/src/builtin/runtime_modules/assembly/registration_reports.rs", "zircon_runtime/src/builtin/runtime_modules/assembly/target_modules.rs", "zircon_runtime/src/builtin/runtime_modules/core_modules.rs", "zircon_runtime/src/builtin/runtime_modules/ids.rs", "zircon_runtime/src/builtin/runtime_modules/ids/module_id.rs", "zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs", "zircon_runtime/src/builtin/runtime_modules/tests/mod.rs", "zircon_runtime/src/builtin/runtime_modules/tests/profile_modules.rs", "zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs", "zircon_runtime/src/plugin/runtime_profile/defaults.rs", "zircon_runtime/src/plugin/runtime_profile/descriptor.rs"]

> 本记录绑定 2026-08-02 当前源码的 M2 实现清单。当前 `in_progress` 表示实现、静态门与二次审查已完成，但 Windows managed Cargo 和原生 milestone acceptance 仍须由 coordinator workflow 取得；不得把 testing-ready 冒充 accepted。

## Scope Delivered

- Minimal profile 只通过 typed `BuiltinRuntimeModuleId`、唯一 target candidate registry、descriptor dependency closure 和唯一拓扑排序器组装；已删除并禁止 `minimal_profile_runtime_modules`、手写 core module 列表和第二套排序路径。
- `AssetModule` 通过 `AssetModuleLifecycle` 读取真实 `ProjectAssetManager` production generation publication gate；写锁持有期间 `ready()` 返回 false，发布完成后返回 true，poison recovery 与 manager 的全局恢复策略一致。
- `zircon_app` plugin groups 只声明 profile/features 并调用 runtime profile assembly，不再维护并行模块成员清单；测试锁定 Minimal profile 选择与依赖闭包。
- profile dependency closure 的 descriptor cache 缺失分支已从运行时 `expect` 硬切为类型化 `CoreError::MissingModule`；registration structure guard 禁止该生产 owner 恢复 panic 路径。
- 父计划状态已同步为 M2 implementation-complete、second-review-complete、managed-validation-pending；不恢复旧 API、alias、shim 或 fallback。

## Fresh Testing Evidence

- focused owner/static guard：8/8 passed；profile hard-cut source guard：8/8 passed。
- Rust 1.94.1 exact 25-source scoped rustfmt check：passed；`git diff --check`：passed（仅工作树既有 LF/CRLF 提示）。
- 第三轮源码审计补充的 profile-selection typed-error source guard、两文件 Rust 1.94.1 rustfmt 与 scoped diff-check：passed。
- touched source file budget：最大 431 行，0 个超过 1000 行；共享 Git index staged paths 为 0。
- Windows managed Cargo：pending coordinator milestone validation；本记录不声明 package compile 或 focused Rust tests 已通过。

## Review

- 独立二次审查首轮结论：Critical 0 / Important 1 / Minor 0；唯一 Important 是本记录曾把 managed-validation-pending 的里程碑写成终态 `completed`。
- 审查确认 profile 组装只有一个 typed registry/dependency/sorter pipeline，Asset readiness 使用真实 production generation signal，App groups 没有重建模块列表。
- 后续源码审计发现并修复唯一运行时 `expect`；类型化错误与负向 source guard 落地后源码结论保持 Critical 0 / Important 0 / Minor 0。
- 状态 finding 已通过把机器可读状态硬切为 `in_progress` 修复；独立复核确认最终 Critical 0 / Important 0 / Minor 0。
- managed validation 终端前不写 accepted closeout；queued/running 验证由 coordinator wakeup 回传，Session 继续后续可实现工作。
