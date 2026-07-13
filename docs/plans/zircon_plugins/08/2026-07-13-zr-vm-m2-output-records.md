# ZrVM M2 接口注册四通道产出记录

> Owner：[`../08-zr-vm.md`](../08-zr-vm.md) · 日期：2026-07-13 · Session：`plugins-08-zrvm-m2-20260713`

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M2 | M2-T1 · `VmCallbackHandle` 与世代失配重解析 | 完成 | 2026-07-13 | runtime-neutral `VmHostInterfaceRegistry` 以 package slot + dense module/function slot + generation 保存回调；`VmPluginManager::invoke_callback` 在调用前按 active generation 重解析并刷新句柄；`stale_generation_resolves_to_new_function` 覆盖 mock backend reload。句柄额外携带 `slot`，用于消除跨包 module/function 碰撞。 |
| M2 | M2-T2 · system / bt_node 通道与 capability gate | 完成 | 2026-07-13 | `runtime.script.extension.system`、`runtime.script.extension.bt_node` 先鉴权后注册；FixedUpdate/Update/Last 三个固定 dispatcher 进入 runtime scene schedule 并声明 conservative world access，三锚点同时由 descriptor 投影到生成的 `plugin.toml`；BT descriptor 与 callback 对 AI adapter 发布；`vm_registered_system_enters_schedule_conservatively`、`vm_bt_node_executes_in_tree`（mock callback leaf）和全通道拒绝测试已落位。 |
| M2 | M2-T3 · RPC / editor operation 通道 | 完成 | 2026-07-13 | `VmRpcHandlerRegistration` 与 `VmEditorOperationRegistration` 进入同一 versioned registry；RPC 直接保存 Net 契约已有的 `RpcPayloadSchema`/`ReflectSchemaRequest`，没有第二套 ZrVM schema，三段式 editor operation 在注册期校验；未授权返回 typed `CapabilityDenied`，授权 descriptor 仅从 active slot 发布。具体 Net/Editor consumer adapter 仍由各自 owner 里程碑承接，runtime 不反向依赖插件。 |
| M2 | Testing · Windows 分层验证与结构审计 | 完成 | 2026-07-13 | 受管 Windows pool `020c7c…` 构建的 runtime 测试二进制：`script::vm::tests::host_interfaces` 4/4、`script::vm::runtime::hot_reload_coordinator::tests` 6/6；`cargo check -p zircon_runtime --features backend-zr-vm,script --locked --offline --jobs 1` 通过（14m45s）。主工作区 `zircon_plugins/Cargo.lock` 保持外来改动原样，改用 Cargo 独立 `--lockfile-path` 从当前 manifests 离线生成锁后仍以 `--locked --offline` 验证当前共享源码：default 9/9、`backend-zr-vm` 12/12；真实后端覆盖四通道激活、generation 2 热重载、回调执行与卸载清理。`audit_plugin_structure.py --json` 为 `classified-and-clear`，manifest/capability/compat-shim/dist-boundary 均 0。 |

全局 output-record 审计仍报告 5 个本会话范围外的 Editor 计划既有违规（Editor 01 record 数量与 Editor UI 01/10/11/index notice）；本记录未被审计器列为违规，本会话未跨域修改这些文件。

## 架构与参考实现对位

- Godot：对照 `dev/godot/core/extension/gdextension.cpp` 的注册、重载与稳定 binding 兼容流程；Zircon 以显式 package slot/generation 和 capability gate 替代隐式实例身份。
- Bevy：对照 `dev/bevy/crates/bevy_ecs/src/system/system_registry.rs` 的 boxed system 注册与保守访问；Zircon 使用每 stage 一个固定 dispatcher，避免为任意 VM code 声明不可靠的静态 ECS access。
- Piccolo：对照 `dev/Piccolo/engine/source/runtime/core/meta/reflection/reflection.cpp` 的名称登记后构建 accessor 流程；Zircon 在注册期将名称编译成 dense callback slots，运行期 consumer 不做字符串目录查找。

## 结构审查吸收

- 新行为按 `host_interface/{callback,descriptor,error,registry}.rs` 和插件四通道叶子 owner 拆分，`mod.rs` 仅接线与精选导出；最大新增生产文件低于 800 行软预算。
- 公共跨模块失败保持 `VmHostInterfaceError` typed surface；生产锁通过集中 helper 恢复 poison；native 参数数量和类型均返回结构化错误，不使用 unchecked 索引或新增长期 `allow(dead_code)`。
- `CapabilitySet::contains` 不再假设公开/反序列化的 capability 向量已排序；真实 manifest 顺序回归测试验证四通道均可鉴权。插件依赖显式启用 `net-contracts`，使 RPC descriptor 的 feature 边界与共享 schema owner 一致。
- 没有兼容 facade、shim、旧路径 re-export 或 concrete AI/Net/Editor manager 反向依赖。
- 模块说明见 [`../../../zircon_runtime/script/vm/host_interface.md`](../../../zircon_runtime/script/vm/host_interface.md) 与 [`../../../zircon_plugins/zr_vm_language/host_interface.md`](../../../zircon_plugins/zr_vm_language/host_interface.md)。
