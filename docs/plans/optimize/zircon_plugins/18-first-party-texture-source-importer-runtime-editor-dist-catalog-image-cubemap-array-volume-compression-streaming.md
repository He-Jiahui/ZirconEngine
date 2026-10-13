---
title: First-Party Texture Source、Importer、Runtime、Editor、Dist、Catalog、Image、Cubemap、Array、Volume、Compression、Streaming 与 Product Integration 工程化差距
category: zircon_plugins
report_id: Plugins18
review_date: 2026-08-19
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_plugins/texture
  - zircon_plugins/texture/plugin.toml
  - zircon_plugins/texture/runtime/src
  - zircon_plugins/texture/editor/src
  - zircon_plugins/texture/dist/src/lib.rs
  - zircon_plugins/texture_importer
  - zircon_plugins/texture_importer/plugin.toml
  - zircon_plugins/texture_importer/runtime/src
  - zircon_plugins/texture_importer/runtime/src/container
  - zircon_plugins/texture_importer/runtime/src/mipgen
  - zircon_plugins/texture_importer/runtime/src/transcode
  - zircon_plugins/texture_importer/dist/src/lib.rs
  - zircon_plugins/asset_importers/texture
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_app/Cargo.toml
  - zircon_app/src/entry
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/src/asset/assets/texture
  - zircon_runtime/src/asset/importer/ingest/import_texture.rs
  - zircon_runtime/src/asset/importer/image_decode.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
tests:
  - zircon_plugins/texture/runtime/src/tests.rs
  - zircon_plugins/texture/editor/src/tests.rs
  - zircon_plugins/texture_importer/runtime/src/tests
  - zircon_plugins/texture_importer/runtime/src/container/tests
  - zircon_runtime/src/asset/assets/texture/descriptor/tests.rs
  - zircon_runtime/src/asset/assets/texture/upload_support/tests.rs
  - zircon_app/src/entry/tests/source_assertions.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_editor/35-texture-image-cubemap-render-target-sampler-compression-streaming-preview-authoring-review.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/Texture.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/TextureDerivedData.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/Texture2DStreamIn_IO.cpp
  - dev/UnrealEngine/Engine/Source/Editor/TextureEditor/Private/TextureEditorToolkit.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Tests/Texture2DTests.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceTexture.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Textures/Texture2DAtlas.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/ShaderLibrary/DebugMipmapStreaming.hlsl
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/RenderGraphTests.cs
  - dev/bevy/crates/bevy_image/src/image.rs
  - dev/bevy/crates/bevy_image/src/image_loader.rs
  - dev/bevy/crates/bevy_image/src/hdr_texture_loader.rs
  - dev/bevy/crates/bevy_image/src/exr_texture_loader.rs
  - dev/bevy/crates/bevy_render/src/texture/texture_cache.rs
  - dev/Fyrox/fyrox-texture/src/lib.rs
  - dev/godot/editor/import/resource_importer_texture.cpp
  - dev/godot/editor/import/resource_importer_layered_texture.cpp
  - dev/godot/tests/core/io/test_image.cpp
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 18 · First-Party Texture Source、Importer、Runtime、Editor、Dist、Catalog、Image、Cubemap、Array、Volume、Compression、Streaming 与 Product Integration 工程化差距

## 1. 结论

Zircon 的 Texture 不是完全空白。`texture_importer` 已有 DDS、KTX1、KTX2、ASTC 的结构解析、维度与 mip/layer 校验、部分 supercompression 解码、RGBA8 mip kernel、normal convention 和 BC5 编码；Runtime 侧已有 typed descriptor、array/cube/lightmap、container upload readiness 与一条独立的 RGBA32F Environment IBL 路径。这些实现应保留并纳入统一编译管线，而不是推倒重写。

但当前三个名为 Texture 的首方包、Runtime 内建 importer、provider catalog、App feature 与 profile 没有形成同一个产品。`zircon_plugins/texture` runtime manager 只统计宽高、mip 与 texel 数，editor 只注册 descriptor/drawer/view/template，且模板 `plugins://texture/editor/authoring.zui` 不存在；dist 只返回 metadata。它却在 stable manifest 中声明完整能力，并由 runtime profile 选择为可用 Texture 包。

真正包含导入算法的 `zircon_plugins/texture_importer` 虽被 builtin plugin catalog 列为 stable source package，却没有进入 `first_party_runtime_catalog` 的依赖、feature 或 provider dispatch；Runtime profile 也不选择它。App 的默认 `target-client` 与 `target-editor-host` 又没有启用能提供 `texture` 的 `base-runtime-plugins`。因此“profile 已选”“catalog 已列”“source 实现存在”“App 实际装配”是四套互相矛盾的事实。

当前可达的实际图片路径仍主要是 Runtime 内建低优先级 importer。无论 builtin 还是插件 image/PSD 路径，HDR/EXR 最终都调用 `to_rgba8()`；测试甚至把 2x2 HDR/EXR 的 16-byte RGBA8 结果视为正确。settings 可以把同一 RGBA8 payload 标为 `rgba16float`，DDS 测试也允许把 DXT1 container descriptor 改写为 `rgba16float` 而不转换字节。格式、payload、颜色空间、压缩、维度与上传布局因而不是同一不可伪造合同。

离线编译链也没有闭合。当前 tracked `mipgen/kernel.rs` 的 Kaiser 分支漏传 normalizer，同时函数体引用未定义的 `kaiser_normalizer`，fallback 又漏传参数，导致包本身存在确定性编译阻断。即便修到可编译，只有 RGBA8 D2/Cube mip 和 BC5 编码是真实算法；其余 compression target、runtime mip policy、BasisLZ 等多为 metadata/结构准入，没有平台 artifact、真实 codec、bulk mip/page、DDC key 或 generation install。

Cubemap/Array manifest 通过 `std::fs::read(parent.join(relative))` 直接读外部文件，旁路 source snapshot、VFS、dependency graph、root containment 与 symlink policy；外部图片再次 `to_rgba8()`。Cubemap 仅支持固定 vertical-cross 切片与局部负 Z 旋转，没有统一坐标系、face orientation、seam/filter 或 irradiance/specular cook 合同；Array 只有文件列表或竖向切行；Volume/D3 仅在 descriptor/container 层出现。

Editor35、Plugins07/08/06、Runtime04/09D 与 Plugins01 已拥有 HDR 数据破坏、当前编译失败、artifact 谎报、重复 owner/假产品、Texture 类型混装、streaming/residency 和 carrier/catalog 的最高优先级问题。本篇不重复累计 P0，登记 **0 项新增 P0、48 项 P1、12 项 P2**；本篇只拥有 Texture 从 source snapshot、recipe、canonical image、platform artifact、runtime generation、editor document、dist carrier 到默认产品可见结果的纵向闭环。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | 冻结事实 |
|---|---:|---|
| `texture` package | 16 / 614 / 20,925 / 7 | stable shell；runtime summary、editor descriptor、metadata-only dist |
| `texture_importer` package | 43 / 9,575 / 323,386 / 172 | 六类 importer、container parser、mipgen、BC5；当前 source 编译阻断 |
| 旧 `asset_importers/texture` | 7 / 521 / 18,346 / 4 | experimental descriptor-only duplicate；无 importer registration |
| runtime/editor provider catalogs | 10 / 1,489 / 53,302 / 16 | runtime 只路由 `texture`；主 importer 与 editor 均缺 provider |
| Runtime Texture contract 与 builtin ingest | 33 / 7,654 / 264,732 / 52 | RGBA8/container payload、typed upload readiness、builtin RGBA8 decode |
| App 组装与 source assertion | 4 / 535 / 19,289 / 6 | assertion 只禁止 App direct fanout，不证明 provider 被链接 |
| **选定物理范围合计** | **113 / 20,388 / 699,980 / 257** | 路径去重后的静态审查范围 |
| package fingerprint | `e8885212283d766a9cf350c7a65c49eb7405d298f24acd61c818a96da587d6b9` | unique path 小写并标准化 `/`，与各文件 SHA-256 组成 `path|digest`，按 path 排序、LF 连接、无末尾 LF 后 SHA-256 |

源 revision 为 `bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch 为 335。上述三个直接 package 路径在审查结束前保持 working tree clean；App、Runtime、catalog 与共享文档存在其他 Session 活动，因此标记 `source_recheck_required: true`。本文没有修改 production 或 tests。

### 2.2 证据等级

本轮逐文件读取三个 package 的 66 个文件，并追踪 provider catalog、App feature、Runtime profile、builtin importer、Texture asset/upload contract和既有测试；参考实现读取 Unreal、Unity Graphics、Bevy、Fyrox 与 Godot 的生产源码和测试。结论属于 E3 静态调用链与合同审查。

没有运行 Cargo、GPU、Editor、NativeDynamic、跨平台或像素测试。原因不是把静态阅读当作动态通过，而是当前 importer 已有可由源码直接证明的编译错误，且本阶段只授权 review。257 个 test attribute 是库存，不是通过数；不存在的模板、未链接 provider 与 metadata-only carrier 也不能由单包 unit test 证明为产品完成。

### 2.3 与既有报告的边界

- Editor35 拥有 Texture/Image/Cubemap/RenderTarget/Sampler/Compression/Streaming 的完整领域模型与 authoring P0；本篇拥有首方 package 纵向装配。
- Plugins07 拥有全体 importer 的 source/dependency/subasset/artifact/sandbox 共性；本篇细化 Texture 特有的格式、mip、cube/array、codec 与 runtime install。
- Plugins08 拥有九个 editor extension 的 command/document/toolkit 共性；本篇只追踪 Texture editor 到同一 artifact/runtime generation。
- Plugins06 拥有 39 包 catalog/profile/capability closure；本篇证明 Texture 的具体 profile-provider-App 断链。
- Runtime04/09D 拥有通用 asset/artifact/cache/residency；本篇定义 Texture artifact 的领域内容，但不得新建第二套 DDC 或 streaming owner。
- Plugins01 拥有 package、native ABI、carrier 与 lifecycle；本篇只要求 Texture 的 Source/Library/Native 形态产生同一可执行语义。

## 3. 应保留的真实基础

1. DDS/KTX1/KTX2/ASTC parser 已处理较多 header、mip/layer、face、offset、alignment 与 format mapping 约束，可迁入 canonical compiler front end。
2. KTX2 zlib/zstd 解码、DFD 解析与 BasisLZ 结构识别可作为受预算 parser 的起点，但不能把“结构可读”写成“runtime 可用”。
3. BC5 normal encode、green-channel flip 与 normal convention 是真实领域逻辑，应纳入 typed recipe 和质量测试。
4. RGBA8 box/Kaiser mip 框架具有可修复的算法骨架，适合作为 CPU reference，而非最终全格式实现。
5. Runtime upload readiness 对 block-compressed layout、mip byte range 和部分 container contract 的检查应前移并共享给 compiler/admission。
6. `TextureArrayAsset`、`TextureCubeAsset`、lightmap、sampler/settings 等 typed DTO 可保留，但必须拆除 string format 和 payload relabeling。
7. Environment IBL 的 RGBA32F、cube LUT 与 PMREM 路径证明仓库已有保留浮点辐射度的实现，不应另造一条有损 importer。

## 4. 参考引擎给出的最低约束

### 4.1 Unreal Engine

`TextureDerivedData.cpp` 将完整 build settings、明确的版本 GUID、普通/VT 变体、target format 与 source identity 纳入 derived-data key；`Texture2DStreamIn_IO.cpp` 按 mip bulk data 批量读取到目标内存，校验 byte size/pitch，追踪 in-flight 同步并支持取消、abandon、cache invalidation 与错误诊断。`TextureEditorToolkit.cpp` 提供通道、曝光、mip、layer、slice、volume/cube 视图和平台 mip size。`Texture2DTests.cpp` 覆盖 source mip lock、normal/virtual texture async compile cancel 及编辑生命周期。Zircon 的最低门不是复制 UE 类型名，而是具备同等 source/build key、可取消编译、分 mip artifact/stream、平台预览与故障证据。

### 4.2 Unity Graphics

RenderGraph Texture 使用 generation-scoped handle 和 typed descriptor 表达 size mode、slices、format、dimension、UAV、mip、MSAA、dynamic scale、clear/discard/memoryless；测试覆盖 create/release、invalid use、import、handle validity、first-use clear、pool cleanup 与 intra-frame aliasing。Texture atlas 有分配、更新、释放与 mip 管理，mip streaming debug shader 提供可视化。Zircon 不能让 sampled asset、render target、temporary graph texture 与 streaming resource 共用一个晚失败的 `TextureAsset` 字符串 descriptor。

### 4.3 Bevy、Fyrox 与 Godot

Bevy HDR/EXR loader 从同一 reader snapshot 产生 `Rgba32Float`，`Image` 把真实 typed format、extent、data、layers 和 sampler 绑定，并测试 byte/format error、layer、resize、array conversion 与 anisotropy。Fyrox 以 typed TextureKind/TexturePixelKind、filter/wrap/anisotropy/compression/mip/green flip/LOD import options 和 byte-count/mip offset 作为较低但完整的合同基线。Godot importer 暴露 lossless/lossy/VRAM/Basis、HQ/RDO、HDR、normal detection、mip limit、roughness limiter、channel remap、alpha border/premultiply、normal Y flip、size limit 与 platform variant；layered importer 区分 array/cubemap/3D，Image tests 验证 resize/mip/compression 与 float formats。Zircon 当前连 HDR preservation、recipe truth 与 layered owner 分离都未达到这些最低线。

## 5. P0 路由，不重复登记

| 既有最高优先级事实 | Canonical owner | Plugins18 处理方式 |
|---|---|---|
| HDR/EXR 被量化为 RGBA8 | Editor35 P0-1、Plugins07 | 不新增 P0；P1 固定 typed canonical image 与 HDR gate |
| 当前 mip kernel 编译阻断与 lock drift | Plugins07 P0-1、Editor35 P0-2 | 不新增 P0；M0 先恢复可达性，保留失败证据 |
| compression/mip metadata 与 artifact 不一致 | Editor35 P0-3、Runtime04 | 不新增 P0；禁止 descriptor relabeling |
| Texture/duplicate importer 对外宣称不存在或不可执行产品 | Editor35 P0-4、Plugins06/07/08 | 不新增 P0；建立单 owner 和 product composition gate |
| 一个 Texture identity 混合 sampled/cube/array/source/render target/SVT | Editor35 P0-5、Runtime09D | 不新增 P0；使用 typed resource classes 与 generation install |

## 6. P1 工程化差距

### 6.1 Package、maturity、catalog、carrier 与产品组合

| ID | 当前证据 | 必须重构为 |
|---|---|---|
| TEX-P1-01 | `texture` stable manifest 的 runtime 只有 summary manager | capability 只能由真实 compiler/runtime consumer 和动态证据发布 |
| TEX-P1-02 | editor 引用不存在的 `authoring.zui` | 真实 Texture document/toolkit/preview 资源，缺资源则入口 unavailable |
| TEX-P1-03 | `texture` dist 只有 descriptor/registration metadata | 可执行、可协商、可卸载且与 source 语义等价的 carrier，或删除 NativeDynamic 声明 |
| TEX-P1-04 | `texture_importer` dist 同样不执行 importer | importer bridge 必须传 source snapshot、recipe、artifact/diagnostic receipt |
| TEX-P1-05 | 旧 `asset_importers/texture` 重复 image/container/PSD 描述但无实现 | 选择一个 canonical package owner，硬切删除重复 manifest、ID 与测试 |
| TEX-P1-06 | builtin catalog 列出主 importer，provider catalog 却无依赖/feature/dispatch | manifest 到 provider 的生成式闭环，缺 linked provider 时 selection fail-closed |
| TEX-P1-07 | runtime profile 选 `texture`，却不选 `texture_importer` | profile 声明 domain capability 和 required provider set，不按包名猜能力 |
| TEX-P1-08 | 默认 App target 不启用 `base-runtime-plugins` | Client/Editor/Dev 的 BuildSet 测试证明每个 selected package 有 linked provider |
| TEX-P1-09 | editor catalog 对 Texture 为 0 | Editor provider 必须显式装配并验证 runtime/compiler dependency 可达 |
| TEX-P1-10 | source assertion 只证明 App 不直连 package | 增加从 preset 到 catalog/provider/registration/runtime-ready 的端到端 composition test |

### 6.2 Source、decode、payload 与 metadata truth

| ID | 当前证据 | 必须重构为 |
|---|---|---|
| TEX-P1-11 | builtin 与插件各自实现 image decode，安装插件会改变结果 | 一个 canonical decode/compiler implementation，builtin 只委托或提供同语义 fallback |
| TEX-P1-12 | HDR/EXR 调用 `to_rgba8()` | 保留 source precision/radiance，输出 RGBA16F/RGBA32F 或明确的受控 tone-map recipe |
| TEX-P1-13 | PSD 最终也只产 RGBA8 flatten | 记录 bit depth、color profile、alpha、layer policy；不支持项明确拒绝 |
| TEX-P1-14 | `TexturePayload` 仅 `Rgba8`/opaque `Container` | typed storage enum 覆盖 integer/float/block-compressed/planar 与 canonical layout |
| TEX-P1-15 | descriptor format 为 string | 稳定、可版本化的 pixel/transfer/color/alpha/channel semantic 类型 |
| TEX-P1-16 | settings 可把 RGBA8 payload 标作 `rgba16float` | format 由 compiler 输出字节决定，recipe 只能请求转换，不能重命名 |
| TEX-P1-17 | DDS test 允许 DXT1 payload 与 `rgba16float` descriptor 并存 | artifact manifest 必须校验 codec/block layout/mip ranges 与 descriptor 完全一致 |
| TEX-P1-18 | width/height/depth/layers/face/mip 分散校验 | 单一 `CanonicalImageLayout` 在 decode 后立即验证 overflow、cardinality 与 byte ranges |
| TEX-P1-19 | color space、ICC/transfer、alpha mode 与 normal/data texture 语义不足 | recipe 显式记录 source interpretation、working space、output transfer 与 swizzle |
| TEX-P1-20 | import result 缺 source digest、decoder/version、recipe digest 与 dependency set | 产生不可变 `TextureBuildRequest` 和可审计 provenance |

### 6.3 Container、Cubemap、Array、Volume、Mip、Compression 与安全

| ID | 当前证据 | 必须重构为 |
|---|---|---|
| TEX-P1-21 | KTX2 解压上限来自攻击者声明的 expected length | engine-owned per-source/per-level/aggregate byte、ratio、time 与 allocation budget |
| TEX-P1-22 | BasisLZ 可结构通过但无 transcode backend | capability admission 绑定真实 transcoder 与目标 GPU format；否则明确 unsupported |
| TEX-P1-23 | unknown/typeless DDS 可过结构解析 | parser、semantic qualification 与 upload admission 分层且 fail-closed |
| TEX-P1-24 | BC5 container path 未证明实际 codec 为 BC5 | container format 与 requested target 必须相等或执行真实 transcode |
| TEX-P1-25 | 只有 BC5 是真实 encoder，其他 compression target 多为 metadata | 建立 codec backend registry、quality preset、determinism/version 与 platform support matrix |
| TEX-P1-26 | mipgen 当前不可编译，修复后也仅 RGBA8 D2/Cube | typed CPU reference + production backend，覆盖 float、sRGB linearization、alpha coverage 与 D3 |
| TEX-P1-27 | Kaiser/box 没有 image semantic policy | normal renormalization、roughness variance、mask/alpha coverage、height/data filter 分离 |
| TEX-P1-28 | runtime mip policy 只改 descriptor count | mip 必须是 artifact 中有 digest/range 的真实 bytes，不得声明下游会生成 |
| TEX-P1-29 | `.zcube/.zarray` 直接 `std::fs::read(parent.join(...))` | 所有 source 通过 VFS/source broker，记录依赖并验证 root、symlink、scheme 与 budget |
| TEX-P1-30 | Cubemap 固定 cross offset/局部旋转，face size 无效时默认 | 显式 projection/handedness/face orientation/seam policy，非法输入分类失败 |
| TEX-P1-31 | Array 只支持文件列表或纵向切行，外部图像 RGBA8 化 | typed layer source、同构验证、float preservation、stable layer identity 与增量 rebuild |
| TEX-P1-32 | D3/Volume 只停在 descriptor/container | volume source、3D mip filter、brick/chunk artifact、upload/stream 与 editor slice/volume preview |

### 6.4 Artifact、Runtime install、streaming 与 consumer

| ID | 当前证据 | 必须重构为 |
|---|---|---|
| TEX-P1-33 | 没有 target/platform/device capability 参与 cook key | source+recipe+compiler+codec+target+quality+schema 构成 deterministic key |
| TEX-P1-34 | 没有 canonical intermediate 与 platform variant manifest | source decode、semantic transform 与 platform compression 分层，可复用中间产物 |
| TEX-P1-35 | 无原子 publication、last-good 或失败保留 | staging、digest verify、atomic publish、generation receipt、rollback/GC |
| TEX-P1-36 | Runtime asset 常持有 whole payload | manifest 与 bulk mip/layer/page 分离，metadata admission 不读取全部字节 |
| TEX-P1-37 | 没有 tail-first mip residency 与 I/O owner | 统一 Residency Manager 按 view demand、budget、priority、deadline 调度 semantic chunks |
| TEX-P1-38 | 无 async generation/cancel/fence/retirement | generation-qualified request/install/cancel，GPU completion 后才能回收旧资源 |
| TEX-P1-39 | compressed container readiness 与真实 device support 分离 | 安装前协商 adapter format、fallback transcode、row/block alignment 和 memory budget |
| TEX-P1-40 | summary manager 不连接 material/render graph/scene consumer | typed TextureHandle/TextureView/Sampler 与 sampled/storage/render-target 使用点形成可追踪产品链 |

### 6.5 Editor、operations、测试与资格

| ID | 当前证据 | 必须重构为 |
|---|---|---|
| TEX-P1-41 | Editor 只有 descriptor/view/template registration | Texture document 持有 source、recipe、revision、dirty/save/reimport/conflict 与 preview generation |
| TEX-P1-42 | 无通道、曝光、mip/layer/slice/cube/volume/platform 预览 | 用实际 platform artifact 和 runtime sampler 渲染，不用 CPU summary 冒充 preview |
| TEX-P1-43 | 无 import option schema/migration | typed recipe UI、版本迁移、unknown/future policy、copy/preset 与 deterministic serialization |
| TEX-P1-44 | 无 async build operation 与真实 progress/cancel | operation factory 返回 phase/progress/diagnostic/artifact/install terminal receipt |
| TEX-P1-45 | reimport/external dependency 没有冲突语义 | expected source/document revision、dependency diff、staging、atomic apply 与 last-good |
| TEX-P1-46 | 测试主要是 parser/registration/descriptor smoke，且接受有损 HDR/format mismatch | 删除错误 golden，增加 radiance、layout、codec、cube seam、mip quality 与 corruption corpus |
| TEX-P1-47 | 无默认 App、Editor、NativeDynamic、跨平台、GPU 像素证据 | BuildSet-bound source/native parity、rendered golden、device matrix、fault/soak/scale lane |
| TEX-P1-48 | stable/partial/complete 状态没有证据绑定 | maturity 由 gate receipt 自动投影；当前未闭合项降级为 experimental/unavailable |

## 7. P2 长期能力

| ID | 长期方向 | 前置条件 |
|---|---|---|
| TEX-P2-01 | Sparse/Virtual Texture page cook、feedback 与 residency | P1 artifact、streaming、GPU lifetime 全部闭合 |
| TEX-P2-02 | GPU accelerated mip、transcode 与 compression | CPU reference、determinism 和跨 vendor quality oracle |
| TEX-P2-03 | DirectStorage/GDeflate 或平台直接 I/O/decode | semantic bulk、budget、cancel/fence 与 fallback |
| TEX-P2-04 | ASTC/BC6H/BC7/ETC/UASTC 等完整 codec portfolio | target matrix、许可证、安全 parser 与质量基准 |
| TEX-P2-05 | Perceptual/semantic adaptive compression | 可复现 corpus、metric、artifact key 与可解释 recipe |
| TEX-P2-06 | Neural texture compression | 传统 codec 基线、model provenance、跨设备 fallback 与帧预算 |
| TEX-P2-07 | Distributed/remote texture build 与 shared DDC | immutable BuildRequest、trusted artifact、tenant/security 与 eviction |
| TEX-P2-08 | Large volume/medical/scientific brick streaming | typed volume、3D mip、sparse residency 与 precision contract |
| TEX-P2-09 | UDIM/tiled material texture sets | stable tile identity、dependency graph、material binding 与 partial rebuild |
| TEX-P2-10 | Runtime adaptive quality、memory pressure 与 multi-device migration | unified residency、telemetry、generation retirement 与 deterministic fallback |
| TEX-P2-11 | Automated platform visual comparison、delta heatmap 与 regression triage | rendered golden、color-managed capture 和 evidence archive |
| TEX-P2-12 | Texture authoring graph、batch processing 与 recipe marketplace | transaction/compiler/security/package trust 和 schema migration |

## 8. 目标架构

### 8.1 唯一产品链

```text
SourceBroker/VFS immutable snapshot
  -> TextureSourceDecoder
  -> CanonicalImage { typed format, extent, layers/faces, color/alpha semantic }
  -> TextureImportRecipe { version, transform, mip, codec, target policy }
  -> TextureBuildRequest { source/dependency/compiler/recipe/target digests }
  -> TextureCompiler graph
  -> TextureArtifactManifest + immutable bulk mip/layer/page blobs
  -> atomic Artifact Store publication
  -> generation-qualified Runtime install / Residency Manager
  -> TextureView + Sampler consumed by Render Graph / Material / Scene
  -> Editor document, preview, reimport and diagnostics read the same generation
```

`texture`、`texture_importer` 与 Runtime builtin 不得继续拥有三套 decode/registration truth。实施时先根据 crate boundary 选择一个 canonical compiler owner；其余入口只能委托。旧 `asset_importers/texture` 必须硬删除，不保留 compatibility manifest、re-export 或重复 plugin id。

### 8.2 必须固定的 typed contract

```text
TextureSourceSnapshot
  source_id, content_digest, byte_length, dependency_snapshots[], trust/budget

CanonicalImage
  dimension, extent, layer_count, face_layout, mip_count
  pixel_format, transfer_function, color_primaries, alpha_mode, channel_semantics
  validated planes/subresources with checked byte ranges

TextureImportRecipe
  schema_version, source_interpretation, transforms
  mip_policy, compression_policy, platform_overrides, quality/error budget

TextureArtifactManifest
  build_request_digest, compiler_version, target/device class
  actual format/dimension/layout, subresource digests/ranges
  dependencies, diagnostics, determinism and quality receipts

TextureGeneration
  asset_id, artifact_digest, generation, device_id
  residency state, in-flight requests, last-use fence, retirement state
```

Sampled image、cube、array、volume、render target、graph transient、streaming/virtual texture 可以共享底层 pixel/layout schema，但不能共享一个允许任意组合的 late-validation identity。Render target 和 transient graph texture 由 RHI/Render Graph owner 创建；importer 只产可持久 sampled artifact。

### 8.3 Package 与 carrier 约束

- manifest、provider catalog、feature preset 与 App target 由一个机器可验证的 package graph 投影。
- Source/Library/Native 共享 registration schema、capability、recipe、artifact 与 diagnostic receipt。
- Native carrier 不能把 Rust object 或未限定 allocator 的 buffer 直接跨 ABI；也不能只返回 metadata 后声称业务已加载。
- maturity 是 BuildSet-bound gate 结果，不是人工写在 manifest 的常量。

## 9. 实施里程碑

| Milestone | 内容 | 退出条件 |
|---|---|---|
| M0 · Truth cutover | 保留 mip compile failure；降级假 stable；选 canonical owner；删除重复 package；修 preset/catalog/App provider closure | G01-G04、G25、G31 |
| M1 · Typed source/canonical image | SourceBroker、budget、typed pixel/color/layout、HDR/EXR/PSD/container admission | G05-G08、G26 |
| M2 · Deterministic compiler | recipe/version、semantic mip、codec backend、cube/array/volume、platform artifact/DDC key | G09-G15 |
| M3 · Runtime generation/residency | artifact manifest、tail-first bulk、async request/cancel/install/fence/retirement、device support | G16-G20 |
| M4 · Editor product | document/operation/reimport/conflict、channel/mip/layer/cube/volume/platform preview | G21-G24 |
| M5 · Carrier/product parity | Source/Native parity、default Client/Editor/Dev composition、cross-platform/device matrix | G27-G28、G31 |
| M6 · Qualification | corruption/fuzz、quality oracle、rendered golden、scale/perf/fault/rollback/release evidence | G29-G30、G32 |

M0 不允许只修函数参数让 Cargo 变绿后继续保留错误产品声明；M1-M3 不允许先做 Texture Editor 外观；M6 的性能比较必须在同 source、recipe、画质、平台、设备、memory budget、warmup、采样与失败策略下进行。

## 10. 资格门

| Gate | 必须证明的事实 |
|---|---|
| G01 | `texture`、canonical importer、Runtime consumer 和所有 selected feature 在 required BuildSet 可编译 |
| G02 | preset 中每个 Texture selection 都解析到唯一 linked provider，missing/duplicate 直接失败 |
| G03 | Source/Library/Native 产生等价 registration、artifact 与 failure receipt |
| G04 | 旧 duplicate importer、plugin ID、manifest 与调用点已硬删除 |
| G05 | HDR/EXR roundtrip 保留 radiance/precision，误差上限由 golden corpus 验证 |
| G06 | descriptor format、payload bytes、subresource layout 与 upload format 不可矛盾 |
| G07 | image/container parser 经 byte/item/depth/ratio/time/allocation budget 与 fuzz/corruption corpus |
| G08 | 所有外部 source 通过 broker/VFS，dependency/root/symlink/scheme 被记录和验证 |
| G09 | Cubemap projection、handedness、face orientation、seam 与 filter 有 reference golden |
| G10 | Array/layer identity、同构、float precision、partial rebuild 与 upload 正确 |
| G11 | D2/Cube/D3 mip 对 color/normal/roughness/mask/alpha coverage 的质量与 determinism 合格 |
| G12 | 每个 advertised codec 都产真实匹配 bytes；unsupported target fail-closed |
| G13 | platform/device variants 有完整 capability matrix 与 deterministic selection |
| G14 | 相同 BuildRequest 产相同 artifact digest；任一 source/recipe/compiler/target 变化会失效 |
| G15 | build/publish 是 staging+verify+atomic commit；失败保留 last-good 且可 GC |
| G16 | Runtime 只安装已验证 artifact manifest，并发布 generation-qualified handle |
| G17 | metadata/tail mip 可先驻留，high mip/layer/page 按 demand/budget 异步读取 |
| G18 | compressed texture streaming 保持 block/range/alignment 正确，不整资产解压或复制 |
| G19 | request/build/upload 支持 cancel、timeout、device loss、stale generation 与 fence retirement |
| G20 | adapter 不支持目标格式时执行已声明 fallback 或拒绝，不在 draw 时晚失败 |
| G21 | Editor admission 绑定真实 provider/compiler/runtime generation，缺一则 unavailable |
| G22 | document/save/reimport/apply 有 revision、transaction、dirty、undo/redo 与 terminal receipt |
| G23 | 通道、曝光、mip/layer/slice/cube/volume/platform preview 渲染真实 artifact |
| G24 | source/dependency change 与并发编辑产生 classified conflict、atomic apply 和 last-good |
| G25 | manifest maturity/capability 由同 BuildSet gate receipt 投影，无人工假 complete |
| G26 | parser/build/install diagnostic 含 source、recipe、subresource、codec、budget 与 causal chain |
| G27 | Windows/Linux/macOS 与支持的 graphics backend 通过同语义 corpus |
| G28 | linked source 与 NativeDynamic 的成功、错误、取消、卸载后行为一致 |
| G29 | 大图、长 mip chain、array/cube/volume、并发导入与 residency 在明示内存/时间预算内 |
| G30 | artifact corruption、publication crash、device loss、rollback 与 release promotion 有可恢复证据 |
| G31 | 默认 Client、Editor Host、Dev profile 均有 preset-to-visible-textured-frame 产品测试 |
| G32 | 代表性 LDR/HDR/normal/data/cube/array/volume/container 在真实 renderer 有颜色管理的像素 golden |

## 11. 禁止的临时修法

- 禁止只给 Kaiser 调用补一个参数就把 Texture Importer 恢复为 stable。
- 禁止继续通过 settings 字符串改写 format，而不转换和验证 payload。
- 禁止把 HDR/EXR tone-map/quantize 到 RGBA8 后仍称为 HDR import。
- 禁止为每个 codec、cube、array 再复制一套 filesystem reader、budget 或 dependency graph。
- 禁止保留旧 `asset_importers/texture` 作为兼容 facade、re-export 或 manifest alias。
- 禁止把 parser success、descriptor registration、manager summary、空 editor view 或 metadata dist 当作 capability complete。
- 禁止在 Editor 先做完整面板，再让它调用不存在的 compiler/runtime generation。
- 禁止用同步 whole-asset read、永久 clone 或每帧全 mip upload 冒充 streaming。
- 禁止新增 Texture 私有 DDC、task pool、residency manager、VFS 或 GPU lifetime owner。
- 禁止通过降低分辨率、禁用 HDR、减少 mip、固定单平台或关闭验证来获得性能数字。

## 12. 本轮输出边界

本篇完成 Texture 首方 package 与产品链的首轮 E3 静态审查，未实施 production 重构。当前结论是：parser 与局部算法有保留价值，但编译可达性、格式真实性、source dependency、artifact/cook、runtime residency、Editor authoring、carrier parity 与默认产品装配均未达到工程级 Texture 系统；在 G01-G32 全部形成 BuildSet-bound 证据前，不得宣称该能力 complete，也不得用当前测试数量推导表现或性能达到、超过 Unreal。
