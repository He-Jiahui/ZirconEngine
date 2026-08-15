---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/temporal.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_velocity.wgsl
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/06-temporal-pipeline.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VelocityRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/TemporalAA.cpp
  - dev/UnrealEngine/Engine/Shaders/Private/VelocityShader.usf
  - dev/UnrealEngine/Engine/Shaders/Private/TemporalAA.usf
tests:
  - current temporal slice 10 of 10 Rust files reviewed, 1434 lines, 22 inline tests
  - direct behavior and product tests 6 of 6 files reviewed, 2164 lines, 22 tests
  - descriptor velocity-order regression added with static RED to GREEN probe
  - scoped rustfmt 10 of 10 temporal files plus changed descriptor clean
  - two managed focused attempts produced no matching Cargo job and executed zero tests
  - current-source F2 pixels and counters, WPR, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_fix_dynamic_blocked
---

# Graphics temporal current-source结构审查（2026-08-15）

## 当前范围与结论

`zircon_runtime/src/graphics/scene/scene_renderer/temporal/**`当前物理清单10/10个Rust文件：1,434行、1,297个非空行、22条内联测试，fingerprint为`6C9FF2636E26F2E5C11BADA86B43E10CE4E29AF0D190B821CA6A2E2EA5A1658C`。另复读直接调用链和6/6个产品/行为测试文件（2,164行、22 tests）。相关生产和测试文件已有其他会话修改，本轮保留reactive-mask等外部改动，只在取得精确租约后修复feature descriptor的velocity顺序并增加一个回归测试。10/10 temporal文件和changed descriptor逐文件通过`rustfmt 1.8.0 --edition 2021 --check --config skip_children=true`。

current source有三项有效进展：零reactive command绑定共享black view，mask pass/写入均为0；稳定black-mask资源组合已有identity bind-group cache；空对象速度`Load+Store`在录制pass前返回。TAA history本身也是固定双slot并交换角色，没有逐帧创建。旧报告中“零命令仍独立clear”和“空Load+Store仍建pass”结论已失效。

## P0：速度缓冲顺序已静态修复，相机矩阵结构问题仍在

审查时feature descriptor固定先执行`velocity-object(Clear+Store)`，再执行`velocity-camera(Load+Store)`。对象shader用current/previous view-projection与current/previous object transform写完整对象运动；随后相机shader以全屏三角对每个像素无条件写纯相机运动，因此覆盖对象结果。原测试只断言资源名、binding源码形状和空pass guard，没有验证pass顺序、attachment ops或最终运动像素。

本轮先加入`temporal_velocity_composes_camera_before_object_motion`回归测试，并以源码探针得到RED（camera首次位置1181，object 702）；最小实现把camera改为首个`Clear+Store`、object改为随后`Load+Store`，同一探针GREEN（camera 703，object 1182），descriptor rustfmt通过。该修复没有改动reactive-mask外部工作。首次管理验证在Cargo前因协调器`session.register`无终态而失败（request `dcbdb525134e468b8802a4cba18d7075`）；第二次wrapper返回exit 1，但协调器`cargo list`没有对应filter的新job，且同一validator session/pool有foreign runtime job `83ccd6d11f974106a40a00df40acff92`运行。因此目标Rust测试仍为0次执行，不能解释为测试RED或GREEN。

正确结构是相机运动先形成背景，再由对象运动覆盖可见对象，或者在一个typed velocity pass内直接输出最终运动。UE `VelocityRendering.cpp:484-640`先判断是否有draw，非并行路径把clear合入首个velocity render-target load action，使用cached parallel mesh draw commands与instance culling；`VelocityShader.usf:97-98`直接使用共享view当前/上一帧矩阵。它没有在对象pass后增加无条件覆盖的全屏相机pass。

Zircon还重复编译同一相机矩阵：scene uniform先构造current pair+inverse；motion history再构造current pair+inverse+previous pair；camera velocity执行器又重复一次。直接temporal链中每camera至少current pair 3次、inverse 3次、previous pair 2次，随后仍有froxel/post等消费者。camera velocity另有私有uniform upload和逐帧bind-group create。PERF-MVP-346/368应以单个`PreparedCameraMatrices`/view uniform为authority，并把速度合成顺序像素门列为P0；在修正顺序前，TAA运动重投影性能数据不具备算法验收意义。

对象velocity和TAA reactive mesh还是unlit pass，却各自创建完整forward-shadow-receiver bind group，携带shadow atlas、light grid、reflection probes、lightmaps、volumetric、transmission、cookies和irradiance resources，只为满足通用mesh pipeline group 1。UE使用typed `FVelocityPassParameters`，只声明view、scene textures、instance culling与目标附件。PERF-MVP-368必须交付pass-specific最小layout/compiled binding bundle，不能只缓存这份错误的全lighting binding。

## P0：任一history功能触发完整资源包

只要TAA、SSAO、GI、SSR、HZB、exposure或volumetric任一history被请求，`prepare_history_textures`就构造同一个`SceneFrameHistoryTextures`。构造函数无条件创建TAA Rgba16Float双纹理、GI Rgba16Float、GI metadata Rgba16Float、AO Rgba8、SSR Rgba16Float、完整Rgba16Float HZB mip链和exposure双buffer，并用2个render pass清理6张全尺寸纹理；尺寸、HZB plan或froxel quality任一变化会替换整包。

按公开texture descriptor计算、不含驱动对齐/allocator粒度/可选volumetric的显存下限如下：

| viewport | fixed history lower bound / handle |
|---|---:|
| 1280x720 | 44.01 MiB |
| 1920x1080 | 97.68 MiB |
| 2560x1440 | 176.02 MiB |
| 3840x2160 | 390.71 MiB |

4K双视图仅固定包已达781.43 MiB。基数是44 bytes/pixel（TAA双8B、GI 8B、metadata 8B、AO 4B、SSR 8B），再加half-next-power-of-two HZB完整mip链8B/pixel。现有显式viewport/camera release避免了无界泄漏，但不能消除每个live handle的固定放大。PERF-MVP-395应硬切为feature-owned lazy slots和独立generation，HZB-only不得创建其他history，改变froxel quality不得重建TAA/HZB。

## P1：TAA固定fragment算法有明显带宽放大

当前resolve固定为全屏fragment pass。源码级每像素逻辑访问为12次depth load（current 1、closest初始化1、3x3搜索9、foreground 1）、10次scene-color load（current 1、3x3 neighborhood 9）、velocity/history/reactive各1次，共25次；4K即207,360,000次逻辑texture loads/frame，尚未计driver/compiler/cache差异。history重投影用`round + textureLoad`最近点，输出同时写4B Rg11b10 resolved target与8B Rgba16Float history，4K仅输出写带宽约94.92 MiB/frame。

UE `TemporalAA.cpp:746-780,978-1089`按platform/quality/scale/VGPR选择compute permutation；`TemporalAA.usf:1095-1411,2189-2415`以group-shared tile复用depth/color neighborhood，history使用bilinear sampler（`TemporalAA.cpp:949-952`），且只提取实际使用的history输出（`TemporalAA.cpp:1154-1167`）。PERF-MVP-624应先以GPU timestamp/RenderDoc证明各阶段实际成本，再交付capability/quality-selected tiled compute路径、pixel fallback和输出/history格式策略；不得把上述源码复杂度直接写成硬件耗时结论。

## 实施顺序与验收

1. 已把descriptor修正为camera clear/write先、object load/write后并加入顺序/ops回归测试；Render06/17仍须运行Rust测试并补最终velocity像素门：静态对象+移动相机、移动对象+静态相机、两者同时、camera cut/missing history。
2. Render06以prepared view matrices作为唯一authority，velocity不再私有重算/upload；Render02/05为velocity/reactive提供unlit typed layout，Render01合并首个clear并cull无draw pass。
3. Render01/04/06/07/18把history pack硬切为feature-owned lazy slots，Render17记录每handle真实VRAM、texture/view create/destroy、clear passes和affected-only rebuild。
4. Render06/17实现并对比TAA fragment fallback与tiled compute；记录25-load模型对应的实际shader duration、cache/occupancy、read/write bytes和质量，不以指令计数替代capture。

矩阵：720p/1080p/1440p/4K，views 1/2/8，velocity none/camera/object/both/cut，reactive commands 0/1/100/10k，history features each/all，stable/resize/quality change，TAA pixel/compute/quality permutations。硬门：对象velocity不被camera覆盖；prepared matrix build<=1/camera/render-region generation；warm velocity bind-group create=0且unlit pass不绑定lighting；feature-off history slot=0、changed-only rebuild；stable TAA无frame pipeline/bind creation，compute相对fragment的GPU/energy收益有3次以上可复现样本且像素/history/cut/resize通过。

当前没有current-source可运行产品二进制；本轮两次focused validator均未生成匹配Cargo job，旧`target/profiling/zircon_editor.exe`不能作为当前证据。因此Rust回归测试、WPR/xperf、GPU timestamp、energy和RenderDoc均没有current-source动态样本，本模块留在`pending.md`，不进入`review.md`。
