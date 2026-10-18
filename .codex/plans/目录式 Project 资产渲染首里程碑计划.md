# 目录式 Project 资产渲染首里程碑计划

## Summary
- 目标是先把“目录式项目 + 资产索引/监听 + 场景/材质/模型加载 + ECS 实例化 + PBR 渲染 + 编辑器 Gizmo 叠加”跑成一条真实主链，而不是继续扩展当前“直接文件路径 + demo 级 renderer”。
- 当前基线可保留并作为回归约束：`zircon_scene` 已有 `EntityId`、层级和 `RenderExtract`，`zircon_graphics` 已有离屏渲染、grid、Gizmo 线框，`zircon_editor` 已有项目文档与 viewport 宿主，`zircon_asset` 已有异步 worker、OBJ、图片直读；`cargo test -p zircon_asset -p zircon_scene -p zircon_graphics -p zircon_editor --lib` 当前全绿。
- 采用的参考组合是：Piccolo 的“世界对象”和“编辑器 Gizmo”分离；Fyrox 的 `ResourceIo + registry + watcher + import` 分层；Godot 的目录式项目、`res://` 资源路径、文件系统扫描和 importer 驱动的持久化资源。
- 选型固定为目录式新项目：`Project/zircon-project.toml`、`Project/assets/`、`Project/library/`。项目内资源统一使用 `res://...`；导入产物统一使用 `lib://...`。不再让编辑器或场景保存绝对文件路径。
- 本里程碑不重开 ECS 存储重写，不新建 `zircon_editor_3d` crate；直接在现有 `zircon_scene`、`zircon_graphics`、`zircon_editor` 上完成第一条可运行主链。

## Public Interfaces
- `zircon_asset` 成为项目和资产权威层，新增并对外固定：
  - `ProjectManifest`、`ProjectPaths`、`ProjectManager`
  - `AssetUri`、`AssetId`、`AssetKind`、`AssetMetadata`
  - `AssetRegistry`、`AssetWatcher`、`AssetImporter`、`ArtifactStore`
  - `TextureAsset`、`ShaderAsset`、`MaterialAsset`、`SceneAsset`、`ModelAsset`
- `zircon_server::AssetServer` 从只暴露 `pipeline_info()` 扩展为项目感知 façade，至少提供：打开项目、查询当前项目、查询资产状态、订阅资产变化、触发导入/重导入。
- `zircon_server::SceneServer` 增加基于 `SceneAsset` 的实例化/保存入口；编辑器不再把完整 `World` 当项目文件本体。
- `zircon_scene` 的运行时组件改成资产引用模型：
  - `MeshRenderer` 不再存 `MeshSource::Path` / `TextureSource::Path`
  - 运行时组件存 `AssetId`
  - `SceneAsset` 文件中存 `res://` URI，加载后统一解析为 `AssetId`
- `zircon_graphics` 增加材质/着色器解析层：
  - 项目内 `.wgsl` 资产编译缓存
  - 材质到 pipeline 的缓存
  - 占位 mesh / 占位材质 / 错误材质
  - `RenderExtract` 继续承载世界对象与编辑器 Gizmo 叠加，但渲染输入改为资产解析后的句柄

## Implementation Changes
- 里程碑 1：项目根与资产图基础。
  - 定义 `zircon-project.toml`，字段固定为项目名、格式版本、默认场景 `res://` URI、library 版本戳。
  - 引入 `res://` 与 `lib://` 规范化路径；拒绝越出 `Project/assets` 的路径；`Project/library` 只存可重建导入物，不允许人工编辑依赖它作为源数据。
  - 用 `notify` 建立 watcher + debounce + 变更日志，支持 add/update/delete/rename，并以“源文件 hash + importer 版本 + 配置 hash”作为 library 失效键。
  - gate：manifest roundtrip、URI 规范化、越界拒绝、watcher 事件折叠、library 失效测试全部通过。
- 里程碑 2：Rust 原生优先的格式与 importer。
  - 首期真接入固定为：PNG/JPEG 纹理、WGSL shader、TOML material、TOML scene、OBJ、glTF/GLB。
  - 依赖固定采用 Rust 原生生态优先：`notify`、`toml`、`image`/`png`/`jpeg-decoder`、`tobj`、`gltf`，WGSL 校验走 `wgpu`/`naga` 编译链。
  - `FBX`、`ASTC`、`PVRTexTool` 只在本里程碑里落 importer/transcoder 扩展点和结构化诊断，不要求真实工作链；这是“Rust 原生优先”约束下的明确边界，不做隐式 fallback。
  - library 中保存稳定导入物与元数据，graphics/runtime 只消费导入产物，不直接读源模型或源纹理。
  - gate：有效导入、缺文件、坏文件、坏 shader、重复导入命中缓存、配置变更触发重导入全部有测试。
- 里程碑 3：场景资产与 ECS 绑定。
  - `SceneAsset` 采用 `*.scene.toml`，按实体数组序列化，至少覆盖层级、变换、相机、方向光、mesh 实例、material 绑定、active 标记。
  - `MaterialAsset` 采用 `*.material.toml`，首期“完整 PBR”定义固定为 glTF metallic-roughness 核心集：`base_color`、`base_color_texture`、`normal_texture`、`metallic`、`roughness`、`metallic_roughness_texture`、`occlusion_texture`、`emissive`、`emissive_texture`、`alpha_mode`、`double_sided`、`shader`。
  - 编辑器 Gizmo 继续保持 render-only overlay，不进入 `SceneAsset`，也不变成普通世界实体；只从选中实体导出 overlay 数据。
  - 项目保存改为“项目 manifest 指向默认 `SceneAsset`”，不再把整个 `World` 内嵌进项目文件。
  - gate：scene/material roundtrip、层级/变换/材质绑定恢复、缺资产占位行为、选中对象 Gizmo 仍正常渲染。
- 里程碑 4：PBR 渲染与编辑器联通。
  - 将当前 `zircon_graphics` 的硬编码 shader 和 `tint + albedo` 流程升级为项目资产驱动的 PBR pipeline；shader 从 `assets/shaders/*.wgsl` 读取，材质从 `MaterialAsset` 解析。
  - `zircon_editor` 打开目录式项目后，能浏览 `assets/`、加载默认场景、导入 OBJ/glTF、保存 scene/material，并在 watcher 发现资源变化后自动刷新 viewport。
  - 现有 grid、selection、Gizmo 继续保留在 graphics/editor 视口链路，不上浮到资产系统，也不下沉到 UI crate。
  - gate：构造临时项目目录，打开后能自动加载默认场景、渲染模型与完整 PBR 材质，并叠加变换 Gizmo；修改 texture/material/scene 文件后 viewport 无需重启即可更新。

## Test Plan
- 单元测试：
  - project manifest 解析与回写
  - `res://` / `lib://` 规范化与越界拒绝
  - asset registry 的 add/update/delete/rename
  - material/scene TOML roundtrip
  - library cache hit/miss/invalidate
- 格式测试：
  - PNG/JPEG 解码
  - OBJ 与 glTF/GLB 导入到统一 `ModelAsset`
  - WGSL 编译成功与失败诊断
  - PBR material 参数和导入材质映射
- 集成测试：
  - 临时 `Project/assets` 扫描后生成 `Project/library`
  - 打开目录项目并实例化默认 scene 为 ECS world
  - 离屏渲染项目场景并断言输出帧非空
  - 修改 watcher 监控下的 texture/material/scene 文件，验证自动重导入和 viewport 刷新
- 验收命令：
  - `cargo test -p zircon_asset -p zircon_scene -p zircon_graphics -p zircon_editor`
  - `cargo test --workspace --locked`
  - 增加一个 headless project render regression，专门验证“目录项目 -> 资产解析 -> 场景实例化 -> PBR 渲染 -> Gizmo 叠加”的完整路径

## Assumptions
- 本计划按“Rust 原生优先”执行，所以 `FBX`、`ASTC`、`PVRTexTool` 不在首个可交付里程碑里承诺真实工作链，只承诺扩展点、错误模型和后续不破坏架构的接入位。
- 目录式项目是有意的 breaking change；旧 `*.zircon-project.json` 不是首期主路径，只在后续需要时再单独做迁移器。
- `SceneAsset` 与 `MaterialAsset` 都采用 TOML；`ShaderAsset` 采用原生 `.wgsl` 文本；library 永远是可删可重建缓存，不作为权威源文件。
- “完整 PBR”在本里程碑内严格限定为 glTF metallic-roughness 核心工作流，不包含 clearcoat、transmission、IOR 等扩展材质模型。
