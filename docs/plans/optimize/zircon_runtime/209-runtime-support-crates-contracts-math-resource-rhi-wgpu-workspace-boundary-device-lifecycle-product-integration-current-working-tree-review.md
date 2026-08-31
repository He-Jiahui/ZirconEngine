---
title: Runtime Support Crates、Contracts、Math、Resource、RHI、WGPU 与产品边界当前工作树复审
category: zircon_runtime
report_id: Runtime209
review_date: 2026-08-31
baseline_head: working-tree
observed_head: f31fd06f69fdaedb70a0a56fe6d0268de1af83a6
doc_type: current-working-tree-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
coordination_tracking: skipped_by_user_request
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/101-runtime-support-crates-contracts-math-resource-rhi-wgpu-workspace-boundary-device-lifecycle-product-integration-current-source-review.md
related_reports:
  - docs/plans/optimize/zircon_runtime/205-runtime-resource-lifecycle-load-ticket-cache-residency-generation-reload-cancellation-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/204-runtime-filesystem-resource-io-path-atomic-transaction-recovery-security-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
owner_plans:
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
zircon_scope:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/crates/zr_contracts
  - zircon_runtime/crates/zr_math
  - zircon_runtime/crates/zr_resource
  - zircon_runtime/crates/zr_rhi
  - zircon_runtime/crates/zr_rhi_wgpu
  - zircon_runtime/src/graphics
reference_scope:
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/DynamicRHI.h
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/RHIResources.h
  - dev/UnrealEngine/Engine/Source/Runtime/D3D12RHI/Private/D3D12Submission.h
  - dev/UnrealEngine/Engine/Source/Runtime/D3D12RHI/Private/D3D12Allocation.cpp
  - dev/godot/servers/rendering/rendering_device_driver.h
  - dev/godot/drivers/vulkan/rendering_device_driver_vulkan.h
  - dev/bevy/crates/bevy_asset/src/handle.rs
  - dev/bevy/crates/bevy_render/src/renderer/render_device.rs
  - dev/bevy/crates/bevy_render/src/gpu_readback.rs
  - dev/Fyrox/fyrox-core/src/pool/handle.rs
  - dev/Fyrox/fyrox-graphics/src/read_buffer.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourcePool.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceRegistry.cs
---

# Runtime Support Crates、Contracts、Math、Resource、RHI、WGPU 与产品边界当前工作树复审

## 1. 结论

Runtime153 识别的主问题没有被推翻。当前工作树确实新增了可保留的工程底座：五个 crate 的 304 个输入已全部进入 Git index；random checkpoint 增加 authority generation；Resource readiness 已有 immutable management/readiness generation、64 shard projection、显式栈 SCC 与 cycle fail-close；RHI handle 已包含 allocator namespace/device/generation/kind/slot generation；WGPU 已有 generation-local registry、last-use retirement、单提交 owner、fault terminalization、surface lifecycle 和有界 diagnostic readback。

但这些进展仍是 **source-present / integration-incomplete / qualification-pending**，不能写成工程闭环。当前 HEAD 只包含 304 个输入中的 80 个，295 个输入仍 dirty；`zircon_runtime` 生产源码直接使用 `zr_contracts`，根 workspace 与 runtime manifest 仍没有声明它。产品 graphics 仍保存裸 `wgpu::Device`，整个 graphics Rust 选择集还有大面积 native WGPU 对象与创建调用；公开的 transitional recorder 只能靠调用者保证 command buffer 与资源来自同一 generation；普通 neutral readback 仍是同步 `Vec<u8>` 且 WGPU production 固定 unavailable。

因此 Runtime153 的稳定账目本轮重判为：

| 级别 | Open | Partial | Closed |
|---|---:|---:|---:|
| P0 | 1 | 1 | 0 |
| P1 | 9 | 5 | 0 |
| P2 | 4 | 0 | 0 |

24 道资格门为 **15 Fail / 9 Partial / 0 Pass**。本报告不新增 canonical finding，只刷新 Runtime153 的 currentness，并把 native recorder provenance、ignored readiness RED、GPU 测试静默跳过和 source-string assertion 记为既有条目的收口阻断证据。

性能方面仍不能宣称优于 Unreal。没有 clean-clone build、统一普通读回、单 native owner、多队列 provider、device-loss recreate、相同场景/画质/硬件 trace 与产品 benchmark；静态源码规模、测试数量或 API 名称均不是性能证据。

## 2. 冻结范围与统计口径

Zircon 统计包含五个 crate 下全部 `.rs`、`.wgsl` 与 `.toml`。fingerprint 以 lower-case repo-relative path 和文件 SHA-256 组成 `path<TAB>hash`，按路径排序、以 LF 拼接后再次 SHA-256。`HEAD tracked` 表示干净检出可获得，`index tracked` 表示当前暂存候选包含，`dirty` 表示 index/工作树相对 HEAD 仍变化；三者不能混为“已提交”。

| 范围 | files / lines / nonempty / bytes / tests / ignored / unsafe | HEAD / index / dirty | fingerprint |
|---|---:|---:|---|
| `zr_contracts` | 15 / 942 / 823 / 28,720 / 10 / 0 / 0 | 1 / 15 / 15 | `4961e472e3e6293b5f4e1127ddfe5bda4b2d822ce6d1b740eeb8fb3985583f53` |
| `zr_math` | 14 / 1,488 / 1,306 / 44,264 / 24 / 0 / 0 | 0 / 14 / 14 | `5486cb7de1fc98399bcdb27ae6956f502e4554c57ee235cd619347cfcd963592` |
| `zr_resource` | 83 / 19,868 / 18,140 / 687,331 / 240 / 10 / 13 | 0 / 83 / 83 | `e672e4914159578727cab37436d73ced80f13702b79d609e33cef284cee876a6` |
| `zr_rhi` | 39 / 10,849 / 9,787 / 357,052 / 83 / 1 / 0 | 16 / 39 / 35 | `2a6a45ff28663288c265c8c92012e84a154d326807da3c7a769e930f54b4fa42` |
| `zr_rhi_wgpu` | 153 / 45,207 / 42,094 / 1,626,605 / 404 / 0 / 2 | 63 / 153 / 148 | `d1aa050925b44c3206826149256e82f7c7348fe586e019282df6ba0e53950980` |
| Zircon union | **304 / 78,354 / 72,150 / 2,743,972 / 761 / 11 / 15** | **80 / 304 / 295** | `be8bbb94fafd66f9c7cbb1e6208c6dc03f91601a7100c7de3d568227c05b16ba` |
| 13 个本地参考文件 | **13 / 16,178 / 13,676 / 600,107 / 26 / 1** | n/a | `40f439ba042b27b0be617f6a6932ea52a93e3cf4034da6e3f48004d4413f6410` |

测试标记是词法计数，不代表全部执行。`unsafe` 同样是词项计数，包含注释/测试上下文；它用于定位审查面，不是漏洞数量。

## 3. 逐文件扫描覆盖

| 子树 | 已扫描输入 | 重点复核内容 | 当前判断 |
|---|---:|---|---|
| `zr_contracts` | 15/15 | crate root、random algorithm/key/state/checkpoint、serde、assembly、全部测试 | 仍只拥有 random；checkpoint generation 收敛是真实进展，跨域 contract owner 未形成 |
| `zr_math` | 14/14 | convention、fallible math、numeric policy、render conversion、space/transform、全部测试 | checked/fallible 底座可保留；precision/narrowing/policy/space authority 未闭合 |
| `zr_resource` | 83/83 | registry、mutation、lease、manager、management/readiness projection、event stream、atomic/durable transaction、tests/profile | generation/SCC/journal 明显进展；全局 authority 写锁、统一 load owner、resync 与 durability receipt 未闭合 |
| `zr_rhi` | 39/39 | capabilities、descriptors、profile/identity、handles、submission、surface、readback、fault、upload/UI、tests | fail-close/generation 类型可保留；identity、descriptor admission、普通 readback、wait 与 provider extension 未闭合 |
| `zr_rhi_wgpu` | 153/153 | validation、legacy device、production device/registry/submission/surface/diagnostics、native bridge、UI surface、WGSL、tests | production owner 已存在；native product 双 owner、provenance、ordinary readback、recreate/qualification 未闭合 |
| workspace/runtime manifests 与 direct consumers | 全量定向扫描 | workspace members/deps、Runtime imports、graphics backend/system texture/UI/native creation | `zr_contracts` Cargo 断链；graphics 仍广泛持有/创建 native WGPU 对象 |

本轮对 `zircon_runtime/src/graphics/**/*.rs` 做的词法面扫描如下。它包含测试与 `include_str!`/源码断言，因此只能证明迁移表面积，不能当作纯 production 调用数：

| token | hits / files |
|---|---:|
| `wgpu::Device` | 702 / 320 |
| `wgpu::Queue` | 87 / 55 |
| `wgpu::Texture` | 2,188 / 283 |
| `wgpu::Buffer` | 1,153 / 234 |
| `wgpu::RenderPipeline` | 205 / 76 |
| `create_texture` / `create_buffer` | 160 / 70；223 / 137 |
| `create_render_pipeline` / `create_bind_group` | 59 / 51；247 / 115 |
| `create_sampler` / `create_shader_module` | 23 / 19；83 / 70 |

## 4. 当前真实所有权

```text
CoreRuntime / product graphics
  RenderBackend
    SystemTextureGenerationOwner -> native Texture/View/Sampler
    Arc<WgpuRenderDevice>         -> generation registry/submission/surface/diagnostics
    wgpu::Device                  -> product native creation escape surface
    wgpu::Queue                   -> test-only outer field

  WgpuUiSurfaceContext (Clone)
    cloned Instance/Adapter/Device/Queue
    Arc<WgpuRenderDevice>
    UI pipelines/caches/readback queue/native command recording

zr_rhi_wgpu::WgpuRenderDevice
  DeviceId + DeviceGeneration + immutable profile
  registry Mutex + submission owner + surface Mutex + diagnostics Mutex
  WgpuNativeRecorderLease -> borrowed Device + caller-qualified native command buffers

zr_resource::ResourceManager
  Arc<RwLock<ResourceAuthority>>
    registry + management projection + payloads + runtime slots + readiness projection
  commit serial Mutex + event publisher
```

目标不能只是把 native 对象藏进更大的 facade。唯一 generation owner 必须能够证明创建来源、代际、依赖、last-use ticket、延迟销毁、读回、device-loss terminalization 与重建；上层只能保留 neutral handle、immutable artifact/descriptor 和 generation-qualified receipt。

## 5. P0 currentness 重判

### Runtime153-P0-001：clean checkout 无法重现五 crate 当前能力 - Partial

相比 Runtime153，304 个输入现已全部进入 index，这是明确进展；但 HEAD 只包含 80 个，295 个仍 dirty。`zr_math` 与 `zr_resource` 在 HEAD 中仍是 0/14、0/83，`zr_contracts` 仅 1/15，production WGPU 大量文件仍只存在于 integration candidate。当前本机 source graph 不能由 `git checkout f31fd06...` 重现，CI、bisect、source archive、SBOM 与安全审计仍无稳定对象。

关闭条件：将 manifest/source/tests/lockfile 作为原子候选提交；clean clone 复算 304 输入 manifest/fingerprint；执行 workspace metadata、targeted check/test、feature matrix；禁止以 staged 完整替代 committed/reviewed/qualified。

### Runtime153-P0-002：`zr_contracts` 被生产消费但 Cargo 图不存在 - Open

根 `Cargo.toml:6-9,58-61` 只声明 `zr_math/zr_resource/zr_rhi/zr_rhi_wgpu`；`zircon_runtime/Cargo.toml:243-246` 也只有这四个 crate。与此同时，`core/framework/random/mod.rs`、`core/runtime/runtime.rs` 与 random authority/registry/service/stream 等生产文件直接导入 `zr_contracts::random`。

关闭条件：原子增加 workspace member、workspace dependency 与 runtime dependency；建立 contracts 不反向依赖 runtime/graphics/editor 的 DAG guard；以 random checkpoint v1 hard-cut/v2 golden、migration policy 和 clean Cargo evidence 证明 schema/service 同步。不能复制类型或增加临时 alias 来绕过依赖图。

## 6. P1 currentness 重判

| ID | 当前 | 本轮证据摘要 |
|---|---|---|
| Runtime153-P1-001 | Partial | outer queue 已 test-only，system texture 有 generation/ticket owner；但 outer Device、UI cloned Device/Queue、native recorder 与大量 native resources 仍绕过 neutral registry |
| Runtime153-P1-002 | Partial | diagnostic readback 已有 budget/batch/map/terminal/delivery；普通 `read_buffer/read_texture` ABI 仍同步且 production unavailable |
| Runtime153-P1-003 | Open | 默认 `wait_for_submission` 仍 `poll -> status -> timeout -> yield_now` |
| Runtime153-P1-004 | Partial | resource handle identity 已显著封印；DeviceId/Generation/Profile/SubmissionTicket 仍公开可造 |
| Runtime153-P1-005 | Open | neutral descriptor 仍 public fields + infallible builder，合法性延迟到 provider validator |
| Runtime153-P1-006 | Open | neutral `RayTracing` 仍存在，WGPU 只在创建时返回 not implemented |
| Runtime153-P1-007 | Open | generation/projection 增强，但 hot acquire/release 仍写锁整个 ResourceAuthority |
| Runtime153-P1-008 | Partial | dependency IDs/readiness/SCC 已存在；统一 load request/owner/budget/cancel/provider/artifact publication 仍缺失 |
| Runtime153-P1-009 | Partial | bounded/coalesced sequence stream 与 projection snapshot 已存在；snapshot+fence 不原子，lag 后无强制 resync/ack/resume |
| Runtime153-P1-010 | Open | Unix/Windows durability 机制不同，公开 API 仍无 typed durability level/evidence receipt |
| Runtime153-P1-011 | Open | `Real=f32`、`RenderScalar=f32`，narrowing receipt 继续是有限值上的恒等转换 |
| Runtime153-P1-012 | Open | 可自定义 checked policy 已有，但 `STRICT` 三阈值仍全 0，缺 domain policy registry |
| Runtime153-P1-013 | Open | `SpaceKind` 仍是动态 tag，缺 From/To transform、origin epoch 与 graph authority |
| Runtime153-P1-014 | Open | `zr_contracts/src/lib.rs` 仍只有 `pub mod random`，没有按域 feature/DAG hard cut |

### 6.1 RHI/WGPU 单 owner 与 native interop

`RenderBackend` 当前保存 `Arc<WgpuRenderDevice>` 和生产 `wgpu::Device`；outer `wgpu::Queue` 已改为 `#[cfg(test)]`，因此旧结论中“生产 outer queue 仍存”已过时。`SystemTextureGenerationOwner` 会核对 device/generation 并生成 upload ticket，也属于真实进展；但其 `SystemTextureResources` 仍直接拥有大量 `wgpu::Texture/View/Sampler`，产品调用继续取得 clone。

`WgpuNativeRecorderLease` 是公开 transitional escape hatch。它向任意 closure 借出 `&wgpu::Device`，并允许 `extend_recorded_command_buffers` 收纳任意 native command buffer；注释要求调用者保证同 generation、不得 clone/retained device，但类型系统无法证明该事实。packet 最终确实由 owner 分配 submission identity 和 enqueue，可是 closure 内创建的 native resource、依赖与 last-use 不会自动进入 neutral registry。`WgpuUiSurfaceContext` 又是 `Clone`，内部保存 cloned instance/adapter/device/queue 与 UI pipeline/cache/readback owner，形成第二 native resource ownership surface。

关闭 P1-001 需要硬切而不是继续增加源码断言：产品长期资源由 neutral handle/registry 创建；native interop lease 必须限制可创建对象、线程、作用域、generation 和依赖登记；不接受调用者自证来源的 command buffer；UI surface 只保留 surface/external-image 所需的最小 interop capability。

### 6.2 Readback、completion 与 identity

`zr_rhi/src/device/render_device.rs:291-297` 仍定义 `read_buffer/read_texture -> Result<Vec<u8>, RhiError>`；WGPU production `device.rs:714-735` 验证 handle 后返回 `ReadbackUnavailable`。另一方面，diagnostic service 已实现 buffer/texture/mip source、staging batch、map callback、completion order、budget/metrics/delivery，并由 enclosing device poll 驱动；UI `GpuReadbackQueue` 也有 budget/cancel/abort，production collection 不再自立第二个 poll owner。这只能把 P1-002 降为 Partial，不能证明 ordinary RHI、capture、streaming 与 Editor inspection 已统一。

`RenderDevice::wait_for_submission` 在 `render_device.rs:241-256` 仍主动循环并 `std::thread::yield_now()`。工程闭环需要 nonblocking query + executor/waker；任何 blocking adapter 必须显式命名、限制 off-thread 并由 thread-role guard 拒绝 render/game/UI 主线程。

RHI 资源 handle 已使用 allocator namespace/device/generation/kind/slot/slot-generation，并拒绝反序列化，这是正确方向。但 `DeviceId::new`、`DeviceGeneration::new`、`RenderDeviceProfile::new`、`SubmissionTicket::new` 均公开，普通调用者仍可伪造互相矛盾的 profile/ticket。最终必须由 DeviceAdmissionAuthority 独占分配，反序列化只能得到 candidate，不能得到 admitted identity。

### 6.3 Descriptor、capability 与 provider extension

`BufferDesc`、`TextureDesc`、`SamplerDesc`、`PipelineDesc` 仍暴露 public fields，builder 返回 `Self`。WGPU 的 `resource_validation`/`pipeline_validation`/command validation 已较完整，但 neutral structural validation 与 provider negotiation 没有形成 `Candidate -> Validated -> Compiled` 类型链，未来 D3D12/Vulkan/mock provider 可能产生不同错误语义。

`PipelineKind::RayTracing` 继续属于 neutral enum，WGPU registry 在创建时才返回“not implemented”。必须改为 provider-owned negotiated extension，只有选中的 provider 能发布版本化 acceleration structure/SBT/pipeline contract；enum 名称不能被上层当成能力承诺。

### 6.4 Resource authority、readiness、event 与 durability

`ResourceManager` 仍以单个 `Arc<RwLock<ResourceAuthority>>` 同时拥有 registry、management projection、payloads、runtime slots 与 readiness projection。`acquire` 写锁全局 authority、downcast payload、修改 runtime state 并刷新 readiness；lease drop 再次写锁且可能删除 payload。这意味着 immutable generation/64 shard projection 并未消除 hot lease churn 与 cold registry/reload 的锁竞争。

Readiness 当前使用显式 traversal stack 计算 SCC，并把 cycle 标为 Failed；这已修正旧递归算法的核心方向。但 `behavior_red.rs` 中“cycle fail-close”和“10,000 deep chain 不增长 native stack”两条关键测试仍 `#[ignore]`，其中注释甚至继续描述旧递归实现。应先解除 ignore、更新测试命名/注释并纳入 managed lane，才能把实现进展变成资格证据。

Resource record 已有 dependency IDs、management/readiness generation 与精确 snapshot pair，但没有覆盖 source/provider/artifact/CPU/dependency/GPU/reload/eviction 的唯一 `ResourceLoadAuthority`，也没有 generation-qualified LoadRequestId、principal/owner、priority/deadline、budget、cancel reason 和 terminal receipt。Runtime205 继续拥有完整产品 load lifecycle；本报告只要求 crate boundary 与并发模型为该 owner 提供唯一底座。

Event stream 有 bounded/coalesced journal、sequence/cursor 与 `Lagged(gap)`，`ResourceManager::projection_snapshot()` 能在一把 authority read lock 下取得 management/readiness pair；但 `subscribe()` 与 snapshot 不是一个原子 admission，receiver 也没有 snapshot fence、ack/resume token 或 lag 后强制 resnapshot。durable import/cook/recovery 事件必须进入独立 journal，不能由 in-process notification stream 代替。

Atomic I/O 在 Unix fsync directory/parent；Windows 使用 `MoveFileExW(...WRITE_THROUGH)`/`ReplaceFileW` 并 sync committed target；其他平台存在 no-op 分支。实现可能是平台上可行的最佳机制，但公开合同没有区分 AtomicVisibility、FileDataDurability、DirectoryEntryDurability 与 BestEffort，也没有 platform evidence receipt，所以 P1-010 保持 Open。

### 6.5 Math、space 与 contracts

`Real` 与 `RenderScalar` 仍都是 f32，`try_to_render_scalar` 的范围检查和 `absolute_error` 对有限值是恒等行为。若 runtime 决定固定 f32，应删除伪 narrowing 名称并只报告 finite/admission；若目标是 large-world/f64 simulation，应实现 origin-epoch-qualified f64 -> f32 conversion、误差/overflow receipt 和 stale-origin rejection。

`NumericPolicy::try_new` 能拒绝非有限/负阈值，是可保留进展；但 `STRICT` 三个 minimum 仍全是 0，Normal/deserialize 等路径继续默认使用它。应建立 simulation/render/import/editor 的命名 policy registry，并让 authoring/import/compiler 使用携带 source context 的 fallible API。

`Position3/Vector3/Normal3 + SpaceKind` 能拦截部分同 tag 错误，但调用者仍可贴错动态 tag 或绕过 wrapper 使用裸 glam。目标边界应是 typed `Transform<From, To>`，或稳定 space ID + validated graph，附 origin/convention/generation receipt。

`zr_contracts` 本轮只增强 random checkpoint generation，crate root 仍只有 `pub mod random`。不应为了填充 crate 批量搬 DTO；应按依赖 DAG 逐域 hard cut：schema/golden -> Runtime producer/consumer -> 删除旧 owner -> feature/DAG guard。

## 7. P2 currentness 重判

| ID | 当前 | 关闭方向 |
|---|---|---|
| Runtime153-P2-001 precision/depth/convention 固定 | Open | canonical world convention 与 render projection policy/receipt 分离；支持 reversed-Z/infinite far/large-world origin |
| Runtime153-P2-002 registry/diagnostic/surface 粗 mutex | Open | 先采 lock hold/wait、churn 与 record concurrency，再决定 shard/frame arena/deferred lane |
| Runtime153-P2-003 单物理队列无可替换 multi-queue owner | Open | provider-neutral graphics/compute/copy plan、timeline、ownership transfer 与 retirement |
| Runtime153-P2-004 hidden public assembly 信任面 | Open | 迁移后改 private/sealed/integration crate，禁止 workspace 任意 caller 绕过 facade |

WGPU `single_serialized_queue` 诚实声明 async compute/copy 为 false，应保留这种 truth。不能为了“看起来完整”伪造多队列；多队列能力必须由未来 D3D12/Vulkan provider 以真实 fence/timeline/ownership transfer 实现。

## 8. 新收口阻断证据，不新增 canonical finding

1. **Native provenance 只靠调用者承诺。** `WgpuNativeRecorderLease` 的 borrowed device 和 arbitrary command-buffer adoption 不能证明资源 generation/last-use，归入 P1-001/G05/G06。
2. **GPU 测试存在环境静默跳过。** 多个 helper 返回 `Option<(Device, Queue)>`，测试使用 `let Some(...) else { return; }`；缺 adapter/feature 时会绿灯退出。必须区分 Unsupported/Skipped/Executed，并让 mandatory CI adapter 缺失成为失败，归入 G23/G24。
3. **静态 source-string assertions 过重。** 五 crate 有 141 个 `include_str!` 与 443 个 `.contains("...")` 词法命中。它们可做 boundary guard，但不能代替类型/编译/运行时行为验证，归入 G02/G05/G24。
4. **关键 RED 仍 ignored。** 11 个 ignored 中包括 readiness cycle/deep-chain、durable I/O profile、management/readiness profile、release performance 与 UI compact style。当前实现可能已满足其中部分，但 ignored 不产生 closure evidence。

## 9. 可保留底座

1. Random 的 stable key、PCG state、single mutable lease、canonical checkpoint、authority-generation hard cut。
2. Math 的 finite check、fallible inverse/perspective、validated transform/unit direction 与 checked custom policy construction。
3. Resource 的 stable ID/locator、typed handle/lease、staged mutation、transaction journal/recovery、immutable management/readiness generation、64 shard projection 与显式栈 SCC。
4. RHI 的 fail-close operation matrix、generation-qualified handle/submission/surface、fault gate、bounded history、diagnostic readback state machine 与 truthful single-queue profile。
5. WGPU 的 context admission、generation-local registry、resource dependency/last-use retirement、single queue submission owner、surface terminalization、diagnostic readback budget/delivery。

这些底座应迁移和加固，不应推倒重写；但保留的前提是先进入可复现 Cargo/source 图，并删除平行 owner。

## 10. 本地参考实现差异

| 参考 | 本地证据中的工程边界 | Zircon 当前差异 | 采用方式 |
|---|---|---|---|
| Unreal RHI/D3D12RHI | DynamicRHI provider、RHI resource lifetime、logical queue/fence/submission thread、deferred deletion/allocation retirement | neutral owner 与 raw WGPU product path 并存，无 D3D12/Vulkan provider、多队列和统一 ordinary readback | 借鉴 owner/retirement 分层，不复制宏、全局单例和历史平台包袱 |
| Godot RenderingDeviceDriver/Vulkan | driver-owned IDs、resource/command/sync primitive、backend 显式实现 | neutral descriptor admission 与 provider compile 未分层，产品仍能创建 native WGPU | 借鉴 driver-owned identity，不牺牲 Zircon generation/lease contract |
| Bevy Asset Handle/RenderDevice/GPU Readback | typed strong/weak handle、drop event、device wrapper、map_async + channel 的异步读回 | Zircon ordinary RHI readback 同步且 unavailable，ResourceLease 不等于完整 load request | 复用异步完成和 typed handle 思路，薄 RenderDevice 不是多后端终点 |
| Fyrox pool Handle/read buffer | generational index stale reject、明确 async read-buffer object | Zircon handle 底座接近，但 profile/ticket constructor 与 native bypass 削弱 authority | 借鉴 stale rejection/future owner，不用规模差异推导性能 |
| Unity Graphics RenderGraph pool/registry | versioned resource handle、use-before-validation、transient/shared/import distinction、pool release/reuse/purge | Zircon registry 尚未成为 render graph transient/alias/pool 的唯一创建与生命周期路径 | 借鉴版本/池/瞬态生命周期，不机械复制 C# managed lifetime |

参考文件只能证明责任划分和可执行模式，不能证明 Zircon 已达到或超过其性能。最终判定必须来自 Zircon clean build、correctness oracle、trace、fault matrix 与同场景 benchmark。

## 11. 目标架构

```text
CoreRuntime (lifecycle owner)
  DeviceAdmissionAuthority
  RuntimeResourceService
  completion/executor/fault/shutdown authority

zr_contracts (pure DTO/traits, domain features, no runtime implementation)
  random / lifecycle / render interop / resource projection / ...

zr_math
  canonical simulation types
  + named numeric policies
  + typed transform graph/origin epoch
  + explicit render conversion receipt

zr_resource
  immutable registry generation
  + sharded hot residency/lease table
  + load/dependency/budget/cancel authority
  + snapshot-fence-delta projection
  + typed durability/artifact boundary

zr_rhi
  candidate -> validated neutral descriptor
  + unforgeable device generation
  + command/submission/readback/surface tickets
  + negotiated provider extensions/queue plan

zr_rhi_wgpu
  sole WGPU generation owner
  + registry/submission/readback/surface implementation
  + narrowly scoped, provenance-recorded native interop

graphics/render graph/UI/capture/editor
  neutral handles + immutable artifacts + generation-qualified receipts
  no stored native Device/Queue/Texture/Buffer/Pipeline owner
```

## 12. 依赖有序重构里程碑

### M0：source/Cargo 可复现性

- 原子提交五 crate manifest/source/tests/lockfile，生成受管 source manifest。
- 接入 `zr_contracts` workspace/runtime dependency 与反向依赖 guard。
- clean clone 复算 fingerprint，执行 metadata、targeted check/test 与 feature matrix。

### M1：identity 与 descriptor admission

- DeviceAdmissionAuthority 独占 DeviceId/Generation/Profile/Ticket 构造。
- descriptor 变为 Candidate -> Validated -> provider Compiled plan。
- wire 反序列化永远回到 candidate，不恢复 admitted identity。

### M2：RHI/WGPU 单 owner hard cut

- system texture/offscreen/surface blit/upload 长期资源迁入 neutral registry。
- 删除 `RenderBackend` 生产裸 Device，收缩 UI context。
- native lease 登记 generation/thread/capability/resource dependencies/last-use，删除 arbitrary adoption。

### M3：统一异步 readback/completion

- ordinary buffer/texture subresource request、layout/format receipt、copy ticket、map completion、cancel/timeout/budget/artifact。
- diagnostic/capture/Editor inspection 共用 completion owner。
- 删除同步 `Vec<u8>` ABI与主线程 yield wait。

### M4：Resource authority 拆分

- immutable registry generation、sharded residency、payload publication、mutation lane 分离。
- 统一 request/dependency/provider/artifact/reload/last-good/cancel/terminal receipt。
- snapshot+fence+delta 原子订阅；lag 强制 resnapshot；durable 事件进入 journal。

### M5：Math/space contract

- 明确 f32 或 large-world f64 policy，删除伪 narrowing 或实现真实 f64 -> f32。
- 建立 domain numeric policy 与 fallible production boundary。
- 建立 From/To transform、origin epoch、projection convention receipt。

### M6：Provider extension 与多队列

- RayTracing 改为 negotiated provider extension。
- provider-neutral graphics/compute/copy plan、timeline/fence、ownership transfer、cross-queue retirement。
- WGPU 保持 truthful single queue；D3D12/Vulkan 独立实现。

### M7：资格恢复

- 解除已满足的 ignored RED/profile gate；GPU test 显式报告 Executed/Unsupported/Skipped。
- device loss/recreate、stale handle、surface resize、memory pressure、readback cancel、shutdown/restart fault matrix。
- lock/queue/resource churn/VRAM/readback latency trace 与预算。

### M8：产品性能资格

- 相同场景、画质、硬件、驱动与 capture window 对比目标引擎。
- 记录 p50/p95/p99 frame、CPU submit、GPU time/bubble、lock wait、allocation、VRAM peak、readback/recovery time 和 dropped work。
- 原始 trace 与 correctness image/hash 同时归档，禁止只给汇总分数。

## 13. 资格门

| Gate | 当前 | 关闭条件 |
|---|---|---|
| G01 clean checkout 包含五 crate 当前能力 | Partial | index 已完整；必须提交并从 clean clone 重现 |
| G02 source manifest/fingerprint 受管可复现 | Fail | CI 独立复算与 source guard |
| G03 `zr_contracts` Cargo 图闭合 | Fail | workspace/runtime dependency + clean check |
| G04 contracts pure DTO/trait、无反向依赖 | Partial | random 方向正确；全域 feature/DAG guard 未闭合 |
| G05 WGPU generation 只有一个产品 owner | Fail | graphics/UI 无长期 native owner |
| G06 handle/submission/surface 全部 generation-qualified | Partial | neutral 底座已在；native bypass/provenance 必须清零 |
| G07 neutral descriptor 构造即合法 | Fail | candidate/validated/compiled 类型链 |
| G08 identity/profile/ticket 不可伪造 | Partial | resource handle 已封印；device/profile/ticket 仍公开 |
| G09 submission terminal history 有界可查 | Partial | 底座已有；产品故障/recreate/retire 闭环缺失 |
| G10 ordinary buffer/texture readback 异步 E2E | Fail | request/ticket/map/artifact/cancel 产品链 |
| G11 render/game/UI 主线程无 blocking/yield wait | Fail | executor/waker + thread-role guard |
| G12 queue topology 真实映射 provider | Partial | WGPU truth 已有；多队列 provider 未实现 |
| G13 resource load request 有 ID/owner/budget/cancel | Fail | RuntimeResourceService E2E |
| G14 dependency/currentness/reload 原子闭合 | Partial | readiness/DAG 有进展；load/reload publication 未统一 |
| G15 hot acquire 不竞争全局 authority 写锁 | Fail | shard/RCU + scale profile |
| G16 resource event snapshot/resync/ack | Partial | snapshot/sequence 有底座；原子订阅/resync/ack 缺失 |
| G17 durability level 跨平台显式并 fault-tested | Fail | typed receipt + crash matrix |
| G18 runtime/render precision 表达真实转换 | Fail | 删除 no-op receipt 或实现真实 narrowing |
| G19 near-degenerate policy 可配置并 fail-close | Fail | domain policy + adversarial tests |
| G20 space conversion 有 From/To/origin epoch | Fail | typed graph + stale reject |
| G21 RayTracing 仅由可执行 provider 宣告 | Fail | extension negotiation + executable provider |
| G22 mutex/queue hotspot 有受管 profile | Fail | lock/queue trace 与预算基线 |
| G23 cross-platform/device-loss 无 stale handle | Partial | fault terminalization 有进展；recreate/re-admission 与强制 GPU lane 缺失 |
| G24 同画质产品性能超过目标基线 | Fail | correctness + managed benchmark + raw trace |

## 14. 首个允许实施的切片

第一步仍只能是 M0，不能从 API 美化、锁拆分或新增 renderer feature 开始：

1. 冻结 304 输入及 root/runtime/random consumer 的准确 source manifest。
2. 原子接入 `zr_contracts` workspace/runtime dependency，禁止 duplicate/alias owner。
3. 把 current index candidate 变为可审计提交，在 clean checkout 复算 `be8bbb94...` 对应 source 图；若候选继续变化，则重新冻结，不沿用本报告指纹。
4. 执行 contracts/math/resource/rhi/rhi_wgpu targeted check/test 与 feature matrix，逐项记录 Executed/Skipped/Unsupported；此后才允许进入 M1。

本轮仅完成 review/index/coverage 文档，没有修改 Rust、Cargo、ABI、tests 或 UI，也没有运行 Cargo、GPU、device-loss、fault、scale、soak、跨平台 durability 或动态 benchmark。Tooling 按用户要求排除；未查询、轮询、等待或实时跟踪协调器。
