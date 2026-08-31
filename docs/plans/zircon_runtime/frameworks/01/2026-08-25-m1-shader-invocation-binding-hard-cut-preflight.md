---
plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
milestone: M1 Phase 1 - core/framework declaration/behavior partition
status: preflight_complete
date: 2026-08-25
related_code:
  - zircon_runtime/src/core/framework/render/shader/compute_dispatch.rs
  - zircon_runtime/src/core/framework/render/shader/fullscreen_pass.rs
  - zircon_runtime/src/core/framework/render/shader/material_property_layout.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/property_layout.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader_package.rs
  - zircon_runtime/src/graphics/shader/builtin_global_shader_contracts.rs
  - zircon_runtime/src/graphics/shader/fullscreen_pass_parameters.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/construct.rs
  - zircon_runtime/tests/fixtures/shader_invocation/compute_binding_probe
  - zircon_runtime/tests/fixtures/shader_invocation/fullscreen_binding_probe
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/ShaderParameterMetadata.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/Shader.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/ShaderParameterStruct.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphUtils.cpp
  - dev/bevy/crates/bevy_render/src/render_resource/bind_group_layout.rs
---

# M1 shader invocation binding hard-cut preflight

## 1. 完成状态

本记录完成 `core/framework/render/shader` 的 current-source 结构复核、算法复核、参考引擎路由、复杂度边界和原子迁移设计。状态是 `preflight_complete`，不是 `source_implemented`、`validation_green` 或 `accepted`。

Frameworks01 已完成 immutable-scope rotation，当前 session 为 `frameworks01-shader-invocation-hard-cut-r12-1b2684b4-20260825`。r12 覆盖结构化 grep 识别的 shader/asset/graphics consumer、新增 invocation owner，以及强化 old-path guard 发现的 Editor viewport root。Editor viewport mixed blob 已按 current hash 从 archived Editor05 attribution 整体 transfer，并把 render-root glob forwarding 收敛为 35 个实际使用的显式 contract；该独立 facade 清理没有移动 shader builder，也没有创建兼容导出。历史 13-file ownership-transfer 预检曾显示 12 个 dirty blob 可转交，唯一 blocker 是 active `mvp00-current-source-convergence-r2-01a00797-20260818` 归属的 `core/framework/render/mod.rs`，协调器返回 `source_owner_executable`。viewport hash 已因合法清理变化，历史 fingerprint 因而失效；剩余 shader 源码仍未修改，在 old-export 删除面与其余 11 个 dirty blob 能够同批合法转交前，Shader ABI 迁移保持 pending。

## 2. Current-source 基线

2026-08-25 对 `zircon_runtime/src/core/framework/render/shader/**/*.rs` 重新采集：

| 指标 | current source |
|---|---:|
| Rust 文件 | 21 |
| 行数 | 4,747 |
| bytes | 164,050 |
| manifest SHA-256 | `c83ab4fd8dc8f15bb95eda63c1b0946fc7a4d6dd08bbc1d868a64742c8293324` |

manifest 算法为按仓库相对路径排序，对每个文件记录 `path<TAB>bytes<TAB>lines<TAB>file-sha256<LF>`，再对 UTF-8 manifest 求 SHA-256。进入源码迁移前必须复算；任一 hash 漂移都使本记录的文件级 ownership 证据失效，但不改变下面的架构结论。

该目录不能整体搬入 `zr_contracts`。至少下列行为仍错误地与 DTO 混在 contract owner：

- `ComputeDispatchBuilder` / `FullscreenPassBuilder` 的 authoring、校验和 plan 编译；
- named resource 查找、kind/access 校验与 ABI binding 分配；
- fullscreen 参数布局推导、字节编码和 slot 查找；
- pipeline label/cache key 的编译期组装。

`zr_contracts` 只应保留 shader kind/stage/value kind、opaque asset reference、entry/resource/parameter layout descriptor、编译后不可变 invocation plan/receipt 和纯不变量方法。compiler、validator、packer、builder 与 WGPU projection 都随 `graphics::shader` implementation owner 迁移。

## 3. 结构性正确性缺陷

### 3.1 参数 ABI 由参数名字典序隐式决定

`FullscreenPassPlan.parameters` 当前是 `BTreeMap<String, ShaderParameterValue>`：

- `parameter_slot(name)` 返回 BTreeMap key 的字典序位置；
- `parameter_byte_len()` 固定为 `parameter_count * 16`；
- `write_parameter_bytes()` 按 values 的同一字典序把每个值编码成一个 16-byte vec4 slot；
- `graphics/shader/fullscreen_pass_parameters.rs` 只缓存 `(name, enum discriminant)` 并再次验证同一顺序。

这意味着重命名一个参数、加入一个排序更靠前的参数，或 shader 源声明顺序与字典序不同，都会改变已有字段的 byte offset。执行路径没有 shader 声明的 offset、alignment、packed size 或 layout hash，因而同一错误假设在 plan 和 uploader 两侧自洽，却不能证明与 WGSL uniform layout 一致。当前测试只证明双方共享同一个隐式排序，不证明 GPU ABI 正确。

### 3.2 资产已有参数声明，但 builder 没有消费

compute/fullscreen `.zshader` 已有 `properties: Vec<ShaderMaterialPropertyAsset>`，并带 name、kind、required、default；然而 builder 的 `build(...)` 只接收 shader kind、entry point descriptors 和 resource descriptors，不接收 property schema 或已编译 parameter layout。因此它无法诊断 missing/unknown/type mismatch，也无法按 shader 声明 offset 打包。

更严重的是 `import_shader_package.rs` 对 Surface、Compute、Fullscreen 都无条件调用 `generate_material_artifact(...)`。它把 compute/fullscreen properties 也投影为 `material_property_layout` 和 material group-2 WGSL，名称与 ABI 职责均不成立。现有 `MaterialPropertyLayout` 的显式 slot、packed size、layout hash 是可复用的设计模式，但该 DTO 同时携带 surface material texture bindings，不能直接冒充通用 invocation layout。

### 3.3 资源绑定没有单一编译权威

资源 numeric binding 当前按 `ShaderResourceDescriptor` 声明序临时分配；`ShaderAsset.pipeline_layout` 又是另一套 layout DTO，但 package import 默认为空，builder 也不消费它。`graphics/shader/builtin_global_shader_contracts.rs` 还手写 shader kind、entry 和 resource arrays，形成 asset declaration、framework builder 和 graphics/WGSL 三方可漂移权威。

单一权威不能只记录 group、binding、kind 和 access。当前 backend-neutral RHI binding layout 还要求 shader visibility；buffer 的 dynamic-offset 与 minimum binding size；texture 的 sample type、view dimension 与 multisample；storage texture 的 format；sampler 的 binding type。若 invocation metadata 或 layout hash 漏掉任一维度，graphics/WGPU executor 仍必须推断或硬编码 ABI，同样会产生双权威。

目标不是把字符串 map 换成更快的 map，而是把 authoring name resolution、类型校验、offset/binding 分配和 layout hash 归并为一次 compile。帧执行只能消费 numeric offsets/bindings 与已打包 bytes。

### 3.4 Current structured consumer 与 ownership 复核

2026-08-25 在 HEAD `1b2684b40ae3eba7abfcdfae3fe7e341b4906ec8` 上，以 builder/plan/resource/pipeline-layout 类型和 `ShaderAsset` construction/material regeneration 调用为根执行 5 组 `git grep`，初始去重得到 54 个 Rust consumer。强化 guard 随后识别 `zircon_editor/src/scene/viewport/mod.rs` 的旧 render-root glob forwarding，current scope 下限增至 55 个 Rust consumer，其中 13 个是当时的 current dirty blob。viewport 的显式 facade 已独立清理，old-owner guard 当前只剩 6 个显式 consumer；55-file 仍是完整迁移 scope 的保守基线，因为 viewport 继续消费 35 个显式 render contracts。该集合只是当前搜索根和 guard 可达的下限，不是完整 schema reference 清单，也不是只迁移两个 builder 文件的许可。正式实施仍须在类型字段变更后用 Rust 编译器和 old-path guard 找出新增 consumer。

| 分类 | current 结论 |
|---|---|
| framework contract/behavior | `compute_dispatch.rs`、`fullscreen_pass.rs` 混合 DTO、compiler、validator、packer；`pipeline_layout.rs` 只有 group/binding/coarse resource type/visibility，`resource.rs` 只有 name/kind/access；两者都不足以表达完整 invocation ABI |
| asset schema/cache/importer | `ShaderAsset` 同时保存 material artifact、粗粒度 `pipeline_layout` 和独立 `resources`；cache 原样持久化后仍无条件 `regenerate_material_artifact()`；package importer 对所有 shader kind 生成 material artifact，并把 `pipeline_layout` 设为空 |
| graphics compiler/executor | `global_pipeline_layout.rs` 运行时再按名字补 WGPU texture/storage-texture 类型；`fullscreen_pass_parameters.rs` 再按 `(name, enum discriminant)` 缓存布局；builtin contract、feature descriptor、HZB/IBL 各自重声明 schema |
| render graph/upward fixtures | `render_graph/{types.rs,tests/resources.rs}` 和全部 `ShaderAsset` literals/round-trip/readiness/product fixtures 必须同批迁移，否则新 schema 会被 default 或旧 constructor 掩盖 |

13 个 dirty blob 的归属风险进一步分为：

- 5 个 current hash 与 archived attribution 完全一致：`asset/assets/shader/{property_layout.rs,zshader.rs}`、`asset/importer/ingest/{import_shader.rs,import_shader_package.rs}` 属 Plugins07，`graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs` 属 Shader06；必须整 blob transfer，不得重写或只取 import hunk；
- `core/framework/render/mod.rs` 是 active `mvp00-current-source-convergence-r2-01a00797-20260818` 的 dirty mixed blob，是当前唯一 active-source ownership blocker；取得该 blob 的显式 transfer/rotation 前不得删除旧 re-export；
- `graphics/feature/render_feature_pass_descriptor/construct.rs`、`graphics/scene/render_product_material_property_tests.rs`、`render_graph/{types.rs,tests/resources.rs}` 的 current hash 已偏离 archived attribution；successor 必须按现 hash 重新取证，不能使用历史 fingerprint；
- `graphics/scene/render_product_streamer_tests.rs` 是未归属 dirty blob，必须先建立 current attribution；`asset/assets/mod.rs` 也已偏离 Frameworks05 历史 attribution，须由实际 asset integration owner 整 blob 接收。
- `zircon_editor/src/scene/viewport/mod.rs` 原先通过 `pub(crate) use zircon_runtime::core::framework::render::*` 把旧 builder surface 引入 Editor viewport。r12 已用 preview `797e5c52fe884744b2115079e13e9980` / fingerprint `5543d769aa30c0b9040b00d035f5b3ec13079036d1d3c5e308a4d17b48e50207` 和 apply `eb363ff1ac3e479faa0c79604830be50` 整 blob 接收，并保留原有 controller/pointer-resolution 变更；当前内容 SHA-256 为 `22c2d1289a65e49b1c7ee9480005d2d2f9c21511f8534d8416fded2e4b9bd661`。旧 glob/builder 零命中，35 个实际使用的 render contracts 显式列出，该路径不再是 Shader ABI ownership blocker。

同一 HEAD 上对仓库内全部 `.zshader` 文档复核后，原有 6 个样例均为 Surface shader；没有真实 Compute 或 Fullscreen `.zshader` 资产。r12 已新增两个真实 compound package：Compute 包含 `element_count/scale`、uniform buffer、read storage buffer、read-write storage buffer 和 `cs_main`；Fullscreen 包含 `tint/exposure`、sampled texture、sampler、uniform buffer 和 `fs_main`。结构化 fixture guard 用 TOML/UUID parser 锁定且只允许这 2 个 package、6 个 v4 UUID、每包 2 properties/3 resources/1 entry；同时逐个资源把 descriptor name/kind/access 与 WGSL name/group/binding/address-space/access/type 配对，锁定 compute/fragment stage、compute workgroup size、meta URL/kind/version/importer、included files，并按 WGSL host-shareable 对齐规则验证 Compute 16-byte 与 Fullscreen 32-byte uniform 字段 offset/size。复用 D 盘受管池 Naga rlib 的 E 盘一次性 validator 对两份 WGSL 均取得 `parse+validate GREEN`。该实现只补齐有效产品输入，不证明 asset import、artifact round-trip、compiled invocation 或 submission/readback 已闭环。

现成 D 盘受管池 `zircon_shader_prewarm.exe` 以这两个 package 为唯一 asset root 并启用 `--validate-wgpu-modules` 后，在 1.721 秒内退出 0，却报告 `source_count=0`、`requested_count=0`、`validated_count=0`、`failed_count=0`。代码级根因已经定位：`zircon_shader_prewarm/manifest/pass_types.rs::asset_scan_pass_types_for_zshader` 对所有非 Surface 文档直接返回空 `Vec<ShaderPassType>`；`manifest.rs:487` 计算该空列表，`manifest.rs:515` 把它传入 source，`manifest.rs:655` 又只遍历 `source.pass_types` 产生 source/request，因此两个 invocation package 被静默归零。后续组装还绑定 material layout/options、geometry source、shading model、surface template 与 `ShaderVariantKey`，所以不能把 Compute/Fullscreen 伪装成 Surface `ShaderPassType` 来绕过。正确 hard cut 是建立独立的 invocation prewarm manifest/key/receipt 路径，由 asset invocation layout 和 `graphics::shader::invocation` 的 compiled ABI 共同驱动，并与 Surface material prewarm 分流。报告保存在 `E:/Git/ZirconEngine/.codex/state/frameworks01-shader-invocation-fixture-report.json`，SHA-256 为 `adb3b35cc4c2d2548ee47482f354a4256685b4d5ba348041a9bddf1a74888cb2`；只有 WGPU module validation `requested_count=2`、`validated_count=2` 后才可通过该门。

历史 13-file transfer-preview request 为 `8f3f2b51d71142cc9754a82a313bccb3`，fingerprint 为 `cf07c705b21f80641c2148280ecbee44a619fcbc0b1426567b7038941b2fbe66`。它只记录当时 12 个 eligible、`core/framework/render/mod.rs` 一个 blocked 的证据，从未 apply。viewport 已由独立 facade 清理合法接收并产生新 hash，因此该 fingerprint 现已失效。MVP00 blocker 解除后，必须对剩余 12 个 dirty blob 重取 current hash 并重新 preview；只有 old-export 删除面和其余 11 个路径同时 eligible 才能原子 apply。

其余 clean 文件同样没有隐式写入许可：部分只有 stale archived attribution，部分完全未归属。r12 immutable scope 已包含 55-file lower-bound manifest、将新增的 `graphics/shader/invocation/**`、hard-cut guard、本记录和 Editor viewport root；任何 active owner 冲突仍使 Shader ABI 源码实现保持 pending。

## 4. 参考引擎结论

### Unreal Engine 主参考

- `ShaderParameterMetadata.h::FMember` 显式保存 `Offset`、`BaseType`、rows、columns、elements，并把这些字段纳入 layout hash；布局不是由名字排序推导。
- `Shader.h::FShaderParameterBindings` 保存 shader 编译后参数/资源 bindings；执行阶段消费 compiled bindings，而不是逐帧重新从名字发现 ABI。
- `ShaderParameterStruct.h::SetShaderParameters` 以 parameter metadata、compiled bindings 和参数内存为输入，清晰分开 declaration metadata、compile result 与 execution payload。
- `RenderGraphUtils.cpp` 对已按 byte offset 排序的 metadata/bindings 使用单调索引合并遍历，避免重复 name lookup，也使复杂度与布局大小线性相关。

Zircon 应采用相同责任切分，不照搬 UE 宏系统或 C++ 内存别名：资产/importer 生成稳定 metadata，graphics compiler 生成 immutable bindings，render graph executor 只消费编译结果。

### Bevy 次参考

Bevy 把 bind group layout、buffer、pipeline cache 等行为放在 `bevy_render::render_resource`，而不是 app/foundation contract 层。它支持本计划的 owner 结论：backend-neutral descriptor 可下沉，资源布局构建与 GPU 对象行为必须留在 graphics/render implementation。

## 5. 目标架构

目标数据流：

```text
.zshader properties/resources/entries
        |
        v
asset import: ShaderInvocationLayout metadata + layout_hash
        |
        v
graphics::shader::invocation compiler
  - resolve authoring names once
  - validate kind/stage/type/access/completeness
  - compile byte offsets and numeric bindings once
        |
        v
CompiledShaderInvocation
  - immutable parameter layout identity
  - packed parameter bytes
  - numeric resource bindings
  - pipeline identity
        |
        v
render graph / WGPU executor
  - O(payload bytes) upload
  - numeric binding projection
  - no alphabetical ABI and no runtime name discovery
```

建议的最终职责，不锁死 Rust 类型命名，但锁死语义：

| Owner | 保留/新增职责 | 禁止职责 |
|---|---|---|
| `core/framework/render/shader` -> `zr_contracts` | `ShaderValueKind`、parameter member descriptor（type/alignment/offset/size）、完整 resource binding descriptor（group/binding/kind/access/visibility，以及 buffer、texture、storage texture、sampler 的类型化 layout 维度）、entry invocation layout、layout hash/packed size、immutable compiled plan DTO | builder、validator、packer、name resolver、WGPU object creation |
| `asset/assets/shader` + importer | 解析 `.zshader` 声明；Surface 生成 material binding layout；Compute/Fullscreen 生成 invocation layout；持久化 artifact schema/version | 对所有 shader kind 无条件生成 material artifact |
| `graphics/shader/invocation` | compile builder、完整诊断、参数 packer、numeric resource binding compiler、layout cache | 向 `core/framework` 回推实现算法 |
| render graph / WGPU executor | 上传 packed bytes，按 numeric binding 建立/复用 layout 与 bind group | 逐帧 BTreeMap 排序、名字查找或重新验证 schema |

`MaterialPropertyLayout` 不得直接复用为通用布局。应拆出共享的 parameter-block primitive，再由 `MaterialBindingLayout` 组合 surface texture bindings，由 `ShaderInvocationLayout` 按 entry point 组合 compute/fullscreen resource bindings。一个 shader asset 可以有多个 entry invocation layout；每个 entry 的 layout hash 必须覆盖共享参数块 identity、该 entry 的 stage/entry identity 和完整 resource layout，而 asset-level aggregate identity 只能组合这些已排序的 entry hashes，不能再次推断 bindings。迁移同批删除旧 material-only 字段被 global shader 误用的路径，不留 type alias 或 forwarding field。

## 6. 原子 hard-cut 实施清单

### A. Contract metadata 与 importer 单一权威

1. 先写 RED tests，证明参数声明顺序与名字字典序不同时仍按显式 offset 编码；新增参数不移动已有显式 offset；missing/unknown/type mismatch/layout-hash mismatch 全部返回 typed diagnostic。资源测试还必须逐项覆盖 visibility、dynamic offset、minimum binding size、texture sample type/view dimension/multisample、storage format 和 sampler type mismatch。
2. 新增通用 parameter-block / invocation layout DTO，layout hash 必须覆盖 algorithm version、entry/stage、每个 member 的 type/alignment/offset/size，以及每个 resource 的 group/binding/kind/access/visibility 和其类型化 layout payload；不得把 backend-neutral RHI 已要求的字段留给 WGPU projection 猜测。
3. Surface 与 Compute/Fullscreen 分流生成 artifact；停止对 global shader 生成 material group-2 artifact。
4. artifact cache schema/version、ShaderAsset construction fixtures、import/readiness tests 同批迁移，禁止 serde/default 把缺失新布局静默当作合法 current artifact。

该波次至少覆盖 `asset/assets/shader/{zshader.rs,shader_asset.rs,property_layout.rs}`、`asset/importer/ingest/import_shader_package.rs`、shader artifact cache payload 及所有 `ShaderAsset` literal/current cache tests。正式 scope 由 current structured consumer inventory 生成，不能只使用本记录的示例路径。

### B. Builder/compiler 行为移入 graphics

1. 建立 folder-backed `graphics/shader/invocation/{mod.rs,compiler.rs,diagnostic.rs,parameter_packer.rs,resource_bindings.rs,tests/...}`，避免继续把职责堆入 684 行 `compute_dispatch.rs` 或 561 行 `fullscreen_pass.rs`。
2. 将 `ComputeDispatchBuilder`、`FullscreenPassBuilder`、shader entry/resource validation、parameter packing 与 pipeline identity compilation 移入该 owner。
3. builder 输入改为 immutable asset invocation layout（或其批准快照），不再传三份可互相矛盾的 `shader_kind/entry_points/resources` slices。
4. compile 输出包含 packed bytes、parameter layout hash 与 numeric resource bindings；executor 不保留 `BTreeMap<String, ShaderParameterValue>` 作为运行 ABI。

### C. Consumer 原子迁移与旧路径删除

当前 builder 生产/混合 consumer 至少包括：

- `graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs`；
- `graphics/feature/render_feature_pass_descriptor/construct.rs`（tests 与 production 同一 mixed blob）；
- `graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs`；
- `graphics/shader/builtin_global_shader_contracts.rs`；
- `graphics/shader/fullscreen_pass_parameters.rs`；
- `render_graph/tests/resources.rs`；
- `core/framework/render/{mod.rs,shader/mod.rs}` 的旧 re-export。

迁移必须在同一 candidate 中完成：改完 import 和 construction API，删除 framework builder exports 与 behavior implementation，添加 old-path count = 0 guard。不得保留 `pub use graphics::...Builder`、同名 wrapper、type alias、deprecated facade 或双写 plan。

`construct.rs`、Shader06 graphics blobs 和 Shader04 历史 builder blobs 必须先按 current hash 重新 preview ownership-transfer；旧 archived attribution 不是写入许可。若任一 mixed blob 有其它活动 owner，整 blob 转给真实 integration owner，不拆取 import hunk。

### D. 产品、性能与功耗验收

1. 静态门：old builder implementation/export 计数 0；global shader 的 material-layout 误用计数 0；invocation executor 的 parameter-name lookup/BTreeMap ABI 计数 0。
2. focused correctness：参数布局顺序/类型/缺失/未知/layout hash 矩阵，完整 resource binding layout/mismatch 矩阵（包括 visibility、buffer、texture、storage texture、sampler 维度），compute/fullscreen compiler tests，artifact round-trip。
3. upward build：受管 Windows `zircon_runtime` lib、`zircon_editor` lib、`zircon_app` production build；target 只允许 D/E/F。
4. product route：至少一个真实 compute 和一个 non-empty fullscreen 参数 pass 走 asset import -> compile -> render graph -> WGPU submission/readback，输出与迁移前基准一致。
5. profile：在正确产品路径上分别测首次 compile、cache hit compile、每帧 parameter update；报告样本数、warmup、P50/P95、CPU time、allocations、uploaded bytes 和 GPU timestamp。没有外接功耗计或 ETW/Windows Energy Estimation Engine 可复现实验时，不声明功耗接近其它引擎。

## 7. 复杂度与性能边界

当前 resource validation 同时构建声明名的 `BTreeSet`、查询请求 `BTreeMap`，并对请求反查声明集合；保守上界为 `O(E log E + E log R + R log E)`，可简写为 `O((E + R) log(E + R))`，其中 E 是声明资源数、R 是请求资源数。global shader 的 E/R 很小，尚无 profile 证明它是主要瓶颈。因此不批准对 BTreeMap 做孤立微优化，也不声明现有耗时或功耗改善。

结构性目标是：

- authoring compile 一次完成 name resolution，期望 `O(E + R + P)` 或带构建期 map 的 `O((E + R + P) log N)`，P 为参数数；
- cache hit 以 `(shader revision, entry, options, layout hash)` 直接取得 immutable bindings；
- 每帧参数更新只按预编译 offset 写入，复杂度 `O(changed payload bytes)`，全量上传为 `O(packed bytes)`；
- executor 不再按 name 排序/查找，不因参数改名改变未相关字段 offset。

只有实现完成并取得上述 profile 数据后，才能判断瓶颈是否消失。算法“最优规模”的验收是执行复杂度随 payload 线性增长且没有按名字重复发现 ABI，不是单次微基准更快。

## 8. Ownership 与下一状态

当前状态保持：

- `research_complete`: current module、asset schema、graphics uploader、Unreal 主参考和 Bevy 次参考已复核；
- `preflight_complete`: 目标 owner、原子波次、复杂度和验收门已锁定；
- `source_implemented`: false（仅指完整 Shader ABI hard cut）；
- `viewport_facade_cleanup`: source implemented，旧 render glob/builder 零命中，35 个实际 contract 显式导出；
- `product_fixture`: source implemented / structured guard GREEN / Naga parse+validate 2/2 GREEN；Compute 与 Fullscreen package 各 2 properties、3 resources、1 entry，prewarm/WGPU product coverage RED（0 sources / 0 validations）；
- `hard_cut_guard`: RED，`6` tests 中 `2` 个通过、`4` 个按预期失败，分别命中 framework 行为、两层旧导出、`6` 个显式旧 owner consumer 和缺失的 folder-backed graphics invocation owner；fixture contract 与 direct/grouped/prefix/chained alias、qualified path、glob、new-owner scanner mutation 自测通过；
- `hard_cut_guard_runtime`: Windows current checkout latest RED run 在 `18.391s` 完成，候选发现使用 builder-symbol/public-forwarding-filtered `git grep`，不做全仓 Rust 文件 materialization；
- `independent_review`: 第一轮 `C0/I2/M0`、第二轮 `C0/I1/M0`；prefix alias、recursive owner scan 与 multiline public-glob statement span 修正后 exact-two-file 复核为 `C0/I0/M0`；viewport mixed-blob/35-symbol curated surface/guard/状态记录的最终窄复核同为 `C0/I0/M0`；fixture 初审 `C0/I1/M0`，补齐资源声明配对、entry/meta 和 uniform layout guard 后终审为 `C0/I0/M2`，两个文档精度项（prewarm `487/515/655` 数据流、非性能测试时长约数）已修正；
- `cargo_validation`: Windows managed job `665de0e602c34826b6266d02d1babe5d` 执行 `validate-matrix.ps1 -Package zircon_editor -SkipTest -VerboseOutput`，约 34.47 秒后自然结束并 release，exit 1、live process 为空；它在编译到 Editor/viewport surface 前被 foreign current source 阻断。新增且未归属的 `zircon_runtime_host/src/viewport_surface.rs`（current hash `758e2dcd9101f8f648cda633296c71709ceee20f8bcd2dea60ff2cee00dc5e47`）于第 174、228 行报 2 个 E0506，原因是 `rollback(&mut self)` 通过 `self.lock_registry()` 持有借用自 `self` 的 `MutexGuard` 时写 `self.completed`。本轮没有修改该 foreign blob，也没有把 35-symbol surface 的编译闭包宣称为通过；
- `performance_claim`: none；
- `power_claim`: none；
- `commit/wecom`: none。

当前 r12 已从 r11 接收 66 个现存 dirty blob，为 3 个 camera-controller 删除 tombstone 恢复显式 lease/attribution，并完成 Editor viewport mixed blob 的独立 transfer 与显式 facade 清理。2026-08-25 最新 12-file transfer-preview request `cefad0f7135c4078a8ba2216b55bdac9` 再次得到 fingerprint `2e030adee42a846134e0aaf7885da24bc5cf04867ffd9a13091c8550015629a1`，显示 10 个 blob eligible，但两个原子输入仍由 executable source owner 持有：current `core/framework/render/mod.rs` hash `b8b5908e5a8c462a8c80081cabfe4c272bf1d51a65b7dd502c42b7f1b9fd7ce8` 属 `mvp00-current-source-convergence-r2-01a00797-20260818`，current `asset/assets/mod.rs` hash `76003c49e5f5159d8b9ee780f0ba41cae3d0eb46cfcc87641498fc9756b4f051` 属 `text01-font-artifact-service-20260825`。该 preview 未 apply。

2026-08-25 baseline epoch 436 的完整 contract-owner 复核又确认，解除上述两个 mixed consumer 并不足以开始 hard cut：`core/framework/render/shader/{asset_kind,definition_value,dependency,entry_point,material_property_layout,pipeline_layout,queue,render_state,resource,stage,variant_prewarm}.rs` 与 `variant_prewarm/budget.rs` 共 12 个 contract blob 仍为 `attribution_missing`；`compute_dispatch.rs` current hash `5779f98dda52eac00bd9dbe9d9d1656ae5fb2cc0606f1c1085276d92856b62ce` 仍指向 archived `shader04-global-executor-closeout-20260714` 且 current hash/baseline 均 stale。`core/framework/render/mod.rs`、`asset/assets/mod.rs`、这 13 个 shader contract/behavior blob、全部 mixed consumers、将新增的 graphics invocation owner、旧 export 删除面和 product fixture 必须形成同一 current-hash owner union；任一项仍为 executable foreign owner、archived stale attribution 或 attribution missing 时，源码迁移继续保持 pending。下一合法动作是由 MVP00、Text01、Shader04/Frameworks01 的真实 integration owner 完成终态或 scope rotation，再对完整 union 重取 current hash、lease、attribution 和 transfer preview；不得沿用历史 12-file fingerprint，不得只转移当前 eligible 子集，也不得用 compatibility facade 暂时通过编译。
