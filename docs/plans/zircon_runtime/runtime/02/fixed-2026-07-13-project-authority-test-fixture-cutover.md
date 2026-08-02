---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: project-authority-test-fixture-cutover
origin_plan: docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
fixing_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
origin_child_dir: docs/plans/zircon_runtime/runtime/02
fixing_child_dir: docs/plans/zircon_editor/editor/09
related_code:
  - zircon_editor/src/core/project/authority.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_save.rs
  - zircon_editor/src/tests/host/manager/support.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
  - zircon_editor/src/tests/host/manager/ui_asset_reference_and_promotion.rs
  - zircon_editor/src/tests/host/manager/ui_asset_reference_and_promotion/theme.rs
  - zircon_editor/src/tests/host/manager/ui_asset_session_preview.rs
  - zircon_editor/src/tests/host/manager/ui_asset_workspace_watcher.rs
tests:
  - .codex/tmp/runtime02_zircon_editor_full_after_coreweak.exe tests::host::manager:: --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib --locked --jobs 1 -- --test-threads=1
resolved_at: 2026-07-13
---


# Editor 09：ProjectAuthority 硬切后 Manager 工程夹具仍把 save 当 create

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md`
- 来源执行切片：Runtime02 service-registry CoreWeak 生命周期上行验收
- 修复责任计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 交接原因：Runtime02 的 service 生命周期精确回归已经通过；Manager 全套测试的剩余失败集中在
  Editor09 正在执行的 `ProjectAuthority` / asset-type hard cut 中。工程创建权已迁给
  `core::project::ProjectAuthority`，但旧 Manager 测试夹具仍直接对不存在的工程根调用
  `EditorProjectDocument::save_to_path`，属于 Editor09 工程/资产测试支持边界，而不是 Runtime02
  service ownership。

## 失败现象与复现证据

当前源码完成 locked Editor lib-test 编译，生成 3153-test 程序。Runtime02 的 Editor lifecycle 四项
精确测试全部 1/1 通过，128 次隔离 Runtime 夹具也 1/1 通过；随后执行：

```text
.codex/tmp/runtime02_zircon_editor_full_after_coreweak.exe tests::host::manager:: --nocapture --test-threads=1
```

自然产生 summary：`66 passed / 17 failed / 3070 filtered out`，线程外部监控为 first 1、peak 36、
last 4，不再呈无界累计。17 项失败中 16 项在调用
`EditorProjectDocument::save_to_path(<尚不存在工程根>, ...)` 时返回
`ProjectManifest(Read: 系统找不到指定的路径)`；另 1 项新建模板在扫描
`assets/shaders/pbr_shader.zmeta` 时因 `asset_kind = "shader"` 与当前枚举只接受 `Shader` 不一致失败。
完整日志：`.codex/tmp/runtime02_editor_manager_suite.stdout.log`、
`.codex/tmp/runtime02_editor_manager_suite.stderr.log`。

## 最低共享层根因

`EditorProjectDocument::save_to_path` 当前已按 Editor09 硬切为“保存已存在工程”：先
`ProjectManager::open`，不再隐式 `ensure_runtime_assets` 或创建 manifest。新的工程身份、模板渲染、
派生目录和 manifest 所有权由 `ProjectAuthority::create_project` 统一承担。Manager 测试支持层尚未迁移
到这一合同，导致多个上层测试重复失败；模板包中的 shader `.zmeta` 还存在一处当前 schema 不可读的
大小写漂移。恢复 save 时隐式创建会破坏已批准的单一 ProjectAuthority 边界。

## 架构修复验收

- 为 Editor Manager 测试提供单一的 ProjectAuthority-backed 工程夹具创建函数；所有需要可打开工程的
  测试先经该 owner 创建，再保存/写入场景或 UI 资产。
- 修正权威 renderable template 中 shader `.zmeta` 的枚举序列化合同，并增加模板创建后
  `ProjectManager::open + scan_and_import` 回归。
- 原复现 `tests::host::manager::` 必须自然返回 0，且完整 Editor lib-test 必须自然产生 summary；不得
  通过过滤失败测试关闭本交接。

## 禁止临时方案

- 禁止恢复 `EditorProjectDocument::save_to_path` 的隐式工程创建、旧 `ensure_runtime_assets`、兼容
  facade 或 create/save 双真源。
- 禁止在 asset-kind 解析器增加只服务测试的大小写 fallback；模板必须写当前权威 schema。
- 禁止逐测试复制 manifest 文件或为 17 个调用点增加特判；迁移共享测试夹具 owner。
- 禁止削弱 Runtime02 的 Manager/full-lib 自然 summary 验收。

## 修复结果与回传

- 根因：Manager 工程测试夹具在 ProjectAuthority 硬切后仍把 EditorProjectDocument::save_to_path 当作不存在工程根的创建入口；renderable template 另有一处非 canonical shader asset_kind。
- 架构修复：新增唯一 ProjectAuthority-backed create_project_with_default_world 测试夹具，所有相关 Manager 场景先经 ProjectAuthority::create_project 建立工程再保存；模板 asset_kind 收口为 Shader，并保留 Runtime root 直到弱 EditorManager 调用完成。
- 验证：当前源码生成的 zircon_editor lib-test binary 执行 tests::host::manager:: --nocapture --test-threads=1，自然结果 83 passed、0 failed、3073 filtered out；renderable template open + scan_and_import 精确测试 1/1 通过。日志 .codex/tmp/editor09-project-authority-manager-suite-shared-current-20260713.log。
- 回传：ProjectAuthority Manager fixture hard cut 已完成，83/83 当前源码回归通过，不恢复 save 隐式创建或大小写兼容解析。
