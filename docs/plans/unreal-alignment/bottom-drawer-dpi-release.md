# 底部 Console 分隔条在 150% DPI 下松手后会跳高

## 1. 哪个界面受影响

在普通项目的 Level Editor 主窗打开并固定底部 Console，从 Console 上沿向上拖动分隔条，在同一窗口松手。生产 Workbench 把主视图、底部停靠区和状态栏放在一个纵向布局中；底部停靠区再布置标题、分隔线与 Console 内容。这是常规停靠面板，不是临时 Content Drawer。以下结果依据公开 ZE 基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37` 的源码推导，尚未运行回归或实机观察。[内置 Console](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs#L52-L59) · [工作台嵌套布局](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/assets/ui/editor/unreal/level_editor_window.zui#L32-L45) · [底部面板](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/assets/ui/editor/unreal/components/bottom_activity_dock.zui#L15-L44)

## 2. 同样的拖动在两个缩放下应得到同样的布局偏好

固定初始高度 `200` 逻辑像素，客户区为 `1920 × 1200` 逻辑像素，手势中不改变 DPI。两例都避开窄窗折叠、紧凑限高和最小高度钳制；向上拖动一次，相当于增加 `20` 逻辑像素。

| 缩放与物理输入 | 应提交的逻辑高度 / 松手后物理高度 | 基线源码推导，非实测 |
| --- | --- | --- |
| 100%；`1920 × 1200 px`，向上 `20 px` | `220` 逻辑像素 / `220 px` | 相同 |
| 150%；`2880 × 1800 px`，向上 `30 px` | `220` 逻辑像素 / `330 px` | 初始 `300 px` 加 `30 px`；把 `330 px` 写作 `330` 逻辑像素，重排后成 `495 px`。Console 上沿比预期多上移 `165 px`，挤占中部主视图空间。 |

[宽高预算](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/dispatch/templates/workbench/drawer_layout.rs#L321-L360) · [坐标合同](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/workbench/autolayout/resolution_context.rs#L92-L119)

## 3. 在哪里混用了单位

Workbench 在逻辑坐标中排版，发布分隔条和面板时把尺寸乘以缩放比例。按下分隔条时，底部高度取自已发布的物理矩形，指针位移和拖动中的临时值也都是物理像素。问题发生在 `finish_drawer_resize_capture`：它更新末次位置，清除临时值，随后把这个物理高度直接交给 `SetDrawerRegionExtent`。布局管理器按逻辑偏好存它；底部组件再取这个值设置高度，发布给 Console painter 时又乘一次 DPI。无需修改指针捕获、绘制器、Taffy 或 Runtime/ECS。[物理框发布](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/dispatch/workbench/componentized_window.rs#L347-L369) · [拖动起点](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/workspace_docking/drawer_resize/capture.rs#L27-L52) · [松手提交](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/workspace_docking/drawer_resize/movement.rs#L73-L93) · [存储布局偏好](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/workbench/layout/manager/apply.rs#L280-L317) · [设置底部高度](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/dispatch/templates/workbench/drawer_layout.rs#L210-L229) · [实际画面消费](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/workbench/docks/bottom.rs#L18-L50)

## 4. Unreal 的生产路径和原有测试

本地 UE 参考为 `6.0.0 / UE5 / CL 0`（`Engine/Build/Build.version:1–9`）。`Engine/Source/Editor/LevelEditor/Private/SLevelEditor.cpp:398–417` 将 StatusBar 挂到主视图下；`Engine/Source/Editor/StatusBar/Private/StatusBarSubsystem.cpp:603–619` 注册 Output Log，`Engine/Source/Editor/StatusBar/Private/SStatusBar.cpp:584–594` 挂载 ToolWidgets 的 `SWidgetDrawer`。`Engine/Source/Developer/ToolWidgets/Private/SWidgetDrawer.cpp:119–170` 中 `SDrawerOverlay` 捕获时记录手柄几何，移动时用 `AbsoluteToLocal` 求本地目标高度，松手交回该高度；同文件 `:555–562,646–674` 用窗口逻辑高度计算保存比例、重开时还原本地高度。`Engine/Source/Runtime/SlateCore/Public/Layout/Geometry.h:428–448` 执行累计变换的逆，窗口比例来自 `Engine/Source/Runtime/SlateCore/Private/Widgets/SWindow.cpp:816–830`。ZE 只借鉴单位边界，不改用 UE 的浮层或百分比存储。

原有 `TestFramework.Driver`（`Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/AutomationDriver.spec.cpp:1095–1156`）对真实 SWindow 内的 `KeyF` 分别断言 `GetAbsolutePosition()` 等于路径几何的 `LocalToAbsolute(0)`、`GetSize()` 等于 `GetLocalSize()`；缺失控件均返回零。这些断言区分两种坐标，未覆盖高 DPI 下拖动 Output Log 并保存尺寸。本次 ZE 回归计划由源码推导，并非移植同形 UE 用例；UE 未运行。

## 5. 已有覆盖和归属

ZE 已有 140% 缩放下窄窗打开 Inspector 的测试，及松手前失焦应取消拖动、不提交偏好的测试；它们不检查正常松手的尺寸单位。本题归 **Editor retained host 的 Bottom 提交**。已对照本地全态 #1–#256 及后续 #257：[#148](https://github.com/He-Jiahui/ZirconEngine/pull/148) 处理工具栏裁剪，[#168](https://github.com/He-Jiahui/ZirconEngine/pull/168) 处理 Play 的捕获释放，[#232](https://github.com/He-Jiahui/ZirconEngine/pull/232) 处理 ScrollBox 自内边距，[#234](https://github.com/He-Jiahui/ZirconEngine/pull/234) 处理响应式验收的控件查找，[#250](https://github.com/He-Jiahui/ZirconEngine/pull/250) 处理重命名的组合字符，[#256](https://github.com/He-Jiahui/ZirconEngine/pull/256) 是预制体路径，[#257](https://github.com/He-Jiahui/ZirconEngine/pull/257) 是 Scene 工具栏图标资源。它们不改这个提交边界。[缩放原测](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/tests/opened_view_visibility.rs#L227-L251) · [取消原测](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/tests/projection_geometry.rs#L391-L470)

## 6. 修复范围与验收计划（未运行）

**只在拥有手势的 Bottom 分隔条正常松手后、发 `SetDrawerRegionExtent` 前**，将最后的物理高度除以发布该 Workbench 框的、已归一的有效缩放比例。保留临时物理值、左右分隔条、取消和钳制；不在 Move 或绘制时再转换。跨窗口、拖动途中变化 DPI 及存盘迁移不在这次范围。只有 tests-only 回归在未修复的基线上取得尺寸断言的实际 RED 后，才应用生产修改。

最小门是 `zircon_editor` 的 `--lib --no-default-features`（默认 test profile），仅运行 `root_bottom_dock_resize_commits_logical_extent_` 两例。用真实 [测试宿主](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/tests/support.rs#L92-L144) 挂载 Console，从已公布的 splitter 发 Down/Move/Up；断言 Move 无提交、Up 恰一条 Bottom `220` 逻辑像素、活动布局偏好与公布外框均正确。100% 是保持不变的对照；150% 预期在基线报告 `330` 的提交和 `495 px` 的外框，修复后为 `220` 和 `330 px`。这些数字目前只有源码推导，驱动、编译或准入失败均不算有效 RED。
