# Editor UI：工具栏命中缓存丢失源裁剪（独立问题草案）

状态：**源码确认，尚未运行行为回归；建议独立 Draft 后先补等价合同测试，再考虑修复。** 本问题提交仅新增调查文档；调查阶段未运行 Cargo、未改引擎或测试，Draft 状态以 GitHub 为准。读取 ZE 为本地 `cb495bb1c952a24c86f4513b8099bc4942c3a87d`；它与公开基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37` 的四个引擎 crate 无差异，以下 ZE 链接固定到后者。UE 只读参照为本机 6.0.0 / BranchName `UE5`，只列短符号、定位及文件 SHA256，不发布代码片段。

## 1．两边不一致、真实路径与调用链

ZE 的源布局已保留祖先裁剪：[工具栏模板:22–26](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui#L22-L26>) 把左按钮组声明为 `clip=true` 的水平 `ScrollableBox`；[runtime 投影:575–580](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/runtime/runtime/projection.rs#L575-L580>) 和 [retained adapter:198–212](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/runtime/retained_adapter.rs#L198-L212>) 继续传递 `clip_frame`。但 `ViewportToolbarSurfaceFrameCache::SurfaceFrameSignature::from_projection` 只缓存 control/component/frame，未存源 clip（[签名定义:30–43](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/dispatch/viewport_toolbar/surface_frame_cache.rs#L30-L43>)、[转换:298–324](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/dispatch/viewport_toolbar/surface_frame_cache.rs#L298-L324>)）；`build_surface_frame` 将按钮全部扁平挂到只裁剪整个工具栏的 root（[构建:356–427](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/dispatch/viewport_toolbar/surface_frame_cache.rs#L356-L427>)）。生产闭环是 [pane 发布:103–145](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/surface_frames/pane_frame.rs#L103-L145>) → 这个缓存帧 → [原生命中:17–24](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/pointer/routing/panes/pane/toolbar.rs#L17-L24>) → [点击回调:10–16](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/pointer/buttons/viewport/toolbar/click.rs#L10-L16>) → [host click:40–58](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/viewport/toolbar_pointer/click.rs#L40-L58>) → [shared dispatch:83–89](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/dispatch/shared_pointer/viewport_toolbar.rs#L83-L89>)。后者的 `sync_surface_frame` 又只取 `entry.frame`、只比较 action/frame（[同步:43–91](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/viewport_toolbar_pointer/sync_surface_frame.rs#L43-L91>)）。因此“上层已经用正确源 grid 挡掉”并不成立：上层读的也是首次失真的派生 grid。相对地，UE `SWidget::Paint` 发布当前裁剪，`FHittestGrid::GetHitIndexFromCellIndex` 查询同时检查 culling rect 与 widget clipping state，最后才判断几何命中（[SWidget:1546–1555,1587–1593](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp#L1546-L1593>)；[HittestGrid:1015–1078](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp#L1015-L1078>)）。

## 2．有界影响，不推测性能

确定的源码反例是：一个按钮的 frame 为 `(12,4,24,20)`，源 clip 为 `(12,4,8,20)`，工具栏 root 为 `(0,0,640,28)`；点 `(32,10)` 在按钮 frame 内但在源 clip 外。当前签名/重建把该按钮有效命中范围扩回 frame 与 root 的交集；仅将 clip 从全宽改为这段窄 clip，签名仍相等，`resolve` 可直接复用旧帧（[缓存比较:177–186](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/dispatch/viewport_toolbar/surface_frame_cache.rs#L177-L186>)）。这是源码推导的最小反例，**未执行该 fixture**。真实模板的左滚动区与右侧固定 60px 组并存（[右组:49–53](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui#L49-L53>)），窄宽度下可能把被裁按钮的动作扩到滚动区外、按钮间隙或其他可见控件附近；最终会不会被更高命中项遮挡，应由真实模板回归选择未遮挡点证明，不能声称所有窄宽度、所有点都会误派发。本问题不声称卡顿、分配或帧率收益，也不涉及 3D viewport affine。

## 3．根因及 UE 的解决方式

根因不是 Runtime 缺少裁剪，也不是只少一次 redraw，而是 Editor 在“按源布局生成可复用命中帧”的边界把**影响命中语义的 clip 从缓存身份和重建数据同时删除**。新帧和缓存复用都不能恢复被丢的信息；内层 geometry-only delta 的 frame 相等判断进一步保持旧错误范围。UE 并非以一个不完整的矩形镜像代替源 widget：`SWidget` 的 persistent state 保留绘制几何/裁剪上下文，`FHittestGrid` 在每次命中时消费当前 clipping state。UE fast path 的 `FSlateInvalidationRoot::PaintFastPath_UpdateNextWidget` 对实际 widget proxy 更新，而不是只比较 label 与矩形（[FastUpdate:665–716](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp#L665-L716>)）。这些代码只支持“快路径也必须保留命中依赖”这一思想，不支持宣称 ZE 应复制整个 Slate 架构。

## 4．Rust 应沿用的思想

让 `ViewportToolbarSurfaceFrameCache` 的 retained signature 是源命中合同的无损投影：至少包含物理坐标下已解析的有效 clip；clip-only 变化必须使对应派生帧/命中段发生变化，而不是伪装 stable identity。可以优先直接复用源 `UiSurfaceFrame` 的 hit authority 并做 typed control-id 映射；若仍需独立桥 surface，则把 clip 与 frame 一并保存在私有 DTO，并在外层缓存与内层 delta 两处都比较/发布。保留源 `control_id`/节点 identity 的稳定性，不把 clip 变更误作另一逻辑按钮，也不以全量重建所有 pane 作为默认修法。

## 5．ABI、ECS、retained UI 差异

UE 使用进程内 `SWidget` 指针、persistent paint state 和 `FHittestGrid`；ZE 此路径由 `zircon_editor` 的 retained-host/模板桥拥有，布局和通用 hit grid 的行为 owner 是 `zircon_runtime::ui`，中立 DTO 在 `zircon_runtime_interface::ui`。ECS 不是本次丢裁剪的发生点；不能把按钮移进 World，亦无需让 app 接管命中。现有 host projection 和 `UiHitTestEntry` 已能表达 clip，窄修复可局限 Editor 的私有 signature/control DTO 与 surface 构建/同步，不必增加 ABI 字段、跨 cdylib 传 Rust widget 指针或暴露 Runtime world。`None`（无局部裁剪）和 `Some(empty)`（已裁空）必须区分；不能借 `.max(1.0)` 把已裁空热区重新开放。

## 6．具体改法草案与 Draft 顺序

先开只含本问题六点调查的独立 Draft；其后在现有 fixture 中补 RED：从带部分 clip 的 retained projection 构建缓存帧，验证 clip 外点不命中；同 frame/control 只改 clip，验证旧快照不变、新帧拒绝旧点、可见点仍命中；再测内层 `sync_surface_frame` 的 clip-only delta。产品落点用真实工具栏模板，在窄尺寸布局后读取源 frame/clip，自动选择“root 内、按钮 frame 内、clip 外且无更前可见按钮”的点，走原生 pane route 与 host callback，断言没有对应命令；若 fixture 找不到这种点，不能把数值猜想记为产品 RED。修复候选：在 signature 增加有效 clip，构建时把源 clip 写入对应 node 的 layout cache；内层桥同步保留/比较 clip，几何发布同时包含 clip 影响，或直接改为只对源 hit result 映射动作，删除重复命中 authority。最后跑相同 fixture 及 existing stable-identity/route-remap/DPI 对照，证明正确裁剪与未变帧复用并存。本轮未补测试、未修复、未执行验收。

## 测试与去重表

| 来源 | 名称 / 路径行 | 已有输入及预期 | ZE 落点 / 本次新增的区别 | 本轮状态 |
|---|---|---|---|---|
| UE 已有相关测试，**不是裁剪 oracle** | `Slate.InvalidationWidgetList.AddBuildRemove`，[测试:24,186–205](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Test/SlateInvalidationWidgetListTest.cpp#L186-L205>) | 多层 VerticalBox/leaf，widget list 构建与子项索引完整 | 只说明 UE 存在稳定 retained identity 测试；不声称从它迁移裁剪用例 | 只读，未运行 |
| UE 对应行为 | [HittestGrid:1031–1078](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp#L1031-L1078>) | frame 内但 clip 外的点不能命中 | 在已检索 Slate/SlateCore Private/Test、UMG Private 中未找到同形“toolbar cache clip-only”现存测试；这是**源码推导，非测试迁移** | 未运行 UE |
| ZE 已有 | `viewport_toolbar_surface_frame_reuses_an_exact_projection_signature`，[projection_fallback:179–239](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/tests/retained_viewport_toolbar_pointer/projection_fallback.rs#L179-L239>) | 同尺寸同映射复用 Arc；改映射发布新帧且不重算布局 | 增加同 frame、只改 clip 的签名/快照对照 | 只读，未运行 |
| ZE 已有 | `stable_frame_identity_skips_reprojecting_toolbar_controls`、`stable_control_identity_publishes_only_the_changed_control_geometry`，[sync tests:12–86](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/tests/sync_surface_frame.rs#L12-L86>) | 同 hit-grid Arc 不投影；frame 位移只发布一个控制几何 | fixture 的 clip 始终等于 frame（[fixture:120–169](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/tests/sync_surface_frame.rs#L120-L169>)），未覆盖部分/空裁剪；就在此扩展内层对照 | 只读，未运行 |
| ZE 产品合同既有 | `root_viewport_toolbar_repeated_click_reuses_published_layout_authority`，[host tests:26–57](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/tests/projection_geometry.rs#L26-L57>) | 同一 host 连点两次，事件两条而布局重算计数不变 | 补真实窄模板 clip 外点无动作；保留正常可见点击和复用对照 | 只读，未运行 |

去重：已读本地 R13 提交 `e7e92a367` 的修改路径（measure/arrange/responsive/grid/masonry），以及当前对应测试；没有本工具栏缓存签名/clip 路径。精确 R13/R14 提交对当前 HEAD 的 ancestry 查询均返回 1，不能把这些未合分支内容称为当前修复。本问题与 #144（3D affine packet）及 #128（PIE 退出/模块卸载）不同根因；近期 #135–145 快照未见同根因 Draft。历史测试状态不构成本次行为通过。

## UE 文件指纹（SHA256，本机读取版本）

| 文件 | SHA256 |
|---|---|
| [HittestGrid.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp>) | `E1DF5B8781A9584B1F625830A0F4D98359238CF49ACF0508551AB519AE6C2C77` |
| [SWidget.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp>) | `AEFD9093803CF15CDA65C8053417CC674B6496735E7C40862F5E89CEDDDC2318` |
| [SlateInvalidationRoot.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp>) | `EC935649054B88977DE0C67523EDF773458C246B2F560F8F7FBEAA18333C82C9` |
| [SlateInvalidationWidgetListTest.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Test/SlateInvalidationWidgetListTest.cpp>) | `8F4FF55BC2056113223F8AD77E703B81457CA342B00A27584C89095A6DCF5332` |

## 工具栏裁剪改变后的命中区域

Windows x64/MSVC 的编辑器 libtest 读取本提交的工具栏模板，在两个按钮采样点命中，然后移动祖先裁剪框。按钮的布局尺寸保持相同。裁剪框只覆盖第一个点时，应当只能点击这个点。裁到空区域时，两个点都应失效。

| 路径 | 观察到的行为 |
|---|---|
| 保留式工具栏外层命中缓存 | 完整、部分、空裁剪三次均在两个点命中。后两次都复用了先前的命中区域。 |
| 指针路由的内层几何缓存 | 三次几何更新标志为 `true,false,false`。两个点三次都能命中，预期依次为两个、一个和零个。 |
| 三个独立路由对照 | 缩放后的命令身份、完整签名复用和单控件几何更新各自通过。 |

七项用例中还有两项报告了独立问题。应用级用例在创建种子项目时因缺少启动 BuildSet 而停止，尚未派发窗口点击。100% 宽幅绘制用例报告通用变换图标缺少文件，已加载的十份资源都匹配原文件，并在进入 150% 用例前结束。本次已测到工具栏的两层命中逻辑，真实窗口的输入派发待补测。

[外层回归](https://github.com/He-Jiahui/ZirconEngine/blob/8c8f03b2e78651cf6ecb8b23f6a8dfa3c0d285c0/zircon_editor/src/ui/retained/dispatch/templates/tests/surface_frame_cache.rs) · [内层回归](https://github.com/He-Jiahui/ZirconEngine/blob/8c8f03b2e78651cf6ecb8b23f6a8dfa3c0d285c0/zircon_editor/src/ui/retained/tests/sync_surface_frame.rs)

## 裁剪变化后的工具栏命中区域

- **外层缓存：** 内建工具栏从完整可见裁到部分可见、再裁到全空。修前复用旧帧，两个点始终命中；修后保留原始按钮框，只更新有效命中区，部分可见时只命中可见点，全空时两点都不命中。
- **内层点击路由：** 原始按钮框不变、只更新裁剪。修前两点仍能触发路由；修后被裁掉的点不再路由，裁到全空后两点均无路由。

Windows x64/MSVC 的 `zircon_editor` 原生库测试使用相同测试和资产。两个回归均由 **失败 → 通过**；缩放后的控件身份与命令、未变投影的帧复用、单控件几何增量三个对照仍通过，共 **5 项通过，0 项失败**。

没有重跑停在工程准入的 App 用例或缺图的宽幅绘制用例；缺图另见 [#257](https://github.com/He-Jiahui/ZirconEngine/pull/257)。桌面实点、项目内实际命令和 GPU 呈现仍待验。
