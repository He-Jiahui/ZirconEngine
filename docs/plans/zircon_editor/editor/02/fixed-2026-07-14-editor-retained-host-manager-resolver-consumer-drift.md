---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: editor-retained-host-manager-resolver-consumer-drift
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
related_code:
  - zircon_editor/src/ui/retained_host/app/assets.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/snapshots.rs
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_editor/src/tests/editing/state.rs
tests:
  - cargo test -p zircon_editor --lib --locked
resolved_at: 2026-07-14
---


# Frameworks05：editor retained-host manager resolver 消费面漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：Editor02 M1 最终 editor consumer 门禁
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：失败来自 Frameworks05 versioned manager handle/use-point resolver 硬切在 editor retained-host 与 editor lib-test 的未完成消费面；Editor02 不拥有 manager service handle 构造与解析契约。

## 失败现象与复现证据

受管 Windows job `3ca02648cc76456ca54b868203e88827` 执行：

```powershell
cargo test -p zircon_editor --lib --locked
```

Render18 E0381 修正后，编译继续进入 `zircon_editor`，最终以 exit `101` 停止。39 条错误中有 38 条归于 Frameworks05：

- `assets.rs` 新增的三个 resolve-at-use helper 没有导入父模块的 `RetainedEditorHost`、`Arc`、`AssetManager`、`EditorAssetManagerContract`、`ResourceManager`，产生 7 条 `E0425/E0405`；
- helper 未编译使 `refresh.rs`、`refresh/snapshots.rs`、`workspace.rs` 的方法解析失效，派生 14 条 `E0599` 与 16 条 `E0282`；
- `zircon_editor/src/tests/editing/state.rs:554` 仍把裸 `Arc<ProjectAssetManager>` 传给只接受 `ProjectAssetManagerAccess` 的 `WgpuRenderFramework::new`，产生 1 条 `E0308`。

完整日志：`E:\ZirconBuilds\editor02-m1-editor-consumer-after-render18-20260714.log`。另 1 条 EditorUI09 私有 pane payload 导入错误已单独交接，不计入本 Failure。

## 最低共享层根因

Frameworks05 已把 retained host 的 manager 字段硬切为 `ManagerServiceHandle` 与 use-point resolver，但 helper owner 文件没有建立当前模块所需的显式导入边界；同时 editor framebuffer 测试夹具没有像 runtime 测试夹具一样建立真实 `CoreRuntime` owner 与 `ProjectAssetManagerAccess`。此前返回的 runtime lib-test failure 没有覆盖 editor consumer，导致 M4 manager hard-cut 被过早声明完成。

## 架构修复验收

- retained-host helper owner 显式导入当前所需类型，三个 helper 成为唯一 resolve-at-use 入口；不得恢复 struct 上长期持有裸 `Arc<dyn Manager>`。
- `refresh`/`workspace` 消费方继续通过 versioned handle 解析，失效 handle 的 typed error 语义保持可见。
- editor rendering 测试建立真实 `CoreRuntime` manager service 与 `ProjectAssetManagerAccess`；不得添加 `From<Arc<ProjectAssetManager>>`、重载构造器或 cfg-only adapter。
- 原 `cargo test -p zircon_editor --lib --locked` 越过全部 38 条 Frameworks05 编译错误。

## 禁止临时方案

- 不恢复 retained host 的旧 `Arc` 字段、不增加兼容 façade、隐式 `Into` 或全局 manager singleton。
- 不用类型标注压制由缺失 helper 引起的派生 E0282；必须修复最低 helper owner。
- 不在 Editor02 world-sync 代码中绕过 editor host 或渲染测试。

## 修复结果与回传

- 根因：Frameworks05 manager-handle hard cut left retained-host use-point helpers without their explicit owner imports, and the editor render fixture still constructed WgpuRenderFramework from a bare Arc<ProjectAssetManager> instead of a runtime-owned ProjectAssetManagerAccess.
- 架构修复：Restored the helper owner boundary with explicit Arc, AssetManager, ResourceManager, EditorAssetManager contract, and RetainedEditorHost imports; rebuilt the render fixture around a real CoreRuntime, RegisteredManagerService, versioned manager handle, and ProjectAssetManagerAccess while retaining the runtime for the access lifetime.
- 验证：Managed Windows job 9a0b6f1cdff144548d082fdd9b5ea636 ran cargo test -p zircon_editor --lib --locked, compiled zircon_runtime and zircon_editor lib-test successfully with none of the original 38 Frameworks05 E0425/E0405/E0599/E0282/E0308 diagnostics, and reached test execution; later functional test failures are separate owner handoffs.
- 回传：Frameworks05 editor consumers now resolve managers at use points through versioned runtime-owned access. The original editor consumer compile regression is closed without Arc adapters, compatibility constructors, or manager singletons.
