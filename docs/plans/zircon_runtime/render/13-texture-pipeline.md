---
related_code:
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/mod.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer.rs
  - zircon_runtime/src/core/framework/render/material/texture_slot_summary.rs
  - zircon_runtime/src/core/framework/render/image.rs
  - zircon_plugins/rendering/plugin.toml
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VT/VirtualTextureSystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VT/VirtualTextureFeedback.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VT/VirtualTexturePhysicalSpace.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/MipGen/MipGenerator.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Textures/RTHandleSystem.cs
plan_sources:
  - .codex/plans/ZirconEngine 资产、Texture、模型、ZShaderZMaterialZMesh 缺口补齐计划.md
  - .codex/plans/Asset Importer 插件化补齐计划.md
---

# 计划 13:纹理体系(mipmap / normal map / 数组与立方体 / 稀疏纹理 / 色彩空间)

## 目标

把纹理从"能上传能采样"补成完整资产管线:

1. mipmap:导入期离线生成为主(含 kaiser/box 过滤选择、法线图专用过滤),运行时 compute 生成兜底(RT/捕获类);mip bias 与各向异性采样设置进质量档。
2. 色彩空间元数据权威化:每张纹理声明 sRGB/linear,导入器据用途默认(albedo=sRGB,normal/mask/HDR=linear),运行时格式选择强制遵守 —— 与计划 07 的"linear 全链"互为表里。
3. normal map 管线:压缩格式(BC5 双通道 + Z 重建)、Y 翻转约定(GL/DX 风格声明)、导入期由高度图生成可选。
4. `Texture2DArray` 与 `Cubemap` 一等资产:数组切片导入(图集页/地形 splat/decal 集),cubemap 六面/equirect 导入(计划 11 消费)。
5. 稀疏/虚拟纹理(SVT):页表 + 物理页池 + 反馈驱动加载的最小可用版,feedback pass 写入需求页,streaming 按预算加载;作为可选 feature(capability + profile gate)。
6. 压缩纹理(BC1-7,KTX2 容器承载)与 transcode 路径定稿(衔接既有 texture importer 插件的 KTX 工作)。

## 现状与差距

- `gpu_texture` 上传与 `texture_slot_summary` 就绪追踪可用,KTX 容器在 texture importer 插件有基础;但 mip 全靠源数据自带、无生成;sRGB/linear 靠格式后缀猜测无元数据契约;无 texture array/cubemap 资产语义;无 SVT;normal map 无压缩与重建约定。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/Graphics/.../universal/Runtime/MipGen/MipGenerator.cs` | 运行时 mip 生成 compute 的组织(downsample 链、深度金字塔同型实现 —— 与计划 04 HZB 共享套路) |
| `dev/UnrealEngine/.../Renderer/Private/VT/VirtualTextureSystem.cpp` | SVT 总控:页请求 → 分配 → 加载 → 页表更新的帧循环 |
| `dev/UnrealEngine/.../VT/VirtualTextureFeedback.cpp` | feedback buffer:着色端写页需求、降采样回读的低带宽设计 |
| `dev/UnrealEngine/.../VT/VirtualTexturePhysicalSpace.cpp` | 物理页池与 LRU 驱逐 |
| `dev/Graphics/.../core/Runtime/Textures/RTHandleSystem.cs` | RT 类纹理的统一句柄与缩放管理(与计划 01 池、计划 07 动态分辨率呼应) |

次参考:`dev/learn-wgpu-zh`(wgpu 纹理上传/mip 写入的基础范式);`dev/bevy/crates/bevy_image`(导入元数据与 `is_srgb` 的资产表达)。

## 目标架构

归属:纹理元数据契约进 `core/framework/render/image.rs` 扩展;mip 生成与 SVT 运行时在 `graphics/scene/resources/`;导入侧改动走 texture importer 插件(零散导入逻辑不进 runtime)。

核心设计:

- `TextureMetadata`:color_space、mip 策略(FromSource | GenerateOffline | GenerateRuntime | None)、normal 约定(None | TangentSpaceDX | TangentSpaceGL)、address/filter/aniso 默认、用途 hint(Albedo/Normal/Mask/HDR/UI);导入器写入 zmeta,运行时强制校验(用途与格式矛盾时诊断)。
- mip 生成:导入期离线(图像库,kaiser;normal 逐 mip 重归一化);运行时 `MipGenPass`(compute downsample 链)服务 RT/探针捕获,与 HZB builder 共享 reduce 框架。
- `Texture2DArrayAsset` / `CubemapAsset`:切片来源列表 + 一致性校验(尺寸/格式);GPU 侧一次性建数组纹理;材质槽位类型系统(`texture_slot_summary`)增加 array/cube 维度。
- SVT(最小版):间接页表纹理 + 物理页 atlas;feedback pass(1/8 分辨率 UAV 写页 id)→ 异步回读 → streamer 按预算加载/驱逐;采样 WGSL include(页表跳转 + 边缘 pad);feature gate,关闭时资产退化为普通流式纹理(最高常驻 mip 链截断)。
- BC 压缩:导入期 transcode(KTX2/BasisU 路径优先),能力检测选择 BC7/BC5/回落未压缩。

## 里程碑

### TX-M1 元数据与色彩空间权威化

实施切片:
1. `TextureMetadata` 契约 + 导入器默认规则 + 运行时校验;现有资产批量迁移(导入器重跑)。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime image --locked` 与 texture importer 插件测试
- 验收证据:albedo 误标 linear 时导入诊断;sRGB 采样产物对拍(计划 07 线性链断言复用)。

### TX-M2 mip 生成与 normal 管线

实施切片:
1. 离线 mip(含 normal 重归一化)+ 运行时 MipGenPass;aniso/bias 接质量档。
2. BC5 normal + Z 重建 WGSL include;Y 约定处理。

测试阶段:
- `cargo test -p zircon_runtime gpu_texture --locked`(mip 链完整性 readback;normal 重建单位长度断言)
- 验收证据:远景纹理无闪烁(抓帧对比);BC5 法线光照与未压缩对拍误差阈值内。

### TX-M3 array 与 cubemap 资产

实施切片:
1. 两资产类型 + 导入(六面/equirect→cube;切片列表→array)+ 槽位类型扩展;计划 11/15 的消费接口就位。

测试阶段:
- `cargo test -p zircon_runtime image --locked`(一致性校验、切片寻址)
- 验收证据:cubemap skybox 用例(与计划 11 EL-M1 共享);array 采样测试图正确分片。

### TX-M4 稀疏虚拟纹理(可选 feature)

实施切片:
1. 页表/物理池/feedback/streamer 闭环;采样 include;gate 与回落。

测试阶段:
- `cargo test -p zircon_runtime svt --locked`(页请求→加载→命中循环的集成测试;驱逐正确性)
- 验收证据:巨幅纹理(>显存)场景按视角加载页(stats:常驻页数远小于总页数);关闭 feature 回落正常。

## 工程落地细化

本章是计划 13 的实施权威(见 index.md §8 第 7 条)。bind group 槽位、storage buffer std430 布局、`zr_` WGSL include、测试命名(`render_<topic>_*` / `render_product_*`)等全局约定直接引用 index.md §8,本章不重复定义。跨计划契约原样消费:计划 01 的 `RgTextureHandle`/`TransientResourcePool`(SVT 物理 atlas 经 `mark_persistent` 标记为持久资源);计划 16 的 `ComputePassDescriptor`/`GpuReadbackQueue`(SVT feedback 回读唯一通道,禁止任何 executor 私自 `map_async`);计划 11 的 `SkyboxSettings` 消费本计划的 `CubemapAsset`(IBL 预滤波归 11,本计划只负责资产格式与导入)。

### 模块与文件落点

现状基础:契约层已有 `core/framework/render/image/`(`RenderImageColorSpace`/`RenderImageDescriptor`/`RenderImageDimension`/`RenderImageUsage`/sampler 族);资产层已有 `asset/assets/texture/`(`TextureAssetDescriptor::with_import_settings`、`TexturePayload::Container`、`upload_support` 的 KTX/DDS/ASTC 就绪判定、`TextureArrayLayout`);GPU 层已有 `gpu_texture/GpuTextureResource` 与 `resource_streamer_ensure_texture.rs`;导入器插件 `zircon_plugins/texture_importer` 已有 `import_image`/`import_psd`/`import_texture_container` 与 `container/ktx/{ktx1.rs,ktx2.rs,ktx2/dfd.rs}`。本计划的增量是:元数据权威化与校验、mip 离线/运行期生成、array/cube 资产、SVT 最小闭环、KTX2 transcode 接缝。

新增文件(runtime 侧):

| 路径 | 职责(一行) |
|------|------------|
| `zircon_runtime/src/core/framework/render/image/metadata.rs` | `TextureMetadata` 及枚举族(契约层,无 wgpu,serde 进 zmeta) |
| `zircon_runtime/src/core/framework/render/image/metadata_validation.rs` | 校验规则表 R1–R11 的实现与 `TextureMetadataDiagnostic` |
| `zircon_runtime/src/core/framework/render/image/svt.rs` | `SvtSettings`/`SvtStats` 纯数据契约(预算、页尺寸、常驻统计) |
| `zircon_runtime/src/asset/assets/texture/array_asset.rs` | `Texture2DArrayAsset`:切片来源列表 + 一致性校验 |
| `zircon_runtime/src/asset/assets/texture/cube_asset.rs` | `CubemapAsset` + `CubemapSourceLayout`(六面/十字/equirect) |
| `zircon_runtime/src/graphics/scene/scene_renderer/mip_gen/mod.rs` | wiring:MipGenPass executor 模块声明 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mip_gen/executor.rs` | `MipGenPassExecutor`:一次 dispatch 写 4 级 mip 的 compute 执行器 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mip_gen/shaders/mip_downsample.wgsl` | 降采样 kernel(box/加权,sRGB 感知开关) |
| `zircon_runtime/src/graphics/scene/resources/svt/mod.rs` | wiring:SVT 子系统模块声明 |
| `zircon_runtime/src/graphics/scene/resources/svt/page_table.rs` | 页表纹理(`rgba8uint`)维护与祖先映射传播 |
| `zircon_runtime/src/graphics/scene/resources/svt/physical_atlas.rs` | 物理页 atlas(持久资源)与页槽位分配 |
| `zircon_runtime/src/graphics/scene/resources/svt/page_pool.rs` | `SvtPagePool`:LRU 空闲堆 + 锁定页(mip tail)管理 |
| `zircon_runtime/src/graphics/scene/resources/svt/feedback_analysis.rs` | 回读数据去重 → `SvtPageRequest` 集合(CPU 侧) |
| `zircon_runtime/src/graphics/scene/resources/svt/svt_streamer.rs` | 页加载预算调度、上传应用、驱逐决策 |
| `zircon_runtime/src/graphics/scene/scene_renderer/svt_feedback/mod.rs` | wiring:feedback pass executor |
| `zircon_runtime/src/graphics/scene/scene_renderer/svt_feedback/executor.rs` | 1/8 分辨率栅格写请求 buffer 的 pass 执行器 |
| `zircon_runtime/src/graphics/scene/scene_renderer/svt_feedback/shaders/svt_feedback.wgsl` | feedback 写入 shader(打包页请求) |
| `zircon_runtime/src/graphics/shader/includes/zr_normal.wgsl` | BC5 Z 重建与 Y 约定函数(计划 08 模板拼接消费) |
| `zircon_runtime/src/graphics/shader/includes/zr_svt.wgsl` | 页表跳转采样函数(indirection → 物理 atlas UV) |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/svt_feedback.rs` | SVT feature descriptor(gate 关闭时 compiled graph 无 pass) |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/mip_gen.rs` | 运行期 mip 生成 feature descriptor(RT/捕获类触发) |

修改文件(runtime 侧):

| 路径 | 改动点 |
|------|--------|
| `zircon_runtime/src/core/framework/render/image/mod.rs` | 仅 wiring:`metadata`/`metadata_validation`/`svt` 模块声明与导出 |
| `zircon_runtime/src/core/framework/render/image/color_space.rs` | 删除 `RenderImageColorSpace::Unknown` 变体(硬切换,见帧时序节删除项) |
| `zircon_runtime/src/core/framework/render/image/descriptor.rs` | `RenderImageDescriptor` 增 `metadata: TextureMetadata` 字段 |
| `zircon_runtime/src/asset/assets/texture/descriptor.rs` | `TextureAssetDescriptor` 增 `metadata` 字段;`with_import_settings` 解析 `usage_hint`/`mip_policy`/`normal_convention`/`compression`/`svt` 键;`normalized()` 调用规则表校验 |
| `zircon_runtime/src/asset/assets/texture/mod.rs`、`texture_asset.rs` | wiring + `TextureAsset::with_metadata`;array/cube 资产构造入口 |
| `zircon_runtime/src/asset/assets/texture/upload_support/compressed.rs` | BC4/BC5/BC6H 块尺寸与 family 映射补全(`TextureUploadCompressionFamily` 扩展) |
| `zircon_runtime/src/asset/importer/ingest/import_texture.rs` | ingest 时执行规则表:错误 → 导入失败诊断,警告 → 导入报告 |
| `zircon_runtime/src/asset/load/texture.rs` | `.zarray`/`.zcube` 装载分支 → `Texture2DArrayAsset`/`CubemapAsset` |
| `zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs` | array/cube 的 view dimension(`D2Array`/`Cube`)与逐层逐 mip 上传 |
| `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer.rs` | 增 `svt_streamer: Option<SvtStreamer>` 字段;fallback 纹理选择改按 `usage_hint`(normal → `fallback_normal_texture`) |
| `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_texture.rs` | SVT 资产分流:gate 开 → 页式驻留,gate 关 → 截断 mip 链走普通路径 |
| `zircon_runtime/src/core/framework/render/material/texture_slot_summary.rs` | `RenderMaterialTextureSlotState` 增 `dimension` 字段(D2/D2Array/Cube/D3 维度匹配诊断) |
| `zircon_runtime/src/asset/assets/material/texture_slot.rs` | `MaterialTextureSlotValue` 增 `expected_dimension`,zshader 槽位声明驱动 |
| `zircon_runtime/src/core/framework/render/backend_types.rs` | `RenderStats` 增 `svt_resident_pages`/`svt_pending_requests`/`mipgen_dispatches` 字段 |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/mod.rs` | 仅 wiring:两个新 descriptor 导出 |

新增/修改文件(texture importer 插件侧,`zircon_plugins/texture_importer/`):

| 路径 | 新增/修改 | 职责/改动点 |
|------|----------|------------|
| `runtime/src/mipgen/mod.rs`、`runtime/src/mipgen/kernel.rs` | 新增 | 离线 mip 链生成:kaiser/box kernel、sRGB 感知降采样、normal 逐 mip 重归一化 |
| `runtime/src/container/ktx/ktx2/transcode.rs` | 新增 | BasisU(ETC1S/UASTC)→ BC 目标族 transcode,藏在 cargo feature `basis-transcode` 后 |
| `runtime/src/cubemap.rs`、`runtime/src/array.rs` | 新增 | `.zcube`/`.zarray` 清单导入 + 十字图切分 + equirect 投影六面 |
| `runtime/src/importers.rs` | 修改 | `import_image` 按 `usage_hint` 写 `TextureMetadata` 默认并调用离线 mipgen;`import_texture_container` 接 transcode 分支 |
| `runtime/src/registration.rs` | 修改 | 注册 cubemap/array 导入器入口 |
| `plugin.toml` | 修改 | 新增 `[[asset_importers]]` 条目 `texture_importer.cubemap`(ext: `zcube`)与 `texture_importer.array`(ext: `zarray`);capability `runtime.asset.importer.texture.transcode` |

### 核心类型与接口

层归属:`TextureMetadata`/`SvtSettings` 是契约层纯数据(`core::framework::render::image`,无 wgpu,可进 zmeta);`SvtStreamer`/`PhysicalAtlas`/`MipGenPassExecutor` 携带 wgpu 类型,固定在 `graphics/**` 实现层;离线 mip kernel 与 transcode 只存在于 importer 插件,runtime 不新增图像处理/transcoder 依赖。

```rust
// ---- core/framework/render/image/metadata.rs(契约层)----
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureMipPolicy {
    /// 容器自带 mip 链,原样上传
    #[default]
    FromSource,
    /// 导入期由 importer 插件离线生成(kaiser/box)
    GenerateOffline,
    /// 运行期 MipGenPass 生成(RT/捕获类)
    GenerateRuntime,
    /// 单 mip,永不生成
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureNormalConvention {
    #[default]
    None,
    /// DirectX 风格(green 向下,采样端翻转 Y)
    TangentSpaceDx,
    /// OpenGL 风格(green 向上,直通)
    TangentSpaceGl,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureUsageHint {
    #[default]
    Albedo,
    Normal,
    Mask,
    Data,
    Hdr,
    Ui,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureCompressionTarget {
    /// 按 usage_hint 推导:Albedo→Bc7、Normal→Bc5、Mask/Data→Bc4/Bc7、Hdr→Bc6h、Ui→Uncompressed
    #[default]
    Auto,
    Uncompressed,
    Bc1,
    Bc4,
    Bc5,
    Bc6h,
    Bc7,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureMetadata {
    pub color_space: RenderImageColorSpace, // Srgb | Linear | Hdr(Unknown 变体删除)
    pub usage_hint: TextureUsageHint,
    pub mip_policy: TextureMipPolicy,
    pub normal_convention: TextureNormalConvention,
    pub compression: TextureCompressionTarget,
    /// 质量档可再偏移;采样器创建时叠加
    pub mip_bias: f32,
    /// 1 = 关闭;质量档上限裁剪
    pub max_anisotropy: u8,
    /// Some(_) 即声明为 SVT 资产;feature gate 关闭时退化为截断 mip 链
    pub svt: Option<SvtSettings>,
}

// ---- core/framework/render/image/svt.rs(契约层)----
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SvtSettings {
    /// 页 payload 像素边长,固定 128(对齐 UE tile 默认)
    pub page_size: u32,
    /// 滤波边沿 pad,固定 4
    pub border_size: u32,
    /// 常驻锁定的粗 mip 起点(mip tail,永不驱逐)
    pub mip_tail_first_level: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SvtStats {
    pub resident_pages: u32,
    pub total_virtual_pages: u32,
    pub pending_requests: u32,
    pub uploads_this_frame: u32,
    pub evictions_this_frame: u32,
}
```

导入校验规则表(`metadata_validation.rs` 实现;ingest 时错误中断导入、警告进导入报告):

| # | 条件 | 级别 | 诊断文案(格式串) |
|---|------|------|------------------|
| R1 | `usage_hint == Normal && color_space == Srgb` | 错误 | `normal map must use linear color space: '{uri}' declares color_space=srgb` |
| R2 | `usage_hint == Albedo && color_space == Linear` | 警告 | `albedo texture '{uri}' declares linear; expected srgb unless intentional` |
| R3 | `usage_hint == Hdr` 且 format 非 float 族(`rgba16float`/`rg11b10ufloat`/`bc6h`) | 错误 | `hdr texture '{uri}' requires a float format, got '{format}'` |
| R4 | `usage_hint == Normal && compression ∉ {Auto, Uncompressed, Bc5}` | 警告 | `normal map '{uri}' should compress as bc5, got '{compression}'` |
| R5 | `compression == Bc6h && color_space == Srgb` | 错误 | `bc6h has no srgb variant: '{uri}'` |
| R6 | `normal_convention != None && usage_hint != Normal` | 错误 | `normal_convention is only valid for usage_hint=normal: '{uri}'` |
| R7 | `mip_policy == None && sampler.mipmap_filter == Linear` | 警告 | `'{uri}' samples with trilinear filter but declares mip_policy=none` |
| R8 | `svt.is_some() && mip_policy == None` | 错误 | `svt texture '{uri}' requires a full mip chain for its mip tail` |
| R9 | `usage_hint == Ui && mip_policy != None` | 警告 | `ui texture '{uri}' rarely needs mips; consider mip_policy=none` |
| R10 | 容器自带完整 mip 链且 `mip_policy == GenerateOffline` | 警告 | `'{uri}' already contains {n} mips; falling back to from_source` |
| R11 | `color_space == Srgb` 且 format 无 `_srgb` 变体(`rg8unorm`/`bc4`/`bc5` 等) | 错误 | `format '{format}' has no srgb variant: '{uri}'` |

```rust
// ---- asset/assets/texture/array_asset.rs / cube_asset.rs(资产层)----
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TextureArrayLayerSource {
    /// 外部纹理引用(逐层一文件,.zarray 清单形态)
    Reference(AssetReference),
    /// 单图按 TextureArrayLayout(RowCount/RowHeight,复用既有类型)切片
    SlicedFromImage { reference: AssetReference, layout: TextureArrayLayout },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Texture2DArrayAsset {
    pub uri: AssetUri,
    /// dimension=D2、array_layer_count=N;全层尺寸/格式一致性在导入期校验
    pub descriptor: TextureAssetDescriptor,
    pub layers: Vec<TextureArrayLayerSource>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CubemapSourceLayout {
    /// 六文件,顺序固定 +X -X +Y -Y +Z -Z(wgpu 层序)
    SixFiles,
    HorizontalCross,
    VerticalCross,
    /// 经距柱状投影,导入期 CPU 重采样到六面
    Equirectangular,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CubemapAsset {
    pub uri: AssetUri,
    /// array_layer_count=6;view 维度 Cube;计划 11 SkyboxSettings 按 AssetReference 消费
    pub descriptor: TextureAssetDescriptor,
    pub source_layout: CubemapSourceLayout,
    pub sources: Vec<AssetReference>, // SixFiles=6 项,其余=1 项
}

// ---- graphics/scene/resources/svt/svt_streamer.rs(实现层,wgpu 允许)----
pub(crate) struct SvtStreamer {
    page_table: SvtPageTable,        // rgba8uint 纹理 + CPU 镜像
    atlas: SvtPhysicalAtlas,         // 持久 atlas(经计划 01 mark_persistent)
    pool: SvtPagePool,               // LRU 空闲堆 + 锁定 mip tail 页
    pending_requests: Vec<SvtPageRequest>,
    budget: SvtBudget,               // max_resident_pages / max_uploads_per_frame
    stats: SvtStats,
}

impl SvtStreamer {
    /// 帧首:应用 io 完成页(atlas 上传 + 页表更新 + 祖先映射传播)
    pub(crate) fn apply_pending_pages(&mut self, queue: &wgpu::Queue) -> u32;
    /// GpuReadbackQueue 回调入口:去重、合并、按预算入队
    pub(crate) fn ingest_feedback(&mut self, data: &[u32]);
    /// 超预算时驱逐 LRU 非锁定页并回写页表
    pub(crate) fn evict_over_budget(&mut self) -> u32;
}
```

zshader/材质槽衔接:zshader 纹理槽声明增加维度标注(`texture_2d` 默认、`texture_2d_array`、`texture_cube`),编译进材质布局后由 `MaterialTextureSlotValue::expected_dimension` 携带;`resource_streamer_validate_material_shader_layout.rs` 在绑定时比对资产 `descriptor.dimension` 与槽位维度,不匹配走 `RenderMaterialTextureSlotFallbackReason` 新变体 `DimensionMismatch`。

### GPU 数据布局与 WGSL 约定

**页表(indirection)纹理**:格式 `rgba8uint`,尺寸 = `virtual_size / page_size`(每虚拟 mip 一个页表 mip 层)。texel 含义:`r = physical_page_x`、`g = physical_page_y`、`b = resident_mip`(该映射实际来自哪个虚拟 mip)、`a = flags`(bit0 resident,bit1 locked)。页表更新在 CPU 侧做祖先映射传播:非常驻 texel 写入最近常驻祖先页的映射(对齐 UE `FTexturePageMap` 的祖先回填语义),因此采样端单次 fetch、无 mip 回退循环。

**物理 atlas**:4096×4096 持久纹理(格式 = 资产压缩格式),页槽 = 128 payload + 双侧 4px border = 136px,容纳 30×30 页;border 在页上传时由相邻 texel 复制填充,保证双线性滤波不跨页串色。

**feedback buffer**(storage,std430):`array<u32>`,长度 = `ceil(w/8) * ceil(h/8)`(1/8 分辨率栅格);每 cell 一个打包请求:`vt_id:8 | mip:4 | page_y:10 | page_x:10`,`0xFFFFFFFFu` 表示无请求。每帧对栅格采样点加帧序抖动(像素内偏移),避免固定栅格永远丢失小物件请求。该 buffer 经计划 01 builder 声明并 `mark_readback`,帧末交计划 16 `GpuReadbackQueue`。

**MipGenPass binding**(group1,pass 级;group0 不占用):

| binding | 类型 | 内容 |
|---------|------|------|
| 0 | `texture_2d<f32>` | 源 mip(m-1)view |
| 1–4 | `texture_storage_2d<rgba8unorm/rgba16float, write>` | 目标 mip m..m+3 的逐层 view(不足 4 级时绑 dummy) |
| 5 | uniform `MipGenConstants { src_size: vec2<u32>, dst_count: u32, srgb_aware: u32 }` | 常量 |

`@workgroup_size(8, 8, 1)`,dispatch = `ceil(dst0.x/8) × ceil(dst0.y/8)`;每线程读源 2×2 均值写 mip m,经 workgroup shared memory 折叠继续写 m+1..m+3。12 级 mip 链 = 3 个 dispatch。`srgb_aware=1` 时读端解码到线性、写端编码回 sRGB(storage 写无自动 sRGB 编码)。

**BC5 Z 重建与 Y 约定**(`zr_normal.wgsl`,只含函数,无 entry point):

```wgsl
// zr_normal.wgsl —— 计划 08 模板拼接消费
fn zr_normal_reconstruct_bc5(rg: vec2<f32>) -> vec3<f32> {
    let xy = rg * 2.0 - vec2<f32>(1.0, 1.0);
    let z = sqrt(clamp(1.0 - dot(xy, xy), 0.0, 1.0));
    return vec3<f32>(xy, z);
}

// convention_dx != 0u 时翻转 green(TangentSpaceDx → 引擎统一 GL 风格)
fn zr_normal_apply_convention(n: vec3<f32>, convention_dx: u32) -> vec3<f32> {
    let y = select(n.y, -n.y, convention_dx != 0u);
    return normalize(vec3<f32>(n.x, y, n.z));
}
```

**SVT 采样**(`zr_svt.wgsl`):

```wgsl
struct ZrSvtConstants {
    virtual_size: vec2<f32>,
    page_size: f32,      // 128.0
    border_size: f32,    // 4.0
    atlas_page_stride: f32, // 136.0 / atlas_size
    max_mip: f32,
    _pad0: f32,
    _pad1: f32,
};

// uv → 页表 fetch → 物理 atlas UV(含 border 内缩)→ 采样
fn zr_svt_sample(uv: vec2<f32>, duvdx: vec2<f32>, duvdy: vec2<f32>, ...) -> vec4<f32>;
// feedback pass 复用的请求打包
fn zr_svt_pack_request(vt_id: u32, mip: u32, page: vec2<u32>) -> u32;
```

### 帧时序与集成点

mip 生成(运行期,仅 `GenerateRuntime` 资源,即 RT/探针捕获):graph 编译期,`mip_gen` feature descriptor 对带 `GenerateRuntime` 标记的 graph 纹理在其最后写者 pass 之后插入 MipGenPass 节点;TX-M2 先以普通 graph compute pass 落地,计划 16 CN-M1 落地后同变更内迁移为 `ComputePassDescriptor` 表达并删除直建路径(16 已把 MipGen 迁移列为其示范切片,顺序协调见"跨计划冲突点")。

SVT 帧间流水(帧 N):

1. 帧首(extract 之后、graph 执行前):`SvtStreamer::apply_pending_pages` —— io 完成页上传 atlas(`write_texture`)、页表 texel 更新 + 祖先传播、LRU 触碰;随后 `evict_over_budget`。
2. graph 内:`svt_feedback` pass(base pass 之后,读 depth 做最近表面判定)以 1/8 栅格写 feedback buffer。
3. graph 末:feedback buffer 经 `GpuReadbackQueue::enqueue`(计划 16 staging ring)。
4. 帧 N+k(k = 队列延迟,2–3 帧):回调进入 `ingest_feedback` —— 去重(HashSet 按打包 u32)、剔除已常驻、按 mip 粗到细排序、预算内提交 io 请求。
5. 任意时刻不可采样的页:页表祖先映射保证采样落到最近常驻粗 mip(mip tail 锁定页为最终兜底),首帧糊但永远可采样。

集成点:`ResourceStreamer::ensure_texture` 遇 `metadata.svt.is_some()` 且 feature gate 开 → 只上传 mip tail 并注册到 `SvtStreamer`;gate 关 → 截断 mip 链(从 `mip_tail_first_level` 起)走既有普通纹理路径,SVT 代码路径完全不实例化(零成本)。

硬切换删除项(同变更内完成,无兼容层):

- 删除 `RenderImageColorSpace::Unknown` 变体及全部 match 分支;现有资产经 `zircon_build.py` 资产步骤重跑导入批量写入权威 `TextureMetadata`。
- 删除导入器中按格式后缀/文件名猜测 sRGB 的隐式默认(`import_image` 的无条件 `rgba8_srgb` 默认改为按 `usage_hint` 决策表)。
- 删除 `resource_streamer_ensure_texture.rs` 中按槽位名字符串挑 fallback 纹理的路径,改为 `usage_hint` 驱动。

### 实施切片细化

**TX-M1 元数据与色彩空间权威化**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| M1.1 契约定稿 | `image/metadata.rs`(新)、`image/svt.rs`(新)、`image/mod.rs`、`image/color_space.rs`、`image/descriptor.rs` | `TextureMetadata` 全字段 + 枚举族;删除 `Unknown`;descriptor 挂 metadata | `cargo check -p zircon_runtime --lib --locked` 过;`Unknown` 引用清零 |
| M1.2 校验规则表 | `image/metadata_validation.rs`(新)、`asset/importer/ingest/import_texture.rs` | R1–R11 实现;ingest 错误中断/警告入报告 | R1/R3/R5/R6/R8/R11 各有失败用例;诊断文案与表一致 |
| M1.3 导入器默认 + 迁移 | 插件 `importers.rs`、`asset/assets/texture/descriptor.rs` | `usage_hint` 默认决策表(albedo=srgb、normal/mask/data=linear、hdr=float);import settings 新键解析 | 全量资产重跑导入零错误;抽样 zmeta 含 metadata 块 |

**TX-M2 mip 生成与 normal 管线**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| M2.1 离线 mip | 插件 `mipgen/{mod.rs,kernel.rs}`、`importers.rs` | kaiser(默认,albedo/data)/box(normal/ui)kernel;sRGB 感知;normal 逐 mip renormalize | 256² 输入产出 9 级;normal mip 全 texel 长度 1±1e-3 |
| M2.2 运行期 MipGenPass | `scene_renderer/mip_gen/*`(新)、`builtin_render_feature_descriptor/mip_gen.rs`(新) | 一次写 4 mip 的 compute;graph 节点插入最后写者之后 | RT 纹理 mip 链 readback 与离线结果误差阈内 |
| M2.3 BC5 normal | `includes/zr_normal.wgsl`(新)、插件 transcode 目标族、`upload_support/compressed.rs` | Z 重建 + Y 约定函数;BC5 上传族 | BC5 法线光照对拍误差阈内(`render_product_*`) |
| M2.4 质量档接线 | `descriptor.rs` sampler 路径、质量档配置 | `mip_bias`/`max_anisotropy` 叠加质量档上限 | 质量档切换后采样器参数断言 |

**TX-M3 array 与 cubemap 资产**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| M3.1 资产类型 | `array_asset.rs`/`cube_asset.rs`(新)、`asset/load/texture.rs`、`gpu_texture_resource_from_asset.rs` | 两资产 + 一致性校验 + D2Array/Cube view 上传 | 尺寸/格式不一致导入报错;GPU 建数组纹理成功 |
| M3.2 导入器 | 插件 `cubemap.rs`/`array.rs`(新)、`registration.rs`、`plugin.toml` | 六面/十字/equirect → cube;清单/切片 → array | 三种 cube 输入形态各一条导入用例通过 |
| M3.3 槽位维度 | `texture_slot_summary.rs`、`material/texture_slot.rs`、`resource_streamer_validate_material_shader_layout.rs` | 维度声明 + 绑定校验 + `DimensionMismatch` fallback | 2D 资产绑 cube 槽走 fallback 且诊断可见;计划 11 `SkyboxSettings` 可按引用消费 `CubemapAsset` |

**TX-M4 稀疏虚拟纹理(可选 feature)**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| M4.1 页表/atlas/池 | `svt/{page_table,physical_atlas,page_pool}.rs`(新) | rgba8uint 页表 + 祖先传播;持久 atlas(`mark_persistent`);LRU 空闲堆 | 纯 CPU 单测:映射/传播/驱逐正确 |
| M4.2 feedback + 回读 | `svt_feedback/*`(新)、`svt/feedback_analysis.rs`(新) | 1/8 栅格写请求;经 `GpuReadbackQueue` 回读(依赖 16 CN-M1 已落地) | 回读数据可解包去重;N 帧延迟语义断言 |
| M4.3 streamer 闭环 | `svt/svt_streamer.rs`(新)、`resource_streamer*` 两文件、`backend_types.rs` | 预算调度、上传应用、驱逐、stats | 请求→加载→命中集成测试绿 |
| M4.4 采样与 gate | `includes/zr_svt.wgsl`(新)、`builtin_render_feature_descriptor/svt_feedback.rs`(新) | 页表跳转采样;gate 关 → 零 pass、截断 mip 回落 | gate 关时 compiled graph 无 svt pass(统计断言);巨幅纹理场景常驻页 << 总页数 |

### 测试与验收清单

单测(命名按 index.md §8 第 6 条;位置 = 各实现文件 `#[cfg(test)] mod tests` 或既有测试树):

| 测试函数 | 断言要点 | 位置 |
|---------|---------|------|
| `render_texture_meta_normal_srgb_rejected` | R1 触发错误,文案含 `color_space=srgb` | `image/metadata_validation.rs` |
| `render_texture_meta_hdr_requires_float_format` | R3 错误;`rgba16float` 通过 | 同上 |
| `render_texture_meta_bc6h_srgb_rejected` | R5 错误 | 同上 |
| `render_texture_meta_svt_requires_mip_chain` | R8 错误 | 同上 |
| `render_texture_meta_defaults_follow_usage_hint` | albedo→Srgb/Bc7、normal→Linear/Bc5、hdr→Hdr/Bc6h | 插件 `tests/`(导入默认在插件) |
| `render_texture_meta_import_settings_roundtrip` | toml 新键 → metadata → zmeta serde 往返 | `asset/tests/assets/texture_importer.rs` |
| `render_mipgen_offline_kaiser_chain_complete` | 256²→9 级,尺寸序列正确 | 插件 `runtime/src/tests/mipgen.rs`(新) |
| `render_mipgen_offline_srgb_aware_average` | sRGB 0x00/0xFF 棋盘降采样 ≈ 0xBC(线性域均值)而非 0x80 | 同上 |
| `render_mipgen_offline_normal_renormalized` | 全 mip texel 长度 1±1e-3 | 同上 |
| `render_mipgen_pass_four_mips_per_dispatch` | 12 级链 dispatch 数 = 3;`mipgen_dispatches` 统计 | `scene_renderer/mip_gen/executor.rs` |
| `render_mipgen_pass_matches_offline_within_tolerance` | GPU readback 与离线链逐 texel 误差 ≤ 2/255 | 同上(gpu 测试标记) |
| `render_texture_array_layer_mismatch_rejected` | 层尺寸/格式不一致导入失败 | `asset/assets/texture/array_asset.rs` |
| `render_texture_cubemap_cross_face_order` | 横/竖十字切分后面序 = +X -X +Y -Y +Z -Z | `asset/assets/texture/cube_asset.rs` + 插件用例 |
| `render_texture_slot_dimension_mismatch_falls_back` | 2D 绑 cube 槽 → `DimensionMismatch` fallback | `texture_slot_summary.rs` |
| `render_svt_request_pack_roundtrip` | u32 打包/解包逆元 | `svt/feedback_analysis.rs` |
| `render_svt_feedback_dedupe_unique_pages` | 重复 cell 合并为单请求 | 同上 |
| `render_svt_page_table_ancestor_propagation` | 子页未驻留时 texel 指向最近常驻祖先 | `svt/page_table.rs` |
| `render_svt_lru_evicts_oldest_unlocked` | 超预算驱逐最久未用且跳过 mip tail 锁定页 | `svt/page_pool.rs` |
| `render_svt_readback_uses_queue_n_frame_latency` | 请求帧 N 提交、帧 N+k 才进 streamer(经 GpuReadbackQueue 假驱动) | `svt/svt_streamer.rs` |
| `render_svt_gate_off_compiles_zero_passes` | feature 关 → compiled graph 无 svt pass、无 atlas 分配 | `svt_feedback/executor.rs` |

产物对拍(`render_product_*` + `ZR_RENDERDOC_CAPTURE_NEXT=1` 抓帧):

- `render_product_bc5_normal_matches_uncompressed`:同法线场景 BC5 vs 未压缩光照差 ≤ 阈值。
- `render_product_mip_chain_no_shimmer`:斜视角棋盘格远景,有/无 mip 链帧间方差对比。
- `render_product_svt_resident_pages_below_budget`:>显存虚拟纹理场景,`SvtStats::resident_pages` << `total_virtual_pages` 且画面收敛。

命令基线:切片期 `cargo check -p zircon_runtime --lib --locked`;里程碑末 `cargo test -p zircon_runtime render_texture --locked`、`render_mipgen`、`render_svt` 过滤词收窄;插件侧 `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_texture_importer_runtime --locked`。

### 参考实现精读笔记

**UE `VT/VirtualTextureSystem.cpp`**:`FVirtualTextureSystem::BeginUpdate` 在帧更新入口调用 `GVirtualTextureFeedback.Map(GraphBuilder.RHICmdList)` 拿上一(几)帧 feedback;`FFeedbackAnalysisTask`/`FeedbackAnalysisTask` 把原始 buffer 去重进 `FUniquePageList`(可多任务并行,分段后合并);`GatherRequests`/`FGatherRequestsTask` 把页列表转成 `FUniqueRequestList`(`MergeRequests` 合并多任务结果),再 `SubmitRequests` 按预算(`Updater->PageUploadBudgetSVT`,来自 `Settings.MaxSVTPageUploads`)产页;`LoadPendingTiles` 展示了无 feedback 的显式区域请求路径(`EncodePage(SpaceID, MipLevel, TileX, TileY)` 打包 u32);`bFlushCaches` 分支对每个 `FVirtualTexturePhysicalSpace` 调 `GetPagePool().EvictAllPages`。Zircon 对应:`SvtStreamer::ingest_feedback`(去重+合并)与帧首 `apply_pending_pages`;取舍:V1 不做 UE 的异步任务化(`bAsyncTaskAllowed`)与 adaptive VT,单线程分析在 readback 回调内完成,数据量(1/8 栅格)足够小。

**UE `VT/VirtualTextureFeedback.cpp`**:`TransferGPUToCPU` 用固定 `MaxTransfers` 的 staging ring(`WriteIndex`/`ReadIndex`/`NumPending`),溢出时丢最旧项(`Fences->Release(ReadIndex)` 后推进读位,统计 `STAT_VirtualTexture_LostFeedback`);`CanMap` 先于 RVT flush 检查可用性防低帧率毛刺;`Map(MaxTransfersToMap)` 返回 `FMapResult`;条目 stride 由 `Desc.bPageAndCount ? 2 : 1` 决定。Zircon 对应:这套 ring+fence 正是计划 16 `GpuReadbackQueue` 的职责,本计划只做消费者 —— 溢出丢旧、N 帧延迟、丢失统计都由 16 统一实现,SVT 侧仅需容忍"丢一帧 feedback 只延迟收敛"。

**UE `VT/VirtualTexturePhysicalSpace.cpp` + `TexturePagePool.h`**:物理空间按 `FVTPhysicalSpaceDescription`(TileSize、格式族、`bHasLayerSrgbView`)建池;`GetTileSizeInBytes` 逐 layer `CalculateImageBytes` 累加;页池 `FTexturePagePool` 用 `FBinaryHeap<uint32, uint16> FreeHeap` 按帧号做 LRU(`GetNumLockedPages() = GetNumPages() - FreeHeap.Num()`,锁定页直接不在堆里);`UpdateResidencyTracking(Frame)` 维护驻留压力并驱动 mip bias 退让。Zircon 对应:`SvtPagePool` 同样用"锁定页不入堆"表达 mip tail;取舍:V1 不做 UE 的 residency 压力自适应 mip bias,超预算只走硬驱逐 + stats 暴露,自适应留给后续质量档联动。

**Unity `MipGen/MipGenerator.cs`**:`m_DepthPyramidCS` 的 `DepthDownsample` kernel 单 dispatch 最多写 4 级(`DepthPyramidConstants` 携带 `_MinDstCount`、`_DstSize0..3`、`_MinDstOffset0..3`,外层 `dstIndex0 += minCount` 循环),线程组 `DivRoundUp(dstSize, 8)` 即 8×8;`m_ColorPyramidCS` 是逐 mip 两 kernel(`ColorDownsample` + `ColorGaussian`)且明确注释 color pyramid 不能 in-place(读写同资源冲突);`m_PreferCompute/m_SupportCompute` 提供 raster PS 回落(`m_ColorPyramidPSMat`)。Zircon 取舍结论:MipGenPass 采用其 depth pyramid 的"一次写 4 mip"形态(storage 绑定数与 wgpu `maxStorageTexturesPerShaderStage` 下限兼容,12 级链 3 dispatch),不采用完整 AMD SPD 单 dispatch(需跨 workgroup coherent 原子,wgpu 可移植性差),也不引入 gaussian kernel(那是后处理 color pyramid 需求,非 mip 链需求);不做 raster 回落 —— wgpu compute 是基线能力。

## 风险与回退

- SVT 回读延迟造成首帧糊:常驻低 mip 兜底页保证永远可采样,清晰度渐进。
- BasisU transcode 依赖引入:放在 importer 插件侧,runtime 不增依赖;无 transcoder 时离线直存 BC。
- 元数据迁移波及全部现有纹理:导入器重跑由 `zircon_build.py` 资产步骤批量执行,迁移前后产物对拍。
