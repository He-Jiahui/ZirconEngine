---
handoff_kind: failure
status: open
created_at: 2026-08-16
summary_slug: build-editor-product-staging-unregistered
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/build-editor.ps1
  - tools/session_coordinator/artifact_product_staging.py
  - tools/session_coordinator/artifact_governance.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_artifact_product_staging.py
  - tools/session_coordinator/tests/test_artifact_governance.py
  - tools/session_coordinator/tests/test_migrations.py
  - tools/session_coordinator/tests/test_server.py
  - tools/tests/build-editor.Tests.ps1
tests:
  - "python -B -m unittest tools.session_coordinator.tests.test_artifact_product_staging tools.session_coordinator.tests.test_artifact_governance tools.session_coordinator.tests.test_migrations -v"
  - "Invoke-Pester -Script .\\tools\\tests\\build-editor.Tests.ps1 -PassThru"
  - ".\\tools\\build-editor.ps1 -OutputDirectory D:\\ZirconBuilds\\editor-ui12-aa-20260816-025812"
  - ".\\tools\\zircon-session.ps1 artifact audit"
---

# Session Coordinator 01: build-editor product staging is unregistered

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M6 current-source Editor build and native WGPU visual acceptance
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：失败发生在 Cargo 启动前的 coordinator artifact governance，最低共享原因属于 Tooling 的生产构建包装与 artifact reservation 契约，不属于 UI/RHI 产品代码。

## 失败现象与复现证据

在 schema64 coordinator 和 Tooling commit `f29459ad1` 已加载、预检 `artifact audit` 返回 `unmanaged:[]` 后执行：

```powershell
.\tools\build-editor.ps1 -OutputDirectory D:\ZirconBuilds\editor-ui12-aa-20260816-025812
```

脚本先创建 `D:\ZirconBuilds\mvp-product-inputs-build-editor-466b291c59b9480da25e7fa00be3018a`，随后第一次 `validate-matrix.ps1` Cargo acquire 以 `unmanaged_artifacts_detected` 拒绝该路径；没有 Cargo/rustc 启动。`build-editor.ps1` 的 `catch/finally` 随后删除 staging，复检 `artifact audit` 再次为 `unmanaged:[]`。

现有 `tools/tests/build-editor.Tests.ps1` 使用 stub validator，只断言参数和文件发布，没有让生产 validator 在 staging 已存在时执行 artifact audit，因此 Pester 可通过而真实 wrapper 失败。

## 最低共享层根因

`tools/build-editor.ps1` 在 coordinator 为 production staging 建立 reservation 之前，直接用 `Directory.CreateDirectory` 创建 `mvp-product-inputs-build-editor-*`。schema64 只认识 coordinator-managed fixture/Cargo/artifact 生命周期；生产 staging 没有对应 lease，因而 validator 的 acquire 依法把调用方刚创建的目录判定为 unmanaged。

## 架构修复验收

- production `build-editor` staging 必须在物理创建前获得 coordinator-issued artifact identity/lease，并在成功 rename 或失败清理后精确 release；不得复用仅面向 Pester 的 `mvp-test-fixtures-*` 语义。
- 真实 `tools/build-editor.ps1` 复现必须越过 artifact audit，启动 managed Cargo，成功发布 editor/runtime/assets，失败路径仍须回到 `unmanaged:[]`。
- 增加使用真实 coordinator artifact audit 的集成测试，覆盖 staging 已存在于 Cargo acquire 前这一顺序，不能只用 stub validator。
- UI12 M6 必须在修复返回后重跑正式 build-editor gate；本 handoff 打开期间不宣称该包装脚本通过。

## 禁止临时方案

- 不得把 `mvp-product-inputs-*` 加入全局忽略名单或放宽 unmanaged audit。
- 不得添加静默 fallback、重复 artifact registry、测试专用绕过或 build-editor 单调用点白名单。
- 不得削弱现有 artifact isolation、安全 rename 或 plan acceptance 标准来隐藏失败。

## 修复结果与回传

Coordinator-owned contract is implemented on the current worktree as schema 65.
`artifact staging-acquire` now binds a live PID/creation identity, a server-generated
`mvp-product-inputs-build-editor-<lease>` path and the declared final bundle path
before physical creation. Publication is a two-phase transition: the Coordinator
seals the staging filesystem identity before rename and accepts the final path only
when the same identity moved there. Failed builds can release only after both paths
are absent. Startup recovery accepts an interrupted publish only for that exact
identity; copied, recreated, foreign-owner or mismatched paths lose their exemption.

`build-editor.ps1` consumes only the Coordinator-issued path. Its success sequence is
`acquire -> build -> begin-publish -> root-bound rename -> complete-publish`; its
failure sequence removes the leased staging tree, disposes the deletion handle, then
releases without replacing the primary build error. It does not modify
`validate-matrix.ps1`, reuse `mvp-test-fixtures-*`, or add an artifact ignore rule.

RED evidence was the expected schema64 assertion, unknown CLI/server commands,
missing governance methods, and Pester `13/15` with both managed-validator paths
failing before acquire. GREEN evidence is `35/35` Coordinator Python tests,
`15/15` build-editor Pester tests, Python compileall and `git diff --check`. The
concurrency regression first reproduced `database is locked`; after external
filesystem/process probes moved outside `BEGIN IMMEDIATE`, it passes in 1.275s.

Open state: `Coordinator contract fixed / UI12 product acceptance pending`. The
origin owner must rerun the real managed `build-editor.ps1`, prove Cargo starts,
publish editor/runtime/assets, return `artifact audit` to `unmanaged:[]`, and finish
the native WGPU visual acceptance. This record does not claim those product gates.

### 2026-08-27 current-source regression refresh

The current-source product-staging, artifact-governance, and migration suites passed
`33/33` in 183.469 seconds. This includes the short SQLite transaction regression,
identity-preserving publish/recovery, exact-owner release, and
`test_restart_recovers_all_missing_mvp_fixture_parent_reservations`, which clears all
missing fixture-parent reservations during startup recovery without a prefix
exemption. No product Cargo or WGPU run was started. The Coordinator contract remains
green, while the real UI12 build/publish/audit/visual acceptance above remains open.
