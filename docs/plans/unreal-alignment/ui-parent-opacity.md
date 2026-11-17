# Runtime UI：祖先 opacity 未作用于子节点命令（既有债务的源码复核）

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE/ZE 测试均未在本轮执行。

状态：**源码确认，未运行回归；不是首次发现。** 旧计划已记录 `RUIR-P1-006`，本次把其中可独立验收的“祖先 alpha 累乘”收窄为一个问题，不夹带离屏 group compositing、滤镜或 renderer 重构。建议独立 Draft 调查先行，再补测试，最后才做可行修复。本轮无 Cargo、源码修改、Git 写操作、远端操作。读取本地 ZE `cb495bb1c952a24c86f4513b8099bc4942c3a87d`，四个引擎 crate 与公开 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37` 无差异；ZE 引用固定到后者。UE 本机 6.0.0 / BranchName `UE5`，只给短符号说明、行号与 SHA256。

## 1．两边不一致、路径行号与真实调用链

UE `UWidget::SetRenderOpacity` 将值传给缓存 Slate widget（[Widget:454–462](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/Components/Widget.cpp#L454-L462>)）；`SWidget::Paint` 把本地 RenderOpacity 混入继承的 widget style，再传给本节点的 `OnPaint`（[SWidget:1494–1497,1655–1657](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp#L1494-L1657>)）；`SPanel::PaintArrangedChildren` 将这份 style 继续传给 child Paint（[SPanel:24–40](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SPanel.cpp#L24-L40>)），`FWidgetStyle::BlendOpacity` 的合同是 alpha 相乘（[WidgetStyle:45–48](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/WidgetStyle.h#L45-L48>)）。ZE 的生产 Runtime UI 则经 `RuntimeUiSurfaceSet::render_submission` → 每个 surface `rebuild_dirty` → `UiSurface::rebuild_render_extract_with_text_frame`（[session:196–233](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/runtime_ui.rs#L196-L233>)、[rebuild:72–100](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/surface/rebuild.rs#L72-L100>)）→ 对 draw order 中每个节点，`UiNodeVisualData::resolve` 只接本节点 metadata/state（[extract:192–224](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/render/extract.rs#L192-L224>)、[visual:23–40](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/render/node_visual_data.rs#L23-L40>)）；`resolve_opacity` 只读本节点值，默认 1（[resolve:253–257](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/render/resolve.rs#L253-L257>)）。通用及专用 painter 消费这个 local opacity，图像/形状/文本的实际 scene renderer 也只读 `command.opacity`（[render:471–507,538–580](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/ui/render.rs#L471-L580>)），不存在此后补乘祖先的消费者。

## 2．有界影响与最小反例

一个合法树：根 Group 的 local opacity=0，子 Button 的 local opacity=1；子节点仍 render-visible，非空 frame。当前提取对子 Button 生成 opacity=1 的表面/文字等命令，根 Group=0 的命令不会把同列表中的子命令变成 0；renderer 只因**各自命令** opacity<=0 才略过。因此“隐藏整个子树的淡出”只隐藏根自身而不隐藏孩子，是从完整生产调用链得出的确定反例，**本轮没有执行该 fixture 或截屏**。两层父 0.5、子 0.5 时，UE 递归合同为子命令有效 alpha 0.25，ZE 当前为 0.5；祖先改变时还可能保留旧子命令。影响仅限具有非 1 祖先 opacity 的后代 UI 绘制，不据此认定输入不可命中或安全隐藏；透明度本来不等同 hit-test visibility。未测帧耗、分配、GPU 或产品像素，故不做性能结论。

## 3．根因及 UE 的解决方式

根因是 Runtime 的平铺 draw-order extraction 没有把树级继承的视觉上下文投影为命令有效值；根 Group 的 scalar opacity 不是 push/pop 状态，后端不会自动形成子树 alpha stack。此后中立 paint-element 适配也只是把每条 `command.opacity` 拷贝到 effects（[command:241–257](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime_interface/src/ui/surface/render/command.rs#L241-L257>)），线性遍历没有祖先查找（[list:18–27](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime_interface/src/ui/surface/render/list.rs#L18-L27>)）。UE 用 `FWidgetStyle` 随递归 Paint 累乘：祖先 opacity 已进入孩子的输入 style，而非要求 child 自己猜父状态。这只是 Slate 的逐 primitive 继承透明度；**不能把它描述为自动离屏合成整个重叠子树**。旧计划中的 isolated group compositing 比本次合同更大，不应塞进同一修复。

## 4．Rust 沿用思想

在 Runtime 先求每节点 effective opacity＝本地 clamp 值×有效祖先 opacity，供所有专用/通用子命令共享；metadata 继续保留 local 值，避免把累计值写回 authored props 造成重复累乘。缓存可按树结构/本地 opacity generation 维护有效值；单叶更新不必扫描无关树，但祖先 opacity 更新必须把其受影响后代加入 render dependency closure。全量提取和节点增量提取应使用同一解析入口，不在 Editor painter、scene renderer、glyph 或 image 后端各自补一套继承算法。祖先不透明度变化仍是 render 域，不应靠改变 visibility、layout、focus 或 hit-test 来“修复”。

## 5．ABI、ECS 与 retained UI 差异

UE 的 style 依靠进程内 widget 递归传参；ZE 要在 `zircon_runtime::ui` 的 retained tree 与 render cache 边界完成继承解析。`zircon_runtime_interface` 已有每命令 scalar opacity，可承载有效值，窄修复不需要改 cdylib ABI，不要跨边界传 C++ widget/Rust trait object 或 World 引用。该路径的 owner 是 Runtime UI，而不是 `zircon_app` bootstrap，也不是 Editor 的 authoring projection；ECS 投影不是祖先 alpha 解析的替代 authority。Editor 使用相同 Runtime surface 时应自然得到同一命令结果；若另有 host 专用 painter，则必须核对是否又把 metadata local alpha 当最终 alpha，不能把修复只落到某个后端并宣称全产品对齐。

## 6．具体改法草案与验收顺序

独立 Draft 首先保留本问题说明与旧计划归属。之后先补全量 RED：父 0/子 1，父 0.5/子 0.5，三层和独立 sibling，对照所有 Button surface/text/icon 命令值；再补增量 RED：只变父 opacity，要求后代命令同步改变、无关 sibling/旧 snapshot 内容保持不变，layout/hit-grid 不推进。现有 `patch_render_nodes` 的局部提取 draw order 仅包含 `changed_node_ids`（[partial extract:616–669](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/render/extract.rs#L616-L669>)），故修复不仅要在 full extractor 加乘法，还要增加祖先视觉依赖闭包，不能遗漏 retained 子命令。实现候选是 Runtime 私有 effective-opacity 索引加 subtree render invalidation；遇不完整索引先 fail-safe full render extract，而不改布局。相同输入还应走实际 Runtime UI render submission，再用已有 [runtime_ui_integration offscreen harness:30–65](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs#L30-L65>)，在不透明背景以非重叠 child-only 的 RGB/亮度贡献或同背景对照验证透明度，不要求 framebuffer alpha 通道必然改变；不得把仅 DTO 数值通过当 GPU 像素通过。本轮这些测试均未编写/运行，未给出实现已可编译结论。

## 测试与去重表

| 来源 | 名称 / 路径行 | 已有输入与预期 | 本问题落点 / 缺口 | 本轮状态 |
|---|---|---|---|---|
| UE 对应合同 | `UWidget::SetRenderOpacity` → `SWidget::Paint` → `SPanel::PaintArrangedChildren`，见上列定位 | 嵌套 parent/child opacity 相乘；0 祖先下的后代有效 alpha 为 0 | 已检索 Runtime Slate/SlateCore Private/Test 与 UMG Private；未找到直接同形的嵌套 RenderOpacity 自动化测试，故**源码推导，非测试迁移** | UE 未运行 |
| ZE 已有 | `render_extract_carries_visual_contract_fields_for_visible_nodes`，[render_extract:5–114](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/tests/shared_core/layout_surface/render_extract.rs#L5-L114>) | root 默认 1、IconButton local 0.75，断言命令 0.75 | 同 fixture 改为有 metadata 的 Group 父与显式子 alpha，覆盖祖先 0/0.5；当前已有输入不能发现缺口 | 只读，未运行 |
| ZE 已有 | `render_patch_publishes_changed_commands_and_retains_other_segments`，[patch_publication:12–89](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/tests/surface_dirty_domains/patch_publication.rs#L12-L89>) | 仅叶节点 1→0.375，要求局部命令更新、其他 segment 保持、layout/hit Arc 复用 | 扩展 parent-only 更新对后代的 render closure；保留无关兄弟和旧快照稳定对照 | 只读，未运行 |
| ZE 产品入口 | `RuntimeUiSurfaceSet::render_submission`，[session:196–255](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/runtime_ui.rs#L196-L255>) | 真实 runtime surfaces rebuild 后生成 shared submission | 同一树进入 submission 后抽查子命令 effective alpha；后续再走 GPU offscreen，不能只复述乘法 | 新回归草案，未运行 |

去重说明：旧 [renderer 计划:179](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/79-runtime-ui-renderer-display-list-paint-order-clip-transform-opacity-atlas-text-glyph-batch-wgpu-submit.md#L179>) 已记录 `RUIR-P1-006`，本次明确不是首次发现；只对其中“祖先 alpha 缺失”提独立可验收草案。R13 提交 `e7e92a367` 的改动是 measure/arrange/responsive/grid/masonry 测试，不是 opacity；本机 exact R13/R14 提交不属于当前 HEAD 祖先，未将未合实现当基线修复。#144 是 mesh viewport affine，#128 是退出/卸库，近期 #135–145 快照未见本根因的独立 PR。本条不接管他人未合工作。

## UE 文件 SHA256

| 文件 | SHA256 |
|---|---|
| [Widget.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/Components/Widget.cpp>) | `6D2F142ABEDF1BC4EABDA5A59776D98134E6945DC8A84CF4D8D63CF0B26F57D5` |
| [SWidget.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp>) | `AEFD9093803CF15CDA65C8053417CC674B6496735E7C40862F5E89CEDDDC2318` |
| [SPanel.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SPanel.cpp>) | `E33E3BE3D652632985F9EE7A0965E0BC57BC1556E3682EAEB1D8E452D8FC57B6` |
| [WidgetStyle.h](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/WidgetStyle.h>) | `5152CF108765E65ABE5C9E9F1F26B29FA9DCAC8F9486EFA2F4F0921E81932708` |
