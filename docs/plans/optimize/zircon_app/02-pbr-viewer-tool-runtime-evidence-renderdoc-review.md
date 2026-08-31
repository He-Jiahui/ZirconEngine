---
related_code:
  - zircon_app/Cargo.toml
  - zircon_app/src/bin/zircon_shader_pbr_viewer
  - tools/shader-pbr-profile-contract.ps1
  - tools/zircon_profile_shader_pbr_viewer.ps1
  - tools/zircon_validate_shader_pbr_viewer_evidence.py
  - tools/zircon_validate_shader_pbr_gpu_timing_evidence.py
  - tools/zircon_validate_shader_pbr_renderdoc_replay.py
  - tools/zircon_summarize_shader_pbr_profile.py
  - tools/session_coordinator/artifact_receipts.py
  - docs/tests/runtime/shader
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/performance/01/2026-08-09-mvp-render-submission-architecture-audit.md
  - docs/tests/runtime/shader/2026-08-13-startup-performance-architecture-review.md
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_m5_deferred_lighting_20260728_dx12_startup.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderCaptureInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderCaptureInterface.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/ImageWriteQueue/Public/ImageWriteQueue.h
  - dev/UnrealEngine/Engine/Source/Developer/FunctionalTesting/Public/AutomationScreenshotOptions.h
  - dev/UnrealEngine/Engine/Source/Developer/FunctionalTesting/Private/ScreenshotFunctionalTestBase.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Tools/LookDev/Compositor.cs
  - dev/Graphics/Tests/SRPTests/Packages/com.unity.testing.urp/Scripts/Runtime/UniversalGraphicsTestBase.cs
  - dev/Graphics/Tests/SRPTests/Packages/com.unity.testing.urp/Scripts/Runtime/UniversalGraphicsTests.cs
  - dev/bevy/crates/bevy_render/src/view/window/screenshot.rs
  - dev/bevy/crates/bevy_dev_tools/src/easy_screenshot.rs
  - dev/Fyrox/fyrox-core/src/task.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/godot/core/object/worker_thread_pool.cpp
  - dev/godot/core/os/thread.cpp
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: partial_p0_01_p0_02_p0_03_foundation_p0_04_p1_01_profile_transaction_static_implementation_complete_pending_managed_validation
source_recheck_required: true
---

# 02 · PBR Viewer、Tool Runtime、Evidence 与 RenderDoc 工程化差距

## 1. 结论

`zircon_shader_pbr_viewer` 不是一个随手拼出的空壳。它已经具备可保留的窄域基础：HDRI/IBL 冷暖缓存分离、staged project publish、Base PSO 有界重检、非阻塞 GPU timestamp 回收、ready sidecar、RenderDoc capture 与 replay 校验、binary/HDRI/artifact SHA、WPR/energy 可用性记录以及唯一 run directory。这些设计使它比普通截图 demo 更接近一个实验室工具。

但它目前仍不能承担“Zircon 产品渲染已达到或超过 Unreal”的验收角色。自动截图与 managed RenderDoc run 都传入 `--screenshot`；viewer 在该模式下明确不绑定 native viewport surface，而是执行离屏 render、CPU readback 和 PNG 写出。它又直接创建一个仅注册 foundation/tasks/asset module 的 `CoreRuntime` 和 `environment_only_pbr_preview` `SceneRenderer`，加载自动生成的单镜面球工程，绕过 App01 审查过的产品 `ProductComposition`、动态 runtime DLL、插件目录、产品 scene、窗口交换链以及完整 deferred profile。当前证据能证明一个窄的 HDRI/IBL/PBR 离屏路径，不等价于 packaged client、native presentation 或完整 renderer。

证据正确性也没有达到图形回归系统的门槛。ready validator 只要求 PNG 至少有两个不同颜色和一个非黑像素；没有 golden/reference image、HDR 线性域比较、SSIM/感知误差、平台/厂商容差、语义采样点或 camera settle。一个“有颜色的错误画面”仍可通过。与此同时，ApplicationHandler 内的窗口、scene、PSO、render、截图、GPU timing 和 capture 错误只打印并 `event_loop.exit()`，`run_app()` 正常返回后进程退出码仍为 0，直接调用方会把失败解释为成功。

本轮登记 4 项 P0、26 项 P1、8 项 P2。重构方向不是继续给 viewer 增加布尔开关，而是建立 `RenderEvidenceRunner`、`EvidenceCapability`、`EvidenceRunManifest` 和版本化 scene corpus：每次运行先声明实际宿主、presentation、scene、renderer profile、backend/device 和 oracle 能力；所有 artifact 作为一个事务提交；失败必须产生非零退出码和结构化 terminal record；visual、GPU timing、RenderDoc、native present 与 packaged product 分别形成不可混用的 gate。没有同场景、同分辨率、同硬件、同采样口径的 Unreal/HDRP 基线，不得宣称性能或表现超过参考引擎。

## 2. 审查边界与证据

### 2.1 当前源码范围

| 集合 | 文件 / 物理行 | 本轮证据 |
|---|---:|---|
| viewer production | 14 / 5,886 | E3：CLI、Winit app、后台 scene load、camera、project staging、render/present、PNG/sidecar、GPU timing、RenderDoc、work path |
| focused test file | 1 / 689 | E2：`app_tests.rs` 有 29 个测试，但不创建真实 EventLoop/window/SceneRenderer/RenderDoc session |
| viewer tests total | 126 test attributes | E2：0 ignored；绝大多数为纯函数、临时文件或 source contract，未运行真实 GPU/窗口 |
| immediate evidence tools | 7 / 4,487 | E3：profile contract、runner、三个 validator、summarizer、artifact receipt |
| tracked shader corpus | 182 / 624,277,393 bytes | E2：107 PNG、17 RDC 与其他输入/报告；不是 current-source 自动回归矩阵 |

viewer production fingerprint 为 `37cfb1a281c6cd8fd95188591b164f91cb14d11b5f5aa981a3c35d60df24c325`。算法与 App01 相同：路径排序、逐文件 SHA-256，再对 `path<TAB>hash<LF>` 清单取 SHA-256。成文前 viewer 与上述七个 immediate evidence tools 均未出现在工作区修改列表；实施前仍需重取，因为相邻 runtime/graphics 和 docs 存在其他 Session 修改。

本轮将 viewer 和直接消费它的证据脚本放在同一个 owner chain 中审查：

1. CLI/config -> work paths -> RenderDoc preload -> EventLoop/ApplicationHandler。
2. generated project -> `CoreRuntime` -> asset/import/cache -> `SceneRenderer` -> mirror sphere scene。
3. redraw/capture request -> Base PSO admission -> offscreen/native render -> CPU presenter/screenshot。
4. ready PNG/sidecar -> GPU timing -> RenderDoc RDC/replay -> run report -> profile summary/analysis -> artifact receipt。

`docs/tests/runtime/shader` 的历史文件只用于检查证据治理和已声称能力，不把旧 PNG/RDC 直接当成 current-source 验收。仓库中的完整 tooling、CI、packaging 与 session coordinator 仍将在 `zircon_tooling` 单独全扫；本轮只审查和 PBR viewer 直接相连的生产/消费合同。

### 2.2 动态证据边界

本轮没有启动 viewer、Cargo、WGPU、WPR、RenderDoc、Editor 或 packaged runtime，也没有重新生成任何图片/捕获。因此：

- 静态证据可以证明退出码传递、surface 选择分支、artifact 写入顺序、sidecar/validator schema、线程 owner、Cargo feature 和 reference scene 内容。
- 历史 artifact 可以证明曾产生特定图像、RDC 或报告，不能证明它们对应当前 fingerprint，也不能证明视觉正确。
- 未执行 device/surface loss、kill/crash、磁盘写满、双进程争用、不同 GPU/backend、HDR、DPI、多窗口或同负载基准。

### 2.3 参考源码给出的工程边界

- Unreal `RenderCaptureInterface` 将 capture 的 game/render/RDG 边界显式化；`ImageWriteQueue` 通过 fence、queue 与 shutdown flush 管理异步图像写出；Functional Testing 的 screenshot path 提供 delay、camera/environment setup/restore、local/global error tolerance、comparison completion 与 timeout。关键不是复制 API，而是 capture、write、compare、completion 都有 owner。
- Unity Graphics 的 HDRP LookDev compositor 以命名 render target 和显式 Begin/End RenderDoc 包围实际 render；URP graphics tests 固定分辨率/scene/camera，等待 frame/end-of-frame，再按 reference image 和平台/厂商阈值比较，并可检查分配。
- Bevy screenshot 以 render-target request entity、`Capturing`/`Captured` 状态、异步 readback 和 observer completion 接入 render schedule，说明 screenshot 应是 renderer-owned request，而非 app 内同步文件副作用。Bevy 本身不是本轮的 golden-image 权威。
- Godot worker/thread API 和 Fyrox task/executor 显示后台工作应进入引擎生命周期；Godot thread 明确要求 join，Fyrox executor 在 resumed/suspended 管理 graphics context。它们用于约束 owner 与 lifecycle，不据此推断其视觉回归能力。
- 仓内 Unity Graphics 不含闭源 Unity Player 主循环；本轮只引用可见的 graphics test/LookDev 源码，不推断不可见实现。

## 3. 当前证据到底能证明什么

| 能力声明 | 当前路径 | 结论 |
|---|---|---|
| HDRI 可解码并形成 IBL artifact | generated project + asset importer/cache + scene restore | **可部分证明**；recipe、尺寸和 staging timing 有记录 |
| environment-only PBR pass 可产出非空图像 | offscreen render -> CPU readback -> PNG | **可证明非空执行**，不能证明 BRDF/IBL 视觉正确 |
| 指定 frame generation 有 GPU timestamp | GPU timing request/report + validator | **可证明窄域 frame 的 measured pass**；managed validator 拒绝 unavailable/timed-out |
| RenderDoc 可抓取并回放 capture | preload bridge + `.rdc` + replay validator | **可证明离屏 draw capture 可回放**；不证明 native present |
| native viewport/swapchain presentation | interactive `render_to_viewport_surface` | **未被自动 gate 覆盖**；截图/capture 模式主动绕过 |
| packaged product runtime/DLL/plugin composition | 无 | **不能证明**；viewer 直接链接 runtime 并自组最小模块 |
| 完整 deferred renderer/material/scene matrix | 单镜面球、environment-only profile | **不能证明** |
| 视觉等价/优于 Unreal 或 HDRP | 无同场景 reference、无误差 oracle | **不能证明** |
| 性能优于 Unreal 或 HDRP | 有 Zircon cold/warm/profile 数据，无同负载 reference | **不能证明** |

任何后续计划或 milestone 若引用本 viewer 证据，都必须同时写出 capability label。`offscreen_readback`、`native_present`、`packaged_product`、`visual_regression`、`gpu_timing` 和 `renderdoc_replay` 不能互相代替。

## 4. 可保留的真实基础

### 4.1 冷暖缓存与 Base PSO admission 有明确状态

profile 为 cold run 创建独立 cache，为 warm mode 先 seed 再复用 shared IBL cache，并校验 `Written`/`Reused`。它还明确声明没有清 DX12/driver cache，避免把进程缓存控制夸大为 GPU driver 冷启动。one-shot 截图/capture 在 environment-only Base pipeline 未 ready 时以非阻塞重检等待，且有 45 秒 deadline。这两点应保留并提升为通用 evidence runner 状态，而不是退回 sleep。

### 4.2 GPU timing consumer 比 ready 图像 validator 更严格

GPU timing report 绑定 requested/completed generation，可记录 direct pass；managed validator 要求 measured status 和所需 pass，拒绝 unavailable/timed-out，并能绑定 ready PNG SHA。该链条已具备“请求 -> 异步完成 -> 特定 generation -> 外部校验”的雏形，应扩展到 visual/capture/present，而不是改成 CPU wall-clock 猜测。

### 4.3 RenderDoc replay 与 profile provenance 有可用骨架

managed runner 在唯一 run directory 中要求恰好一个 `.rdc`，replay validator 会在回放前后核对 SHA、限制时间/日志并保留 snapshot。profile 又记录 viewer binary、HDRI、关键 source manifest、命令/validation ticket 和 artifact receipt。当前问题是覆盖和事务不完整，不是这些字段没有价值。

### 4.4 Generated project 的 staged publish 避免直接暴露半套文件

首次生成使用 staging directory，准备完毕后 rename 发布，并处理并发发布者。这比逐个向最终目录写文件更可靠。目标架构应在此基础上增加 content manifest、fsync、完成标记、版本迁移和残留清理，而不是删掉 staged publish。

### 4.5 Scene 的 Drop 顺序有显式意图

`PbrMirrorScene::Drop` 显式按 world、surface、renderer、CoreRuntime 的依赖方向释放。这种 owner 意图值得保留。缺口在于后台任务可能尚未把 scene 交给 app、event loop lifecycle 不完整，以及整个进程没有可观测 shutdown phase。

## 5. P0：会制造假成功或错误能力结论

### PBR-P0-01 · 运行期致命失败最终返回成功退出码

`main()` 只传播参数解析、RenderDoc preload、EventLoop 创建和 `run_app` 自身的错误。进入 `ApplicationHandler` 后，窗口/CPU presenter/scene load/Base PSO/resize/render/screenshot/GPU timing/capture completion 失败均采用 `eprintln!` + `event_loop.exit()`；没有 terminal error 从 app 返回给 `main`。因此 `run_app()` 可以正常结束，进程返回 0。

这会让 direct CLI、CI、IDE task 和任何只看 process status 的 orchestrator 把失败当成功。managed profile 通常会因 artifact 缺失而二次失败，但这只是脚本补救，不能修复 binary contract。

必须引入 app-owned `TerminalOutcome`，在 event loop 退出后由 `main` 映射到稳定非零 exit code；同时原子写出结构化 terminal record，包含 phase、error category、source chain、artifact commit state 和 cleanup outcome。测试至少覆盖 scene failure、PSO timeout、render failure、write failure、capture failure 与 user cancel 六类退出码。

#### 2026-08-25 静态实施记录：PBR-P0-01

状态：`implementation_complete_pending_managed_validation`。`main` 现在保留应用对象直到 `run_app` 返回，并把 app-owned `TerminalOutcome` 转成进程退出码；RenderDoc preload 和 EventLoop 创建的早期失败也在已验证的非 C: work root 下原子写出同一 schema。`TerminalOutcome` 有 3 个终态（`succeeded`、`failed`、`cancelled`）、10 个失败类别，失败码稳定映射为 11--20，用户取消为 130，成功为 0。它记录 phase、错误类别/消息、viewer/HDRI source chain、screenshot/GPU timing/RenderDoc 三项 artifact commit state，以及主资源释放、loader 完成/取消后 join、超时和 join panic 五种 cleanup state。

窗口创建、presenter、scene load、Base PSO admission/readiness/timeout、render、screenshot、GPU timing、RenderDoc capture 与 status/frame present 的致命路径都先记录失败再请求 event-loop exit；正常 one-shot 完成与用户关闭分别写入 success/cancel。P0-02 后 native surface bind 或 render 失败直接成为 `FramePresent/Presentation` terminal failure，不能再降级到 CPU presenter；要求 RenderDoc 产物但未获得可验证 `.rdc` 时则在 terminal record 中标为 `not_committed`。

`terminal_outcome.rs` 的单元测试覆盖所有失败类别的非零码、cancel 与 success/failure 的区分、主失败不被 cleanup failure 覆盖，以及 JSON 原子替换后保留 source/artifact/cleanup 字段。`app_tests.rs` 锁定所有 `event_loop.exit()` 只能落在三条 outcome owner 路径或已先写入 scene-load failure 的单独路径；`main.rs` 锁定两条 app 创建前的失败路径包含 source chain。`rustfmt --check` 与 scoped `git diff --check` 已通过；Rust tests、真实 process exit、WGPU/窗口/RenderDoc、截图及性能/功耗采样均未执行，等待 coordinator-managed Windows receipt。

PBR-P0-02、PBR-P0-04 的静态实现见本节后续记录。PBR-P0-03 的 display-oracle 基础设施已开始实施，但版本化 corpus、current baseline 和线性 HDR oracle 尚未具备；Phase 0 artifact transaction 亦仍待实施，因此整个计划仅为部分 P0 静态实现完成。

### PBR-P0-02 · 离屏诊断证据被放在可误认成产品/native 能力的位置

`finish_scene_load()` 仅在 `screenshot_path.is_none()` 时绑定 native viewport surface。managed measured run 与 RenderDoc run 都设置 screenshot，因此两者固定走 scene offscreen render、CPU readback；capture 也包围这条离屏路径。viewer 同时绕过 `zircon_app` 产品 composition、dynamic runtime session、plugin/catalog 和产品 scene，只注册最小 Core module 并使用 environment-only preview。

sidecar 写出 `screenshot_presentation=cpu_readback` 是诚实基础，但没有 machine-enforced capability taxonomy，也没有阻止上层文档把它升级为 native/product/complete renderer evidence。

必须把 runner 分成至少 `OffscreenDiagnostic`、`NativePresent`、`PackagedProduct` 三个不可隐式降级的 host mode。每个 artifact manifest 写入 mode、composition fingerprint、renderer profile、surface/present count、scene ID 和 capture target；验收器拒绝 capability 不匹配。native mode surface 失败必须失败，不能静默 CPU fallback 后继续获得 native 标签。

#### 2026-08-25 PBR-P0-02 实施设计：viewer capability contract，不伪装产品宿主

当前 `zircon_shader_pbr_viewer` 是独立的诊断 binary，不含 `zircon_app` 产品 composition、catalog、plugin/session 或 product scene。因此它只实现 `offscreen-diagnostic` 和 `native-present` 两种强类型 mode；`packaged-product` 仍由产品入口承担，viewer 必须拒绝而不是给同一 mirror-sphere 路径贴上产品标签。mode 在参数解析时收敛：离屏模式必须请求 screenshot，native 模式禁止 screenshot/GPU-timing readback。保留省略 mode 的旧 CLI 兼容性时，解析器只可根据 screenshot 是否存在推导出二者之一，随后在 `ViewerConfig` 固化，渲染过程不得再改写其能力。

ready sidecar 将升级 schema 并声明 host mode、静态 viewer composition ID、固定 mirror-sphere scene ID、capture target 与 GPU scene-surface present count。受管 profiling 脚本显式传递 `offscreen-diagnostic`，Python validator 的 current-schema gate 要求该五元组精确匹配 CPU readback evidence；旧 schema 仅作为历史读取兼容。native mode 的 viewport-surface failure 直接走 terminal failure，不能把 native run 降级到 CPU presenter 后继续。该切片只改变运行能力标签和失败路径，不增加 shader、IBL bake 或 frame hot-loop 算法；没有新的性能、功耗或 GPU 时间声明。

#### 2026-08-25 静态实施记录：PBR-P0-02

状态：`implementation_complete_pending_managed_validation`。`ViewerConfig` 已收敛 `ViewerHostMode::{OffscreenDiagnostic, NativePresent}`。显式的 `packaged-product` 被该 standalone diagnostic binary 拒绝；省略 mode 时，旧 CLI 仅由 screenshot 是否存在解析成一个固定 mode，之后 app 不再根据 surface 失败或 presenter 状态改写它。离屏 mode 要求 screenshot，native mode 拒绝 screenshot/GPU timing readback；native viewport surface 失败直接成为 `FramePresent/Presentation` terminal failure。

ready-frame schema 从 v13 升至 v14，新增 `host_mode`、`host_composition_id`、`scene_id`、`capture_target` 和 `gpu_scene_surface_present_count`。当前 screenshot evidence 必须是 `offscreen-diagnostic`、`zircon_shader_pbr_viewer_standalone_diagnostic_v1`、`single_pbr_mirror_sphere`、`offscreen-scene-renderer-cpu-readback` 与 count 0；current Python validator 拒绝其他组合。profile script 显式传递 mode 和 validator expectation，v2--v13 仍只允许 legacy inspection，不是 current acceptance。

Rust format、scoped diff、Python AST 和 source contracts 已静态通过；Cargo/Python unit execution、WGPU/窗口、当前 PNG、RenderDoc replay、CPU/GPU timing、energy 和 power 未执行，等待 coordinator-managed Windows receipt。`PackagedProduct` runner、完整 composition fingerprint、artifact transaction 与 P0-03 的完整 scene corpus/linear-HDR oracle 仍未实现，故不关闭 P0/Shader 06 milestone。

### PBR-P0-03 · 图像 gate 只验证“有颜色”，错误画面仍可通过

ready validator 的像素正确性门槛是至少两个 distinct colors 和至少一个 nonblack pixel。它没有 reference image、HDR linear buffer、color-space contract、SSIM/perceptual metric、局部/全局误差、semantic probes、mask、vendor/backend tolerance，也没有等待稳定帧或相机/环境恢复协议。

这无法发现 BRDF 符号错误、roughness/metallic 反转、cubemap face/handedness 错误、曝光偏移、tone map 退化、shadow/normal/IBL 漏接、左右翻转、局部全黑或大面积 clipping。当前历史“人工看起来正确”不能替自动 correctness oracle。

必须建立版本化 scene + camera + HDR source + linear reference + display reference 套件。比较层至少提供 exact semantic probes、HDR-relative error、perceptual image metric、局部/全局阈值、平台/vendor policy、diff/heatmap 和批准流程；reference 更新必须是显式 review artifact。single mirror sphere 只保留为 smoke case，不能作为 PBR acceptance case。

#### 2026-08-25 PBR-P0-03 实施设计：display oracle 先与 provenance 绑定

在 renderer 提供可比较的线性 HDR readback 前，先建立窄的 display-output oracle，不伪造 HDR 或跨设备感知等价。`tools/zircon_pbr_visual_oracle.py` 接受一个版本化 JSON manifest；manifest 绑定同目录 reference PNG 的 SHA-256、非空 provenance 字段、全局 RGB mean-absolute-error/p99/error-pixel-fraction 阈值，以及命名的语义矩形区域。PNG 解码、CRC、尺寸、RGBA8 格式和像素预算由该模块单一负责；ready validator 将已解码 candidate 传给 oracle，避免同一 PNG 的第二次读取/解压。误差比较以一次 RGB 扫描和 256 桶直方图求 p99，不为整帧分配误差数组。

`tools/zircon_validate_shader_pbr_viewer_evidence.py --display-visual-oracle <manifest>` 仅在明确提供 manifest 时启用 gate，并在 JSON summary 回写 oracle/reference SHA、比较像素数、全局误差和语义区域误差。manifest 的 expected metadata 必须与 ready sidecar 匹配，因此离屏 CPU readback、scene、backend 或 host capability 不能与另一条基线混用。`tools/zircon_profile_shader_pbr_viewer.ps1 -DisplayVisualOracle <manifest>` 复用同一受控路径检查，将该 manifest 转发至 cold/warm/RenderDoc 的 ready validator，并在每次 run report 与 profile summary 写入 SHA-256 指纹。reference PNG 与 manifest 必须由受管 Windows run 后一同批准并置于 `docs/tests/runtime/shader` 的受控 oracle 目录；现有带日期的 PNG 仅是历史证据，不能倒灌为 current baseline。

#### 2026-08-25 静态实施记录：PBR-P0-03 foundation

状态：`partial_implementation_complete_pending_managed_validation`。已新增 display oracle 的纯函数测试，覆盖 provenance 不匹配、全局误差拒绝、语义区域与 ready-frame JSON summary；主 validator 已复用抽出的 PNG decoder，避免两份解析器漂移。Python AST、scoped diff 和 source-contract 静态检查通过；Python unit execution、managed viewer output、reference approval、RenderDoc、HDR readback、GPU timing、性能及功耗测量均未执行。

这一 foundation 不关闭 PBR-P0-03：single mirror sphere 仍只可作 smoke case；缺少的版本化 feature corpus、固定 camera/environment settle、linear HDR-relative comparison、perceptual metric、vendor/backend policy、heatmap/diff artifact 与 review/transaction approval 继续作为后续的独立合同。profile runner 只能在受管 baseline manifest 已批准后显式传入 `--display-visual-oracle`，不得默认选取仓库中的历史 PNG。

#### 2026-08-25 静态修复记录：current evidence recipe/schema propagation

状态：`implementation_complete_pending_managed_validation`。运行时 canonical IBL recipe 已因 PMREM roughness-to-mip 合同升级至 `2026_08_24_0007`，但 current evidence validator 仍接受旧的 `202608090006`，profile summarizer 也仍将 v13 视为 current schema；这会使新 viewer 的 ready evidence 被错误拒绝，或让 summary 与 producer 发生 schema 漂移。本次将 validator 的 current recipe 设为 `202608240007`，将 summarizer current schema 收敛到 v14，并让 viewer 的 Rust sidecar fixture 从 runtime `IBL_BAKE_ALGORITHM_VERSION` 导出值，避免测试重复维护 recipe literal。

2026-08-26 follow-up：Shader06 的低粗糙度 PMREM D_GGX/PDF 修复把 canonical recipe 推进到 `2026_08_26_0008`；viewer current validator 同步要求 `202608260008`。上一段的 v7/`202608240007` 是 roughness-LOD 修复时点的历史身份，不再是当前源码验收身份。该同步只防止旧 PMREM/sidecar 冒充 current-source evidence，不代表新的 WGPU、截图、RenderDoc、性能或功耗验收已经完成。

v2--v13 仍只允许显式 `--allow-legacy-schema` inspection；测试分别覆盖不完整 v13 的字段拒绝和完整 v13 的兼容读取，默认 current path 仅接受完整 v14 capability/BRDF-LUT/provenance。Python AST、Rust formatter、runtime-to-validator source contract 和 scoped diff 将在本次编辑后重新静态检查。未运行 Python unit、Cargo、WGPU、RenderDoc 或生成截图，因此没有产生动态正确性、性能、功耗或验收结论。

### PBR-P0-04 · 后台 scene 构建线程被 detach，退出时不可取消、不可 join

`BackgroundTask<T>` 只保存 mpsc receiver；`thread::spawn` 返回的 `JoinHandle` 被立即丢弃。任务没有 cancellation token、deadline、progress、join 或 Drop 协议。用户在 HDRI decode、project staging、asset import/cache、renderer/device 初始化期间关闭窗口时，app 可退出而后台线程仍在执行；进程终止会截断尚未提交的磁盘/GPU 工作，也无法报告 teardown 是否完成。

必须由 runner/task service 持有 join handle 和 cancellation token，scene build 各阶段检查取消，退出按 stop-admission -> cancel -> bounded join -> artifact rollback/quarantine -> renderer/core drop 执行。若 deadline 后仍不能停止，terminal record 必须为 teardown failure；不得以退出码 0 或只有 stderr 结束。

#### 2026-08-25 PBR-P0-04 实施设计：专属 loader owner，不滥用 I/O lane

当前 `BackgroundTask` 是唯一 consumer，且其任务会创建 `CoreRuntime`/WGPU renderer 再把 typed `PbrMirrorScene` 移交给 event thread。`BoundedKeyedIoLane` 的合同是有 key 的 `FnOnce() -> Result<(), Failure>` I/O work、fence 和 retained-byte admission，不能承载这个 typed renderer payload；`JobScheduler` 现有 handle 也没有取消已运行 scene construction 的合同。因此不把 viewer 偷塞进这两个不匹配的 owner。仍保留专属具名 loader，改用 runtime 的 named-thread creation wrapper，并由 `BackgroundTask` 持有 `JoinHandle`、`Arc<AtomicBool>` cancellation token 和 completion receiver。

取消语义采用 Zircon `DynamicSceneSpawnTask` 的已验证模式：开始前、每一个 scene bootstrap phase 后以及 publication 前检查 token；取消后丢弃未发布 scene，不让它进入 app。Unreal `FAsyncTask` 的 `Cancel`/`TryAbandonTask` 同样只安全撤销未开始工作，运行中工作必须 cooperative；Godot 的 worker pool 先停止 admission、让 worker 到安全点再 `wait_to_finish`。因此本设计明确不承诺强杀正在进行的 HDR decode/IBL bake。event-loop callback 只 request cancel，`main` 在 loop 返回后执行一次有界 drain；正常完成必须 join，deadline 到达则写 `task_shutdown` terminal failure 和 `background_loader_shutdown_timed_out` cleanup state，随后由非零进程退出阻止 artifact 接受。

#### 2026-08-25 静态实施记录：PBR-P0-04

状态：`implementation_complete_pending_managed_validation`。`BackgroundTask` 现由 runtime 的 named-thread wrapper 创建并持有 `JoinHandle`，任务和 `PbrMirrorScene::new_with_cancellation` 共享 `Arc<AtomicBool>` token。worker 在执行前和 publication 前检查 token；scene 在 HDRI preflight 前后、project assets、runtime bootstrap、project open、world load、renderer bootstrap 与 IBL restore 后检查 token。因此取消不会把已取消的 typed scene 发布到 event-loop owner。

所有 terminal owner 先 request cancellation；`run_app` 返回后 `finish_after_event_loop` 对 loader 执行一次 2 秒 bounded join，再按结果记录 `completed_and_joined`、`cancelled_and_joined`、`shutdown_timed_out` 或 `join_panicked`。后两者把原本 success/cancel 终态提升为 `task_shutdown` 的非零失败；若已有 scene/render 等主失败，主 phase/category/code 保持不变，额外 teardown 信息进入 `cleanup_error`。`try_take` 也在把结果交给 event loop 前 join worker，避免成功或 scene-load error 路径遗留 handle。

首轮取消检查点为 HDRI preflight 前后、project assets、runtime bootstrap、project open、world load、renderer bootstrap 和 IBL restore 边界。它先解决 handle ownership、取消传播、publication 抑制和终态真实性；内部 decoder/PMREM 的细粒度 cancellation、artifact rollback/quarantine 和 shared runner service 仍需在 Phase 0 artifact transaction/asset worker 重构中完成。超时不是 worker 已停止的证明，故只产生拒收用的失败 record，不能作为 artifact acceptance。实施前的静态调查确认该路径不在 frame hot loop；steady-state render CPU/GPU work 不增加，唯一新控制操作是 scene-load stage boundary 的 atomic acquire load。没有运行测量前不对该成本、功耗或 shutdown latency 作数值声明。

## 6. P1：缺失的工程合同

### 6.1 Artifact、provenance 与缓存完整性

| ID | 当前差距 | 需要重构的合同 |
|---|---|---|
| PBR-P1-01 | ready PNG、sidecar、GPU timing、validation JSON、run report、summary/analysis 分别直接写入；只有 HDR exposure cache 使用原子替换。crash/kill 可留下看似存在的半套证据。 | `EvidenceArtifactTransaction`：临时目录/临时文件、flush/fsync、逐项 SHA、manifest 最后提交、目录原子 publish；consumer 只接受 committed manifest。 |
| PBR-P1-02 | 唯一 GUID run directory 解决命名冲突，但没有 `running/committed/failed/quarantined` 终态、resume 或 stale-run scavenger。 | run lease、heartbeat、terminal state、失败原因、恢复/隔离和保留策略；禁止仅凭目录存在判定完成。 |
| PBR-P1-03 | standalone ready sidecar 有文件名、尺寸和渲染字段，却不绑定 PNG SHA、HDRI SHA、viewer binary SHA、source manifest 或 run ID；同名同尺寸图片可被替换后通过弱 validator。 | sidecar 成为签名/哈希绑定 manifest，至少绑定所有输入、binary、composition、PNG、schema 和 validation policy；managed 与 standalone 使用同一合同。 |
| PBR-P1-04 | profile 的 human-auditable critical source list 只覆盖 viewer 14 个 production 文件中的 6 个；遗漏 background load、Base recheck、camera、main、presenter、project assets、RenderDoc 和 work paths。 | 从 Cargo/owner graph 生成完整 source closure，或直接绑定 reviewed production fingerprint；source list 不能手工维护为“关键文件猜测”。 |
| PBR-P1-05 | `viewer_project_assets_are_ready()` 只检查六个预期路径是否为文件；测试甚至用任意 fixture 文本证明“ready”。corrupt、tampered 或旧 schema 同名文件会被复用。 | project content manifest 记录 schema、每文件 SHA/size/semantic type、generator version 与 input recipe；打开前逐项校验，损坏则 quarantine + rebuild。 |
| PBR-P1-06 | generator 仅在 manifest 不存在时写入；已存在 manifest 不与当前预期路径/scene/renderer recipe 对比，runtime 后续又从磁盘打开它。 | manifest 是单一真值：加载、validate、migrate 或重建；禁止 in-memory assumptions 与 disk manifest 分叉。 |
| PBR-P1-07 | crash 后遗留 `.stage_`/`.incomplete_` 目录没有启动清理、lease、age 或 owner policy。 | staging journal + owner PID/run ID + age；启动时安全识别并 quarantine/回收，不得无条件删除活跃发布者。 |
| PBR-P1-08 | profile timeout 直接强杀 viewer；没有先发 cancel、等待 terminal manifest、drain capture/write queue 或区分 hang 与 slow stage。 | 两阶段 timeout：cooperative cancel + bounded graceful shutdown，随后才 hard kill；两种结果分别记录，hard-killed run 永不提交。 |
| PBR-P1-09 | schema 多处固定为 v1/文本键值，但没有 capability negotiation、reader min/max version、迁移或 unknown-field policy。 | typed/versioned manifest library，由 producer、validator、summarizer 和 receipt 共用；显式兼容范围与迁移测试。 |

### 6.2 Tool runtime、平台与 presentation lifecycle

| ID | 当前差距 | 需要重构的合同 |
|---|---|---|
| PBR-P1-10 | `PbrMirrorViewerApp` 用大量 `Option`/boolean 表达 loading、ready、screenshot、timing、capture、exit；非法组合靠分支顺序避免。 | 显式 `ViewerState`/`RunPhase` 状态机，typed transition 与 terminal outcome；每个 phase 有 owner、deadline、cancel 和 cleanup。 |
| PBR-P1-11 | 单窗口 handler 忽略 `WindowId`，没有 `suspended()`/`exiting()`，`CloseRequested`/`Destroyed` 只退出；surface/device 生命周期不与平台 suspend/resume 对齐。 | window/viewport identity、resumed/suspended/exiting、surface generation 与重建；至少真实覆盖 destroy-before-load、suspend-during-capture 和 resume-after-loss。 |
| PBR-P1-12 | renderer/device 在后台 loader thread 创建后移动到 event thread；当前 desktop 路径可编译，但没有 backend/platform thread-affinity capability 或失败策略。 | device/queue ownership 明确属于 render service；后台只做 CPU/import 工作，或由平台 capability 证明可跨线程；Web/mobile/Metal 单独 gate。 |
| PBR-P1-13 | binary `required-features = ["target-client"]`，因此工具带入 dynamic API、animation、graphics、nav、script、UI、desktop platform/input 等完整 client bundle。 | 独立 `render-evidence` feature/product role，仅引入真实依赖；packaged-product mode 再显式选择完整 client。 |
| PBR-P1-14 | artifact/work path 在 binary 中硬拒绝 C:，fallback 硬编码 `D:/ZirconEngineArtifacts`，RenderDoc 示例也绑定 D:；这是某实验机策略，不是引擎能力。 | storage policy 下沉到 harness/config；工具只校验可写空间、路径隔离和容量，支持任意合法 volume/container/CI workspace。 |
| PBR-P1-15 | P0-02 已令 standalone viewer 的 native surface bind/render failure fail closed；但 broader diagnostic/product mode 仍没有统一的结构化 downgrade/recovery policy，也没有 native/product integration coverage。 | mode 决定 downgrade policy；diagnostic 可降级但 manifest 标红，native acceptance 必须 fail closed。 |
| PBR-P1-16 | surface/device loss、outdated/suboptimal、resize/minimize、DPI/HDR/color profile 不形成恢复状态机；render/presenter error 多数直接退出。 | 与 RHI owner 对齐的 recoverable/fatal error taxonomy、surface generation、device recreation、color/output metadata 和注入测试。 |

### 6.3 Scene 与 renderer 代表性

| ID | 当前差距 | 需要重构的合同 |
|---|---|---|
| PBR-P1-17 | 默认工程只有一个自动生成的完美镜面球、零 direct light、environment-only profile；无法覆盖 roughness/metallic、纹理、normal、shadow、skinning、instancing、alpha、transmission、clearcoat、SSS、probe blend 或 post process。 | 版本化 scene corpus，按 material/light/geometry/post/streaming feature 分层；每个 scene 声明 coverage 与 oracle，禁止一个 scene 给全 renderer 背书。 |
| PBR-P1-18 | viewer 自组最小 CoreRuntime、generated manifest 与 scene，未加载真实 cooked/product project，也不通过 App01 的 `ProductHost`/dynamic runtime/plugin composition。 | 同一 runner 提供 isolated renderer 与 packaged product adapter；后者必须加载真实 product artifact，并把 composition fingerprint 写入 manifest。 |
| PBR-P1-19 | screenshot path 直接调用 offscreen render；interactive native path没有自动 framebuffer/present capture，无法验证两条路径色彩、资源、shader permutation 和同步等价。 | renderer-owned capture request 可挂在 offscreen target 或真实 viewport pre-present；建立同 frame dual-path comparison 和 present completion proof。 |

### 6.4 RenderDoc、GPU timing 与性能方法

| ID | 当前差距 | 需要重构的合同 |
|---|---|---|
| PBR-P1-20 | app 在 capture 后只取 latest capture，没有先记 baseline count、要求恰好 `+1`，也不在 app 内绑定返回路径与请求 template。managed unique directory/单 RDC 检查能缓解但不是 bridge 自身合同。 | capture session 记录 baseline、capture ID、expected template/target、begin/end frame generation，并要求 count 精确增长和路径归属。 |
| PBR-P1-21 | app 层只检查 RDC 是非空普通文件；强 replay/SHA snapshot 校验位于外部 profile。direct viewer capture 可被误当成已验证证据。 | capture artifact 状态分 `captured`、`replayed`、`validated`；只有 replay validator 产生的 committed manifest 能获得 validated label。 |
| PBR-P1-22 | managed RenderDoc run 因同时请求 screenshot 固定抓离屏路径，没有 native swapchain capture 模式，也没有捕获 product host frame。 | 分离 offscreen/native/product capture recipes；manifest 记录 capture target、surface/viewport、present count 与 frame graph signature。 |
| PBR-P1-23 | GPU timing 能绑定 frame generation，但 one-shot 通常测首个 ready screenshot frame；没有 warm-up/settle、连续帧分布、outlier policy 或视觉输出稳定性联锁。 | correctness 先稳定，再采 N 帧；报告 median/p95/min/max、frame identity、frequency/calibration、pass coverage，并把异常/缺失样本 fail closed。 |
| PBR-P1-24 | cold/warm 只控制进程与 IBL cache，明确不清 driver/DX12 cache；这是诚实的，但当前名称仍容易被解释成完整 shader/driver cold start。 | cache-domain taxonomy：project/asset/IBL/PSO/driver/OS；每项写 controlled/uncontrolled，结论只落在受控 domain。 |
| PBR-P1-25 | 没有与 Unreal/HDRP 同 scene、分辨率、quality、shader warm state、GPU/driver/power policy 的对照 runner；Zircon 自身数字不能支持“更快”。 | paired benchmark protocol、机器清单、thermal/power lock、随机化顺序、重复/置信区间、原始 trace 与同画质 visual gate。 |

### 6.5 测试与长期证据治理

| ID | 当前差距 | 需要重构的合同 |
|---|---|---|
| PBR-P1-26 | 126 个 test attribute、0 ignored，但没有真实 EventLoop/window/SceneRenderer/WGPU/RenderDoc 集成测试；同时 `docs/tests/runtime/shader` 跟踪 624,277,393 bytes，其中 17 个 RDC 占 461,326,034 bytes，`.gitattributes` 未配置 LFS。 | 分层 CI：纯函数、headless GPU、native present、RenderDoc lab、packaged product；大 capture 迁移到内容寻址 artifact store，仅在 Git 保留小型 golden、哈希 manifest、批准记录与 retention/currentness index。 |

#### 2026-08-25 PBR-P1-01 实施设计：profile completion receipt 作为唯一提交点

`shader-pbr-profile-publication.ps1` 已经把一次 profile 写入
`profile-captures/.staging/<profile-id>`，并在该 root 外以 create-new 写入
completion receipt。下一步不新建第二套 artifact store：将该 receipt 收敛为
profile transaction 的唯一提交 marker。

1. receipt 必须严格绑定 summary 所在的 staging root、stable profile id、`completed`
   状态和本次 root 下的完整 regular-file 闭包；每一项只允许安全相对路径、SHA-256
   和字节数，重复、漏项、额外项、符号链接或 root escape 都拒绝。
2. receipt 只在 profile summary、每个 run report、ready/timing/replay validation、可选
   WPR/energy/RenderDoc 和 analysis 都已写完并通过 producer-side checks 后创建；receipt
   之后不得再向 staging root 写 artifact。crash、kill 或任何前序错误只能留下没有
   completed receipt 的 staging root（或 immutable incomplete receipt），不得被
   consumer 当作完成 profile。
3. 复用的 Python summarizer 增加显式 completion-receipt gate。capture 自身在提交前运行
   summarizer 是 producer-side validation；后续任何需要把 profile 当作已完成证据的
   consumer 必须传入 receipt 并重放 root 闭包/hash。这样不制造 report/receipt 循环依赖，
   同时避免把尚未存在的 receipt 作为 capture 中间步骤的输入。
4. 此 transaction 覆盖 profile runner 控制的 PNG、sidecar、GPU timing、validation、run
   report、WPR/energy/RDC/replay、summary 与 analysis。viewer 进程内的 terminal outcome
   仍由 app 原子写入，属于该 runner 的输入 artifact；跨 host 的统一 `RenderEvidenceRunner`
   与 app 内 artifact transaction 仍是后续 Phase 0 工作，不能宣称本小步已完成全部 P1-01。

当前静态实现让 publication 扫描 artifact 时拒绝 reparse point，并新增
`Assert-ZirconShaderPbrProfileCompletion` 对 create-new receipt 的 schema、allocated root、稳定
顺序、完整文件闭包、SHA-256 和字节数逐项重放。capture 写出 receipt 后立即调用该 verifier；
此后若错误发生，不再写入会污染已提交闭包的 incomplete receipt。Python summarizer 新增显式
`--completion-receipt` gate，供后续 consumer 在读取 profile 前重放同一闭包。PowerShell parser、
Python AST/import、source contract 和 scoped diff 已静态通过；未运行 Pester、Python unit、viewer、
Cargo、WGPU、RenderDoc 或生成任何当前截图。该记录只将 P1-01 的 profile transaction foundation
标为 static implementation complete，完整 `RenderEvidenceRunner`、app-owned transaction、stale-run
scavenger 和受管动态验证仍未完成。

P1-26 的重点不是“删除所有历史证据”。17 个 RDC 和 107 个 PNG 对回溯有价值，但把数百 MiB 二进制直接永久放在源码历史里会放大 clone、diff、CI cache 和审查成本，又没有 current-source 状态机。迁移必须先生成不可变 SHA manifest、保留 owner/日期/tool version/源 fingerprint 和可恢复位置，再按保留策略处理 Git 历史；不得直接破坏用户证据。

## 7. P2：可维护性与操作体验

| ID | 当前差距 | 建议 |
|---|---|---|
| PBR-P2-01 | CLI 为手写 parser，positional HDRI 可被后值覆盖，缺少统一 `--version`、machine-readable help/error 和稳定 exit-code 文档。 | 使用 workspace 统一 CLI contract，保留兼容迁移期；输出 schema 与 exit code 版本化。 |
| PBR-P2-02 | stdout/stderr 混合 loading、profiling、warning 与 terminal 文本，上层只能抓日志或再读文件。 | stdout 只输出结构化 event/result 或明确 human mode；diagnostics 走 logger，terminal record 为权威。 |
| PBR-P2-03 | window title、`screenshot_written`、capture/timing pending 共同推断 ready，而不是单一 phase。 | title 仅投影 `RunPhase`；自动化不解析 title。 |
| PBR-P2-04 | `SoftbufferPresenter` 把 RGBA 转 XRGB、忽略 alpha并清空未覆盖区，但没有色彩 profile、transfer function 或 HDR 声明。 | 以 explicit output transform/pixel format 描述 CPU fallback；禁止拿它与 HDR/native 输出直接比较。 |
| PBR-P2-05 | project/cache version 字符串散落在 work path 与 generated assets 中，升级依赖人工同步。 | 单一 typed schema/version owner，提供 migration/rebuild reason。 |
| PBR-P2-06 | generated asset report 统计固定六个 viewer 文件，不覆盖后续 importer/cache/bundle 写入，指标名容易被解释成完整启动 I/O。 | 区分 generator writes、asset import writes、cache writes、artifact bytes 与 fsync/publish time。 |
| PBR-P2-07 | RenderDoc capture path 强制小写 `.rdc` 等 Windows 上不必要的文本策略，错误信息又带实验机默认路径。 | 按平台规范化/比较扩展名，错误输出显示 capability 和实际配置来源。 |
| PBR-P2-08 | scene load 完成 wake 后可在 event callback 中同步开始重渲染/证据写出，长工作会阻塞 event dispatch。 | 完成事件只提交 state/`request_redraw`；render、readback、write 在明确 schedule/queue 中执行并可取消。 |

## 8. 目标架构

### 8.1 `RenderEvidenceRunner` 是唯一运行 owner

```text
EvidenceRunSpec
  -> Capability admission
  -> Host adapter (isolated / native / packaged)
  -> Scene corpus load + content verification
  -> Warmup / readiness oracle
  -> Capture requests (image / timing / RenderDoc / present)
  -> Validation
  -> Artifact transaction commit
  -> Ordered shutdown
  -> TerminalOutcome + process exit code
```

runner 拥有 cancellation、deadline、task joins、renderer/host、artifact transaction 和 terminal outcome。Winit app 只是一个 host adapter，不再独自决定文件副作用和 process success。

### 8.2 能力必须是可校验数据，而不是文档形容词

建议最小 capability 集：

```rust
enum EvidenceHost { IsolatedRenderer, NativeViewport, PackagedProduct }
enum EvidenceOutput { LinearOffscreen, CpuReadbackPng, NativePresent }
enum EvidenceOracle { Smoke, SemanticProbe, GoldenImage, GpuTiming, RenderDocReplay }
```

`EvidenceRunManifest` 记录实际 host/output/oracle、scene content hash、composition/renderer profile、binary/source/input hash、backend/adapter/driver/OS、resolution/color space、warmup/sample、capture frame generation、artifact hashes、validation policy 和 shutdown outcome。validator 根据 requested capability 与 actual capability 做 admission，不能靠字符串约定。

### 8.3 Scene corpus 与 reference 更新有独立治理

scene 不再由 viewer 内嵌字符串临时生成后只看文件存在。每个 case 包含：

1. content-addressed project/cooked scene 与输入资源；
2. 固定 camera、time、seed、resolution、exposure、quality 与 renderer profile；
3. expected feature coverage 和禁止 fallback；
4. HDR/SDR reference、semantic probes、tolerance policy；
5. reference provenance、reviewer、生成 source fingerprint 和 superseded history。

reference 更新必须产生 before/after/diff/heatmap，不允许测试失败时自动覆盖 golden。

### 8.4 Artifact store 与源码仓解耦

Git 保留小型、长期稳定的 golden 和 manifests；RDC、WPR、超大 trace、重复中间 PNG 进入内容寻址 artifact store。manifest 以 SHA-256、size、media type、source/run/capability、retention 和可恢复 locator 绑定。任何迁移先验证远端可取回和 hash，再决定是否改写历史。

## 9. 分阶段重构计划

### Phase 0 · 先消灭假成功和不完整提交

- 引入 `TerminalOutcome`、非零退出码、结构化 terminal record。
- 用 explicit `RunPhase` 替代关键布尔组合。
- 后台 scene task 加 cancel/join/deadline，退出执行 bounded drain。
- ready/timing/capture/report 改为单一 artifact transaction，增加 committed marker。
- standalone 与 managed validator 统一 hash/schema/capability 合同。

完成门槛：对 scene/render/write/capture/hang 注入失败，进程退出码、terminal record、run state 和残留清理完全一致；kill 后不存在可被 validator 接受的半套 run。

### Phase 1 · 分离 isolated、native 与 product 三类证据

- `RenderEvidenceRunner` 抽离 Winit app；host adapter 显式选择 isolated/native/packaged。
- native capture 接入真实 viewport/pre-present，并记录 present completion。
- packaged adapter 复用 App01 的产品 composition/DLL/plugin owner，不再手工伪造等价启动。
- surface/device lifecycle 接入 resume/suspend/loss/recreate；禁止 acceptance mode 隐式 CPU fallback。
- source closure 和 build receipt 覆盖全部 owner 文件/依赖。

完成门槛：三个 mode 产生互不混淆的 capability manifest；native/product gate 能证明真实 surface/present/composition，offscreen artifact 不可能冒充它们。

### Phase 2 · 建立视觉正确性 scene/reference 矩阵

- smoke mirror sphere 降级为快速连通性测试。
- 增加 dielectric/metallic/roughness grid、textured/normal、multiple lights/shadows、skinned/instanced、alpha/transmission/coat/SSS、probe blend、post/HDR scene。
- 保存 linear HDR reference 和 display output；实现 semantic probes、global/local/perceptual errors、diff/heatmap。
- 固定 warmup、camera、time/seed、exposure、resolution 和 vendor/backend tolerance。
- CI 分 headless GPU、native lab、RenderDoc lab、packaged nightly。

完成门槛：故意引入 roughness、cubemap orientation、exposure、tone map、shadow 和 color-space 错误时，正确 case 必须稳定失败并给出可定位 diff。

### Phase 3 · 性能对标与长期证据治理

- 采稳定帧分布而不是单一 first-ready frame；同时记录 CPU trace、GPU pass、memory/residency、pipeline/cache domain。
- 设计与 Unreal/HDRP 同场景同画质 paired benchmark，固定硬件/driver/power/thermal 与随机化运行顺序。
- 建立 artifact CAS、retention/currentness index 和 reference approval workflow。
- 将数百 MiB 历史 capture 迁移前生成完整 SHA manifest 并验证可恢复性。

完成门槛：报告既能重放 Zircon run，也能重放 reference run；统计、图像正确性和环境控制缺一项时只标 diagnostic，不产生“更快/更好”结论。

## 10. 验收 Gate

| Gate | 必须证明 |
|---|---|
| G1 Exit truth | 每类致命失败返回稳定非零码；user cancel、timeout、hard kill 与 internal failure 可区分。 |
| G2 Terminal record | terminal outcome 原子写入并绑定 run/spec/binary/source；cleanup failure 不被主错误覆盖。 |
| G3 Task shutdown | scene build 可取消且 bounded join；退出后没有写入、GPU/asset/cache work 或 detached thread。 |
| G4 Artifact transaction | crash/kill/disk-full 任一点都不能留下可被接受的 committed run。 |
| G5 Cache integrity | 任一 generated project/cache 文件篡改、截断或 schema 过期都会 quarantine/rebuild，而非按存在复用。 |
| G6 Capability isolation | offscreen/native/product 三模式的 validator 互相拒绝错误标签。 |
| G7 Native present | 自动证明指定 viewport/surface generation 的 render、present 与 completion；fallback 不可冒充。 |
| G8 Product composition | packaged evidence 绑定真实 product/DLL/plugin/project composition fingerprint。 |
| G9 Visual oracle | PBR/IBL/color/shadow 故障注入稳定触发 semantic/HDR/perceptual failure，并产 diff。 |
| G10 GPU timing | 指定稳定 frame 的 required passes 全部 measured；unavailable、timeout、generation mismatch 均失败。 |
| G11 RenderDoc | count 精确增长、capture path/target/frame 绑定、SHA 稳定、replay 成功后才标 validated。 |
| G12 Platform lifecycle | resize/minimize/suspend/resume/surface loss/device loss/close-during-load 有确定状态与恢复/失败结果。 |
| G13 Provenance | binary、完整 source closure、inputs、scene、config、device/driver、artifacts 和 policy 全部内容绑定。 |
| G14 Corpus governance | 每个 scene/reference 有 owner/version/coverage/approval/currentness；不可自动覆盖 golden。 |
| G15 Benchmark parity | Zircon 与 Unreal/HDRP 同场景、同画质、同环境，报告分布和置信区间；正确性 gate 先于性能。 |
| G16 Artifact retention | 大 artifact 可按 SHA 取回并校验；Git clone 不再承担无界 capture 历史。 |

## 11. 与现有计划和证据的关系

- App01 已把 PBR viewer 明确排除出产品 host 统计。本报告确认这种拆分正确，并进一步规定：只有 Phase 1 packaged adapter 可以把 viewer/evidence 与 ProductHost 能力重新关联。
- Runtime09A/09C/09F1 已分别审查 RHI/render graph、material/shader/PSO、environment/IBL/probe。viewer 的成功不能关闭这些计划中的架构缺口；它只证明特定 consumer path 曾执行。
- `2026-08-13-startup-performance-architecture-review.md` 和后续 M5 文档对 offscreen/diagnostic 边界已有部分诚实说明，应保留为历史 provenance；新 manifest 需要把这种说明变成 machine-enforced capability。
- 历史 PNG/RDC 不因本报告自动失效或删除。它们进入“historical/unverified-current-source”状态，直到 source fingerprint、inputs、capability 与 validator 全部可绑定。
- 本报告不实施 renderer 算法修正，也不把工具脚本全域审查冒充 `zircon_tooling` 完成；后续 tooling 轮次仍需覆盖 workspace、CI、validator framework、package、artifact service 和 session coordinator 全部代码。

## 12. 非目标与禁止捷径

1. 不以增加更多 source `.contains()` 测试代替真实 process/GPU/window/capture gate。
2. 不以“图片非黑”“RDC 非空”“RenderDoc 可打开”代替视觉正确性。
3. 不把 CPU readback screenshot 命名为 native swapchain evidence。
4. 不把 isolated mirror sphere 的 timing 推广到完整产品 renderer。
5. 不通过延长 sleep/timeout 掩盖 readiness、task owner 或 frame identity 缺失。
6. 不在没有可恢复 SHA manifest 的情况下直接删除或改写历史 artifact。
7. 不在没有同画质 reference gate 的情况下宣称性能或表现优于 Unreal/HDRP。
8. 不为兼容当前脚本继续扩散未版本化文本键值；producer/consumer 应迁移到共享 typed schema。

## 13. 差距统计

| 级别 | 数量 | 主题 |
|---|---:|---|
| P0 | 4 | 失败退出码为 0、离屏证据能力膨胀、无视觉 oracle、后台 task 无取消/join |
| P1 | 26 | artifact/provenance/cache、tool lifecycle/platform、scene代表性、RenderDoc/timing/benchmark、测试与大证据治理 |
| P2 | 8 | CLI/日志、ready投影、CPU output、版本、I/O指标、路径文本策略与 event scheduling |

下一步进入 `zircon_runtime_interface` ABI/FFI/handle/version/foreign ownership 全域审查；本报告只登记重构，不开始修改 viewer 或证据脚本。
