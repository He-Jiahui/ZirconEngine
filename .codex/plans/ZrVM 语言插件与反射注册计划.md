# ZrVM 语言插件与反射注册计划

## Summary
- 先在 ZirconEngine 内完善脚本反射/宿主导出机制，再把 `E:\Git\zr_vm` 作为 `zr_vm` VM 后端插件接入现有 `zircon_runtime::script::vm`。
- 所有 VM 访问 ZirconEngine 的能力都走稳定 `HostHandle`、capability 和反射描述，不把 Rust 对象裸指针暴露给 VM。
- `zr_vm` 以 `zircon_plugins/zr_vm_language/runtime` 插件落地；默认 ZirconEngine 主运行时仍可在没有 `zr_vm` 时构建，启用该插件才要求本机有 `zr_vm` 构建产物。

## Key Changes
- 在 `zircon_runtime::core::framework::script` 定义反射契约：`ScriptHostModuleDescriptor`、`ScriptHostTypeDescriptor`、`ScriptHostFunctionDescriptor`、`ScriptHostValueKind`、`ScriptHostCallContext`、`ScriptHostResult`。
- 在 `zircon_runtime::script::vm::host` 增加宿主导出注册表，由 `PluginHostDriver` 持有；注册表负责 descriptor 校验、capability 绑定、`HostHandle` 分配、回调分发。
- 首批注册 ZirconEngine 基础库为 VM native modules：
  - `zr.zircon.foundation`：config/event/log/time 查询与发布。
  - `zr.zircon.asset`：资源定位、状态、列表、修订号。
  - `zr.zircon.scene`：level/world handle 创建、加载、保存、summary 查询。
  - `zr.zircon.render`：只读渲染后端信息和基础 render stats。
  - `zr.zircon.math`：向量/矩阵/颜色等纯值类型描述，优先只暴露值构造和纯函数。
- 扩展 VM package 模型，支持除现有 `bytecode` 外的 `zr_vm` 项目源包：
  - `plugin.toml` 使用 `backend = "zr_vm:project"`。
  - `[zr_vm] project = "plugin.zrp"`，`entry_module = "main"`，`execution_mode = "binary"`。
  - lifecycle 约定为可选导出：`activate()`、`deactivate()`、`saveState(): string`、`restoreState(state: string)`。
- 新增 `ZrVmBackendFamily` / `ZrVmBackend` / `ZrVmPluginInstance`，把 ZirconEngine 反射 descriptor 转换成 `zr_vm_rust_binding::ModuleBuilder`，并在每个 `zr_vm` runtime 上保持 native module registration 生命周期。

## Implementation Order
1. 反射基础层：先实现 descriptor、注册表、重复名称/无效类型/capability 校验，以及 mock backend 下的注册和调用测试。
2. 宿主基础库：用现有 `core::framework` trait 和 `core::manager` service name 接入 foundation、asset、scene、render、math；所有对象返回稳定 handle 或纯数据。
3. package 协议：让 `discover_vm_plugin_package` 识别 `zr_vm:project`，保留旧 bytecode 包兼容 mock/unavailable backend。
4. `zr_vm` 插件：在 `zircon_plugins/zr_vm_language/runtime` 添加 runtime plugin crate，依赖 `zr_vm_rust_binding`，通过环境变量 `ZR_VM_RUST_BINDING_LIB_DIR` 链接本地 `zr_vm` 动态库。
5. 热替换：沿用 `HotReloadCoordinator`，`saveState/restoreState` 映射到 `VmStateBlob`；宿主 handle 表不随 VM 实例替换而失效。
6. 文档与示例：补一个最小 `.zr` 插件示例，演示 `%import("zr.zircon.foundation")` 和热替换状态迁移。

## Test Plan
- `cargo test -p zircon_runtime script::vm`：验证反射注册、capability、host handle、package 发现、mock backend、热替换。
- `cargo test -p zircon_plugin_zr_vm_language_runtime`：在 `ZR_VM_RUST_BINDING_LIB_DIR` 存在时验证真实 `zr_vm` 编译、native module 注册、调用导出和卸载。
- 边界测试：重复 module/type/function、未知 capability、无效 handle、缺失 `zr_vm` 动态库、缺失 lifecycle export、热替换失败回滚。
- 压力测试：重复 load/unload/hot_reload 100 次；native callback 中递归调用宿主导出；并发请求先用全局互斥串行化，因为 `zr_vm_rust_binding` 测试显示 C 状态仍是进程级共享。
- 验收命令：`cargo check --workspace`、`cargo test --workspace`、`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime`。

## Assumptions
- “以 plugin 的形式纳入 `zr_vm`”解释为 ZirconEngine runtime plugin + VM backend plugin，不走旧式 Rust native dynamic plugin ABI。
- v1 不暴露 editor authoring API，不让 VM 直接改 renderer/GPU 对象；先暴露稳定、可测试的 runtime 基础库。
- `zircon_hub/src/app/binding.rs` 是 Hub UI 绑定层，不作为 VM 反射系统落点。
- `zr_vm` 源码仍在 `E:\Git\zr_vm`；后续可再决定是否改成 git submodule 或 vendored third-party。
