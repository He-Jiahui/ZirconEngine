# 虚幻风格编辑器入口

当前编辑器默认资产是 `zircon_editor/assets/ui/editor/unreal/level_editor_window.zui`，组合 `unreal/editor_shell.zui` 和 `unreal/starship.zui`。工具栏、Viewport、Outliner、Details、菜单、内容抽屉和状态栏均由 `unreal/components` 提供。

## 本地设计参考

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Styling/StarshipCoreStyle.cpp`
- `dev/UnrealEngine/Engine/Source/Editor/EditorStyle/Private/StarshipStyle.cpp`
- `dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/LevelEditor.cpp`

当前编辑器使用 Starship 色彩和 Roboto 字体，保留稳定控件 ID、绑定和事件。内部 `WorkbenchWindowRoot` 名称是接口标识；资产入口由 `unreal/level_editor_window.zui` 提供。

## 当前导出与验收入口

真实 Penpot 导入和导出必须绑定上述原始资产及其所属组件。默认资产描述、内建模板注册和真实导出 owner 校验均指向该路径。

布局 catalog 已更新当前编辑器记录：移除 11 项源文件已删除的旧界面记录，加入 11 项当前虚幻风格资产。其余 317 项历史记录保持原文。新增记录的视觉验收状态为 `pending`；侧栏本地化解析和真实产品状态输入的诊断仍保留。

完整验收需要当前引擎构建的普通编辑器窗口、对应的 GPU 帧与布局数据，以及真实 Penpot 保存、关闭、重开、导出的同一文件证据。旧版本截图与模拟导出不作为本轮画面一致性证据。
