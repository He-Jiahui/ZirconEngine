---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
resolved_at: 2026-07-14
summary_slug: vm-reflection-catalog-test-support-import-drift
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_plugins/08-zr-vm.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_plugins/08
related_code:
  - zircon_runtime/src/script/vm/tests/support.rs
  - zircon_runtime/src/script/vm/reflection/catalog.rs
  - zircon_runtime/src/script/vm/mod.rs
tests:
  - cargo test -p zircon_runtime --locked
---


# Plugins08：VM reflection catalog 测试支持未同步当前 owner 导入

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：M1 测试阶段 / default-feature runtime gate
- 修复责任计划：`docs/plans/zircon_plugins/08-zr-vm.md`
- 交接原因：`VmReflectionCatalog` 的生产 owner、可见性与 VM host fixture 均属于 Plugins08 M1 unified reflection；Editor02 不拥有 script VM 测试支持模块。

## 失败现象与复现证据

协调器管理的 Windows job `13130c87201b444db8bcb9654a7f118a` 执行：

```text
cargo test -p zircon_runtime --locked
```

lib-test 编译在 `zircon_runtime/src/script/vm/tests/support.rs:193` 报 `E0433`：
`VmReflectionCatalog::default()` 无当前作用域导入。编译器指出当前 public re-export 可由
`crate::script::VmReflectionCatalog` 导入。完整日志位于
`E:\ZirconBuilds\editor02-m1-runtime-default-after-generation-20260714.log`。

## 最低共享层根因

Plugins08 将 VM reflection catalog 落到新的 production owner/re-export 后，script VM test support 仍依赖父模块通配导入隐式带入该类型；当前父边界不再导出这个名字，fixture 初始化与真实 owner 可见性发生漂移。

## 架构修复验收

- 测试支持从当前唯一 production owner/受支持 re-export 显式导入 `VmReflectionCatalog`。
- 不把 catalog 复制回旧父模块、不新增 alias 或通配 re-export 只为测试服务。
- 原 default-feature `cargo test -p zircon_runtime --locked` 不再失败于该 `E0433`。
- Plugins08 的 focused reflection/catalog 测试继续覆盖 catalog 默认构造与 host context wiring。

## 禁止临时方案

- 不新增 test-only catalog stub、旧模块 façade 或重复 registry 真相。
- 不删除 fixture 的 reflection catalog 字段或以 `cfg` 跳过初始化。
- 不把该编译错误归入 Editor02 world-sync 实现。

## 修复结果与回传

- 根因：VM test support relied on a parent glob import after VmReflectionCatalog moved behind the supported crate::script re-export.
- 架构修复：Imported VmReflectionCatalog explicitly from crate::script; no alias, facade, stub, or duplicate registry was added.
- 验证：Managed runtime validation no longer reports the catalog E0433; managed zircon_plugin_zr_vm_language_runtime build/test passed 16/16.
- 回传：Editor02 default-feature gate may resume past the VM reflection catalog support import.
