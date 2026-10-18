# ZirconEngine Unity 式编辑器优先补齐计划

## Summary

- 当前引擎已经具备“架构 MVP”：核心运行时、模块生命周期、ECS 场景、资源/渲染/UI/插件注册骨架、NativeDynamic ABI v2、编辑器扩展注册表都已经存在。
- 当前还不具备“Unity 式最小产品能力”：编辑器插件 UI 扩展、资源导入工作流、Inspector/自定义编辑器、插件安装管理、播放模式、发布面板和跨平台打包链路仍不完整。
- 本轮优先路线按你的选择定为：**Unity 编辑器优先**。先把编辑器扩展能力做成可用 SDK，再反向稳定运行时插件、导出链路和多平台发布。

## Key Changes

- 建立 Editor Plugin SDK v1：
  - 扩展现有编辑器插件注册能力，正式支持插件窗口、菜单、工具栏命令、Inspector/Component Drawer、资源导入器、自定义资源编辑器、Project/Hierarchy/Scene 视图贡献。
  - 增加插件生命周期：加载、启用、禁用、卸载、热重载、进入/退出播放模式、场景切换、资源变化、UI 消息分发。
  - 参考 Fyrox 的编辑器插件生命周期、Unreal 的 Editor Module/AssetTools/ToolMenus、Godot 的 editor plugin 扩展入口。

- 补齐 Unity 式编辑器最小闭环：
  - Project 资源浏览、Hierarchy 场景树、Inspector 属性编辑、Scene/Game 预览、Console/Diagnostics、Plugin Manager 作为首批稳定窗口。
  - 所有编辑器命令接入统一 Operation/Undo/Redo；插件贡献的命令也必须声明是否可撤销。
  - 资源导入先覆盖模型、纹理、材质、场景、UI 文档，插件可以注册 importer 和 asset editor。

- 稳定运行时/编辑器插件边界：
  - 保留当前 native dynamic 插件、linked/library embed、source template 三条路线。
  - VM 插件继续作为接口预留；在真实 `zr_vm` 接入前，只允许 mock/unavailable backend 参与测试，不作为 MVP 交付门槛。
  - 插件 manifest 增加或固化：插件类型、运行目标、编辑器贡献、依赖、能力声明、宿主 API 版本、支持平台。

- 发布能力作为编辑器功能接入：
  - 在编辑器中加入 Build/Export 面板，先暴露 Windows/Linux/macOS 桌面目标。
  - Android/iOS/WebGPU/WASM 暂列为第二阶段目标；移动端和 Web 默认不支持 native dynamic 插件，优先使用 linked/source/VM 路线。
  - 发布策略参考 Unity Graphics 的 BuildTarget/RenderPipeline 选择、Unreal 的 Runtime/Editor module 拆分、Godot 的扩展加载/重载边界。

## Public Interfaces

- `zircon_runtime_interface` 作为插件 ABI/DTO 稳定层继续保留，所有插件 SDK 对外类型从这里或专门的 SDK facade 暴露。
- 编辑器插件接口需要形成稳定 contract：
  - `EditorPluginDescriptor`
  - `EditorExtensionRegistry`
  - `EditorOperationDescriptor`
  - asset importer/editor contribution descriptor
  - inspector/component drawer descriptor
  - plugin lifecycle hooks
- 导出配置需要形成稳定 contract：
  - desktop export target: Windows/Linux/macOS
  - package mode: source template, library embed, native dynamic
  - plugin compatibility diagnostics
  - editor-side build report DTO
- 不改变当前四主包方向：`zircon_app`、`zircon_runtime`、`zircon_editor`、`zircon_runtime_interface`。

## Implementation Milestones

1. **编辑器插件 SDK v1**
   - 把现有插件目录、扩展注册、操作注册、能力声明收敛成一套公开 SDK。
   - 加入插件启停、热重载、失败隔离和诊断报告。
   - 提供 2 个真实示例插件：一个编辑器窗口插件，一个资源导入/Inspector 插件。

2. **Unity 式最小编辑器工作流**
   - Project/Hierarchy/Inspector/Scene/Game/Console/Plugin Manager 成为默认布局。
   - 插件可以贡献窗口、菜单、工具栏、Inspector、资源导入器。
   - 播放模式需要最小实现：进入运行态、退出回滚编辑态、隔离运行时状态。

3. **资源与场景 authoring 闭环**
   - 资源导入写入可追踪 artifact/metadata。
   - Scene 中实体、组件、材质、网格、灯光、相机可在 Inspector 编辑并保存。
   - 插件组件需要能显示、编辑、序列化，并在插件卸载时给出明确保护或降级诊断。

4. **桌面发布面板**
   - 编辑器内生成 Windows/Linux/macOS 发布计划和 build report。
   - 支持 linked/library embed/native dynamic 插件打包诊断。
   - CI 从 Linux-only 扩展到桌面矩阵，至少覆盖 build、test、native plugin fixture、export plan contract。

5. **第二阶段平台扩展**
   - WebGPU/WASM：建立 wasm target、浏览器启动器、资源加载策略，禁止 `libloading` 路线。
   - Android/iOS：建立 app host、资源包、触控输入、平台权限、渲染 surface 策略。
   - 移动/Web 插件策略默认只支持静态链接、source template 或 VM，不承诺动态库热加载。

## Test Plan

- 插件 SDK：合法/非法 manifest、重复插件 ID、依赖缺失、能力不足、宿主 API 版本不匹配、启停/卸载/热重载。
- 编辑器扩展：插件窗口、菜单、Inspector、资源导入器、Operation/Undo/Redo、插件禁用后 UI 贡献消失。
- 资源工作流：导入模型/纹理/材质/场景，重启编辑器后 metadata 和 artifact 可恢复。
- 场景编辑：创建实体、编辑组件、保存/加载、进入播放模式、退出后回滚。
- 发布面板：Windows/Linux/macOS export plan、插件兼容性诊断、native dynamic fixture 可加载、缺失动态库报错清晰。
- 回归参考：对照 Bevy 插件 lifecycle、Godot GDExtension reload、Unreal Editor Module、Fyrox EditorPlugin、Unity BuildTarget/render pipeline 行为做验收清单。

## Assumptions

- 本轮不把 Android/iOS/WebGPU 作为第一交付门槛，而是先确保 Unity 式编辑器插件体验成立。
- NativeDynamic 插件只承诺桌面平台；移动端和 Web 走静态链接、源码模板或未来 VM。
- 高级渲染能力如 virtual geometry、hybrid GI 不作为编辑器 MVP 的阻塞项。
- `zircon_plugins` 继续作为插件工作区存在，但需要修正文档与实际成员不一致的问题，例如 physics/animation 的状态标注。
