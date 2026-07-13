# ZrVM M5 热替换生产化产出记录

> Owner：[`../08-zr-vm.md`](../08-zr-vm.md) · 日期：2026-07-14 · Session：`plugins-08-zrvm-m2-20260713`

## 状态与完成项目

| 里程碑 | 切片 | 状态 | 证据 |
|---|---|---|---|
| M5 | M5-T1 · `VmStateBlob` v2 | 完成 | `VmStateBlob { schema_version, types, payload }` 已落地；`from_json`/`to_json` 提供完整跨语言 envelope，`from_reflected_objects`/`reflected_objects`/`validate_reflected` 统一校验重复类型、未声明对象类型和重复字段。`state_blob_round_trips_with_schema` 通过。 |
| M5 | M5-T2 · 字段迁移 | 完成 | `VmStateTypeSchema` 直接嵌入 `ReflectTypeRegistration`，没有第二套字段模型；当前名优先、旧名映射、默认值填充、移除字段丢弃和必填字段失败均为 typed error。`schema_change_migrates_fields` 通过。 |
| M5 | M5-T2 · 失败回滚 | 完成 | coordinator 保存旧 state、旧 `VmPluginHostContext` 和四通道/回调表 generation snapshot；失败时清理新世代、以旧 capability/source/root 重新激活、恢复旧 state、精确恢复旧注册。`migration_failure_rolls_back_old_module` 和 load-stage registration retry 回归通过。 |
| M5 | Real adapter protocol | 代码完成，feature 验证归 M4 | 两份 real backend 均硬切到完整 JSON `VmStateBlob` 的 `saveState`/`restoreState`，并读取可选 JSON `stateSchema`。feature fixture 生成带 type hash/type table 的 reflected blob 和统一反射 schema；因 `E:/Git/zr_vm/build` 不存在，本记录不宣称真实 feature 已编译或运行。 |
| M5 | Review · 三轮独立复核 | 完成 | 首轮发现旧 state/context/registration 未精确回滚、blob 不变量分散和公开 API 文档缺失；二轮发现真实路径只有 schema、没有源 envelope；三轮确认上述 P1 全部关闭，无新增 P0/P1。旧公开协议文档随后同步硬切为完整 envelope。 |

## TDD 与 Windows 验证

- RED：加入旧 state restore、旧 capability/package root、load 阶段注册保留和 blob 构造不变量断言后，`script::vm` 为 **82 passed / 4 failed**；失败点与评审问题一一对应。
- GREEN：固定 Windows toolchain `1.94.1-x86_64-pc-windows-msvc`，受管 retained target `e6b9e81a…`，`--locked --offline --jobs 1`：
  - `cargo test -p zircon_runtime --lib --no-default-features --features core-min,script,net-contracts script::vm -- --test-threads=1`：**86 passed / 0 failed / 3967 filtered**。
  - `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime -- --test-threads=1`：在 real-envelope 最终硬切前的默认插件路径为 **11 passed / 0 failed**，doctest **0/0**；最终 fixture/source 通过 scoped rustfmt 与 diff 检查。最终默认插件重跑因 `zircon_runtime` 根 `Cargo.lock` 被其他会话并发改写/共享 target 长时间占用而未取得新的可归属退出码；不将该环境状态表述为测试失败。
- 最终真实 feature 测试未运行：本机 `E:/Git/zr_vm/build` 缺失；此项属于 M4，不降格为默认后端替代验证。

## 结构与规范吸收

- 新生产 owner 最大为 `host_interface/registry.rs` 467 行、`state_migration.rs` 388 行，均低于结构约定软上限；`mod.rs` 仅接线。
- `VmHostInterfaceGenerationSnapshot` 将 rollback 原子性放在注册表 owner 内，没有把四类 map 复制到 coordinator。
- 所有稳定 M5 公共类型、字段、构造、编码/解码、迁移函数和错误均补充 rustdoc；错误保持 typed，只有复合 rollback cleanup 以 `VmError::Operation` 汇总多重失败。
- scoped rustfmt、`git diff --check` 通过；插件结构审计为 `classified-and-clear`，manifest/registration/capability/dist 违规均为 0。

## 明确边界

- `type_hash` 当前是 revision 元数据：源表携带、目标迁移后重写；迁移选择以稳定全限定 `ReflectTypePath` 为主，不以 hash 相等作为 gate。
- 空类型表表示 opaque envelope；非空类型表必须通过 reflected 校验。直接 struct literal/裸 serde 仍可构造未检对象，但正式 real lifecycle 入口强制走 `from_json`。
- M4 仍负责真实 ZrVM external library、collector/root、schema export 与 feature matrix 的实际编译/执行；M5 不替代 M4。

## 模块文档

- [`../../../zircon_runtime/script/vm/state_migration.md`](../../../zircon_runtime/script/vm/state_migration.md)
- [`../../../zircon_plugins/zr_vm_language/state_migration.md`](../../../zircon_plugins/zr_vm_language/state_migration.md)
