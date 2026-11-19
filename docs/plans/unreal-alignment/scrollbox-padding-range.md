# 非虚拟滚动框：自身内边距被多算成可滚动距离

原 ZE 基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。本 PR 首次提交只含调查和回归方案，不改布局器。讨论的是 `.zui` 中**节点自身的 `layout.padding`**，不是父槽的内边距、弹性滚动或虚拟列表。

## 1. 正常用户路径

打开普通 UI v2 文档：垂直 ScrollableBox 内有两个固定高 40 的按钮，上、下内边距各 10，外框高 100。内框正好高 80；用户在按钮上滚动时，内容不应移动。在相同表面先用外高 70 滚到底，再扩至 100，旧偏移也应归零。[文档解析器明确区分自身 padding 与 slot.padding](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/v2/surface_tree/layout.rs#L54-L76)，而 [ScrollableBox 摘取轴、间距与虚拟配置](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/v2/surface_tree/layout.rs#L358-L385) 是已有类型。[UI 资源预览使用同一编译后表面](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/asset_editor/preview/preview_host.rs#L60-L78)，不是要求新增编辑器功能。

仓库已有 [ReactBits 文档夹具的实际自带内边距滚动框](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/tests/fixtures/ui/reactbits_agent_workspace.zui#L259-L265)。它是**测试资产**而非发布产品故障实录。在抽查的 274 份模板、示例与 Editor 等范围内没有找到相同直接声明的实例；名字带 ScrollBox、实际布局却是 VerticalBox 的控件不计入。不会据此声称某个现成游戏界面已经复现。

## 2. 生产者、消费者和错误量

1. [measure_content_size](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/layout/pass/measure.rs#L103-L107)量子项；公共返回 [加入节点自身内边距](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/layout/pass/measure.rs#L134-L146)。[测量结果存入节点 layout_cache](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/layout/pass/measure/traversal.rs#L138-L161)，其主轴尺寸包含两边内距。
2. 另一路在 [先计算内容框](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/layout/pass/arrange.rs#L129-L143) 后把 [已内缩的 content_frame 交给滚动布局](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/layout/pass/arrange.rs#L275-L287)。[常规路径的 content_extent 与 viewport_extent](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/layout/pass/arrange.rs#L769-L802) 却分别使用**含内距的缓存**和**不含内距的内框**；[offset 的硬边界](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/layout/scroll.rs#L40-L59) 直接作差。对于这里取值非负的垂直例，子项 C=80、总自内距 P=20、内框 V=80；实现推得 `max(C+P−V,0)=20`，而同坐标系应得 `max(C−V,0)=0`。
3. [正常未被控件处理的滚轮分发](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/surface/event_routing.rs#L139-L161)从最近的可滚动祖先开始，只有实际有位移才消费；[偏移钳制及失效标记](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/tree/node/scroll.rs#L31-L57)复用上述错误状态。[子项按 offset 安排](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/layout/pass/arrange.rs#L829-L847)随后把按钮真的移走，并非只是滚动条外观或命中网格缓存未失效。

## 3. 影响和可证伪的数值

| 普通配置 | 正确最大偏移 | 按当前源码推导 |
|---|---:|---:|
| 外高 100、两项共 80、自内距 20 | 0 | 20 |
| 外高 70，其余相同 | 30 | 50 |
| 在前一行底部把同一表面扩至高 100 | 重新钳制为 0 | 仍可留下 20 |
| 外高 70、同样两项、无自内距 | 10 | 10 |

在第一或第三行，按钮本应从 y=10、50 开始；若错误地留下偏移 20，则从 −10、30 开始。因为内层误报“已滚动”，它还可能比真实可滚的外层先吃掉一个滚轮；这一嵌套影响需要另外的命中/消费证据，不能拿单层案例代签。初始提交是源码取证；尚未在此提交运行测试或 GUI。

## 4. UE 的方案与已有原测

`SScrollBox::GetScrollOffsetOfEnd`（`Engine/Source/Runtime/Slate/Private/Widgets/Layout/SScrollBox.cpp:579–584`） 和 `SScrollBox::ScrollBy`（`Engine/Source/Runtime/Slate/Private/Widgets/Layout/SScrollBox.cpp:1221–1249`）用同一 ScrollPanel 的内容尺寸与安排后的可视几何计算正常硬上限；可选 overscroll、消费策略另处处理。`SScrollBox::OnMouseWheel`（`Engine/Source/Runtime/Slate/Private/Widgets/Layout/SScrollBox.cpp:1165–1188`）是实际滚轮入口。它给 ZE 的参照是**相减的是同一坐标系的两个范围**，不是声称 UE 的 slot padding、前/后填充、始终消费选项与 ZE 的 node padding 完全等价。

UE 已有 `TestFramework.Driver / Element / Hover`：`AutomationDriver.spec: 下滚和上滚后的 Hover`（`Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/AutomationDriver.spec.cpp:417–445`）分别从顶部或底部使屏外按钮进入视野；`SAutomationDriverSpecSuite 内实际的 SScrollBox`（`Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/SAutomationDriverSpecSuite.cpp:181–198`）不是伪造几何。该原测并不覆盖“外高恰好等于内容加内距之后，再 resize”这一组合，没有借用别的 ListView/TileView 断言。UE 依据为本地 6.0.0 / UE5 / CL0 的镜像；未运行 UE，不分发 UE 源且没有可靠 upstream SHA。

## 5. ZE/Rust 的实现边界和去重

继续让 node frame 管绘制/命中，让内框管子项安排，并保留测量缓存对父节点报告的期望尺寸。最小候选是在**非虚拟**滚动分支把轴向内容和视口统一到内框坐标；不要普遍删除 padding、扩大裁剪、强制滚轮不消费或在 resize 时一律清零。[物化虚拟列表以逻辑项数计算大小](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/layout/pass/arrange/virtual_list.rs#L38-L66)是不同路径，不能再次从它的逻辑长度里扣内距。若将来要处理负内距、材质自量或配置了虚拟化但未物化的变体，应先分别测量；这里不将它们列作已实测范围。

已交叉核查全部状态 PR 的标题/正文、相关历史路径、全部开放文件及既有布局计划。本案区别于 [#148](https://github.com/He-Jiahui/ZirconEngine/pull/148) 的工具栏裁剪缓存、[#150](https://github.com/He-Jiahui/ZirconEngine/pull/150) 的 opacity 与 [#174](https://github.com/He-Jiahui/ZirconEngine/pull/174) 的文本输入。现存 [无内距虚拟滚动的原回归](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/tests/shared_core/scroll_mutation/virtual_scroll.rs#L85-L103)也不会揭露这个两种坐标重复相减的问题。

## 6. 等价回归和最小修复门

先在本 PR 单独提交回归：通过 `.zui` loader、编译/展开和表面布局产生真实按钮、发送正常滚轮，核实派发是否被默认滚动消费、确切 offset、两个按钮的 arranged/Quad/hit。分别比较刚好放下、溢出后同实例扩大、无内距控制；不手工设置滚动偏移、不用一次重建新表面冒充 resize。需先确认正对照真实到达并在错误边界上获得 RED，才允许修正生产，修后用同一测试文件对照而不改断言。输出的是表面布局/渲染命令与命中，不是 GPU 像素、OS 硬件滚轮或整个发布编辑器验收。

本地 UE 文件 SHA-256：

- `Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/AutomationDriver.spec.cpp`：`62a0e0a384be81fae4e10ec29925fd6058332fe850842daed0aa853163422da6`
- `Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/SAutomationDriverSpecSuite.cpp`：`3ce1a412a68316cedd113bf579f0bf9e147b76c97a5ea45b46571a87eeee3d82`
- `Engine/Source/Runtime/Slate/Private/Widgets/Layout/SScrollBox.cpp`：`1b64336bf8356f3b46644a060e8d9349d133f44f4f86bf8ea9ec3be548eef178`

## 7. [实测失败] 真实滚轮、内容范围与当前按钮位置

从正常 `.zui` 文档加载、编译及展开为同一 `UiSurface`，给真实按钮坐标发送默认滚轮，重建并读取真实布局树、按钮 Quad、命中与 handled_by。基线下的错误是 offset 和实际按钮位置被改变，而不是滚动条的小数舍入。

本次 Git 输入 `25569db598005184535f0c7b075607056bee7d76`；[实际回归用例](https://github.com/He-Jiahui/ZirconEngine/blob/25569db598005184535f0c7b075607056bee7d76/zircon_runtime/tests/scroll_padding.rs)，该输入的测试产物 SHA-256 `d4947f01e066e238ddbe2bafd8e29a75cde5fcdd1f4dea1eeee7e091fd4cf58d`。独立集成测试实际运行 **1 过 / 4 败**。

| 冻结的用例 | 本次结果 |
|---|---|
| `scroll_padding_exact_fit_does_not_scroll` | 失败 |
| `scroll_padding_exact_fit_inner_leaves_first_wheel_to_outer_owner` | 失败 |
| `scroll_padding_horizontal_uses_left_and_right_not_top_and_bottom` | 失败 |
| `scroll_padding_overflow_clamps_to_start_as_the_viewport_grows` | 失败 |
| `scroll_padding_zero_padding_keeps_existing_end_and_fit` | 通过 |

| 时点 | 实测 offset | 正常值 |
|---|---:|---:|
| `fit-after-wheel` | 20 | 0 |
| `overflow-end` | 50 | 30 |
| `overflow-after-resize` | 20 | 0 |
| `zero-padding-end` | 10 | 10 |
| `zero-padding-fit` | 0 | 0 |
| `horizontal-end` | 30 | 10 |

单独 `ui` 的首次真实构建没有生成测试二进制（共享 cooked 资产模块找不到 zstd，E0433）；本节的行为计数只来自关闭默认、显式 `native-host,ui` 后的 Windows 测试产物，两条结果不合计。

另按同一代码输入单独编译/运行**未修改**的 `zui_reactbits_agent_workspace_contract::reactbits_agent_workspace_builds_a_responsive_flow_surface`；该测试包含 640×520、900×620 和 1280×800 三个原尺寸。结果：**0 过 / 1 败**。不是上述五个滚轮反例。 该次先在原用例通过 node_path 查找 retained 控件时停止（第 25 行），在三个尺寸的 compute_layout 调用之前；没有据此签收任何尺寸。

覆盖的是正常文档表面的 CPU 布局、呈现指令与输入派发。没有据此声称物理鼠标、OS 窗口、GPU 像素、所有虚拟列表或 UE 原测已通过。
