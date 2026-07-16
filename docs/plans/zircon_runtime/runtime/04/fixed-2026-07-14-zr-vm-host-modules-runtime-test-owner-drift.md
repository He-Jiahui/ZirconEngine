---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: zr-vm-host-modules-runtime-test-owner-drift
origin_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
fixing_plan: docs/plans/zircon_plugins/08-zr-vm.md
origin_child_dir: docs/plans/zircon_runtime/runtime/04
fixing_child_dir: docs/plans/zircon_plugins/08
related_code:
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge_performance_baseline.rs
  - zircon_runtime/src/script/vm/tests/host_exports.rs
  - zircon_runtime/src/tests/runtime_absorption/script_binding/inventory.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs
tests:
  - cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked
resolved_at: 2026-07-14
---


# Plugins 08：ZrVM host modules Runtime 测试 owner 漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 来源执行切片：Runtime04 migration journal recovery broad-asset regression repair
- 修复责任计划：`docs/plans/zircon_plugins/08-zr-vm.md`
- 交接原因：Plugins08 已把真实 ZrVM backend 硬切到插件 crate 并删除 Runtime 内旧 owner，但 Runtime 内仍有实现感知测试读取已删除文件；这些消费者必须随 Plugins08 owner 迁移一起收束，Runtime04 不应通过跨 crate 文本读取重新建立反向依赖。

## 失败现象与复现证据

Runtime04 先修复迁移 journal 文件名后，受管 Windows job
`d8ee57b2e13949e28870d7c00447df76` 执行：

```powershell
cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
```

lib-test 编译在执行 Runtime04 精确回归前以 exit `101` 停止：

```text
error: couldn't read zircon_runtime/src/tests/plugin_extensions/
../../script/vm/backend/zr_vm_project_backend/real_backend/host_modules.rs
zircon_runtime/src/tests/plugin_extensions/
extension_registry_bridge_performance_baseline.rs:179:9
```

旧 Runtime owner 已删除，当前真实实现位于
`zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs`。同一旧路径还被
`zircon_runtime/src/script/vm/tests/host_exports.rs` 与 script-binding inventory 保留。完整日志：
`E:\ZirconBuilds\runtime04-migration-journal-current-coremin-lib-rebuild-20260714.log`。

## 最低共享层根因

Plugins08 的 crate owner 硬切只迁移了生产实现，没有同步迁移所有实现感知测试和结构清册。
`extension_registry_bridge_performance_baseline` 属于 Plugins11/Plugins08 的真实 ZrVM callback
实现断言，却仍编译在 `zircon_runtime` 的 test tree 中；更新为跨 crate `include_str!` 只会把错误的
Runtime -> plugin source dependency永久化。

## 架构修复验收

- 真实 ZrVM `host_modules.rs` 的 call-site capture/host export 结构断言由插件 crate 自己拥有并通过。
- Runtime test tree 只保留 runtime-neutral bridge/host-export contract，不读取插件实现源文件。
- 所有旧 `script/vm/backend/zr_vm_project_backend/real_backend/host_modules.rs` 测试与清册锚点被迁移或删除。
- 插件受管测试通过，原 Runtime core-min `--lib` 复现不再在该 include 路径失败。
- Runtime04 的 migration journal 精确回归随后从当前源码产物重跑。

## 禁止临时方案

- 不增加旧 Runtime backend compatibility module、re-export、复制文件或占位 `host_modules.rs`。
- 不把 Runtime 测试改为跨 crate `include_str!` 或以 feature/cfg 跳过实现断言。
- 不弱化性能基线、host export 契约或 Runtime04 验收条件来隐藏编译失败。

## 修复结果与回传

- 根因：Concrete ZrVM backend moved to the plugin crate, while runtime tests and structure inventories still read the deleted runtime implementation owner.
- 架构修复：Removed runtime-to-plugin implementation source reads and stale inventories; retained runtime-neutral bridge contracts; consolidated dense call-site and poison-lock implementation guards in the plugin owner without compatibility files.
- 验证：Managed runtime lib-test executable: script::vm::reflection::tests 14/14 and hot_reload_coordinator reflection 1/1; managed plugin test executable 18/18; full runtime library compiled, with broad integration gate blocked only by unrelated AssetMetaDocument and SourceCubemapMipChain consumer drift.
- 回传：Runtime04 may resume past the deleted ZrVM host_modules owner; Plugins08 now owns the concrete backend implementation and its source guards.
