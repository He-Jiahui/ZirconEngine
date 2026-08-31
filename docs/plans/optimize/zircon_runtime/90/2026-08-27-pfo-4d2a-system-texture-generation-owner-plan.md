# PFO-4d2a System Texture Generation Owner Plan

## 状态

- 日期：2026-08-27
- 当前状态：`pfo_4d2a4e_measurement_foundation_source_implemented_static_review_complete_dynamic_baseline_pending`
- 范围：环境 BRDF LUT、黑色 cube 回退、环境采样器及后续 system texture 的 WGPU device-generation 生命周期
- 证据边界：本文记录源码与仓库内 Unreal/Lumen 参考的结构审计；没有把静态调用计数解释为 GPU 性能、功耗或动态正确性数据。

## 结构重审结论

当前 `SceneRendererCore` 直接拥有 `SceneEnvironmentBrdfLut` 和 `SceneEnvironmentCubemap`。BRDF CPU payload 虽由进程级 `OnceLock` 缓存，但每个 core 仍创建和上传一份原生纹理；环境回退又为 source、specular、irradiance 三个绑定槽创建三张格式、尺寸和内容完全相同的 1x1x6 cube。真实环境尺寸或 mip 变化时三张业务纹理必须重建，但与尺寸无关的 filtering sampler 也被同步重建。

这不是单次 `create_texture` 的局部问题，而是 system texture 没有明确的 device-generation owner：CPU 产物、原生资源身份、bindable view、提交可见性和 device-loss 重建分散在 core 构造与动态环境路径中。直接暴露 `WgpuResourceRegistry` 的原生 getter 会重新建立绕过 neutral RHI 的数据面，因此不采用。

## Unreal / Lumen 对齐

1. Unreal `Renderer/Private/SystemTextures.cpp` 通过 `TGlobalResource<FSystemTextures>` 按 RHI 生命周期初始化系统纹理，并把稳定资源注册进 RDG blackboard；black cube 与 `PreintegratedGF` 都不是每个 view family 临时创建。
2. Unreal 的可迁移语义是“每个有效 RHI generation 一个持久 owner”，不是 Rust 进程全局 raw WGPU singleton。跨 device generation 共享原生 texture/view 是非法的。
3. `dev/LumenInUE5.5.4WithComputeShader/App.cpp` 的静态 `sPreintegratedGF` 同样表明预积分 BRDF 应有稳定身份，但其 D3D12 全局句柄只作为资源/Pass 语义参考，不作为 Zircon 绕过 neutral owner 的接口模板。

## 目标设计

1. 在 `RenderBackend` 所属的 WGPU generation 边界安装 `SystemTextureGenerationOwner`；owner 与 `Arc<WgpuRenderDevice>` 同生共死，不使用进程级 raw WGPU `OnceLock`。
2. BRDF LUT 与黑色 cube 的 CPU artifact 独立于 WGPU，允许跨 generation 复用不可变字节；原生 texture/view/sampler 必须按 generation 唯一创建。
3. owner 先构建完整候选 bundle，再通过既有 upload/submission authority 受理上传；只有所有资源和上传都成功受理后才原子发布 immutable lease。失败不得留下可消费的半初始化 bundle，并允许下一次重试。
4. scene/RDG 消费 typed lease 或 binding bundle，不取得 queue、poll、flush 或 registry 原生快照权限。device-loss 后旧 lease 由 generation 校验拒绝，retirement 服从真实 submission ticket。
5. 动态 source/specular/irradiance 环境纹理仍由 scene environment owner 管理，因为其内容和尺寸属于场景状态；它们复用 generation-owned sampler 和 black-cube fallback lease。
6. 在 neutral binding lease / recorder 具备完整承载能力前，不把 raw view getter 临时扩散到 `RenderDevice`。PFO-4d2a0 先收敛当前 owner 内可以证明等价的物理身份，随后再硬切 generation owner。

## PFO-4d2a0 回退身份前置切片

1. source、specular、irradiance 的冷启动回退均绑定同一张 1x1x6 RGBA16F black cube；三个 Rust handle/view clone 只共享一个原生资源身份，不改变 bind group ABI。
2. black cube 只上传一次 48-byte RGBA16F payload，删除另外两次相同的 queue texture write。
3. 环境 sampler 只在 `SceneEnvironmentCubemap::fallback` 构造时创建；动态纹理 rebind 保持 sampler identity，不制造无效 bindable resource churn。
4. 动态真实环境的三张不同 mip/尺寸纹理仍分别创建和按 frame staging transaction 上传，本切片不错误合并不同语义资源。

## 量化门槛

- PFO-4d2a0 静态门槛：fallback 原生 cube create `3 -> 1`、view create `3 -> 1`、直接 black-cube upload `3 -> 1`；单个 cubemap owner 生命周期 sampler create `1 + rebind 次数 -> 1`。
- 完整 generation owner 稳态门槛：同一 device generation 中 BRDF LUT create/upload 为 1，black cube create/upload 为 1，环境 sampler create 为 1；任意新增 `SceneRendererCore` 不再增加这些计数。
- 算法：初始化为 `O(system artifact bytes)` 时间和内存，热帧 lookup 为 `O(1)`；禁止每帧扫描 owner registry 或复制 LUT/cube payload。
- 动态验收：通过 WGPU diagnostics/RenderDoc 验证唯一身份、上传和 bind group 绑定；通过真实窗口 PNG 验证回退及真实环境视觉；记录冷启动与稳态 CPU/GPU p50/p95/p99、resource-create 数、upload bytes 和功耗。
- 动态 WGPU、截图、RenderDoc、profile 和功耗数据统一留在验收阶段；未取得证据前不得宣称完整 generation owner、性能瓶颈消失或视觉验收通过。

## PFO-4d2a0 当前完成项与静态结果

- `SceneEnvironmentCubemap::fallback` 现在只创建一张 1x1x6 `Rgba16Float` black cube 和一个 cube view；source/specular/irradiance 三个绑定槽通过 WGPU handle clone 共享同一物理资源身份，bind group ABI 不变。
- black cube 初始化只编码并上传一次 48-byte payload。源码结构计数由 texture create `3 -> 1`、view create `3 -> 1`、direct fallback upload `3 -> 1`。
- filtering sampler 只在 cubemap owner 冷构造时创建。真实环境尺寸/mip rebind 仍事务性创建三张不同语义纹理，但 sampler create/publish 均为 0，避免与内容无关的资源 churn。
- 已增加边界化源码契约测试，锁定 fallback 单物理身份、单上传以及 dynamic rebind 不创建/替换 sampler；局部 `rustfmt --check`、调用计数脚本和 scoped `git diff --check` 通过。
- 本切片未运行 Cargo 或动态 WGPU。完整 generation-local shared owner、BRDF/black-cube/sampler 跨 core 复用、shadow fallback 迁移、PNG、RenderDoc、profile 与功耗仍 pending。

## PFO-4d2a1 Shadow Binding Lease

### 调用图与问题

- 每个 `SceneRenderer` 当前创建一个独立 `RenderBackend`/WGPU device 和一个 `SceneRendererCore`；因此把资源字段机械搬到 backend 尚不能提供可量化的多-core复用。
- 同一 core/device generation 内存在一个真实重复 consumer：`ShadowMapRenderer` 复用完整 scene bind-group layout，为满足绑定校验另外创建3张1x1x6 cube、1张1x1 BRDF texture、1个 sampler 和1个 SH9 buffer。
- shadow depth/alpha-mask pass需要每个 atlas slot 的独立 camera uniform，但没有第二套环境内容 producer。额外环境对象只是合法 binding fallback，且当前 shadow cube/LUT 没有初始化 payload。

### 实施边界

1. scene environment 冷构造时显式发布拥有 handle clone 的 `ShadowSceneEnvironmentBindingLease`。lease 保持同一 WGPU generation 的 black cube texture/view、BRDF LUT texture/view、sampler 与 SH9 buffer 存活。
2. `ShadowMapRenderer` 移入 lease，仅以同一 black cube view 填充 source/specular/irradiance 三个槽；每个 shadow slot 仍创建自己的 camera uniform 和 bind group，保持 view-projection 隔离。
3. 删除 shadow 私有 cube/LUT/sampler/SH9 create helpers；不收窄公共 scene layout，不改变 shader permutation 或 material alpha-mask 行为。
4. lease 不提供 queue、poll、submit、registry getter或跨 generation复制；它是完整 generation owner 前的只读消费契约，后续可由 `SystemTextureGenerationOwner` 直接发布。

### 静态与动态门槛

- full-scene cold shadow 环境增量：texture create `4 -> 0`、view create `4 -> 0`、sampler create `1 -> 0`、SH9 buffer create `1 -> 0`。
- shadow slot 数增长时环境 resource create始终为0；slot camera uniform/bind-group的后续持久化属于独立 hot-path工作，不与本切片混报。
- 源码测试锁定 shadow product constructor不接收 device、不包含 environment resource create，并验证三个 cube binding引用同一 lease view。
- Cargo、真实 WGPU shadow draw、PNG、RenderDoc、resource identity capture、CPU/GPU profile与功耗继续留到统一验收阶段。

### 当前完成项与静态结果

- 新增 `ShadowSceneEnvironmentBindingLease`。scene bundle 在冷构造时从已初始化 black cube、真实 BRDF LUT、稳定 sampler 和 SH9 buffer 克隆 WGPU handles，lease 保持这些同-generation 资源存活。
- `ShadowMapRenderer::new` 不再接收 device，只消费 layout 与 lease；shadow 私有3张cube、1张BRDF texture、sampler、SH9 buffer及其create helpers已删除。
- shadow scene bind group的source/specular/irradiance三槽引用同一个lease black-cube view，BRDF/sampler/SH9各引用一次；atlas slot camera uniform仍按slot隔离。
- 静态计数：shadow constructor texture/view/sampler/buffer create均为0，legacy environment helper为0，black-cube binding reference为3。full-scene shadow cold增量由4 texture + 4 view + 1 sampler + 1 buffer降为0。
- 六个触及Rust文件的局部`rustfmt --check`、构造/绑定调用图脚本与scoped `git diff --check`通过。Cargo、真实WGPU、shadow PNG、RenderDoc、profile、功耗及完整backend generation owner仍pending。

## PFO-4d2a2 Array Copy Contract 与 Generation Owner

### 结构重审与阻塞根因

- `TextureCopyRegion` 当前只表达二维 `width/height` 和单个 `origin_z`，WGPU upload/command bridge 因而把 `depth_or_array_layers` 固定为 1。继续把 black cube 拆成六个 feature-owned 写入会绕过 neutral copy contract 的缺口，并把一个连续 48-byte 系统资源错误放大为六个 native staging write。
- `SceneEnvironmentBrdfLut::new` 与 `SceneEnvironmentCubemap::fallback` 仍在每个 core 构造期间直接创建和上传原生资源。现有 `ShadowSceneEnvironmentBindingLease` 只消除了同一 core 内 shadow consumer 的重复资源，尚未建立 backend device-generation owner。
- 每个 `SceneRenderer` 当前仍创建独立 `RenderBackend`/WGPU device。本切片只承诺同一 backend/device generation 内唯一，不虚构跨 device 的原生资源共享；跨 renderer 共享必须等待 backend 生命周期上移。

### 实施顺序

1. 先把 `depth_or_array_layers` 纳入 neutral `TextureCopyRegion`，序列化旧输入默认值为 1；upload byte budget、边界校验、deterministic executor、native queue write 和 command encoder 统一消费同一深度。
2. 多层线性数据的 required bytes 使用 `O(1)` checked arithmetic：`(depth - 1) * rows_per_image * bytes_per_row + (height - 1) * bytes_per_row + row_bytes`；禁止按层临时复制或为校验扫描 payload。
3. 在 `RenderBackend` 的 WGPU generation 边界安装惰性 `SystemTextureGenerationOwner`。普通 compute/test/surface backend 不生成 BRDF 或提交无用纹理；首个 scene acquire 才构建完整候选：一张 1x1x6 RGBA16F black cube、一张 RG16F BRDF LUT 和一个 filtering sampler。
4. black cube 与 BRDF LUT 组成一个 `WgpuTextureUploadBatch`，由 `WgpuRenderDevice` 受理为一张 Copy ticket 并在 owner 内完成一次 native flush。只有 flush 返回后才原子发布 immutable lease；失败时 owner 保持未发布，局部候选释放并允许同 generation 重试。
5. `SceneRendererCore` 只接收 generation-qualified typed lease；fallback cubemap 和 BRDF wrapper 从 lease 克隆绑定 handle，不接收 queue，也不重新创建 system texture/sampler。动态 source/specular/irradiance 纹理与 scene SH9 保持 scene owner。
6. generation owner 字段必须先于 `WgpuRenderDevice` 声明，使 Rust drop 顺序先释放 owner retained handles，再释放 submission/device owner；`SceneRenderer` 内 backend 字段必须最后声明，使 core、streamer、target、history 与所有 scene lease 先释放，代际 owner 最后释放。

### 静态与动态门槛

- neutral array-copy 静态门槛：region 默认 depth 为 1；所有 native texture copy extent 不再硬编码 1；multi-layer required-byte 和 destination bounds 均 checked；2D 现有调用行为不变。
- generation owner 静态门槛：backend 冷构造的 system resource create/upload 为 0；首个 scene acquire 恰好创建 black cube 1、BRDF LUT 1、filtering sampler 1，提交并 flush 一个含两项 texture write 的 Copy batch；同 generation 后续 acquire create/upload/submit 均为 0；scene fallback/BRDF 构造中的 `queue.write_texture`、system texture create 和 sampler create 降为 0。
- 算法门槛：CPU LUT artifact 进程内不可变缓存仍为 `O(lut texels * integration samples)` 冷构造、`O(1)` 热复用；generation native 初始化为 `O(system artifact bytes)`，热帧 lease lookup 为 `O(1)`，无 registry 扫描、payload clone 或每帧上传。
- 动态验收继续统一留在 milestone validator：WGPU array-copy tests、真实 scene/shadow draw、PNG、RenderDoc resource identity、Copy ticket/bytes、冷启动与稳态 CPU/GPU p50/p95/p99、功耗。取得这些证据前不宣称性能瓶颈消失或 milestone accepted。

### PFO-4d2a2 当前完成项与静态结果

- neutral `TextureCopyRegion` 已增加 `depth_or_array_layers`，构造与旧 serde 输入默认值均为 1，默认值不会写入新序列化输出。多层 source byte budget、origin-z 边界和 destination end 均使用 checked `O(1)` 算术；deterministic buffer/texture、texture/buffer、texture/texture executor 按 layer/row 搬运，WGPU 三个 command-copy extent 与两个 queue-upload extent 均透传 neutral depth，原生提交中的硬编码 depth `1` 为 0。
- `RenderBackend` 已安装惰性 `SystemTextureGenerationOwner`：普通 backend 冷构造 system texture create/upload 为 0；首个 scene acquire 的源码契约为 black cube texture 1、BRDF LUT texture 1、filtering sampler 1、连续 texture write 2、Copy ticket 1、device-owned flush 1，固定 payload 为 `48 + 16,384 = 16,432 bytes`。flush 成功后才发布 lease，失败不发布；同 generation 后续 acquire 返回已发布 lease，create/upload/ticket/native submission 均为 0。
- BRDF CPU artifact 改为进程级 `OnceLock<Arc<[u8]>>`，冷路径只计算和编码一次，后续 device generation 只克隆 `Arc`，不再把缓存的 16 KiB LUT 复制成新的 payload owner。原生 texture/view 仍按 device generation 独立创建，不跨 device 共享 WGPU handle。
- scene BRDF wrapper 与 black-cube fallback 只投影 typed generation lease；两条 system-resource 消费路径的 texture/sampler create 和 `queue.write_texture` 均为 0。真实 source/specular/irradiance 环境纹理仍由 scene owner 按内容与尺寸重建，没有被错误并入 system texture。
- owner acquire 与 `SceneRendererCore` 都校验 `DeviceId + DeviceGeneration`。`RenderBackend` 中 system owner 先于 device owner 声明，`SceneRenderer` 中 backend 最后声明，确保 scene/native resource handle 先释放、device generation owner 最后释放。
- 按代码结构规范把 startup 聚合、Core 分阶段计时和 Base PSO 预热报告迁入具名 `startup_report.rs` owner；`scene_renderer.rs` 由 886 行降至 666 行，startup report owner 为 221 行，均低于 800 行 review warning，公共 re-export 路径保持不变。
- 已增加 legacy serde、连续 array-layer deterministic roundtrip、owner lazy/publish ordering、neutral-to-WGPU depth 透传、scene consumer 零旁路和文件预算结构门禁。28 个 exact Rust 文件的 `rustfmt --check --edition 2021 --config skip_children=true`、scoped staged/unstaged `git diff --check`、trailing-whitespace 扫描通过；全 `zircon_runtime` 显式 texture range-error constructor 缺失 depth 字段计数为 0；owner/array/scene 静态门禁 failure 均为 0。
- 本状态仍未运行 managed Cargo、真实 WGPU scene/shadow draw、PNG、RenderDoc、CPU/GPU p50/p95/p99、功耗或 device-loss 验收，因此不标 milestone accepted，不写 accepted output row，也不提交性能瓶颈已消失的结论。

## PFO-4d2a3 Neutral Fallback Family 与 Core Raw Queue Hard Cut

### 当前源码重审与参考边界

- 生产态 graphics 仍有 12 处 `queue.write_texture`。其中 post-process black/white/HZB/effect LUT 5 处、transmission 1 处、irradiance volume 1 处、lightmap atlas fallback 1 处，均是同一 device generation 内不可变的 neutral fallback；resource streamer asset fallback、动态 post-process LUT、reflection probe PMREM 与 viewport icon atlas 仍有各自内容/驻留 owner，不属于本切片。
- `MeshPipelineCache::construct` 只因 light-cookie、transmission、irradiance 与 lightmap 冷资源接收 raw queue；`ScenePostProcessResources::new` 只因 5 张 fallback 接收 raw queue。继续在 feature constructor 内初始化这些纹理会维持隐式提交边界、重复物理身份与错误的 scene 生命周期。
- Unreal `FSystemTextures` 在 RHI 生命周期内拥有 white、black、black-alpha-one、normal、array、volume 与 cube dummy；`FLightFunctionAtlas` 则是独立 scene owner，disabled binding 只借用 `GSystemTextures.GetWhiteDummy`。Zircon据此只把 neutral binding 资源并入 generation owner，不把 1024 atlas、asset texture、probe PMREM 或 dynamic scene texture误并为全局资源。
- 现有 2026-07 RenderDoc capture 为 12,085,874 bytes，可作为历史功能证据，但不匹配当前 dirty source/build identity，不能充当本切片的当前 GPU timing、提交次数或功耗基线。`LightCookieAtlasBuildPass` 每次以 `LoadOp::Clear(WHITE)` 覆盖整张 atlas，构造期另行上传 4,194,304-byte 全白 payload 在源码语义上冗余；按性能约束，本次只记录该候选，实时 profile 前不把它与 neutral owner 切片混成“已完成 atlas 优化”。

### 目标结构与实施顺序

1. `SystemTextureGenerationOwner` 的 candidate bundle扩展为 9 张不可变纹理：black cube、BRDF LUT、RGBA8 black、RGBA8 black-alpha-one、RGBA8 white、RGBA16F black、irradiance-volume black、2D effect LUT 与 3D identity LUT；同时发布一只 generation-local linear-clamp sampler。
2. RGBA16F black 同一物理纹理发布 D2 与单层 D2Array 两个 view，分别供 HZB/其他二维 fallback 与 lightmap atlas fallback 使用；禁止为 view dimension 再创建第二张相同纹理。transmission、post-process black 与 black-alpha-one保持不同 alpha 语义。
3. 9 项 payload 进入首个 scene acquire 的同一个 `WgpuTextureUploadBatch`、同一 Copy ticket 与同一次 owner flush；publish 仍只发生在 flush 成功后。同 generation 重复 acquire 的 create/upload/ticket/native submission 仍全部为 0。
4. `SceneRendererCore` 以借用方式把 typed system-texture lease传给 `MeshPipelineCache` 与 full post-process resources。两者删除 raw queue 参数；advanced-lighting/lightmap/post-process wrapper只克隆精确 texture/view/sampler handle，不得到 backend、queue、registry或跨 generation getter。
5. effect LUT 生成算法和现有 oracle迁入 system texture owner附近的具名 payload owner；避免 post-process feature继续拥有设备级 payload生成。owner主文件保持低于800行，新增资源/载荷职责进入子模块。
6. resource streamer fallback、asset LUT、reflection probe与viewport icon保持原 owner和上传路径；这些动态/资产路径必须后续接入统一 upload service，不以本切片的 system texture lease绕过 residency、revision 或 frame budget。

### 量化与验收门槛

- 首个 acquire 的 system upload由 `2 textures / 16,432 bytes` 扩展为 `9 textures / 16,764 bytes`，但 Copy ticket与native flush仍各为1；332-byte neutral payload不再通过8个 feature-owned raw queue write发布。
- full-scene core冷构造的 feature-owned neutral texture create由8降为0，raw `queue.write_texture`由8降为0；RGBA16F HZB/lightmap两张物理纹理收敛为1张纹理、2个typed view。`MeshPipelineCache` 与 `ScenePostProcessResources` 的 raw queue constructor参数均降为0。
- 系统资源初始化复杂度保持 `O(system artifact bytes)`，热 acquire与各consumer投影均为 `O(1)`；禁止 registry扫描、payload深复制、按consumer重复上传或每帧neutral重建。
- Cookie atlas的4,194,304-byte构造上传删除、persistent slot/dirty-region更新与GPU timing属于独立后续性能切片。删除前必须取得匹配当前 build identity 的CPU allocation/submit与RenderDoc pass/resource基线；本节不得据历史capture宣称该瓶颈已消失。
- 静态门禁必须锁定9项system upload、16,764 bytes、单ticket/flush、consumer零raw write、core constructor零raw queue、RGBA16F单纹理双view与文件预算。动态验收仍需managed Cargo/WGPU、真实scene PNG、RenderDoc resource identity/Copy事件、device-loss重建、CPU/GPU p50/p95/p99与功耗；完成前状态只能是source/static complete、dynamic pending。

### 当前状态

- `source_implemented_static_review_complete_dynamic_validation_pending`：generation owner 已扩展为 9 项不可变纹理 bundle，固定 payload 为 `16,764 bytes`，首个 acquire 仍只使用 1 个 Copy ticket 与 1 次 owner flush；publish 仅在 flush 成功后发生，同 generation 后续 acquire 的 create/upload/ticket/native submission 均为 0。
- `MeshPipelineCache`、full post-process、transmission、irradiance volume 与 lightmap fallback 已改为只消费 typed system-texture lease；RGBA16F black 由 1 张物理纹理发布 D2/D2Array 两个 view。17 个 mesh WGPU fixture 已迁移到 typed lease，feature-owned neutral texture create/raw write 均降为 0。
- effect LUT payload owner 已迁到 generation owner 子模块；owner 主文件为 319 行、resources 子模块为 467 行，另有纯 CPU payload/字节预算与结构门禁。完成本切片后生产态 raw `queue.write_texture` 由 12 降为 5。
- 已运行一次 managed validator：`.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter runtime_90_neutral_fallback_family_has_one_generation_owner -VerboseOutput`。目标目录为 `D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`；Cargo 在编译 Rust 前以 101 退出，因为共享 dirty manifest/lock 状态触发 `cannot update Cargo.lock because --locked was passed`。未移除 `--locked`、未改写 lockfile，也未把该结果记为动态通过。

## PFO-4d2a4 Asset/Scene Texture Upload Transaction

### 原剩余 5 处 raw upload 所有权重审

1. resource streamer white/normal fallback 经二次重审后修正归属：两者的 `GpuTextureResource.id` 为 `None`，不参与 asset revision/residency，实际是缺失材质纹理时的稳定绑定资源。Unreal `MaterialUniformExpressions.cpp` 在无效二维材质纹理时直接绑定 `GWhiteTexture` 及其 sampler；因此 Zircon 应把 white/default-normal 原生资源并入 generation system bundle，再由 streamer 投影 descriptor/bind-group，而不是建立第二套资产上传事务。
2. post-process LUT asset 具有 resource revision/residency 语义，后续改为 prepared asset upload；system effect LUT 只覆盖固定 identity/oracle，不替代动态 LUT asset。
3. reflection probe PMREM 具有 cubemap revision、LRU slot 与 frame visibility 三重约束。旧实现按 `8 mips * 6 faces` 直接执行 48 次 `queue.write_texture`，并在统一场景提交成功前就把 revision 写进 slot allocator，失败帧会错误复用未提交内容。本切片优先修复该结构性问题。
4. viewport icon atlas 是惰性 cache owner，后续需要 pending publication 与 frame upload transaction，不能借 system texture owner 绕过 atlas cache identity。
5. light-cookie atlas 是独立 scene owner。当前构造期上传 `1024 * 1024 * 4 = 4,194,304 bytes` 全白 payload，而每次 build pass 又以 white clear 覆盖整张 atlas；它需要匹配当前 build 的 allocation/submit/RenderDoc 基线后再独立删除冗余上传并设计 persistent slot/dirty-region 更新。

### Unreal / RDG 参考结论

- `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentCapture.cpp` 的 reflection capture 路径先完成 mip/face GPU upload，再设置可供 shading 使用的状态；异步入口把上传放进 render command，并在 GPU upload 边界后才 finalize source data。Zircon 因此必须区分 prepared/pending 与 submitted/ready，不能以 CPU revision lookup 代替 GPU 发布事实。
- Unreal `LightFunctionAtlas.cpp` 采用 consumer/profile gate 与 RDG texture，disabled/default binding 借用 `GSystemTextures` white dummy；generation pass 使用 clear/no-action load 语义，不在 feature constructor 中额外提交整张 white texture。Zircon 后续 light-cookie 切片按此分离 disabled neutral binding、atlas owner 与 frame graph pass。

### PFO-4d2a4a Reflection Probe 实施边界

1. PMREM payload 的 mip 内六个 face 在源和目标上均连续，因此每个 mip 生成 1 个 `depth_or_array_layers = 6` 的 layered upload；descriptor 数由 48 降为 8，descriptor 构建为 `O(mips)`，不再按 face 分配/提交。
2. 8 个 mip descriptor 共享 1 个 `Arc<[u8]>`，固定 128px、8 mip、RGBA16F cube payload 为 `1,048,560 bytes`。上传与 planar/probe/header buffer 更新共同进入该帧唯一 `WgpuResourceUploadBatch`；compiled 与 direct scene 都只有 1 个 frame resource packet。
3. slot allocator 使用 `Ready | Pending(prepare_epoch)`。同 epoch 重复 cubemap 复用已排队 slot；未发生成功 scene submission 的 pending slot 在下一 epoch 必须重传。commit 同时校验 cubemap、revision、slot 与 epoch，迟到 commit 不能发布后续重试。
4. pending publication 只保存本帧实际变更项，commit 为 `O(changed probes)`；LRU acquire/touch/evict 保持 `O(1)`，没有 allocator/HashMap clone。epoch 回绕时只失效 pending 状态，不改变已提交 Ready 资源。
5. compiled/direct scene 仅在 `submit_graphics_command_buffers_with_frame_diagnostics_and_surface` 成功返回后调用 `commit_pending_uploads`；enqueue/flush/submit 失败保留未发布语义，下一帧重试。probe upload 文件中的 raw `queue.write_texture` 为 0。

### 后续分层

1. PFO-4d2a4b：resource streamer white/default-normal generation projection，已完成源码与静态审查。
2. PFO-4d2a4c：post-process LUT prepared asset upload 与 revision publication。
3. PFO-4d2a4d：viewport icon atlas pending publication/frame transaction。
4. PFO-4d2a4e：在取得匹配 build identity 的 CPU allocation、submission、RenderDoc pass/resource 与 GPU timing 基线后，删除 light-cookie 4 MiB 构造上传并收敛 atlas 更新模型。

### 当前状态

- `source_implemented_static_review_complete_dynamic_validation_pending`：反射探针 raw texture write `48 -> 0`，frame upload descriptor `48 -> 8`，face loop `1 -> 0`，payload owner allocation 为 1，allocator clone 为 0；生产态 graphics raw `queue.write_texture` 总数由 5 降为 4。
- exact 11 个 Rust 文件的 `rustfmt --check --edition 2021 --config skip_children=true`、scoped `git diff --check` 与静态事务门禁通过。`resources.rs` 731 行、slot allocator 219 行、upload owner 218 行，均低于 800 行 review warning。
- Cargo/WGPU、真实 scene PNG、RenderDoc capture、resource identity/Copy 事件、device-loss、CPU/GPU p50/p95/p99 与功耗仍未取得当前 build 证据；沿用上述共享 `Cargo.lock --locked` 动态阻塞，不重复运行 validator，不标 milestone accepted，不提交 commit/WeCom 消息。

## PFO-4d2a4b Material Fallback Generation Projection

### 结构修正

- white/default-normal 不具有 asset id、revision 或 residency 状态，不应以 resource streamer 构造期 raw queue write 模拟资产发布。产品路径只需 generation-qualified texture/view/sampler lease；测试 helper 的独立 WGPU fixture 通过 `#[cfg(test)]` 保留，不进入产品所有权与 raw write 计数。
- system white 保持 1 张 `Rgba8Unorm` 物理纹理，同时发布 linear 与 `Rgba8UnormSrgb` typed view。post-process 继续消费 linear view，材质 missing-image fallback 消费 sRGB view；不会因为 color-space ABI 创建第二张全白纹理。
- default normal 新增 1 张 `Rgba8Unorm` 物理纹理和 `[128, 128, 255, 255]` 的 4-byte payload。system bundle 从 `9 textures / 16,764 bytes` 扩展为 `10 textures / 16,768 bytes`，但 Copy ticket 与 owner flush 仍各为 1，同 generation 热 acquire 仍为 0 create/upload/submit。
- `TextureSamplerCache` 以 generation linear-clamp sampler 预置默认 key；white/default-normal projection 与后续默认 sampler asset 共用该 `Arc<wgpu::Sampler>`，不在 streamer 构造期创建第二个等价原生 sampler。

### 当前状态

- `source_implemented_static_review_complete_dynamic_validation_pending`：产品 resource streamer fallback 的物理 texture create `2 -> 0`、raw texture write `2 -> 0`、queue constructor 参数 `1 -> 0`；production graphics raw `queue.write_texture` 总数由 4 降为 3，仅剩 post-process LUT asset、light-cookie atlas 与 viewport icon atlas。
- system white 物理纹理计数为 1、typed view 为 2；default-normal 物理纹理为 1；system upload count/bytes 为 `10 / 16,768`，owner enqueue/flush 各为 1。fallback、sampler cache、streamer construction 与 system resources 均低于 800 行 review warning。
- exact 11 个 Rust 文件的 `rustfmt --check --edition 2021 --config skip_children=true`、scoped `git diff --check`、产品构造/资源计数与 raw upload owner 静态门禁通过。Cargo/WGPU、真实 PNG、RenderDoc resource identity、device-loss、CPU/GPU profile 与功耗仍 pending；不标 milestone accepted。

## PFO-4d2a4c Post-process LUT Asset Upload Transaction

### 结构重审与 Unreal 对齐

- 旧 `PostProcessLutTextureResource::from_rgba8_asset` 在资源构造函数中直接调用 `queue.write_texture`，随后 `ResourceStreamer` 立即把 revision/resource 写入 `post_process_lut_textures`。该路径没有进入 `RenderBackend` submission authority、没有 frame ticket，也无法在 queue admission 或 scene submission 失败时撤销错误发布。
- effect-stack 状态判断原先调用 `load_texture_asset`，会深拷贝包含 LUT `Vec<u8>` 的完整 `TextureAsset`；随后上传路径再次读取资产。32³ RGBA8 LUT 的有效 payload 为 `32 * 32 * 32 * 4 = 131,072 bytes`，因此旧路径在进入 WGPU write 前已有一份不必要的 128 KiB 资产深拷贝。
- Unreal `PostProcessCombineLUTs.cpp:153` 用 volume descriptor 表达完整三维 LUT，`761-784` 先注册持久 view-state LUT 或在 graph 中创建临时 LUT，`825/855` 再把生成工作放入 RDG compute pass。可迁移语义是“持久资源身份与 graph/submission 工作分离，并在统一图提交中建立可见性”，不是 feature owner 私自写 queue 后立即宣称 ready。

### 实施结果

1. `PostProcessLutTextureResource::prepare_from_rgba8_asset` 现在只创建物理 texture/view，并返回 `PostProcessLutTextureUploadWork { resource, upload_batch }`。完整三维 payload 使用 1 个 `TextureCopyRegion::with_depth_or_array_layers(depth)`，descriptor/上传循环由按 slice 潜在扩展收敛为固定 1；生产代码中的 `wgpu::Queue` 与 `queue.write_texture` 均为 0。
2. upload layout 以 checked `O(1)` 算术同时得到 width、height、depth、bytes-per-row、rows-per-image 与 byte length。上传只从原子 snapshot 的有效范围建立 1 个 `Arc<[u8]>` payload owner；完整 `TextureAsset` 深拷贝为 0，超长尾部不会进入 GPU 上传。
3. `ensure_post_process_lut_texture` 保留 requested-revision 热路径；effect-stack 3D 路径则把用于 shape 判定的同一 `ResourceSnapshot<TextureAsset>` 直接交给发布函数。单个 3D LUT 请求的资产读取由 2 次降为 1 次原子 snapshot，资产 payload 深拷贝由 1 次降为 0。
4. LUT upload 通过 `RenderBackend::enqueue_copy_texture_upload_batch` 取得 1 个 Copy ticket，并以 `TextureCopyUpload + ResourceId` 记入 `RenderFrameSubmissionTransaction`；只有 ticket 记录成功后才同帧发布 resource/revision，不增加私有 flush。irradiance-volume texture 复用同一事务入口。
5. cold frame failure 继续复用 texture submission receipt。`Failed | Cancelled | DeviceLost` 的资源 ID 会同时撤销普通 texture、mip state、依赖 material 与 `post_process_lut_textures`；已 `Submitted | Completed` 的上传不被错误删除。稳定帧仍无 cache scan，失败回滚只扫描冷路径 receipt/cache。

### 静态量化与未完成证据

- LUT resource：raw write `1 -> 0`、raw queue type `1 -> 0`、upload batch `0 -> 1`、layered region `0 -> 1`、upload loop `0`、payload owner allocation `1`、完整 asset clone `1 -> 0`。
- frame transaction：snapshot route `1`、backend enqueue ticket `1`、resource ticket record `1`、私有 flush `0`；32³ RGBA8 payload 为 `131,072 bytes`，CPU preparation 为 `O(payload bytes)` 的一次不可变 payload copy，cache hit 为 `O(1)` revision lookup且零 upload。
- 生产态 graphics raw `queue.write_texture` 总数由 3 降为 2，仅剩 light-cookie atlas 与 viewport icon atlas。exact 6 个 Rust 文件的 `rustfmt --check --edition 2021 --config skip_children=true`、scoped `git diff --check`、resource/transaction/snapshot-route/failure-rollback 静态门禁通过；相关生产文件均低于 800 行 warning。
- 沿用共享 dirty `Cargo.lock --locked` 阻塞，不重复运行 managed validator。Cargo/WGPU、真实 scene PNG、RenderDoc Copy/resource identity、device-loss、CPU/GPU p50/p95/p99 与功耗仍 pending；本状态不标 milestone accepted，不提交 commit/WeCom 消息，也不宣称动态瓶颈或功耗已收敛。

## PFO-4d2a4d Viewport Icon Atlas Upload Transaction

### 结构重审与 Unreal 对齐

- 旧 viewport icon cache 在首次命中时直接创建 texture/view/bind group、调用 `queue.write_texture`，并立即把条目标记为 `Ready`。该写入没有进入 frame resource ticket；queue admission、后续 frame preparation 或 scene submission 失败时，CPU cache 仍会把未被统一提交边界确认的候选当作稳定资源。
- viewport icon atlas 与 screen-space UI atlas 是不同 owner：前者只缓存 camera/directional-light 两个 scene-gizmo sprite，后者管理通用 UI atlas page。把二者强行合并会污染 cache identity；正确收敛点是它们共同追加到已有 `frame_texture_uploads`，而不是共享业务 atlas。
- Unreal `SlateRHIResourceManager.cpp:723-746` 先建立动态纹理 resource/proxy，并通过 `ENQUEUE_RENDER_COMMAND` 把 source data 与 RHI 初始化交给 render submission owner；`1050-1057` 把 atlas 更新集中在 `UpdateTextureAtlases/ConditionalUpdateTexture`。`SlateRHIFontTexture.cpp:136-166` 同样把 dirty atlas 的更新集中在 render-thread command。可迁移语义是 cache identity、pending source data 与 RHI submission 分离，而不是 draw consumer 直接写 queue 后立即发布 ready。

### 实施结果

1. `IconEntry` 现在为 `Unloaded | Missing | Pending { sprite, upload } | Ready`。首次 decode 只创建候选 texture/view/bind group，并把唯一 `Vec<u8>` 直接移动进 `WgpuTextureUpload::from_owned_bytes`；无 payload clone、无 raw queue write。候选 binding 可供同帧 draw 编码使用，但不会提前获得 `Ready` 身份。
2. atlas 保存可重放 upload debt。每次 scene-gizmo prepare 完成 icon discovery 后只遍历固定两个槽，把每个 `Pending` upload 的 texture handle 与 `Arc` payload 各克隆一次并入现有 `frame_texture_uploads`；同一图标在同帧出现任意次数仍只有一个 upload descriptor。prepare/enqueue/ledger/scene 任一阶段失败时 pending 保留，下一次 frame preparation 自动重放，不依赖 frame generation 是否递增。
3. direct 与 compiled 路径均复用已有唯一 `WgpuResourceUploadBatch` 和 `FrameResourceUpload` ticket，不增加 copy ticket、native flush 或私有提交。两条路径都只在 `RenderFrameSubmissionTransaction::validate_scene_submission` 成功后调用 `commit_pending_icon_uploads`，此时 `Pending -> Ready` 并释放 retained CPU payload。
4. `ViewportOverlayRenderer` 只调用 `SceneGizmoPass::commit_pending_icon_uploads` 窄接口，不穿透访问私有 atlas 字段。overlay prepare、compiled helper 与 icon owner 都删除 `wgpu::Queue` 参数；draw/binding owner 不再拥有 submission authority。
5. RGBA8 layout 使用 checked width/height/byte-length 算术并要求 payload 长度精确匹配。固定槽重放与 commit 为 `O(2) = O(1)`；稳定 `Ready` 帧无 payload allocation、无 upload descriptor、无 ticket/flush 增量，cache lookup 为直接 slot 索引。

### 静态量化与未完成证据

- viewport icon 生产路径：raw texture write `1 -> 0`、raw queue type `1 -> 0`、owned upload constructor `0 -> 1`、private flush `0`；一个新图标只有 1 个 texture upload descriptor，同帧重复 draw 不增加 descriptor。生产态 graphics raw `queue.write_texture` 总数由 2 降为 1，仅剩 light-cookie atlas。
- direct/compiled 静态顺序均通过：`icon prepare < frame resource enqueue < scene transaction validate < icon commit`。exact 14 个 Rust 文件的 `rustfmt --check --edition 2021 --config skip_children=true`、scoped `git diff --check`、生产源码 owner 计数与事务顺序门禁通过；核心新增/修改 owner 文件均低于 800 行，existing compiled `render.rs` 为 899 行、低于 1000 行 hard stop 且本切片只增加窄接线。
- 沿用共享 dirty `Cargo.lock --locked` 阻塞，不重复运行 managed validator。Cargo/WGPU、真实 scene icon PNG、RenderDoc Copy/resource identity、失败注入/device-loss、CPU/GPU p50/p95/p99 与功耗仍 pending；本状态不标 milestone accepted，不提交 commit/WeCom 消息，也不宣称动态瓶颈或功耗已收敛。

## PFO-4d2a4e Light-cookie Atlas Measurement Foundation

### 优化前结构重审

- 当前 atlas 构造先分配 `1024 * 1024 * 4 = 4,194,304 bytes` 的临时全白 `Vec<u8>` 并直接 `queue.write_texture`。每次 `LightCookieAtlasBuildPass` 又以 `LoadOp::Clear(WHITE)` 清理全部 `1,048,576` 像素，然后为每个已解析 texture entry 创建 bind group、设置 128x128 viewport 并绘制一个 fullscreen triangle。
- 源码可证明构造上传与首次 build clear 在内容上重复，但不能证明它在真实产品冷启动中的 CPU allocation、driver upload、submission 或功耗占比；同样不能证明稳态主要成本位于 frame-plan 排序、每 entry bind-group create、全图 clear 还是实际 slot draws。按优化门槛，未取得匹配 current build 的 profile/RenderDoc 前不删除上传、不引入 persistent slot/dirty region cache。
- 当前 `frame_plan.rs` 正有共享工作区内的独立 CPU 排序优化与 release benchmark 变更。本切片不修改该文件，也不把它的候选 benchmark 解释成 atlas GPU/descriptor 瓶颈证据。

### Unreal 对齐与观测合同

- Unreal `LightFunctionAtlas.cpp:26` 为 atlas generation 建立独立 GPU stat；`30-39` 明确按 material identity 去重并以单 atlas + constant-buffer mapping 供多个 consumer 使用；`54-78` 把 atlas enable、slot resolution、edge size 和 light count设为可观测配置；generation 受 consumer gate 驱动，不是 feature constructor 的隐式上传。
- Zircon 先补齐同级观测：`light_cookie/atlas_construct`、`light_cookie/initial_white_upload`、`light_cookie/frame_plan` 与 `light_cookie/atlas_encode` 四个 CPU scope；每个成功 compiled scene frame发布 rebuild count、输入 cookie count、planned entry count、resolved draw count、unresolved entry count、blit bind-group create count、full-clear pixel count和generation initial-upload bytes。
- 统计由 `LightCookieAtlasResources` owner 在 compiled frame 开始时 O(1) reset，rebuild 后 O(1) saturating aggregate，scene submission/transaction 成功出口统一 emit。失败帧不混入成功帧分布；profiling关闭时只保留固定字段加法，不增加资源扫描、分配、submit、poll或GPU命令。
- `planned - resolved` 只表示当前 streamer 无法解析的 atlas entry；`input - planned` 同时包含 duplicate light 与64槽容量截断，未增加额外集合/扫描前不得把二者伪装成独立指标。bind-group create 与resolved draw当前一一对应，保留两条series用于和后续cache实验做ABI稳定的前后对照。

### 动态判定门槛

1. Windows产品场景按0/1/4/16/64 cookie和25% texture-miss矩阵预热60帧、采集300帧、重复3次；启动capture另采集atlas construct/initial upload，稳态记录frame-plan/encode CPU scope及全部counter的p50/p95/p99/max。
2. RenderDoc核对每帧pass count、full clear、draw count、viewport、bind-group/resource identity和构造期Copy事件；记录capture与current build identity，产物放入`docs/tests/runtime/render`。
3. 同机同driver记录CPU frame、GPU atlas pass、native submission、allocation/RSS/VRAM和功耗。只有构造Copy、全图clear或per-entry binding任一项达到可重复的实际占比，才选择对应的单变量结构优化。
4. 若persistent slot/dirty region被证实必要，目标复杂度为frame update `O(changed cookies)`、stable frame `O(1)`、slot lookup `O(1)`，并以revision/generation + scene submission成功发布；在证据前不实现该模型。

### 当前完成项与静态结果

- 新增独立 `LightCookieAtlasProfile` owner，成功帧发布 8 条series：rebuild、input cookies、planned entries、resolved draws、unresolved entries、blit bind-group creates、full-clear pixels和generation initial-upload bytes。统计只在已有rebuild结果上做saturating add，没有新增cookie/resource遍历、临时集合或锁。
- `LightCookieAtlasResources::rebuild` 已加入 `light_cookie/frame_plan` 与 `light_cookie/atlas_encode` CPU scope。compiled scene在frame入口reset，且只在scene transaction验证成功后emit；静态顺序为 `reset < graph execution < scene validation < emit`。失败帧不会污染成功帧分布。
- 本measurement切片有意保留构造期raw write 1、build pass full clear 1，且新增native submit 0、poll 0。这样动态基线可以分别观察冷启动Copy、CPU plan/encode与GPU generation pass，不用先修改被测行为。
- exact 5个Rust文件的`rustfmt --check --edition 2021 --config skip_children=true`、scoped tracked diff check、untracked source whitespace与8-counter/4-scope/零新增循环提交的静态门禁通过。`profile.rs` 107行、`resources.rs` 208行、compiled tests 502行；existing compiled `render.rs` 901行，低于1000行hard stop，本切片只增加frame lifecycle两处窄接线。
- managed Cargo仍受已记录的共享dirty `Cargo.lock --locked`阻塞，本切片未重复运行。300帧profile、RenderDoc、PNG、GPU timing、allocation/RSS/VRAM与功耗仍pending；没有删除4 MiB上传、没有实现persistent slot/dirty region，也不宣称任何瓶颈已消失。
