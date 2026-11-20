# 颜色 LUT 烘焙应使用本次相机提交的生效图

## 1. 正常提交可选中烘焙，却得到黑色低亮像素

调用 `RenderFramework::submit_frame_extract` 时，调用方可以提供相机、效果参数和后处理体积。渲染器按视口选择生效参数。在 Windows 上向真实产品图提交非中性调色帧，RTX 3060 Laptop 的 DX12 和 Vulkan 后端都报告已执行 `post.color-lut-bake`。读回的 32×32×32 个 RGBA16F 格点中，32,768 个 alpha 全部偏离预期的 1。输入线性 RGB `(0, 0.18, 0.5)` 时，内部 LUT 路径的最终 sRGB8 为 `(0, 0, 0)`，同设置的直接路径为 `(0, 109, 156)`。两种后端均使用同一个测试程序。既有用户 LUT 和非中性调色测试也在烘焙结果断言处失败。[现有产品用例](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/tests/render_product_post_process.rs#L91-L203)

## 2. 编图和命令录制分别查询两份图

提交阶段依据相机、质量设置和预算生成后处理栈及生效图，再把它们存入该帧的 renderer-owned snapshot。`gpu.post_process()` 通过 `ViewportRenderFrame` 读取这份快照。[生效图构建](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/runtime/framework/extract/submission/build.rs#L351-L415) · [帧访问器](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/types/viewport_render_frame.rs#L89-L102)

`color_lut_bake_postprocess_executor` 却在 `gpu.frame_extract().post_process.graph` 中查找 `ColorLutBake`。源图缺少节点时，它在录制计算命令之前返回成功。图执行记录仍会登记该 executor。原生跟踪显示，生效图已分配 32³ 的 D3 纹理，并按 `exposure-resolve → color-lut-bake → uber` 排序。真正的 dispatch 只发生在 `record_color_lut_bake_to_resource` 中。[烘焙守卫](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/execution/builtin_postprocess_executors.rs#L59-L81) · [命令录制](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/execution/context/gpu/post.rs#L235-L288)

## 3. Unreal 的生产路径和真实测试依据

UE 的 `Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessing.cpp` 第 1094–1113 行从当前 View 选择 CombineLUT，随后把选定的纹理交给 tonemap。`Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessCombineLUTs.cpp` 第 735–784、840–864 行使用当前 View 的结果与参数建立 RDG pass。`Engine/Source/Runtime/RenderCore/Public/RenderGraphUtils.h` 第 479–493、538–565 行使用该 pass 捕获的参数录制计算命令。

UE 已注册的 `System.Renderer.ColorManagement.ACES2OutputTransform` 要求 NonNullRHI。`Engine/Source/Runtime/Renderer/Private/Tests/ACESOutputTransformTest.cpp` 第 73–80、208–267 行建立真实 RDG 计算和拷贝，执行后从 GPU 读回颜色值。ZE 回归沿用其计算与读回方法，数值判据使用 ZE 自身的 Reinhard 算子。UE 测试本身验证 ACES2 变换。

## 4. ZE 的等价回归

在两个独立视口提交同一组 Reinhard 参数、曝光 1 和线性 RGB `(0.25, 0.5, 0.75)`。第一帧沿用提交前的源图，另一帧仅对同一份数据的副本调用 `post_process.rebuild_graph(false, false)`。先运行第一帧，然后分别捕获两者的 HDR 源和最终像素。

两帧均应在生效图中建立烘焙 pass。按 `executor_id` 读取该 pass 的 `dispatch_count`，再核对 32³ 格点、alpha 及最终线性 RGB。解析预期为 `(0.2, 1/3, 3/7)`，最终像素的线性绝对误差上限为 `0.007`。若仅第一帧缺少 dispatch，而手动重建的一帧具有正确格点和像素，就能把故障定位到守卫选择的图。[逐 pass 统计](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/render/backend_types/graph_reports.rs#L439-L503) · [体积纹理读回](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/core/render/compiled_scene_readback.rs#L69-L103)

## 5. 修复位置与适配范围

在烘焙执行器中通过 `gpu.post_process().graph` 检查节点。该访问器读取渲染器为当前帧选定的覆盖值，低层调用没有覆盖值时继续使用原提取数据。录制和采样便使用同一个生效栈。GPU 图仍拥有纹理、曝光资源和计算命令的生命周期。[GPU 上下文访问器](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/execution/context/gpu.rs#L458-L460)

## 6. 与已有问题的关系及验收条件

已核对 247 个 PR，含 113 个开放、133 个合并和 1 个关闭的 PR。本问题影响合法低亮输入和整张 LUT 的写入。#154 处理已经烘焙的 LUT 对超出单位立方体颜色的采样。#120 曾涉及同一执行器文件，#137 修正图形测试的模型夹具借用。依次验证本问题的低亮双视口、既有非中性与外部 LUT 用例，再与 #154 组合验证高光。[高光采样 #154](https://github.com/He-Jiahui/ZirconEngine/pull/154) · [历史变更 #120](https://github.com/He-Jiahui/ZirconEngine/pull/120) · [测试夹具 #137](https://github.com/He-Jiahui/ZirconEngine/pull/137)

## 7. 低亮双视口的原生对照

Windows DX12 在 RTX 3060 Laptop 上执行了同一份相机参数。生效图在两个视口均包含 `post.color-lut-bake`。直接提交的视口记录 0 次烘焙 dispatch，32,768 个格点的 alpha 偏离预期，最终 sRGB8 为 `(0, 0, 0)`。调用方先重建提取图时，视口记录 1 次 dispatch，格点 alpha 和 RGB 全部通过，最终 sRGB8 为 `(124, 156, 175)`。

用已烘焙的 LUT 输入 `(1, 2, 4)`，三个通道的最终 sRGB8 均为 187。低亮调度回归由本草稿负责，域外通道采样见 #154。 [双视口测试](https://github.com/He-Jiahui/ZirconEngine/blob/d0c2b5e5b78c51758a7bc226623da1c31efccab5/zircon_runtime/src/graphics/tests/render_product_post_process.rs#L316) · [高光采样 #154](https://github.com/He-Jiahui/ZirconEngine/pull/154)
