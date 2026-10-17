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

## 本轮核验记录（2026-10-15 UTC）

- `real-r15` 已完成真实 Penpot 保存确认、关闭原上下文、重新认证打开及 ZUI/PNG 导出。导出 ZUI 与当前 `unreal/level_editor_window.zui` 字节一致；源文件 SHA256 为 `94924073dfe77eaa54aa9c8736267158e1e7aa9ca4a8cb6fdfeab5510ab68964`。
- 实际官方导出图已检查：画布边界修复后直接导出为 960×640，`real-r14` 中的左上额外空白已消除，未经过导出后裁剪。PNG SHA256 为 `3425eb9d4d5395819340021647f0807601be4e19718225801d4943d93e14c89b`。关闭状态的菜单宿主及 Outliner 28×28 筛选按钮沿用当前源资产。
- 菜单选中项的前景配色和跨组件焦点返回存在待修复问题；已定位实际源文件，不能将当前默认关闭状态的截图计为这些交互通过。
- 上述结果仅证明原始资产的真实持久化与导出。完整动态界面、引擎普通窗口、场景、焦点及弹层交互、双端布局和像素对照仍未验收。
- 按用户要求继续当前 125% 桌面预览。100% 与 150% 的正式尺寸组仍待实际系统缩放环境验证。

本轮完整证据归档于公共产物目录 `E:/cargo-targets/zircon-local/reports/penpot-unreal-01a0f04d`，清单为该目录的 `artifacts.json`。保留真实失败、来源指纹和未完成状态。
