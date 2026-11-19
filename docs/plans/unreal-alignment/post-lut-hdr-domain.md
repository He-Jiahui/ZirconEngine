# 后处理独立源码复核：内部烘焙 LUT 在 tone mapping 前夹掉 HDR 输入

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE/ZE 测试均未在本轮执行。

- 结论：**可建立一个独立、仅问题定义的 Draft；源码确定性高，GPU/产品复现尚未执行。** 本问题已有 `CR-POST-SHADER-0004` TODO；本次是独立确认真实可达调用链及测试缺口，不宣称首次发现。
- 审查源基线：`cb495bb1c952a24c86f4513b8099bc4942c3a87d`。公开 blob 固定到 lead 指定的已发布基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`；只读 `git diff --quiet` 确认这两提交的 `zircon_runtime`、`zircon_runtime_interface` 子树一致（exit 0）。下列 ZE 链接均使用该固定发布基线。
- UE 身份只据本地 [Build.version:1–9](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L1-L9>)：`6.0.0`、`BranchName=UE5`、changelist 0；不推测商业发行版/提交。UE 仅短引及路径/哈希，不附源码副本。
- 权限：只读调查；未修改两侧代码，未运行 Cargo/UE/GPU，未做 Git 写操作/远端动作。唯一新增文件是本报告及同目录的 RHI 领域记录。

## 1. 不一致、精确位置与两侧可达调用路径

**ZE 不是在最终 LDR 输出才夹色，而是把 HDR scene-linear 输入直接当作内部 LUT 的 `[0,1]` 格点坐标。** 因此先于解析 tone mapping 丢失高光差异。

| 阶段 | 固定基线源码、符号、契约 |
|---|---|
| 输入确为 HDR | [core/constants.rs:1–3](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/core/constants.rs#L1-L3>) 的 `SCENE_COLOR_HDR_FORMAT=Rgba16Float`，注释明确内部线性 HDR、最终才写 sRGB；[resource_descriptors.rs:75–86](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/pipeline/pipeline/resource_descriptors.rs#L75-L86>) 对 `SCENE_COLOR` 同样明确 SDR 输出也使用该 HDR backing。|
| 实际 HDR 来源 | [fallback_mesh.wgsl:675–726](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/mesh/shaders/fallback_mesh.wgsl#L675-L726>) 的 `fs_main` 将 albedo/emissive 或 lit/emissive 相加后输出，无 `[0,1]` 上界；[post.wgsl:741–744,908–913](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/post/shaders/post.wgsl#L741-L744>) 还会在查 LUT 前累加 bloom/GI。此处只证明合法数据域可超过 1，不声称某现有项目的实际亮度。|
| bake 可达 | [stack.rs:108–124,367–368](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/render/post/stack.rs#L108-L124>) 的完整构造入口：非默认 grading、启用 tonemap、启用 user LUT 任一即可启用 `ColorLutBake`。|
| graph→实际 compute | [builtin_postprocess_executors.rs:59–82](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/execution/builtin_postprocess_executors.rs#L59-L82>) `color_lut_bake_postprocess_executor` → [context/gpu/post.rs:235–274](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/execution/context/gpu/post.rs#L235-L274>) `record_color_lut_bake_to_resource` → [execute_color_lut_bake/mod.rs:16–72](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/post/execute_color_lut_bake/mod.rs#L16-L72>) `execute_color_lut_bake`，绑定本帧曝光并实际 dispatch。|
| bake 的域 | [color_lut_bake.wgsl:120–132](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/post/shaders/color_lut_bake.wgsl#L120-L132>) `cs_main`：`global_id/(lut_size-1)` 直接作为 source color，随后才 `apply_tonemap`/grading。仅覆盖线性 `[0,1]`。|
| 真实消费者选择 | [context/gpu/post.rs:170–175,194–230](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/execution/context/gpu/post.rs#L170-L175>) 将 graph 的 `COLOR_LUT` 传入 `execute_post_process`；[resources/execute/run/execute.rs:71–72,115,214–249](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/post/resources/execute/run/execute.rs#L214-L249>) 的 `select_effect_lut_texture_views` 在没有可用 user LUT 优先返回时选择 `BakedColorLut3d`，写 shader mode 4。|
| 信息损失点 | [post.wgsl:595–600,613–625](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/post/shaders/post.wgsl#L595-L625>) `sample_effect_lut_3d` 先 clamp 到 `[0,1]`；`apply_tonemap_and_lut` 的 mode 4 直接调用它。|
| 并无稍后补偿 | [post.wgsl:929–935](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/post/shaders/post.wgsl#L929-L935>) 的 `fs_main` 在 mode 4 后跳过再次 grading；[output_transfer.wgsl:29–38](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/post/shaders/output_transfer.wgsl#L29-L38>) 仅从 tonemapped 纹理搬运颜色，不重新 tone map。|

**UE 可达对照：**

1. [PostProcessing.cpp:1094–1113](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessing.cpp#L1094-L1113>) 在启用 tonemap 的普通路径调用 `AddCombineLUTPass`；[1627–1642](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessing.cpp#L1627-L1642>) 将同一 `ColorGradingTexture` 和 `SceneColorSlice` 放进 `FTonemapInputs` 再调 `AddTonemapPass`，不是一份未消费的缓存。
2. [PostProcessCombineLUTs.cpp:735–784](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessCombineLUTs.cpp#L735-L784>) 创建/登记 LUT，[840–864](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessCombineLUTs.cpp#L840-L864>) dispatch `FLUTBlenderCS`；[PostProcessCombineLUTs.usf:488–495](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Shaders/Private/PostProcessCombineLUTs.usf#L488-L495>) 的 `MainCS` 用网格坐标调用 `CombineLUTsCommon` 并实际写纹理。
3. [PostProcessCombineLUTs.usf:345–366](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Shaders/Private/PostProcessCombineLUTs.usf#L345-L366>) 的 `CombineLUTsCommon` 把格点解码为线性色：短引 `LogToLin(LUTEncodedColor) - LogToLin(0)`；另一现存分支是 PQ shaper，不是无条件把线性色限制为 1。
4. [PostProcessTonemap.cpp:868–885](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessTonemap.cpp#L868-L885>) 将场景输入、LUT、clamp sampler 绑定，像素路径在 [1104–1135](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessTonemap.cpp#L1104-L1135>) 实际 `AddDrawScreenPass`；texel-center 参数见 [945–949](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessTonemap.cpp#L945-L949>)。
5. [PostProcessTonemap.usf:529–535](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Shaders/Private/PostProcessTonemap.usf#L529-L535>) 的普通 tone-map 分支实际调用 `ColorLookupTable(FinalLinearColor)`；[190–210](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Shaders/Private/PostProcessTonemap.usf#L190-L210>) 先做配对编码，短引 `LinToLog(LinearColor + LogToLin(0))`，再寻址 LUT。编码定义见 [GammaCorrectionCommon.ush:88–118](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Shaders/Private/GammaCorrectionCommon.ush#L88-L118>)。

独立性：不涉及普通 mesh/viewport affine、perspective side-plane、TBN/Mikk 或 glTF fixture。根据 lead 提供并已读取的 [PR snapshot](<D:/CodexData/worktrees/ze-ue-audit-20261117/reports/ue-module-audit-20261117/github-pr-snapshot.json>)，所列 PR135–145 无此 LUT 问题；现存 TODO 应原样注明，不能包装成未知新发现。

## 2. 有界实际影响与非臆测的判定

- 条件限于：已启用内部 bake，消费者实际选择 **mode 4**；例如无 user LUT、neutral grading、启用 Reinhard tonemap。已成功准备的 user LUT 有选择优先权，不在此结论范围。
- 对任意有限的中性 HDR 输入 `(c,c,c)`，`c>=1` 在当前 mode 4 中均寻址到同一最后格点。输入 1 与 2 的返回值相同是源码的确定推论，不是截图测量。
- 同一现存解析 Reinhard 分支、white point 1、曝光乘数 1、bias 0、neutral grading，契约为每通道 `F(c)=c/(1+c)`。因此 `F(1)=1/2`、`F(2)=2/3` 是代码代数推导，二者应可区分；它们本来都能表示为 SDR 输出。不能用“最终目标是 LDR”解释提前丢失。
- 更一般会压平超过该输入域的高光/改变通道比例；影响限于这条错误 bake 域的结果。不声称全场景必然受影响、具体项目受影响像素数、FPS/显存损失或实机色差。
- 尚未运行原始消费者的 GPU witness；故表述为“源码确定缺陷、实际产品呈现待验证”，不提供伪造 RED/GREEN。

## 3. 根因及 UE 如何解决

根因是把**用户归一化 LUT 的采样地址函数复用为内部 HDR tone-mapping LUT 的地址函数**，而 producer/consumer 都没有 scene-linear↔LUT-domain shaper。LUT 本身可用、计算 pass 可执行、最终输出编码也不是根因。

UE 把有限 LUT 网格的坐标域与场景线性域分开：bake 解码网格、运行颜色变换；消费时把实际场景线性色配对编码，再应用 texel-center 坐标。现存 log 分支使用黑色偏置保证零输入可编码，并覆盖大于 1 的输入。这里借鉴的是双向域契约，不要求 ZE 实现 UE 的 ACES2、PQ/HDR display、局部曝光或多套缓存。

已有 ZE 测试并未排除此问题：

- [color_lut_bake_pipeline.rs:4–22](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/post/tests/color_lut_bake_pipeline.rs#L4-L22>) 仅检入口/源码片段。
- [post_process_pipeline.rs:51–60](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/post/tests/post_process_pipeline.rs#L51-L60>) 甚至固定要求 mode 4 直接使用同一归一化采样，不能把它当行为正确性 oracle。
- [color_lut_readback.rs:96–99,186–204](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/render/post/color_lut_readback.rs#L186-L204>) 从 `[0,1]` 格点生成 expected source；[compiled_scene_readback.rs:106–130,162–175](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/core/render/compiled_scene_readback.rs#L106-L130>) 验证格点颜色变换而非真实 HDR 消费寻址。
- [render_product_post_process.rs:91–203,360–400](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/tests/render_product_post_process.rs#L360-L400>) 虽经过产品图，但现有无 mesh fixture 的 clear RGB 均小于 1；它不能证明 HDR 输入保真。

## 4. Rust/WGSL 沿用 UE 思想，不另造架构

继续由现有 `ScenePostProcessResources` 和 graph executor 负责 bake/消费；只给**内部 LUT**定义同一组固定 shaper 常量及互逆 encode/decode。可以沿用本地 UE 的有界 log 思路（14-stop/中灰基准及黑偏置从同一组常量导出），必须明确可表达上界及越界策略，不宣称无限 HDR。

bake 的 source 变为 `decode(grid_coord)`，mode 4 的输入变为 `encode(scene_linear)`；采样仍用现有 3D 纹理、clamp sampler、texel-center scale/offset。mode 1–3 的 user LUT 输入域不变。用现有编译期拼接方式共享很小的 WGSL helper，而不是增加 LUT 管理器、调度系统或公共渲染特性。

CPU 回读 reference 也应对**编码网格先解码**再运行现有解析变换；这是同一域契约的必要配套，不是把 expected 改成输出本身。另保留独立真实输入的 GPU 比较，防止 producer 和 reference 同错。

## 5. wgpu / ABI / ECS 差异

- UE 是 HLSL/RDG/RHI；ZE 使用 WGSL、wgpu 3D `rgba16float` storage/采样纹理及既有 RenderGraph。不可搬 UE 线程模型、资源引用类或 descriptor 布局。
- `ColorLutBakeParams` / `PostProcessParams` 是 Rust `repr(C)` GPU uniform 边界（[bake params:5–11](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/post/params/color_lut_bake_params.rs#L5-L11>)）。固定域修复可不增字段、不改 mode 编号、不改动态 runtime ABI；不在 `zircon_runtime_interface` 添加 wgpu 对象或 Rust trait object。
- 场景/ECS 仍只提供既有曝光、tonemap、grading 数值；无需新增 component、改变体积叠加或相机提取。
- 不改变 scene HDR/final sRGB 格式。sRGB 编码、f16 量化、LUT 插值必须在验证 oracle 中分层处理；UE ACES2 的 nits golden 不能作为 ZE 拟合 ACES/Reinhard 数字 oracle。

## 6. ZE 精确改法草案（尚未实施）

1. **先开问题定义 Draft，再只加等价回归，测原始实现。** 在现有后处理测试 owner 中，用真实 `ScenePostProcessResources` bake 和实际 `fs_main` 消费 32³ 内部 LUT，输入为明确上传的 scene-linear `Rgba16Float` `{0,0.18,0.5,1,2,4,8}` 中性灰以及至少一组混合通道；不要用可能夹值的 importer/UNORM fixture。
2. 固定 profile 启用 color grading 支持、关闭 bloom/GI/AO/AA/风格效果，settings 选 Reinhard、white point 1、bias 0、manual exposure multiplier 1、neutral grading、无 user LUT。同一 source/settings/color space、相同 viewport，分别使用实际 baked mode 4 与同一生产 shader 的解析 mode 0。确认统计证明 `color-lut-bake`、`post.uber` 实际执行。
3. 主要 oracle：独立 CPU `F(c)`、GPU analytic、GPU baked 三方对照；必须检查 HDR 1 与 2 不再相等。首个回归的线性 RGB absolute error 上限提议 `0.005`，应在执行前用选定 shaper/32 格点的插值误差与 f16 量化界审定，不能为跑绿临时放宽。最终 sRGB capture 是第二层产品见证，须先用同一输出变换计算 expected，不能把 float 与 8-bit 字节直接比较。记录原实现 RED 的具体差异，而非“shader 可解析”。
4. 仅改 [color_lut_bake.wgsl:129–132](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/post/shaders/color_lut_bake.wgsl#L129-L132>) 的 source-domain decode，以及 [post.wgsl:617–618](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/post/shaders/post.wgsl#L617-L618>) 的 mode-4 encode；保留 user mode 3 的旧归一化采样。小 helper 若单列，接入现有 [shader_sources.rs:5–9](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/post/resources/shader_sources.rs#L5-L9>) 与 [color_lut_bake_pipeline.rs:7–11](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/create_pipeline_bundle/color_lut_bake_pipeline.rs#L7-L11>) 两个编译入口。
5. 同 PR 调整 [compiled_scene_readback.rs:106–130](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/core/render/compiled_scene_readback.rs#L106-L130>) 的内部 LUT reference 使其接受 decode 后场景颜色；不要把通用 user-LUT/identity 工具的 `[0,1]` 契约一并改掉。同步替换只认旧源码字符串的断言为分域+行为断言，保留 user 2D/strip/3D 控制组。
6. 原测试不变、同设备/settings 再验证修复 GREEN；才提交同 PR 产品修复。无需依赖 PR135/144/138/140/141/142/143。当前没有测试补丁或生产补丁。

### 测试迁移/推导表

| UE 现存测试或源码契约 | 输入、预期 | ZE 现有落点与正确分类 |
|---|---|---|
| `System.Renderer.ColorManagement.ACES2OutputTransform` / `FACESOutputTransformTest`，[ACESOutputTransformTest.cpp:77–80,124–138,271–308](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Tests/ACESOutputTransformTest.cpp#L77-L80>) | 包含灰 0/0.02/0.18/0.5/1/4、亮 8、负值及混色；检查有限非负、黑到黑、中性保持、非递减灰阶、峰值界。 | 该测试调用真实 ACES2 forward/inverse，不调用 baked LUT 消费；ZE 适配位置与限制见下一行。 |
| 同上测试的 ZE 映射 | 可复用 HDR 输入集和“真实生产变换→GPU readback→不变量”方法；UE numeric golden/round-trip 是 ACES2 专属。 | [render_product_post_process.rs:91–203](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/tests/render_product_post_process.rs#L91-L203>)、[compiled_scene_readback.rs:69–102](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/core/render/compiled_scene_readback.rs#L69-L102>)。**只能称输入/方法适配；不是已有 LUT 寻址测试迁移。** |
| 未找到直接 `LinToLog↔LogToLin` / baked-vs-analytic LUT 自动化原测试 | 搜索了 UE `Engine/Source/**/*Test*.*` 中的 `CombineLUT`、`ColorLookupTable`、`Tonemap`、`EyeAdaptation`，`Engine/Plugins/Tests/**/*.cpp` 同关键字，以及 `Runtime/Renderer/Private/Tests`；找到上项 ACES2 测试、编辑器曝光设置/flags 和 test-image 路径，但没有直接 LUT 域回归。 | 不声称“迁移 UE 原 LUT 测试”。[PostProcessCombineLUTs.usf:345–366](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Shaders/Private/PostProcessCombineLUTs.usf#L345-L366>) + [PostProcessTonemap.usf:190–210](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Shaders/Private/PostProcessTonemap.usf#L190-L210>) 推导 producer/consumer 配对契约；新增 ZE 真 GPU 等价测试。 |
| ZE 自有解析 Reinhard 契约（非 UE 原测试） | 同一曝光/white/grading 下，HDR 1/2/4/8 各自逼近 `c/(1+c)`；非递减一项不足以抓“高光平台”，须明确与解析结果比较。 | 以 [post.wgsl:621–625](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/post/shaders/post.wgsl#L621-L625>) 为生产解析分支；[color_lut_bake_pipeline.rs:4–22](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/post/tests/color_lut_bake_pipeline.rs#L4-L22>) 补真实 bake+consumer 测试，[render_product_post_process.rs:350–400](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/tests/render_product_post_process.rs#L350-L400>) 补端到端 capture。尚未添加/执行。 |

### 可行性、验证门与未验证状态

- 可行性：局部 shader 域映射和 test-only reference 适配；不扩 UE 特性集，不改变资源生命周期或 ABI。最大风险是低亮插值精度、有限域上界和 user LUT 被误套新域，均有明确控制组。
- Draft 开门：本报告的源码结论足够；开 PR 是 lead 后续授权动作，本次未执行。随后必须先拿原始实现的 GPU RED，再在同 PR 修复。
- CPU/结构门：encode/decode 零点、有限性、单调、往返、端点/越界；现有 naga 解析和 WGSL uniform 布局测试。它们不替代 GPU 等价。
- GPU 门：Windows 原生 adapter 明确记录 backend/设备；不得静默 skip。现有资源契约测试可用 `ZIRCON_REQUIRE_GPU_TESTS=1` 强制设备，后处理 witness 也应对设备缺失失败。明确 `zircon_runtime --lib --no-default-features --features native-host,graphics`，仅选本问题新增测试及既有 `color_lut_*`、`post_process_shader_*`、`render_product_post_non_neutral_tonemap_grading_changes_final_frame`、user-LUT 控制组；profile/精确过滤由执行者按新增测试名预先声明。编译产物仅用 drive-root cargo-targets。**本轮不运行这些命令。**
- 未验证：没有 Rust 编译、单测执行、原实现 GPU RED、候选修复 GREEN、UE executable、RenderDoc 或项目实机截图。不以旧测试存在或 TODO 替代结果。

### 后处理领域覆盖/未清事项

| UE 模块 | ZE owner | 已读测试覆盖 | 本轮结论/未清事项 |
|---|---|---|---|
| Renderer/PostProcess、相关私有 shader | `zircon_runtime::core::framework::render::post`；`graphics::scene::scene_renderer::post_process`；graph builtin executor | 上述 bake/消费源码测试、GPU产品 clear-color/LUT 回读测试；UE ACES2 自动化 | 唯一 Draft 候选为本报告的内部 HDR LUT 域缺陷。曝光/DOF/运动模糊 shader 只做了定向阅读，不宣称全面审计。 |
| Engine 体积设置与 Renderer 曝光 | ZE volume evaluator、history domains | [volume_evaluator.rs:66–88](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/render/post/tests/volume_evaluator.rs#L66-L88>) 明确固定其衰减行为；[prepare_history_textures.rs:89–118](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene_renderer_history/prepare_history_textures.rs#L89-L118>) 区分空间历史与曝光域 | 体积公式/曝光历史策略不能只因与 UE 或直觉不同就报 bug；无独立证据，本轮不立第二问题。用户自定义 LUT 优先级也不在本 Draft 顺带修理。 |

### UE 证据 SHA256（本地完整源文件，只列哈希）

| 源文件 | SHA256 |
|---|---|
| [Build.version](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version>) | `3DCD8B6654872D013F2B00AA5F0EF1E0DB89D4E8627CE820554675A27F228EA6` |
| [GammaCorrectionCommon.ush](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Shaders/Private/GammaCorrectionCommon.ush>) | `EA65801253C3950508F754886FACFDEC3CA03D701EE6D63C27DA453BA0572954` |
| [PostProcessCombineLUTs.usf](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Shaders/Private/PostProcessCombineLUTs.usf>) | `A3CC797AC35023B3842803D96561F4AD76E1B6A1C38E4D40F6435F67A35282B9` |
| [PostProcessTonemap.usf](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Shaders/Private/PostProcessTonemap.usf>) | `BC37ED8DCA9E80519F44FFB9CB00F2204E163AE3BDDC4D96C5CBBEDCF92723DA` |
| [PostProcessing.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessing.cpp>) | `2E5A4C058316EF037587C40CF1FD4502C61DFE5234DD72ED51998440DF6BF3CA` |
| [PostProcessCombineLUTs.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessCombineLUTs.cpp>) | `C05DFC80311FF9661F26C7A1F9EBAC94052D9878405BA02ED83AA6896515B149` |
| [PostProcessTonemap.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessTonemap.cpp>) | `9A1AD49A52F1712F7E30D7E010BBB672602213D0C4CF0861957232FE012D819D` |
| [ACESOutputTransformTest.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Tests/ACESOutputTransformTest.cpp>) | `470318A6A23D3F7FC091C670BC59439AC134A9F843EC266250977A77A2399F18` |

## 7. 同一着色器的高光对照

在 RTX 3060 Laptop 上通过 Chromium WebGPU 执行原有烘焙着色器的 `cs_main` 和后处理着色器的 `vs_main`、`fs_main`。表中是线性 RGBA16F 读回值。曝光为 1，使用 Reinhard 算子与中性调色。

| 输入亮度 | 直接映射 | 内部 LUT |
|---:|---:|---:|
| 1 | 0.500000 | 0.500000 |
| 2 | 0.666504 | 0.500000 |
| 4 | 0.799805 | 0.500000 |
| 8 | 0.888672 | 0.500000 |

低亮混色输入和外部三维 LUT 对照均通过。UE 原测的灰阶与亮色输入位于 `Engine/Source/Runtime/Renderer/Private/Tests/ACESOutputTransformTest.cpp` 第 121–138 行。这里的数值判据来自 ZE 自身的 Reinhard 实现。

本次提交给 Runtime 产品图增加了双路径用例。两条路径使用相同的曝光与 HDR 场景输入。直接路径从后处理功能描述符移除烘焙 pass 和可选的 `COLOR_LUT` 读取，以便比较最终图像。 [产品图回归源码](https://github.com/He-Jiahui/ZirconEngine/blob/3f13501656eba837da9da5d29deef8f13ffc7124/zircon_runtime/src/graphics/tests/render_product_post_process/internal_lut_hdr.rs)
