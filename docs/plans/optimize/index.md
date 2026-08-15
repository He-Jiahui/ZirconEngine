# ZirconEngine 工程级差距审查

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本目录承接 ZirconEngine 对 Unreal、Fyrox、Bevy、Godot 与仓内 Unity Graphics 参考源码的全量工程化差距审查。审查规则、分层顺序和完成定义由 [00-engine-wide-review.md](00-engine-wide-review.md) 统一拥有；物理扫描进度由 [coverage.md](coverage.md) 维护。

当前已完成 `zircon_runtime` 从core lifecycle到advanced surface lighting、temporal AA/velocity/history/upscaling，以及exposure/color/bloom/DOF/motion blur/SSR/terminal composition的连续首轮纵向深审，整个引擎仍处于 `in_progress`；runtime UI以及App、ABI、Editor、Plugins、Hub和Tooling大范围仍待审。没有进入编号子计划并达到 E2/E3 的模块不得视为已审查，也不得由本页给出“完整”结论。

## 分类

| 分类 | 当前审查队列 | 目录 |
|---|---|---|
| `zircon_runtime` | 生命周期/registry → task/event/diagnostics → resource/scene → platform/systems → graphics | [zircon_runtime](zircon_runtime/index.md) |
| `zircon_app` | bootstrap、产品循环、动态 runtime library、完整停机 | [zircon_app](zircon_app/index.md) |
| `zircon_runtime_interface` | ABI、FFI、句柄、版本协商、跨库所有权 | [zircon_runtime_interface](zircon_runtime_interface/index.md) |
| `zircon_editor` | authoring state、transaction、viewport、content workflow、扩展性 | [zircon_editor](zircon_editor/index.md) |
| `zircon_plugins` | 插件 SDK、发现、装载、隔离、重载与卸载 | [zircon_plugins](zircon_plugins/index.md) |
| `zircon_hub` | 项目/引擎安装、启动、更新、进程与错误恢复 | [zircon_hub](zircon_hub/index.md) |
| `zircon_tooling` | workspace、derive/codegen、验证、CI、打包、profile | [zircon_tooling](zircon_tooling/index.md) |

## 当前最高风险

- Runtime 生产入口只激活模块，未形成统一的反向停机调用链；进程级 task/timer worker 又未参加动态会话销毁。
- 模块 deactivation 在可拒绝通知之前执行 `cleanup`，失败后却恢复 `Running`，破坏生命周期原子性。
- 模块状态转换缺少并发所有者、等待/取消和合法迁移校验；服务卸载又向公开调用方返回不可撤销的强 `Arc<T>`。

详细证据和重构路线由 [zircon_runtime/01-core-runtime-lifecycle-registry-review.md](zircon_runtime/01-core-runtime-lifecycle-registry-review.md) 与 [zircon_runtime/02-core-runtime-events-tasks-review.md](zircon_runtime/02-core-runtime-events-tasks-review.md) 分别拥有。
