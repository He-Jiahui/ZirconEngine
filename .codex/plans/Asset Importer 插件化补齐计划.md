# Asset Importer 插件化补齐计划

## Summary
- 采用硬切插件化：`AssetManager/ProjectManager` 只负责扫描、meta、registry、artifact、失败记录和热重导入，不再按扩展名内建分支导入格式。
- 所有格式通过 `AssetImporterRegistry` 注册：Rust runtime 插件和 NativeDynamic 插件都能贡献 importer。
- 第一轮“两层落地”：纯 Rust/稳定依赖格式真实导入；FBX、PSD、KTX/DDS/ASTC、HLSL/CG 等重型格式先接入可选插件接口和明确失败诊断，后续专用插件补真实解码。

## Key Changes
- 在 `zircon_runtime::asset` 增加通用 importer contract：
  - `AssetImporterDescriptor { id, plugin_id, priority, source_extensions, full_suffixes, output_kind, importer_version, required_capabilities }`
  - `AssetImportContext { source_path, uri, source_bytes, import_settings }`
  - `AssetImportOutcome { imported_asset, dependencies, migration_report, diagnostics }`
  - `AssetImporterRegistry` 按 full suffix 优先匹配，例如 `.ui.toml`、`.scene.toml`、`.prefab.toml`，再按普通扩展匹配。
- 扩展 `RuntimeExtensionRegistry` 和 `PluginPackageManifest`，允许插件注册 asset importers；editor 的 `AssetImporterDescriptor` 继续只表示菜单/操作/UI 贡献，不作为 runtime 导入执行源。
- 修改项目扫描：
  - 单个文件失败不再中断整个扫描；生成 `ResourceState::Error` 记录、诊断、source hash、importer id/version。
  - `*.meta.toml` 继续作为 sidecar 忽略。
  - `foo.toml` 作为普通 TOML DataAsset；`foo.xxx.toml` 必须有合法 typed importer，否则失败。
- 新增/扩展资产类型：
  - `DataAsset`：支持 `.toml` 默认加载；`.json/.yaml/.yml/.xml` 通过数据 importer 插件加载。
  - `TextureAsset`：保留 `rgba8` 解码输出，同时预留 compressed/container payload，用于 DDS/KTX2/ASTC/cubemap/DXGI 类纹理。
  - `ShaderAsset`：保存 `source_language`、原始 source、规范化 `wgsl_source`、入口点和 Naga 验证诊断。
- 格式第一轮：
  - Model：真实支持 `.gltf/.glb/.obj`；`.fbx/.dae/.3ds/.dxf/.ply/.stl/.usd/.usda/.usdc/.usdz` 注册为可选模型插件能力，未安装 backend 时生成失败记录。
  - Texture：真实支持 `png/jpg/jpeg/bmp/tga/tiff/gif/webp/hdr/exr/qoi/pnm`；`psd/dds/ktx/ktx2/astc/cubemap/dxgi` 走可选插件能力与诊断。
  - Audio：用统一音频解码插件输出 PCM f32，支持 `wav/mp3/ogg/flac/aif/aiff/opus` 能力；缺编解码 backend 时失败但不中断扫描。
  - Shader：`wgsl` 直接 Naga 校验；`glsl/vert/frag/comp/spv` 转 Naga IR 后输出 WGSL；`hlsl/cg/fx` 通过 NativeDynamic 工具链插件进入 SPIR-V/Naga/WGSL，工具缺失时失败。
- NativeDynamic loader：
  - 增加 `runtime.asset.importer.native` 能力。
  - 通过 ABI v2 command 调用 `asset.import/<importer_id>`，输入 `ZRIMP001` envelope：metadata JSON + 原始文件 bytes。
  - 输出 `ZRIMO001` envelope：neutral import DTO + artifact bytes/diagnostics；host 负责反序列化、校验 kind、写 artifact。
  - panic、denied capability、malformed buffer、invalid output kind 都转成失败文件诊断。

## Public APIs / Interfaces
- `ImportedAsset` 增加 `Data`，`ResourceKind` 增加 `Data`；同步更新 `zircon_runtime_interface::resource` re-export。
- `AssetMetaDocument` 增加 importer id/version、source schema version、target schema version、migration summary；旧 meta v1 通过 migrator 升级到当前版本。
- typed TOML importer 统一实现 `AssetSchemaMigrator`：支持旧版本内存迁移到当前 artifact；未来版本和低于最低支持版本失败。
- 新增插件包布局：`zircon_plugins/asset_importers/{model,texture,audio,shader,data}/runtime`，并在独立 `zircon_plugins/Cargo.toml` 注册成员。

## Test Plan
- Registry：重复扩展名、priority 覆盖、full suffix 优先级、未知 `.xxx.toml`、plain `.toml` data fallback。
- Project scan：有效文件导入、无效文件失败记录、失败后继续扫描、meta 稳定 UUID、importer version/config hash 更新、watch reimport 失败到成功。
- Formats：迁移现有 glTF/OBJ/PNG/JPEG/WGSL/WAV 测试；新增 BMP/TGA/TIFF/HDR/EXR、TOML/JSON/YAML/XML、GLSL/SPIR-V、MP3/OGG/FLAC/AIFF 覆盖；FBX/HLSL/PSD/KTX/DDS/ASTC 未装 backend 时诊断稳定。
- NativeDynamic：fixture 注册 importer、成功导入 DataAsset、capability denied、panic、malformed buffer、wrong output kind、unload/state callback。
- TOML migrations：`ui.toml` 旧 schema 迁移、future schema 失败、`scene.toml/prefab.toml/meta.toml` version policy。
- Milestone validation：先运行 `cargo test -p zircon_runtime --locked asset`、`cargo test -p zircon_runtime --locked plugin_extensions`、`cargo test --manifest-path zircon_plugins/Cargo.toml --locked`，最后运行仓库 validator `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1`。

## Assumptions
- 先接入 NativeDynamic loader 执行边界，但不让 native 插件跨 ABI 持有 Rust trait object、`Arc`、`wgpu` 对象、world/editor state。
- 当前已有其他会话在改 plugin manifest、sound、editor extension、runtime interface；实现时必须基于当时最新文件合并，不覆盖这些未完成改动。
- 参考依据：Unity/Unreal texture/audio/model import docs、Naga 29 docs，以及本地 Bevy `AssetLoader`、Godot `ResourceImporter`、Fyrox `ResourceLoader`、Unreal `UFactory/IAssetTools` 结构。
  Sources: [Unity texture import](https://docs.unity.cn/2018.4/Documentation/Manual/ImportingTextures.html), [Unreal textures](https://dev.epicgames.com/documentation/en-us/unreal-engine/textures-in-unreal-engine?application_version=5.6), [Unreal audio](https://dev.epicgames.com/documentation/pt-br/unreal-engine/importing-audio-files), [Unreal FBX](https://dev.epicgames.com/documentation/en-us/unreal-engine/fbx-content-pipeline?application_version=5.6), [Unreal glTF](https://dev.epicgames.com/documentation/en-us/unreal-engine/gltf-file-format-support-in-unreal-engine?application_version=5.6), [Naga docs](https://docs.rs/crate/naga/latest).
