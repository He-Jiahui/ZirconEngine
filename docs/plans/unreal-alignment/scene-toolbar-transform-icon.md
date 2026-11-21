# 内建 Scene 工具栏的模式按钮加载了占位图

## 1. 用户可见的现象

内建 Scene 工具栏把第一个按钮画成占位图。这个按钮宽 28 个逻辑像素，只显示图标，绘制器需要用 Universal Transform 图案标明用途。ZE 的另一份组件化视口模板分别提供选择、平移、旋转和缩放按钮。本文的复现对象是内建 Scene 工具栏。[内建按钮](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui#L55-L60) · [组件化视口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/assets/ui/editor/unreal/components/viewport.zui#L101-L131)

## 2. 实际绘制结果

Windows x64/MSVC 的编辑器原生测试从内建工具栏生成真实绘制节点，在 640 × 28 逻辑像素、100% 缩放下调用工具栏绘制器。绘制记录把模式按钮标成 `missing-icon`，并报告缺少已就绪的文件像素和可确认的源文件。记录中的另外十份图像均已加载，摘要与磁盘文件匹配。现有用例在 100% 这一轮退出，退出码为 101。它还包含 150% 缩放和全部 16 个图标的检查，这些检查位于失败点之后。[原生绘制用例](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/workbench/docks/tests/viewport_toolbar.rs#L5-L74)

## 3. 资源在哪一步选错

工具栏声明的图标是 `editor_pages/scene_viewport/tools/universal-transform.svg`。资源查询按这个相对地址查找，随后进入缺图降级路径。版本库中实际的图标位于 `editor_pages/tools/universal-transform.svg`，SVG 内的名称是 `Universal transform`。模式按钮的命令仍能进入路由，因此只检查按钮或命令是否存在会漏掉此次绘制错误。[资源查找](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/paint/visual_assets/candidates/query.rs#L61-L95) · [绘制像素的来源](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/paint/visual_assets/template.rs#L58-L96) · [已有 SVG](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/assets/icons/editor_pages/tools/universal-transform.svg#L1)

## 4. Unreal 对照与已有测试

本地 Unreal 参考树为 `6.0.0 / UE5 / CL 0`。`Engine/Source/Editor/UnrealEd/Private/EditorViewportCommands.cpp:156–161` 注册平移、旋转、缩放和组合模式。`Engine/Source/Editor/EditorStyle/Private/StarshipStyle.cpp:4987–4994` 为各模式选择具名样式，其中组合模式 `EditorViewport.TranslateRotateMode` 使用 `Starship/EditorViewport/TranslateRotate3D` 的 SVG。

`Engine/Plugins/Experimental/EditorTRSGizmo/Source/EditorTRSGizmoTests/Private/Tests/TransformGizmoTests.cpp:18–39` 中，`Translate_Direct_Axis_Facing` 选择平移模式后，对 X 轴手柄执行左键拖动。`Engine/Plugins/Experimental/EditorTRSGizmo/Source/EditorTRSGizmoTests/Private/Tests/TransformGizmoTestUtilities.inl:329–350` 读取拖动前后的 Actor 变换，断言该轴位移非零。这项原测验证命令下游的操作行为。ZE 的绘制回归还需直接核对图标文件及绘制位置。此次 UE 对照依据为原测试与实现源码。

## 5. 现有测试与问题归属

ZE 既有的 16 像素可读性用例通过字面量列表读取图像，列表中同样存了错误地址。占位图也能产生可见像素。测试应从真实工具栏读取节点选择的资源，再确认该资源的文件身份。#148 改变祖先裁剪时检测命中缓存，#215 检测 File 菜单的翻译。这次在相同布局、裁剪和语言下即可复现缺图。[旧图标列表](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/contract/paint/tests/visual_assets_tests_support.rs#L7-L17) · [可读性检查](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/paint/tests/editor_pages.rs#L4-L32) · [工具栏裁剪 #148](https://github.com/He-Jiahui/ZirconEngine/pull/148) · [菜单翻译 #215](https://github.com/He-Jiahui/ZirconEngine/pull/215)

## 6. 修复与验收

将模式节点的图标地址指向已有的 Universal Transform SVG。分别在 100% 和 150% 缩放下绘制完整工具栏，检查 `activate_scene_mode` 对应的文件路径、SHA-256、加载状态及按钮矩形内的图像绘制命令。再由已有宽幅工具栏用例遍历全部 16 个控件。这样可以同时观察本图标的恢复和其它控件的绘制结果。[工具栏投影入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/host_lifecycle/startup/bridges/factory.rs#L40-L56)

## 同一程序复测：模式图标通过了 100% 的局部检查

同一 Windows Editor 测试程序复测时，100% 下的模式图标通过了原用例的资源归属、加载和绘制范围断言。原用例仍因后面的 SetPreviewSkybox 失败；未验证 150% 或新增 Rust 用例。

保留原 Windows x64 Editor libtest、正常开发资产根与其中的 `projected_toolbar_paints_all_16_source_icons_at_100_and_150_percent`，只替换运行时 ZUI 的图标地址。这次没有重新编译测试程序，也没有重跑旧失败。修前 stdout 在 100% 首轮的 `assetAudit.complete` 停下：Universal Transform 图标是一个无真实源文件记录的占位图。

修后同一原用例越过了资源审计以及 16 个源控件的计数，但仍退出 101（0 passed、1 failed）。原始输出停在另一控件的断言：`icon SetPreviewSkybox must be loaded with its source owner`。这是后续边界，不是整条回归通过，也未在此进一步诊断 Skybox。

100% 目标通过可从原用例顺序确定：它先跑 100%，再跑 150%；内建模板按左组的 authored child 顺序投影，`ActivateSceneMode` 在 `SetPreviewSkybox` 之前。若当前断言发生在首轮，前面的目标已完成；若发生在后续轮，首轮早已完成。目标的原断言查找同一 `sourceNodeId` 的已加载资源，并检查控件矩形中存在 `Image` 命令；先前通过的审计还回读实际资源字节。原日志未单独打印该行的 SVG 路径、SHA 或图像命令和资源键的配对，也未打印失败轮次。此次仅认定 100% 的目标节点，不把 150%、新增精确回归或普通桌面呈现记为已执行。

[原来的两轮断言](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/workbench/docks/tests/viewport_toolbar.rs#L14-L75) · [不重排的节点采集](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/runtime/runtime/projection.rs#L503-L523) · [一行图标资源修复](https://github.com/He-Jiahui/ZirconEngine/blob/58decef48911a4ea9d8707872393dce4546278f9/zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui#L55-L60) · [另行加入、尚未执行的精确回归](https://github.com/He-Jiahui/ZirconEngine/blob/a404673de70b0aabedf6568a7fd5fb2b9b880258/zircon_editor/src/ui/retained/contract/workbench/docks/tests/viewport_toolbar.rs)
