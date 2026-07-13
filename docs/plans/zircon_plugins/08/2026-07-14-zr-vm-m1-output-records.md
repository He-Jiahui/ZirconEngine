# ZrVM M1 反射注册表统一产出记录

> Owner：[`../08-zr-vm.md`](../08-zr-vm.md) · 日期：2026-07-14 · Session：`plugins-08-zrvm-m1-20260714`

## 状态与完成项目

| 里程碑 | 切片 | 状态 | 证据 |
|---|---|---|---|
| M1 | M1-T1 · `ZrReflect` derive 与统一注册契约 | 完成 | 新增 workspace proc-macro crate `zircon_reflect_derive`，支持 struct/enum、字段/虚拟字段访问器、组件/资源/序列化/editor/remote/script visibility 属性和 dense `u32` 字段槽。`zircon_runtime_interface::reflect` 新增 `ZrReflect`、`ZrReflectValue`、独立 `ReflectScriptVisibility` 与类型级文档；脚本宿主描述器改为由 `ReflectTypeRegistration` 权威投影，ABI projection 不再维护第二套字段元数据。 |
| M1 | M1-T2 · 内置手写反射硬切 | 完成 | `scene/reflect/fixed/**` 已删除且无兼容模块、别名或空壳；普通内置组件统一走 `derived_component_registration::<T>`，Hierarchy/ActiveInHierarchy 只保留 World-owned 不变量适配；派生写入经 `World::insert` 回灌 change detection。Runtime 15 旧路径守卫已按最终派生反射边界修复并回传为 [`fixed-2026-07-14-derived-reflection-hard-cut-guard.md`](fixed-2026-07-14-derived-reflection-hard-cut-guard.md)。 |
| M1 | M1-T3 · VM 类型反向注册 | 完成 | `TypeRegistry::register_vm_type` 校验 plugin owner/id/component backing，把统一反射字段完整投影为 `ComponentTypeDescriptor`，再接入动态组件适配；`vm_type_round_trips_as_dynamic_component` 通过。 |
| M1 | M1-T4 · dense call site | 完成 | `ScriptCallTable` 在模块加载期把类型路径/字段名解析为稳定 `type_slot/member_slot`；运行期只调用 `ReflectComponent::{read,write}_field_by_slot`。测试同时安装 named 与 dense callback，确认运行期 named dispatch 计数保持 0。 |
| M1 | 结构约定与评审问题吸收 | 完成 | 反射 owner 按 interface / derive / runtime adapter / plugin call-site 分层，`mod.rs` 仅接线；所有新增生产文件低于 800 行，最大 `script.rs` 为 678 行。外部输入保持 typed error，向量有限值检查和层级写入使用直接分支；旧 `fixed/**` owner 的恢复由结构测试锁死。 |

## TDD 与 Windows 验证

- `zircon_reflect_derive`：Windows `validate-matrix` build/test 通过，**5 passed / 0 failed**；字段名与字段槽访问器 token 均有覆盖。
- `zircon_runtime_reflection_macros`：Windows build/test 通过，**10 passed / 0 failed**；宏生成的统一 registration、ABI projection 和 fallible module descriptor 路径通过。
- `zircon_runtime_interface`：Windows build/test 通过，**244 passed / 0 failed**；script visibility 与类型级 documentation 的 serde/契约覆盖通过。
- `zircon_plugin_zr_vm_language_runtime`：Windows build/test 通过，**13 passed / 0 failed**；`call_site_resolution_happens_once` 与 `runtime_calls_use_dense_slots_without_field_name_dispatch` 精确测试均通过。
- `zircon_runtime`：`validate-matrix -Package zircon_runtime -SkipTest` 在受管兼容池 `4069e384…` 以退出码 0 完成；覆盖最终源码的新测试二进制进一步得到：
  - Runtime 15 原复现：**1 passed / 0 failed / 7965 filtered**；
  - `scene::tests::ecs_reflect`：**61 passed / 0 failed / 7905 filtered**；
  - VM 反向注册：**1 passed / 0 failed / 7965 filtered**；
  - `script::vm::tests::reflection_docs`：**5 passed / 0 failed / 7961 filtered**。
- TDD 修复轨迹：最终 Runtime 回归前，`ecs_reflect` 曾为 **59 passed / 2 failed**；最低层原因分别是 source guard 将 dense helper 名称误识别为旧 named helper，以及层级不变量调用被格式化为多行链。helper 明确命名为 `read_dense_slot` / `write_dense_slot`，层级写入改为直接 `match world.set_parent_checked(...)` 后，向上 61 项全部转绿。

## 审计与文档同步

- `python tools/audit_plugin_structure.py --json`：`m1_gate_status = classified-and-clear`；manifest、registration、capability、distribution、skeleton migration debt 违规均为 0。
- failure handoff validator：**87 artifacts / 0 errors**；failure graph audit 仅保留 3 个既有跨计划 cycle，与 Plugins 08 M1 无关。
- scoped `git diff --check` 通过，仅有仓库既有 LF/CRLF 提示；生产文件预算均低于结构约定软上限。
- 模块文档已同步：[`../../../zircon_runtime_interface/reflect.md`](../../../zircon_runtime_interface/reflect.md)、[`../../../zircon_runtime/scene/reflect.md`](../../../zircon_runtime/scene/reflect.md)、[`../../../zircon_runtime/script/vm/zr_vm_host_reflection.md`](../../../zircon_runtime/script/vm/zr_vm_host_reflection.md)、[`../../../zircon_plugins/zr_vm_language/runtime.md`](../../../zircon_plugins/zr_vm_language/runtime.md)。各文档保留 machine-readable related-code / implementation-files / tests / plan sources 头。

## 明确边界

- M4 仍待真实后端接通：2026-07-14 验收时 `E:/Git/zr_vm/build` 不存在，因此本记录不宣称 `real-zr-vm` external library、collector/root 或 feature matrix 已编译运行；这不降低 M1 的默认后端与统一反射验收。
- 全库 output-record audit 仍报告 6 项其他计划既有问题（Editor 5 项、Plugins 05 Navigation 1 项）；Plugins 08 的 `08/` 目录未超过十记录限制，本记录未越权迁移其他 owner 的证据。
- 共享 main 工作树包含大量其他会话改动与 staged 文件；`close-session-goal-milestones` 把 foreign staged scope 设为硬停止条件，因此 M1 的代码/测试切片已完成，但服务级 milestone commit 尚未执行。没有手工 stage、commit、push 或 finalize，也没有吸收或取消暂存外部会话内容。
