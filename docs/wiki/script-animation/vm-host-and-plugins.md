---
related_code:
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/vm/backend/vm_backend.rs
  - zircon_runtime/src/script/vm/backend/backend_registry.rs
  - zircon_runtime/src/script/vm/host/mod.rs
  - zircon_runtime/src/script/vm/plugin/mod.rs
  - zircon_runtime/src/script/vm/runtime/mod.rs
implementation_files:
  - zircon_runtime/src/script/vm/backend
  - zircon_runtime/src/script/vm/host
  - zircon_runtime/src/script/vm/plugin
  - zircon_runtime/src/script/vm/runtime
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
tests:
  - zircon_runtime/src/script/vm/tests/plugin_runtime.rs
  - zircon_runtime/src/script/vm/tests/host_interfaces.rs
  - zircon_runtime/src/script/vm/tests/lifecycle_failures.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests
doc_type: module-detail
---

# VM、Host 与脚本插件

## 分层

脚本系统由四层组成：

1. `VmBackend`/`VmBackendRegistry`：按 backend family 选择实际 VM 实现；`UnavailableVmBackend` 明确表示当前构建没有可执行 backend。
2. Host registry：注册 builtin/bridge/gameplay host module、函数、系统、RPC、editor operation 和 capability。
3. Plugin package：发现 `VmPluginManifest`/payload，按 `VmPluginManagementPolicy` 装载到 slot。
4. Runtime manager：维护 slot state、GC、状态 blob、热重载和 `ScriptSceneRuntimeSystem` 的帧调用。

VM backend 的最小合同是：

```rust
use zircon_runtime::script::{VmBackend, VmError, VmPluginHostContext, VmPluginInstance,
    VmPluginPackage};

struct MyBackend;
impl VmBackend for MyBackend {
    fn backend_name(&self) -> &str { "my-vm" }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        let _ = (package, host);
        Err(VmError::BackendUnavailable(
            "my-vm backend is not linked in this build".to_string(),
        ))
    }
}
```

示例明确返回 `BackendUnavailable`，表示该 backend 尚未链接；应用不能把 unavailable 当作运行时 fallback。生产宿主应注册真实 backend，或在能力报告中显示 unavailable。

## Host module 与能力

`register_builtin_host_modules`、`register_bridge_host_module` 和 `register_gameplay_host_module` 将宿主函数暴露给脚本。每个导出通过 `HostCapabilityRecord`/`CapabilitySet` 声明能力名、版本和调用约束；脚本只能调用已授予的 capability。`render_script_host_modules_markdown`/`write_script_host_modules_markdown` 可生成面向工具和开发者的 host API 文档。

重要宿主入口包括：

- `VM_HOST_INTERFACE_MODULE`：通用接口注册。
- `BRIDGE_HOST_MODULE` / `BRIDGE_HOST_CAPABILITY`：跨域 bridge。
- `VM_SYSTEM_CAPABILITY`：按 `VmSystemStage` 注册脚本 system。
- `VM_EDITOR_OPERATION_CAPABILITY`、`VM_RPC_HANDLER_CAPABILITY`、`VM_BT_NODE_CAPABILITY`：编辑器、RPC 和行为树扩展。

## Package 发现与限制

`discover_vm_plugin_package*` 根据 `VmPluginDiscoveryRequest` 扫描项目 source，并用 `VmPluginDiscoveryLimits` 限制 manifest、payload、目录和并发读入。调用方应在产品 profile 选择后再发现包，把路径规范化和错误交给 discovery owner，而不是自己读取任意 DLL/脚本文件。

## Slot 生命周期与热重载

一个脚本插件 slot 通常经历：`discovered -> admitted -> loading -> running -> stopping -> unloaded`。`HotReloadCoordinator` 负责在安全点：

```text
stop new calls -> run GC/roots -> save VmStateBlob
 -> cleanup old instance -> load new package
 -> migrate state schema -> restore -> resume calls
```

`VmStateBlob` 使用 `VM_STATE_SCHEMA_VERSION_V3`、类型 identity 和 field values；迁移失败必须保留原实例或返回明确的 `VmStateMigrationError`，不能半恢复。GC 使用 `VmGcBudget` 控制每帧微秒预算，root registration 返回 `VmGcRootToken`，token 退休后不可继续访问对象。

## 场景脚本系统

`ScriptSceneRuntimeSystem` 在 `SCRIPT_SCENE_RUNTIME_SYSTEM_SET` 中按 `ScriptSceneLifecyclePhase` 参与帧循环，暴露 update/fixed-update 系统常量。它通过 `VmReflectionWorldAccess`/`VmReflectionWorldOperation` 读取或提交 World 操作；脚本不持有 Rust `World` 引用。

## 状态与限制

- 该域必须启用 `script`；未启用时公共 API 不存在或 profile 不注册模块。
- backend registry、host capability 和 plugin manifest 是三个独立门；有包不代表有 backend/能力。
- FFI/host callback 必须收口 panic、限制 payload 和检查 session/slot generation。
- VM 插件热重载是受 policy、GC deadline、state schema 和 host call quiescence 约束的运行时操作，不是任意线程替换 trait object。
