---
related_code:
  - zircon_hub/src/lib.rs
  - zircon_hub/src/tauri_app/mod.rs
  - tools/cargo-zircon/src/main.rs
  - tools/cargo-zircon/src/lib.rs
implementation_files:
  - zircon_hub/src
  - tools/cargo-zircon/src
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_tooling/session_coordinator/01
tests:
  - zircon_hub/src/tauri_app/runtime_state/tests.rs
  - tools/cargo-zircon/src/plugin
doc_type: category-index
---

# Hub 与工具链

Hub 与工具链是引擎源码、项目文件和可分发产物之间的操作层。`zircon_hub` 是 Tauri 桌面应用和可选本地服务；`cargo-zircon` 是面向工程师和 CI 的命令行工具；Session Coordinator 负责把 Cargo 验证放进受控的目标目录、租约和证据流水线。

## 文档地图

| 页面 | 适用场景 |
| --- | --- |
| [Zircon Hub](zircon-hub.md) | 创建/打开项目、选择引擎、启动编辑器、查看任务状态 |
| [`cargo-zircon`](cargo-zircon.md) | 插件脚手架、manifest 同步、校验、产品 receipt |
| [导出与项目打包](export-and-packaging.md) | 生成项目包、构建 runtime/editor、签发和验证产物 |
| [Session Coordinator](session-coordinator.md) | 受管验证、Cargo lane、validation copy、故障路由 |

## 责任边界

```text
Hub UI/Tauri commands
    -> HubRuntimeSession / view model / background task
    -> project + engine + plugin catalogs
    -> editor process / local service / build command

cargo-zircon
    -> plugin declaration and plugin.toml
    -> validation-batch bridge
    -> product receipt build/issue/verify

validate-matrix.ps1 + coordinator
    -> source snapshot + target admission + Cargo
    -> machine-readable evidence and cleanup
```

Hub 不直接拥有 Runtime 的 ECS 或渲染状态；它只持有项目、进程、任务和展示模型。`cargo-zircon` 也不替代 Cargo，它负责生成和验证 ZirconEngine 特有的元数据与构建协议。

## 状态标签

- **已实现**：源码有可调用入口并有聚焦测试。
- **受限**：功能由 Cargo feature、平台、凭据或外部服务门控。
- **内部实现**：只供 Hub、CI 或 coordinator 使用，不是第三方稳定 API。
- **规划中**：文档描述设计方向，当前入口不能据此保证产品行为。
