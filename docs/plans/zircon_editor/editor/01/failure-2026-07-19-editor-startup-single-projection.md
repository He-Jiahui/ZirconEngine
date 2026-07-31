---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: editor-startup-single-projection
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/01
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/entry_runner/editor
  - zircon_app/src/entry/builtin_engine_entry.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_editor/src/ui/host
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration/manager.rs
tests:
  - editor startup single project-open counter
  - first-party registration single-build counter
  - GUI and CLI operation startup parity
---

# Editor01：editor启动single projection

## 现象与根因

GUI和CLI operation启动先为capability/session构造`first_party_runtime_plugin_registrations_for_config`，随后first-party bootstrap再次构造同一集合。`editor_entry_config`通过`ProjectManager::open`读取项目插件，retained host的startup session又通过project authority打开同一项目。入口还重复store config、重建builtin catalog并复制descriptor/report。startup artifact没有唯一generation owner，多个阶段只能重新投影。

## 修复验收

- 一次启动产生一个prepared project/session artifact，manifest read/parse/project open每project generation至多1次。
- first-party registrations/catalog/module descriptors每entry generation构造至多1次，由capability、session和bootstrap共享；联动Editor12保持plugin lifecycle失效准确。
- GUI与CLI operation消费同一startup pipeline，错误优先级、diagnostics和module order保持。
- Editor/Dev project规模1/100/1000 plugins记录open/parse/build/clone bytes与F0 wall/p95；结果回传PERF-MVP-427。

## 禁止临时方案

不得在GUI和CLI分别加缓存，不得缓存跨project/plugin generation的owned快照，不得改变启动失败诊断顺序。

## 修复结果与回传

2026-07-23 current-source复核：`entry_runner/**`11/11、1,988行、43 tests、组合指纹`4a6d25...a354`确认GUI与CLI operation仍各自先构造runtime registrations供capabilities/session，再由first-party bootstrap重复构造；GUI open-project仍先`ProjectManager::open`读取plugins。native load-report registration vectors已move/extend，无旧深clone回归。没有current-source Cargo或open/build counter。

Open state: `待 Editor01联动Editor12发布single prepared startup artifact并回传GUI/CLI启动计数证据`。

2026-07-28 Editor12 native registration handoff：`EditorManager::native_editor_plugin_registration_reports` 已能以 native load report 解码 cdylib 的 `SerializedContributionBatch` 并构造 host registration，但标准 GUI 路径 `prepare_editor_startup -> EditorHostRunConfig` 仅传入 `first_party_editor_plugin_registrations_for_config` 的结果，项目 `zircon_plugins/` 中的 native editor contribution 因而从未注册到 host。修复必须使用同一 prepared project root 生成一次 native load report，同时完成 project selection 过滤与 registration materialization；不得为完成 manifest 再做一次 `discover()`，不得重新打开 project。仅 `ProjectManifest.plugins` 中 `enabled` 且支持 `EditorHost` 的 native editor package 可以注入；未列出的 native package 保持 `native_project_selection` 的默认 disabled，空 target list 仍遵循 `supports_target` 的 all-target 语义。Editor12 负责提供这一单次报告/过滤 API，Editor01 只将其追加至已有 host configuration。验收需覆盖 enabled、disabled、非 EditorHost 与无 prepared project，并证明标准入口不再只有 first-party registration vector。

2026-07-28 审查澄清：staged runtime ABI 的 GUI session 只接受 profile、project manifest 和 wake sink，且 GUI source guard 明确禁止回退 `create_linked_with_profile_and_project`；它必须保持 projectless，项目激活仍归 `EditorUiHost`。因此“共享 runtime registration projection”在新版边界中表示 capability 计算、runtime core bootstrap 与传给 gateway 的 capability 共享同一 prepared artifact，而不是把旧 linked-session registration 注入重新引入动态 session。后续 current-source 验证必须确认这条单向投影；不得以修复名义恢复旧 linked runtime 路径。

2026-07-29 current-source implementation: `prepare_editor_startup` 现在是 GUI 与 CLI operation 共用的唯一 prepared artifact owner；`ProjectAuthority::open_project`、`first_party_runtime_plugin_registrations_for_config` 与 `first_party_editor_plugin_registrations_for_config` 在 entry source 各仅出现一次。GUI 仅从 prepared project root 经 `EditorManager::selected_native_editor_plugin_registration_reports` 物化一次 native contribution，并只追加 manifest enabled 且支持 `EditorHost` 的 registrations；entry 不再直接 discover native plugin。两条启动路径均使用 projectless `RuntimeSession::create_with_profile`，project activation 保留给 host。`gui_startup.rs` 已加入 source guards 覆盖上述单次调用、两次 append registration 与旧 linked session 禁止。此为 source-ready 证据，不含 1/100/1000 plugin 产品计数、受管 Cargo 或 current-source independent review。

2026-07-30 validation dependency: R5 的 `entry_runner/editor.rs`、`gui_startup.rs` 与 `first_party_runtime_plugins.rs` 以 `rustfmt --check --config skip_children=true` 通过；对 `entry_runner/editor.rs` 的递归格式检查在未租约的 sibling `editor/tests/cli_operation.rs` 既有格式漂移处停止。该漂移属于 CLI operation test owner，R5 未修改该文件；在其被对应 owner 规范化前，不得把递归 rustfmt 或后续复合门结果归因为 R5，更不得据此声明 Cargo 或 failure 已通过。

2026-07-30 validation dependency resolved: `editor01-cli-operation-format-r1-20260730` 已取得 `editor/tests/cli_operation.rs` 单文件租约并执行 rustfmt；该文件的 `rustfmt --check` 与 whitespace check 通过。该文件仍是历史未跟踪工作树状态，故本修复不产生可从 `HEAD` 归因的 diff；它只恢复 R5 的递归格式前提，不替代 1/100/1000 行为门、source-bound Cargo、独立复审或 failure return。

## 产出记录与时间

| 时间 | 状态 | 产出与证据 | 后续 |
| --- | --- | --- | --- |
| 2026-07-29 CST | 部分修复，未回传 | `zircon_app/src/entry/entry_runner/editor.rs` 已移除硬切后失效的 `core::editor_plugin::EditorPluginRegistrationReport` 导入，改为 `zircon_editor` 正式根导出；`gui_startup.rs` 增加旧模块路径和 `RuntimeSession::create_with_profile_and_project` 禁止守卫。GUI 与 CLI operation 均已统一使用 projectless `RuntimeSession::create_with_profile`，静态守卫为 `legacy_editor_plugin_imports=0`、`projectless_runtime_session_calls=2`；作用域 `rustfmt --check --config skip_children=true` 与 `git diff --check` 通过。 | failure 仍 open：必须继续完成 prepared project 的单次 native registration 投影、GUI/CLI 计数证据及受管 Cargo 验证；不得以本次路径收敛返回 fixed。 |
| 2026-07-29 CST | `OPEN / source-ready single projection` | GUI 与 CLI operation 共享 `prepare_editor_startup`；entry source 的 project open、first-party runtime registration 与 first-party editor registration 各为一次。GUI 通过 prepared project 的 manager-owned native report 追加一次 selected native registration，并按 manifest enabled/`EditorHost` target 过滤；`gui_startup.rs` 已静态锁定一次 projection、两次 append、零 entry `NativePluginLoader` 和两条 projectless runtime-session 调用。 | 未运行 Cargo、独立 review 或 1/100/1000 plugin open/parse/build/clone/F0 计数。failure 保持 open，后续须在 source-bound managed gate 中完成 GUI/CLI parity 与性能证据。 |
