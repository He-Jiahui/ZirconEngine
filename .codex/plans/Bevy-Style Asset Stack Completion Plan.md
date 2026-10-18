# Bevy-Style Asset Stack Completion Plan

## Summary

把 `asset / image / gltf / shader / mesh / material` 收束成 Bevy 风格的统一资产模型，但落点保持 Zircon 现有架构：`zircon_runtime::core::resource` 继续做身份、状态、事件和 payload 存储；`zircon_runtime::asset` 做项目扫描、importer、artifact、热重载和 typed asset facade；`zircon_runtime::graphics` 做 render prepare/cache。

主要 Bevy 参照：本地 `dev/bevy` 为准，版本是 `0.19.0-dev`。关键源码包括 [Assets<T>](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_asset/src/assets.rs:288>)、[Handle](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_asset/src/handle.rs:134>)、[AssetEvent](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_asset/src/event.rs:50>)、[AssetLoader/LoadContext](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_asset/src/loader.rs:29>)、[LoadState](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_asset/src/server/mod.rs:1993>)、[Image/ImageLoader](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_image/src/image.rs:606>)、[GltfAssetLabel/GltfLoader](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_gltf/src/label.rs:33>)、[Mesh](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_mesh/src/mesh.rs:227>)、[Material/StandardMaterial](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_pbr/src/material.rs:144>)、[Shader/ShaderCache](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_shader/src/shader.rs:35>)。外部文档参考 docs.rs：[`bevy_asset`](https://docs.rs/bevy_asset/latest/bevy_asset/)、[`bevy_gltf`](https://docs.rs/bevy_gltf/latest/bevy_gltf/)、[`bevy_image`](https://docs.rs/bevy_image/latest/bevy_image/)、[`bevy_mesh`](https://docs.rs/bevy_mesh/latest/bevy_mesh/)、[`bevy_pbr`](https://docs.rs/bevy_pbr/latest/bevy_pbr/)、[`bevy_shader`](https://docs.rs/bevy_shader/latest/bevy_shader/)。

## Public APIs / Interfaces

- 在 `zircon_runtime::asset` 增加 Bevy-style typed facade：`Asset` trait、`Handle<TAsset>`、`Assets<TAsset>`、`AssetEvent<TAsset>`、`AssetLoadState`、`DependencyLoadState`、`RecursiveDependencyLoadState`、`AssetLoadStates`。实现上复用现有 `ResourceHandle<TMarker>`、`ResourceManager`、`ResourceRecord`、`ResourceEvent`、`ResourceLease`，不新建平行存储。
- 在 `zircon_runtime_interface::resource` 增加 `MeshMarker / Mesh` 资源种类；`TextureAsset`、`ShaderAsset`、`MaterialAsset`、`MeshAsset`、`ModelAsset`、`GltfAsset` 都实现统一 `Asset` contract。
- 扩展 importer 输出为多资产集合：`ImportedAssetEntry { locator, asset, dependencies, diagnostics }`，root locator 无 label，subasset 使用现有 `ResourceLocator #label` 语法。glTF、atlas、shader include、material texture 依赖都进入同一个 dependency graph。
- `ProjectAssetManager` 增加统一入口：`load<TAsset>(locator) -> Result<Handle<TAsset>, AssetLoadError>`、`assets<TAsset>() -> Assets<TAsset>`、`load_state(handle)`、`dependency_load_state(handle)`、`recursive_dependency_load_state(handle)`、`load_states(handle)`、`is_loaded*` convenience predicates、`subscribe_asset_events<TAsset>()`。现有 `load_model_asset/load_texture_asset/...` 在实现中改走 typed facade。
- `TextureAsset` 升级为 Bevy `Image` 对齐的数据模型：extent、dimension、format、mip_count、array_layers、sampler、color_space、render_asset_usages、raw/container payload。`ShaderAsset` 增加 import path、shader_defs、imports、source format 和 validation report。`MaterialAsset` 升级为 Standard PBR + custom shader/material-graph 引用模型。`ModelAsset` 变成容器，具体几何进入 `MeshAsset`。

## Milestones

1. **M1: Asset Facade Foundation**
   - 实现 `Asset / Handle<T> / Assets<T> / AssetEvent<T> / AssetLoadState`，把 Bevy 的 `Assets<T>` 和 `Handle<T>` 模型转译到现有 `ResourceManager`。
   - `LoadState` 映射规则固定：无 record 为 `NotLoaded`；`Pending/Runtime Loading` 为 `Loading`；`Ready + payload` 为 `Loaded`；`Error` 为 `Failed`; `Reloading` 保持 Zircon 扩展态并兼容为 loading 类状态。
   - Testing stage: typed handle kind mismatch、get/acquire/release、event typed filtering、load state mapping、last-good payload 保留。

2. **M2: Dependency Graph And Events**
   - 把 `AssetImportOutcome.dependencies` 落到 `ResourceRecord.dependency_ids`，并新增 locator-to-id 解析失败诊断。
   - 实现 recursive dependency load state，行为参考 Bevy `LoadedWithDependencies` 与 glTF loader 测试。
   - Testing stage: direct dependency、nested dependency、missing dependency、failed dependency、rename/remove/reload event 顺序。

3. **M3: Multi-Asset Import Output And Labels**
   - 将 importer contract 从单个 `ImportedAsset` 切到 `ImportedAssetEntry` 集合，artifact store 为每个 label 写独立 artifact。
   - 使用 `.zmeta.entries[*].uuid` 为 root 和每个 labeled subasset 提供独立持久 UUID，并用 `AssetId::from_asset_uuid(entry.uuid)` 生成资源 ID；`url#label` 仍是可读定位和修复线索，不再参与身份派生。
   - Testing stage: root + subasset artifact、label 稳定 ID、重复 label 报错、unknown label load 返回 structured error。

4. **M4: Image / Texture Completion**
   - 以 Bevy `ImageLoader` 为参考补齐 settings、sampler、format、mip/layer/container 元数据；保留现有 PNG/JPEG 等真实解码和 DDS/KTX/KTX2/ASTC container header 解析。
   - texture importer 输出 `TextureAsset` descriptor；压缩容器不强制 CPU RGBA 展开，交给 graphics prepare 判断 GPU 支持。
   - Testing stage: PNG/JPEG/BMP/TGA/TIFF/GIF/WebP/HDR/EXR/QOI/PNM decode，PSD flatten，DDS/KTX/KTX2/ASTC header，invalid header diagnostics。

5. **M5: Mesh As First-Class Asset**
   - 新增 `MeshAsset`：primitive topology、indices `u16/u32`、named/custom vertex attributes、morph targets、skin inverse bindposes、bounds、render usages。
   - `ModelAsset` 改为 container：保存 mesh/material/scene-level references，不再直接持有 flattened vertices/indices。
   - Testing stage: OBJ/glTF mesh import、custom attributes、missing normals tangent generation、index width preservation、large mesh boundary、old direct primitive tests 迁移到 `MeshAsset`。

6. **M6: Shader And Material Completion**
   - `ShaderAsset` 对齐 Bevy `Shader`：WGSL/SPIR-V/Naga source、import path、imports、shader_defs、entry points、validation diagnostics；`ShaderCache` 按 `ResourceId + revision + shader_defs` 失效。
   - `MaterialAsset` 对齐 Bevy `StandardMaterial` 的 PBR 主干：base color、metallic/roughness、normal/occlusion/emissive、alpha、double-sided、unlit、UV channel、clearcoat/specular/anisotropy 等 glTF extension 字段；custom material 通过 shader/material graph 引用扩展。
   - Testing stage: WGSL valid/invalid、shader import dependency invalidation、StandardMaterial TOML roundtrip、material texture dependency、transparent/alpha mask pipeline key。

7. **M7: glTF Bevy-Parity Importer**
   - glTF importer 输出 `GltfAsset` root，加 labels：`Scene{i}`、`Node{i}`、`Mesh{i}`、`Primitive{i}`、`Material{i}`、`Texture{i}`、`Image{i}`、`Animation{i}`、`Skin{i}`，参考 Bevy `GltfAssetLabel`。
   - 支持 scenes/nodes/hierarchy/transforms、materials/textures/images、skins、animations、extras、names、coordinate conversion、custom vertex attributes；不支持的 glTF extensions 进入 capability diagnostics。
   - Testing stage: Bevy glTF loader 测试语义迁移，包括 node hierarchy、skin node、custom asset source images、image failure as load failure、loaded-with-dependencies。

8. **M8: Render Prepare And Hot Reload**
   - 在 graphics 建立 `PrepareRenderAsset<T>` 风格路径：`PreparedTexture`、`PreparedMesh`、`PreparedShader`、`PreparedMaterial` 都按 `ResourceId + revision` cache。
   - shader/material/texture/mesh dependency 变化后只重建受影响 prepared resources；reload 失败保留 last-good prepared resource 并上报 diagnostics。
   - Testing stage: revision cache hit/miss、shader import invalidation、material texture reload、mesh reload、failed reload keeps last-good render output。

9. **M9: Editor Diagnostics And Export**
   - Asset Browser 改用 `Assets<T>`、typed events、load state、dependency state；展示 importer capability、subasset labels、diagnostics、artifact locator、revision。
   - Export manifest 写出 handle graph、subasset labels、required importer/render capabilities，确保缺插件时可诊断。
   - Testing stage: editor resource tree、subasset selection、diagnostic display、export manifest includes dependencies、disabled importer produces stable failure.

10. **M10: Documentation And Acceptance**
   - 更新 `docs/zircon_runtime/asset/*`、`docs/assets-and-rendering/*`、插件 importer docs，记录 Bevy 对照、Zircon divergence、public API、tests。
   - Milestone validation: scoped `cargo test -p zircon_runtime --lib asset`、`cargo test -p zircon_runtime --lib graphics`、`cargo test --manifest-path zircon_plugins/Cargo.toml --locked`，最后运行仓库 validator `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1`。

## Assumptions

- 范围限定为 `asset / image / gltf / shader / mesh / material`，不在本计划内重做 Bevy ECS、app plugin group 或完整 renderer。
- Bevy 是行为和公共模型参照，不复制 crate 布局；Zircon 的权威落点仍是 `zircon_runtime::core::resource`、`zircon_runtime::asset`、`zircon_runtime::graphics`。
- 采用硬切换：新增 `MeshAsset` 与 multi-asset importer 后，不保留长期旧 `ModelPrimitiveAsset` 直存顶点路径。
- 现有 importer plugins 保留为能力来源，但输出契约统一升级；未安装 native/backend 的格式必须生成稳定 diagnostics，而不是静默降级。

## Current Completion State (2026-05-23)

当前计划的底层资产身份、typed facade、依赖状态和 `.zmeta`/shader-material 主链已经推进到比原始 M1-M3 描述更具体的状态：

- M1/M2 的 typed facade 和 dependency state 已经拥有 `AssetLoadState`、`DependencyLoadState`、`RecursiveDependencyLoadState` 与 `AssetLoadStates` 分层；`ProjectAssetManager` 暴露 root/direct/recursive 状态查询和 `is_loaded*` 组合谓词，状态查询只读 registry/runtime state，不触发 resident restore。
- M3 的 multi-asset identity 已经跟 `.zmeta` 硬切规则对齐：root entry 使用 document UUID，subasset entry 使用自己的持久 UUID，`ResourceId` 从 entry UUID 派生。旧的“source UUID + label”身份派生方案不再是计划方向。
- `.zmeta` / package URI / compound `.zshader` / `.zmaterial` schema 已经由 `.codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md` 收束；后续 asset-stack work 应从当前实现状态继续，而不是重新引入基于可变 label 的身份派生或 runtime `*.meta.toml` 兼容路径。
- M5 的 mesh first-class path 已进入下一小切片：`MeshAsset` / `.zmesh` 现在记录 morph target attribute maps 和 skin inverse-bind matrices，保持 legacy `ModelPrimitiveAsset` 兼容转换不消费这些新字段，并用 focused mesh tests/docs 锚定 roundtrip 与 vertex-count validation。Built-in and plugin glTF labeled mesh primitive subassets now propagate primitive morph target displacement maps and node-level skin inverse-bind matrices into `MeshAsset`; standalone `Skin{n}` labels now emit JSON `DataAsset` rows with joint `Node{n}` locators, optional skeleton metadata, and `Skin{n}/InverseBindMatrices` matrix payloads, while `Animation{n}` remains the diagnostic placeholder until animation-channel import lands.
- M6 shader 主链继续补齐：`.zshader` 已支持可选 `import_path` 并持久流入 `ShaderAsset.import_path`；`shader_defs` 会复制到 `RenderShaderVariantKey.defines`；`[[imports]]` 声明现在无论是否带 redirect 都会保留在 `ShaderAsset.imports`，只有带 `{ uuid, url }` redirect 的行会进入 shader dependency graph。
- 当前活跃的 facade/load-state 切片正在 `zircon_runtime::asset::facade` 和 `docs/zircon_runtime/asset/facade.md` 中收敛，其他会话同时占用渲染验证队列；新的实现切片需要重新做会话协调后再进入 Rust 文件。
