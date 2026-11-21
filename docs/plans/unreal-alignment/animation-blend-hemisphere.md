# Draft 调查：基础姿态混合的独立四元数规范化会选错共同半球

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；D: 审计快照、工作记录与原生回执仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE 测试均未执行；ZE 原生测试结果见文末各阶段节，均为作者本机 Windows x64/MSVC 回执，不是 CI 结果。

状态：**问题定义 + 回归 Draft（PR #147）；原实现 RED 已在作者本机记录，尚无生产修复。** 截至 head `abc69300`，本 PR 只含本文与测试文件 `zircon_plugins/animation/runtime/src/evaluation/pipeline/tests/pose_blend.rs`，未改生产代码。第 1–6 节为 2026-11-17 只读源码阶段原文，其中“本轮”均指该阶段。阶段索引：
- 第 7 节（测试 `0613b6f1`，记录 `8a0dcbab`，2026-11-18）：Base 叶子 10 案，6 过 4 败；
- “两帧动画图求值复现手骨转向”（测试 `cb6a5bca`，记录 `8689ba4e`，2026-11-20）：16 案，11 过 5 败；
- “三个宽角姿态对短弧混合的限制”（`abc69300`）：几何论证与独立数值模型，非引擎实测，约束后续修复策略。
修复策略待 John 决定（见第 4 节“待 John 决定”）。

ZE 只读工作副本 HEAD 为 `cb495bb1c952a24c86f4513b8099bc4942c3a87d`；按 lead 更正，所有 ZE 链接固定到已发布基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。已用只读 `git diff --name-only e893… cb495… -- zircon_runtime zircon_plugins zircon_editor zircon_runtime_interface` 核对这四个源码树无提交差异，并核对本问题相关路径无工作副本修改。保留他人的脏技能。UE 为本地 6.0.0 / BranchName UE5，见 Build.version:1–10（UE 本机参考树 `Engine/Build/Build.version:1–10`）；以下 UE 内容仅短符号描述，不复制实现。

## 1. 两边差异、路径/行号/符号及可达调用

**ZE 问题点不是未实现高级动画节点，而是现有 Base 混合的旋转算术。**

| ZE 实证点 | 短描述 |
| --- | --- |
| [runtime_system.rs:23–61](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/runtime_system.rs#L23-L61>) | `register_runtime_system` 将 `animation.evaluate` 注册到 PostUpdate，执行 `run_animation_runtime_system` → `tick_animation_world`。这是插件生产调度入口，不是孤立数学工具。 |
| [parameter_apply.rs:420–481](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs#L420-L481>)、[tick.rs:228–247](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs#L228-L247>) | 已启用 graph 的 `AnimationGraphPlayerComponent` 加骨架组件可生成 `PendingGraphPoseSample`，然后进入 `resolve_graph_pose_requests_with_demand`。 |
| [graph_evaluate.rs:34–77](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/graph_evaluate.rs#L34-L77>)、[134–195](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/graph_evaluate.rs#L134-L195>) | 采样有效 clip 贡献；Base 贡献汇入 `blend_graph_base_poses`。 |
| [pose_blend.rs:63–141](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_blend.rs#L63-L141>)、[154–178](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_blend.rs#L154-L178>) | 每骨骼归一有效权重后，对每个输入分别调用 `canonical_rotation`：取绝对值最大分量，若其为负就整体取反；将结果直接线性求和后 normalize。**它没有比较输入与共同参考/累加器的点积。** |
| [channel_validation.rs:81–114](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/channel_validation.rs#L81-L114>)、[channel_sample.rs:21–28](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/channel_sample.rs#L21-L28>)、[sample.rs:278–304](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/sample.rs#L278-L304>) | 第 2 节反例使用有限、单位、非零四元数和单键通道，可以通过已有校验；单键返回原值，旋转采样只归一长度，不会提前修复共同半球。 |
| [tick.rs:400–425](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs#L400-L425>)、[pose_apply.rs:13–51](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_apply.rs#L13-L51>) | 混合结果进入姿态快照；能解析到后代节点时还会写回局部 Transform。因此不是仅 UI 显示值差异。 |

UE 对应生产链：`FAnimNode_MultiWayBlend::Evaluate_AnyThread`:112–168（UE 本机参考树 `Engine/Source/Runtime/AnimGraphRuntime/Private/AnimNodes/AnimNode_MultiWayBlend.cpp:112–168`） → `FAnimationRuntime::BlendPosesTogether`:364–383（UE 本机参考树 `Engine/Source/Runtime/Engine/Private/Animation/AnimationRuntime.cpp:364–383`） → `BlendTransform<Accumulate>`:57–62（UE 本机参考树 `Engine/Source/Runtime/Engine/Public/AnimationRuntime.h:57–62`） → `TTransform::AccumulateWithShortestRotation`:976–998（UE 本机参考树 `Engine/Source/Runtime/Core/Public/Math/TransformNonVectorized.h:976–998`）。后者按 incoming 与当前 accumulator 的点积选加/减，结果再统一归一化；vectorized 对应入口:1060–1068（UE 本机参考树 `Engine/Source/Runtime/Core/Public/Math/TransformVectorized.h:1060–1068`） 使用 shortest-path quaternion accumulation。这里只把非 ISPC 路径作为数值对照，没有声称执行过 UE SIMD/ISPC。

模块边界也已核对：Engine.Build.cs:140–163（UE 本机参考树 `Engine/Source/Runtime/Engine/Engine.Build.cs:140–163`） 私有依赖 AnimationCore；229–243（UE 本机参考树 `Engine/Source/Runtime/Engine/Engine.Build.cs:229–243`） 动态列出 AnimGraphRuntime/MovieScene/Tracks；AnimGraphRuntime.Build.cs:10–24（UE 本机参考树 `Engine/Source/Runtime/AnimGraphRuntime/AnimGraphRuntime.Build.cs:10–24`） 公共依赖 Engine/AnimationCore；AnimationCore.Build.cs:9–21（UE 本机参考树 `Engine/Source/Runtime/AnimationCore/AnimationCore.Build.cs:9–21`） 是更低层数学/骨权重模块，不应把上述 Engine 姿态混合误归到它。

本文件引用 UE 文件的 SHA256（字节级标识；不将受限实现搬入 ZE）：

| UE 文件 | SHA256 |
| --- | --- |
| `Engine/Build/Build.version` | `3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6` |
| `Engine/Source/Runtime/Engine/Engine.Build.cs` | `1c6a698c731ca203fc9648a53c13bcc3768fae1fb91cd9ec658a3abfa962e212` |
| `Engine/Source/Runtime/AnimationCore/AnimationCore.Build.cs` | `862eccc11e90f86f164a662a106418d2527467a51acf5126a44b47b6b761bb54` |
| `Engine/Source/Runtime/AnimGraphRuntime/AnimGraphRuntime.Build.cs` | `252b91593a1966a0e2d0697a89c7901d575411afe14dd95b7bf83ad351286e61` |
| `Engine/Source/Runtime/AnimGraphRuntime/Private/AnimNodes/AnimNode_MultiWayBlend.cpp` | `f209e1399e3461bb1c8e14be0ade86717c678ab23915b4d23784c775a0e20c19` |
| `Engine/Source/Runtime/Engine/Private/Animation/AnimationRuntime.cpp` | `34c8a0c17a16e0fb3f22944072d862a5c00a20d29ec9a723fa22172357b63b65` |
| `Engine/Source/Runtime/Engine/Public/AnimationRuntime.h` | `0a042e3ff1c5b56c394b868bbef39a49010406b0a483a84a947eaeb7b0398381` |
| `Engine/Source/Runtime/Core/Public/Math/TransformNonVectorized.h` | `79d02e1d40ca7a95b1e81562f39849a10f867053af674547e2b33da234d7800b` |
| `Engine/Source/Runtime/Core/Public/Math/TransformVectorized.h` | `e788f59596803558db4dc7585ef36e08791e7c645c8fe48c54bd2a345b00f219` |
| `Engine/Source/Runtime/Core/Tests/Math/UnrealMathTest.cpp` | `f0276a149caab14a3cdae4f873e7fb151b701696c1e72022407bb8d6d2101cbe` |
| `Engine/Source/Runtime/Core/Public/Tests/TestHarnessAdapter.h` | `36a8b1a845d187176ba75336e5bb7354502a549059d4e7291b992a86920e5a51` |
| `Engine/Source/Runtime/Core/Public/Misc/LowLevelTestAdapter.h` | `49ac1a309de93d00f5a4e5e949dfcce795a7ffe1efa5e905694f849b4d65d8c7` |

## 2. 有界实际影响

最小合法 fixture：一个骨骼、两个单键常量旋转 clip、Base Blend 权重各 0.5，无 mask/additive/IK，translation=0、scale=1。四元数均按 ZE `xyzw`：

| 项 | 由现有源码直接推出的值，不是执行回执 |
| --- | --- |
| 输入 | `qa=(0.8,-0.6,0,0)`，`qb=(0.6,-0.8,0,0)`；两者长度 1、点积 0.96。 |
| 正确同半球中点 | `normalize(qa+qb)=(1/√2,-1/√2,0,0)`；两输入相差约 32.52°，中点应距各输入约 16.26°。50/50 时 normalized-linear 与 shortest slerp 的中点一致，不涉及两种插值曲线之争。 |
| 当前 canonical 后 | `qa` 保持，`qb` 因最大绝对分量 y 为负而取反；二者点积变为 -0.96。 |
| 当前输出 | `normalize((qa-qb)/2)=(1/√2,1/√2,0,0)`。它与正确中点的绝对点积为 0，代表 **180° 姿态误差**，不是 `q`/`-q` 的等价符号差。 |
| 几何可观测量 | 正确中点把局部 X 轴转到 -Y，当前结果转到 +Y。任何挂在该骨骼下、非轴对称的节点都可能明显翻转。 |

范围限于走该 helper 的基础姿态多输入混合，例如已启用图播放器；共用 `blend_weighted_poses` 的状态图/嵌套图也需回归，但本 Draft 不据此再开问题。单 clip 未混合、普通通道 slerp、PoseBuffer 的 `shortest_slerp` 不因这个反例自动判错。不声称 GPU 蒙皮产品可达、不声称全部动画都有该错误，也不声称已在编辑器中重现。最小实例已足以显示现有算法违反近邻旋转混合，而不要求新增任何高级功能。

## 3. 根因及 UE 的解决方式

`canonical_rotation` 能让**同一个旋转**的 `q` 与 `-q` 使用同一数值代表，但不能保证**不同近邻旋转**仍落在同一半球：绝对值最大分量从 x 切换到 y 且其符号不同就会发生不连续。把每个输入独立 canonical 再相加，混淆了“唯一符号代表”与“相对共同参考的最短弧”。最终 normalize 只能修长度，不能恢复已选错的弧。

UE 的 `AccumulateWithShortestRotation` 是相对当前累加器进行符号调整，`BlendPosesTogether` 末尾归一。对于第 2 节两输入，无论先放哪一个，点积为正，后一个都不会被反向，因此得到正确的共同半球中点。

ZE 已有 [PoseBuffer blend.rs:149–153](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pose_buffer/blend.rs#L149-L153>) 也先按 dot 纠正相对符号；这里是旁证，不是要求把多输入 normalized-linear 混合改成多次 slerp。

## 4. Rust 如何沿用 UE 逻辑

可以沿用的核心是不变量：**骨骼旋转贡献在加权求和前必须相对共同参考/累加器选同半球，再对最终结果 normalize**。无需移植 UE C++、SIMD 布局或对象模型。

不能机械照搬的地方是 ZE 已承诺输入重排稳定：[既有 masked blend 计划:9–24](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/08c-p1-11-masked-blend.md#L9-L24>) 明确要求重排及 `q/-q` 数值结果稳定。UE 相对变化中的 accumulator 选择符号，对一般三输入以上不自动满足任意重排不变；不能为通过两输入反例而悄悄删除这个合同。

拟评估的 Rust 方案：逐骨骼先过滤 mask/非法权重，确定稳定参考贡献；其余输入先根据与参考的 dot 调整符号，再进行确定顺序的 weighted sum，最终才做一次符号 canonical（如仍需要数值代表唯一）及 normalize。若采用严格 UE 累加器方案，则必须将 source-slot 顺序设为明确合同并证明不破坏当前重排要求；两方案的多输入边界要在测试阶段确定。现有 [compiled_graph/evaluate.rs:73–84](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/compiled_graph/evaluate.rs#L73-L84>) 已按稳定贡献 key 排序，可复用其顺序来源，但 `GraphWeightedPose` 目前未保存此身份，不能假设任意调用者都自动有相同次序。

## 5. ECS / ABI / 数据结构差异

- UE 通过 AnimNode / `FCompactPose` / `FTransform` 求值；ZE 是实体上的播放器组件，插件 PostUpdate 系统消费资源快照，再发布 immutable pose snapshot。需保持 world replacement epoch 及 event admission/defer 的既有事务路径，不要将纠正 quaternion 符号扩成调度重构。
- [AnimationPoseOutput:8–11](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/animation/pose_output.rs#L8-L11>) 是带骨名/局部 Transform 的序列化 Rust DTO；[AnimationPoseBone:5–9](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/animation/pose_bone.rs#L5-L9>) 不是 UE compact-pose 内存布局。四元数反例和 dot 规则与坐标系左右手、Euler 顺序无关，移植测试时仍不能把 UE Rotator 欧拉值不加核对地当作 ZE 欧拉输入。
- [GraphWeightedPose:12–18](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_blend.rs#L12-L18>) 及混合 helper 是插件内部 Rust 类型/函数；本问题原则上不要求新增稳定 ABI 字段、修改资源 schema 或提高 [插件 ABI v3](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/plugin.rs#L18-L24>)。若为了稳定参考选择新增 source key，应先留在内部贡献记录，不能把 `Vec`、ECS World 或 UE 对象跨动态边界传递。
- 保留每骨骼 mask 后的权重归一、零/负/非有限权重过滤、translation/scale 既有语义。不要把本修复与骨架路径身份、逆绑定矩阵、GPU 蒙皮实现或高级 Pose Library 混在同一 Draft。

## 6. 具体 ZE 修改草案、测试迁移表与可行门

**顺序仍为：本问题六点文档独立 Draft → 同一 Draft 补等价/回归单测并实测 RED → 再评估窄修复；本轮不执行后两步。**（2026-11-17 阶段原文。其后第二步已完成，见第 7 节及其后；第三步“窄修复”仍未开始，待第 4 节决定。）

拟修改 owner：只在 [pose_blend.rs 的 Base rotation accumulation:100–178](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_blend.rs#L100-L178>) 引入共同半球选择，把逐输入 canonical 从数值求和路径移出；必要时通过内部贡献身份稳定参考/顺序。不要改变 additive 分支或全局通道插值。测试首先落现有 [pipeline/tests/pose_blend.rs](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/tests/pose_blend.rs>)，再用已有 [physics_tick.rs:378–430](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick.rs#L378-L430>) 的真实 Level tick 图混合 harness 加生产路径 witness。

| 分类 / UE 已有测试名与定位 | 输入 / 期望 | ZE 现有落点 / 限制 |
| --- | --- | --- |
| **优先列出已有 UE 测试，但不是此缺陷直接迁移**：`FVectorRegisterAbstractionTest` / `System.Core.Math.Vector Register Abstraction Test`，UnrealMathTest.cpp:2357（UE 本机参考树 `Engine/Source/Runtime/Core/Tests/Math/UnrealMathTest.cpp:2357`），内部 `TestQuatLerp` 子例 3652–3721（UE 本机参考树 `Engine/Source/Runtime/Core/Tests/Math/UnrealMathTest.cpp:3652–3721`）。 | float 输入由两组 Rotator `(300,-45,0)` / `(10,270,30)` 构造，alpha=±0.25；double 对照 `(30,45,0)` / `(10,-270,-130)`，alpha=±0.25。期望 quaternion Lerp 特化、Slerp 与旧参考 slerp 在 `1e-6` 内一致，覆盖正/负 raw dot。 | 可为已有 [animation_compiled_evaluator_contract.rs:532–546](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/animation_compiled_evaluator_contract.rs#L532-L546>) 的 clip quaternion 插值控制扩充输入；必须先按轴/旋转序映射成等价 quaternion。**不能将 UE 的 slerp 期望直接套到任意 alpha 的 normalized-linear 多姿态混合上，也不声称该测试已检测 canonical seam。** |
| **源码推导，非 UE 测试迁移**：未找到相同 seam 的 UE 自动化测试。 | 单骨骼 qa/qb 如第 2 节、0.5/0.5，期望与 `(1/√2,-1/√2,0,0)` 旋转等价、输出有限单位；同时比较旋转后的 X 轴应为 -Y，而非仅比较 quaternion 分量。 | [pipeline/tests/pose_blend.rs:10–94](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/tests/pose_blend.rs#L10-L94>)；增加正序、逆序、分别替换 q/-q、mask 排除无效首贡献、第三贡献稳定性。保留现有归一/非法权重/重排/符号对照。 |
| **源码推导，非 UE 测试迁移**：生产入口 witness。 | 一骨架、两个单键常量旋转 clip、Blend 参数 0.5、GraphPlayer playing，tick 0 秒已足够采样。快照旋转应与正确中点等价；若建命名子节点，则局部 X 方向同样应为 -Y。 | [physics_tick.rs:378–430](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick.rs#L378-L430>) 已有 `level_tick_blends_animation_graph_clip_pose_weights` 只验 translation；复用注册资源/播放器/真实 tick 入口，不以独立数学脚本替代产品链。 |

搜索边界如实记录：已搜索 UE `Runtime/Engine/Private/Animation` 及 `Runtime/Engine/Private/Tests`、`Runtime/AnimGraphRuntime`、`Runtime/AnimationCore/Private/Tests`、`Runtime/Core/Tests/Math`、`Editor/AnimGraph`、`Editor/AnimationBlueprintEditor` 的 `IMPLEMENT_*TEST` / `TEST_CASE*` 及 blend/pose/quat/shortest 相关符号；AnimationCore 的 BoneWeights tests 是顶点骨权重，不是姿态 quaternion oracle，Engine 中找到的动画 ISPC codec tests 也不是 Base mixer seam 测试。`TEST_CASE_NAMED` 的自动化适配依据 TestHarnessAdapter.h:12–16（UE 本机参考树 `Engine/Source/Runtime/Core/Public/Tests/TestHarnessAdapter.h:12–16`）、LowLevelTestAdapter.h:55–118（UE 本机参考树 `Engine/Source/Runtime/Core/Public/Misc/LowLevelTestAdapter.h:55–118`）。未穷尽全部 UE 插件/外部测试库，因此只说“本次范围未找到”，不说 UE 没有测试。

可行门与依赖：

1. **Draft 门**：已核对当前开放 PR #135–145、近期已合测试 #131–134 和本地相关动画计划/历史；未发现本问题的独立 Draft。不将该范围冒充全部历史 PR 正文穷尽。去重范围补记（2026-11-17）：上述去重只覆盖 #131–145 快照。同日另有 #146–#179 开放；跨 PR 索引见 Draft #169 的 `docs/plans/unreal-alignment/module-coverage.md`（未合入）。按标题核对 #146–#179，未见与本问题同根因的 PR；这不是正文级查重。#180–#261 未经该标题核对。近期作者本机 PR 快照 `github-pr-snapshot.json`（未随 PR 发布）仅 #131–145。既有 masked blend 计划描述的是已修的全局权重衰减，此处是不同根因、不同旋转 witness；Pose Library 大计划仅提到 hemisphere/order 政策，没有这条反例。
2. **测试门**：尚未编译或运行上述 ZE/UE 测试；需在独立 Draft 内先补测试并实测原码 RED。后续最小包是 `zircon_plugin_animation_runtime`；其 [manifest:9–22](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/Cargo.toml#L9-L22>) 引入 runtime animation/physics-contracts，真实 tick harness 还依赖 physics 插件。精确 features/targets/profile 必须由后续执行者重新声明，本轮没有 Cargo 命令或通过回执。更新（2026-11-20）：第 7 节已执行 `test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --no-default-features --features native-host --lib pose_blend::tests:: --locked --offline --jobs 1 -- --test-threads 1`（原文未写 `cargo` 前缀，⟨FILL：确认是否为 `cargo test …`⟩），Rust 1.94.1、Windows x64/MSVC、`test` profile、退出 101，回执 SHA-256 见第 7 节。“两帧动画图求值复现手骨转向”的 16 案运行尚缺：命令 ⟨FILL⟩、运行时 commit ⟨FILL⟩、工具链 ⟨FILL⟩、退出码 ⟨FILL⟩、回执/stdout/stderr SHA-256 ⟨FILL⟩。本 PR 回归均为 `--lib` 单测；两条图级用例直接调用 `resolve_graph_pose_requests_with_demand`，第 6 节表第 3 行计划的 `physics_tick.rs:378–430` 真实 Level tick 产品 witness 尚未新增，因此 physics 插件依赖尚未进入执行门。
3. **修复可行门**：共同半球的局部算术修正可行性高，但多输入确定性与 tie/near-zero accumulator 处置尚未设计验证。不得仅用两输入 PASS 宣告修复完成；需保留 08C P1-11 的 reorder 数值合同，覆盖 masks、q/-q、近 seam、小权重和三输入 permutations，再决定最小生产 diff。

   更新（2026-11-20）：上述覆盖已落为 `pipeline/tests/pose_blend.rs` 中 12 个 `anim147_*` 回归（原有 4 案保留不改），原实现结果：
   - 跨缝等权 `anim147_equal_weight_seam_uses_shortest_midpoint`：失败
   - mask 排除首贡献 `anim147_leg_mask_excludes_first_anchor_and_rescales_applied_weights`：失败
   - 非法权重不作参考 `anim147_invalid_first_weight_cannot_anchor_rotation`：失败
   - q/−q `anim147_equivalent_quaternion_signs_preserve_physical_rotation`：通过
   - 三近邻 6 排列 × 8 符号 `anim147_adjacent_rotations_preserve_orientation_across_orders_and_signs`：失败
   - 广角排列与符号 `anim147_wide_rotations_preserve_orientation_across_orders_and_signs`：通过
   - 图级跨缝两帧 `anim147_r32_graph_clip_crosses_canonical_seam_without_pose_jump`：失败
   - 图级广角权重交叉 `anim147_r32_graph_wide_weight_crossing_does_not_switch_pose_branch`：通过
   - 极小离群权重 `anim147_r32_vanishing_outlier_cannot_switch_the_close_pair_branch`：通过
   - 普通非等权 NLERP `anim147_r32_ordinary_pair_matches_normalized_linear_interpolation`：通过
   - 恰正交平局 `anim147_r32_exactly_orthogonal_pair_has_one_equidistant_tie`：通过
   - 带 +90° 桥的近正交 `anim147_r32_near_orthogonal_pair_with_a_bridge_cannot_snap_on_dot_sign`：通过

   合计 11/16 通过。
4. **验收边界**：本轮未启动 UE 二进制、ZE Runtime/Editor、GPU 或渲染截图；无法宣称 GPU 蒙皮、可视化抖动幅度、性能或跨平台已验收。调度/ABI/资源格式不变的推断仍要在最终 diff 中复核。

## 7. 原实现的 Base 叶子原生回归（2026-11-18）

[仅追加回归的提交 `0613b6f19d508671c1c7e53bc123f98771008126`](https://github.com/He-Jiahui/ZirconEngine/blob/0613b6f19d508671c1c7e53bc123f98771008126/zircon_plugins/animation/runtime/src/evaluation/pipeline/tests/pose_blend.rs)。Windows x64/MSVC，Rust 1.94.1，插件独立 workspace 的 `test` profile，`test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --no-default-features --features native-host --lib pose_blend::tests:: --locked --offline --jobs 1 -- --test-threads 1`。运行的是 `blend_graph_base_poses` 的内建 libtest，未构建整个插件工作区或运行 Level/GPU/UE。

原有四案不改且均通过；新增六案逐名测试结果：

| 用例（`anim147_` 后缀） | 区分什么 | 实际 |
|---|---|---|
| `equal_weight_seam_uses_shortest_midpoint` | 两项近邻跨规范符号缝；独立物理中点 | 失败 |
| `leg_mask_excludes_first_anchor_and_rescales_applied_weights` | 排掉未选首骨，按腿骨的有效权重归一（含平移和缩放控制） | 失败 |
| `invalid_first_weight_cannot_anchor_rotation` | 非法权重不应成为腿骨方向参考 | 失败 |
| `equivalent_quaternion_signs_preserve_physical_rotation` | 同一旋转的 q/−q，两个顺序 | 通过 |
| `adjacent_rotations_preserve_orientation_across_orders_and_signs` | 三个近邻、6 排列与 8 个独立正负号的物理等价 | 失败 |
| `wide_rotations_preserve_orientation_across_orders_and_signs` | 广角两套权重（含等权），不预设绝对角，仅比较物理朝向 | 通过 |

**总计 6 过、4 败、0 忽略、112 过滤，退出 101。** 失败用例在第一个不满足的断言停止，不谎称它后面的排列、另组输入或权重档位已经执行。原始输出首个实际误差：

- `adjacent_rotations_preserve_orientation_across_orders_and_signs`：`([0, 1, 2], 0): abs(dot)=0.99984777; rotated X=Vec3(-0.034894228, -0.99939114, 0.0), expected Vec3(-0.0, -1.0, -0.0)`。
- `equal_weight_seam_uses_shortest_midpoint`：`("Z seam", 0): abs(dot)=0; rotated X=Vec3(0.0, 0.99999994, 0.0), expected Vec3(-0.0, -1.0, -0.0)`。
- `invalid_first_weight_cannot_anchor_rotation`：`0.0: abs(dot)=0; rotated X=Vec3(0.0, 0.99999994, 0.0), expected Vec3(-0.0, -1.0, -0.0)`。
- `leg_mask_excludes_first_anchor_and_rescales_applied_weights`：`0.25: abs(dot)=0; rotated X=Vec3(0.0, 0.99999994, 0.0), expected Vec3(-0.0, -1.0, -0.0)`。

实际 Windows 工作树（CRLF）测试源 SHA-256 `5271548a88503729344d33ffd253c94632a29f76e2f76adddc34b85731a91af5`，Git blob 按 LF 的 SHA-256 `b9edf5f4ff998e39231f128a51ee1629c5e679e696558529f007a1845bd926d1`；换行归一后源码一致。回执 `1b569bf98d8f236a7addd4a358925b6d095adc56d7612703174f83cd40c506d4`，stdout `d8ba8287bdf1c188cf4444eaceaf2e405c2aad0318baf4e4a7ef2b056e745a7d`（3,452 B），stderr `e2a8a644a75896a9c5b5ab2af85df71e55a20efb727c61980aedba6504e3d326`（269,786 B）；17 项选取输入复核。UE `UnrealMathTest.cpp:3652–3721` 的 `TestQuatLerp` 中 float 旋转的输入是正 RawCosom，double 旋转的输入是负 RawCosom，±alpha 与 dot 正负是两回事。它只校验双输入插值，不是这些 Base 多输入 oracle，未执行 UE。

**暂不提交生产修复。** 第一有效输入作锚会引入与重排有关的大误差；取每骨最大权重（含确定平局）也不能仅凭这十案宣告可用：在同一组 `−170°/0°/+170°`、外侧权重由 `(1+2^-15, 1-2^-15)` 互换、中间固定 1 的独立标量设计检查中，该拟议策略前后物理方向相隔约 **106.611°**，而原逐项规范在此特定比较仅约 0.000489°。这些数**不是引擎实测**，也没有为所有广角规定某个 UE 角。下一门是确定可接受的共同半球策略，并用同组冻结测试、广角权重交叉/正交换号与真实 Level 图采样验证；不改 additive、ABI、平移缩放或现有无贡献回退。

## 两帧动画图求值复现手骨转向

真实骨架和剪辑经过 Blend 图求值，手骨两帧预期只转 0.25°，实际转了 179.750°。第 7 节的十项（原有 4 案 + 首批 6 案）仍为 6 过 4 败；本节新增六项 5 过 1 败；合计 16 项 11 过 5 败。 两次采样的根骨遮罩、根骨朝向和手骨平移均已通过断言。在其中一段剪辑从 −89° 转向 −91° 时，混合后的手骨应从 −89.375° 转向 −89.625°。测试从两次实际输出的单位四元数求出相对转角。

| 新增用例 | Windows 原生结果 |
|---|---|
| 具名骨架与剪辑的两帧 Hand 方向 | 失败 |
| 三段广角剪辑轻微调整混合权重 | 通过 |
| 极小离群权重对邻近两骨姿态的影响 | 通过 |
| 普通二输入的非等权归一化线性插值 | 通过 |
| 恰好正交时，输入顺序与表示符号的等距中点 | 通过 |
| 存在第三个固定姿态时，邻近输入的小幅变化 | 通过 |

同一程序以 `zircon_plugin_animation_runtime` 的 `native-host` 配置运行，16 个具名用例逐项计数。两条图级用例注册了骨架、剪辑、Blend 参数和遮罩，经生产图解析、采样与姿态混合取得结果。

UE 的 `System.Core.Math.Vector Register Abstraction Test` 中，`TestQuatLerp` 覆盖了四元数正负点积。`Engine/Source/Runtime/Core/Tests/Math/UnrealMathTest.cpp:3652–3721` 调用的四元数 `FMath::Lerp` 特化使用 Slerp。这一原测提供方向选择参照。这里普通非等权用例按 ZE 当前的归一化线性插值检查。多输入广角区间和权重跨越相等点还需要约定稳定的姿态选择。

[六个新增回归及图输入](https://github.com/He-Jiahui/ZirconEngine/blob/cb6a5bca0cf789a6e0bf0ec6b19eb8bdf2e6baac/zircon_plugins/animation/runtime/src/evaluation/pipeline/tests/pose_blend.rs#L334-L691)

## 三个宽角姿态对短弧混合的限制

同一根骨骼绕 Z 轴分别旋转 `A = −170°`、`B = 0°`、`C = +170°`。让内层 Blend 以 `u` 混合 A、C，再让外层以 `v` 混入 B，三段剪辑的权重就是 `((1−v)(1−u), v, (1−v)u)`。下面考察只按本帧输入求值、且每一条双输入边都精确使用短弧归一化线性插值（NLERP）的方案。[图的权重计算](https://github.com/He-Jiahui/ZirconEngine/blob/8689ba4e2ca7af6f332403b13cb3f57183bad085/zircon_plugins/animation/runtime/src/evaluation/compiled_graph/evaluate.rs#L37)

按当前符号约定，A·B、B·C、C·A 三个单位四元数点积约为 `−0.087156`、`+0.087156`、`+0.984808`。沿参数方格的边界走 A→C→B→A，每段都选择短弧时，连续跟踪的四元数会从 `q` 走到 `−q`。两者表示同一个旋转，但这个符号变化约束了方格内部的插值。

把两个参数都按 `2⁻¹⁶` 取样，再将每个小方格沿对角线分为两个三角形。将每条边两端四元数的点积符号相乘，内部边各出现两次，边界的总乘积为 `−1`。若有点积为零，该边两端物理转角已相差 180°。否则至少有一个三角形的三个点积 `a,b,c` 满足 `abc < 0`。如果三条边的物理转角都小于 120°，则三个点积的绝对值都大于 `1/2`，其 Gram 行列式 `1 − a² − b² − c² + 2abc` 会小于零，与三个实单位向量的 Gram 行列式非负矛盾。因而某个轴向或对角邻步上，**每个参数至多变化 `2⁻¹⁶`，姿态却至少变化 120°**。

独立数值模型复算了边界的 262,144 个点，边界上最大的相邻转角小于 `0.003205°`。对同一组三姿态，按当前生产公式逐项选符号、加权、归一化，模型得到整个网格相邻转角的上界为 `0.010924°`。这两个数来自几何复算。上节记录的 ZE 原生回归为 **11/16 通过**，其中手骨两帧预期转动 `0.25°`，实际转动 `179.750453°`。[生产混合公式](https://github.com/He-Jiahui/ZirconEngine/blob/8689ba4e2ca7af6f332403b13cb3f57183bad085/zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_blend.rs#L63) · [手骨两帧断言](https://github.com/He-Jiahui/ZirconEngine/blob/8689ba4e2ca7af6f332403b13cb3f57183bad085/zircon_plugins/animation/runtime/src/evaluation/pipeline/tests/pose_blend.rs#L354)

一个可单独验证的修复方向是按图、骨骼和剪辑版本预先固定单位参考四元数 `p`。若遮罩允许的全部连续采样都满足 `|p·q| > 1/√2` 并留有浮点余量，就可以朝 `p` 统一取号，此时任意两项也落在彼此的短弧一侧。候选实现需要验证手骨跨帧、宽角素材的处理，以及同 UUID 换入新剪辑、骨架或遮罩后的重新判断。

本节为数学论证与独立数值模型，不是 ZE 或 UE 实测；Gram 步骤可直接复核（`|a|,|b|,|c|>1/2` 且 `abc<0` 时 `1−a²−b²−c²+2abc < 0`）。模型脚本未随 PR 发布（本 PR 只含本文与 `pose_blend.rs`）。本节结论是第 4 节“待 John 决定”第 1 条的依据。
