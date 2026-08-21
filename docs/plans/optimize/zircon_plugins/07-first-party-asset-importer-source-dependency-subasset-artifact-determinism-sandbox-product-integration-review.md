---
related_code:
  - zircon_plugins/asset_importers
  - zircon_plugins/gltf_importer
  - zircon_plugins/obj_importer
  - zircon_plugins/texture_importer
  - zircon_plugins/audio_importer
  - zircon_plugins/opus_importer
  - zircon_plugins/ui_document_importer
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/src/builtin/runtime_modules/manifest.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_decode.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_meshopt.rs
  - zircon_runtime/src/asset/importer/ingest/import_obj.rs
  - zircon_runtime/src/asset/importer/ingest/import_texture.rs
  - zircon_runtime/src/asset/importer/ingest/import_sound.rs
  - zircon_runtime/src/asset/importer/ingest/import_data_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_v2_asset.rs
  - zircon_runtime/src/asset/importer/ingest/model_mesh_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
tests:
  - zircon_plugins/gltf_importer/runtime/src/tests.rs
  - zircon_plugins/gltf_importer/runtime/src/test_fixtures.rs
  - zircon_plugins/asset_importers/model/runtime/src/tests
  - zircon_plugins/texture_importer/runtime/src/tests
  - zircon_plugins/texture_importer/runtime/src/container/tests
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/05-shader-wgsl-family-importer-compiler-artifact-native-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Interchange/Core/Public/InterchangeSourceData.h
  - dev/UnrealEngine/Engine/Source/Runtime/Interchange/Core/Public/InterchangeTranslatorBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/EditorFramework/AssetImportData.h
  - dev/UnrealEngine/Engine/Source/Runtime/Interchange/Engine/Public/InterchangeAssetImportData.h
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Private/Texture/InterchangeTextureFactory.cpp
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Private/Audio/InterchangeAudioSoundWaveFactory.cpp
  - dev/godot/editor/import/editor_import_plugin.h
  - dev/godot/editor/import/editor_import_plugin.cpp
  - dev/godot/editor/import/resource_importer_texture.h
  - dev/godot/editor/import/resource_importer_texture.cpp
  - dev/godot/editor/import/resource_importer_wav.h
  - dev/godot/editor/import/resource_importer_wav.cpp
  - dev/godot/editor/import/3d/resource_importer_scene.h
  - dev/godot/editor/import/3d/resource_importer_scene.cpp
  - dev/bevy/crates/bevy_asset/src/loader.rs
  - dev/bevy/crates/bevy_gltf/src/loader/mod.rs
  - dev/bevy/crates/bevy_gltf/src/label.rs
  - dev/bevy/crates/bevy_image/src/image_loader.rs
  - dev/Fyrox/fyrox-impl/src/resource/gltf/mod.rs
  - dev/Fyrox/fyrox-impl/src/resource/gltf/animation.rs
  - dev/Fyrox/fyrox-impl/src/resource/gltf/material.rs
  - dev/Fyrox/fyrox-sound/src/buffer/streaming.rs
  - dev/Fyrox/fyrox-texture/src/loader.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/IESImporter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/AssetProcessors/NormalMapFilteringTexturePostprocessor.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 07 · First-Party Asset Importer Source、Dependency、Subasset、Artifact、Determinism、Sandbox 与 Product Integration 工程化差距

## 1. 结论

本轮逐文件覆盖`zircon_plugins/asset_importers`中除shader family外的audio/data/model/texture旧包，以及新的glTF、OBJ、Texture、Audio、Opus、UI Document六个first-party importer package，并向下核对Runtime内建importer、registry选择、ProjectAssetManager合并、first-party catalog与App组合。直接冻结138个Zircon文件、24,318行、824,699 bytes；其中七个插件根共115个文件、约17,519行、649,541 bytes。参考侧冻结Unreal、Godot、Bevy、Fyrox与Unity Graphics共25个文件、16,172行、645,781 bytes。

当前实现不是“只有壳”。Texture importer已有较完整的DDS/KTX1/KTX2/ASTC结构校验、metadata检查、cubemap/array manifest、离线mip和BC5路径；Runtime内建glTF已经能产生Texture、Material、Mesh、Scene、AnimationClip、AnimationSkeleton、Skin与inverse-bind subasset；新Audio importer能从内存快照解码WAV及Symphonia codec，并保留常见channel layout；registry也有COW generation、重复matcher拒绝、available优先和确定性排序。这些基础应保留。

但产品链目前存在更严重的“新插件比内建实现更差”问题。`AssetImporter::default()`注册版本2、优先级10的内建glTF，能生成真实动画；first-party catalog唯一链接的导入器却是版本1、优先级120的`gltf_importer`，它会遮蔽内建实现，并把每个动画写成文本为“not implemented yet”的`DataAsset`。同一稳定package还绕过`context.source_bytes`，用`gltf::import(&source_path)`重新读主文件及外部文件，导致导入结果不再绑定调用者读取的source snapshot，也没有把外部buffer/image登记为source dependency。

Texture importer当前tracked baseline还包含确定的源码级编译阻断：`downsample_box_color_pixel`多了未使用的`kaiser_normalizer`参数，`downsample_kaiser_color_pixel`使用未声明的同名变量，fallback调用又少传一个参数。与此同时KTX2 zstd/zlib解压上限直接取文件声明的`expected_length + 1`，没有独立的工程预算，恶意或损坏输入可驱动大内存分配。模型helper也在生成normal时用文件索引直接访问position，没有在边界前验证index。

新OBJ importer丢弃`tobj`返回的全部material，因而`.mtl`、材质绑定和纹理依赖完全消失；Texture cubemap/array manifest通过`std::fs::read`直接读取相邻文件但不写import dependency；Audio codec把整段音频展开成`Vec<f32>`，忽略loop/marker/loudness/多轨且把部分decode error直接跳过；Opus新旧两条高优先级路径都只是diagnostic importer，没有实际decoder。旧`asset_importers`又同时保留声明-only、diagnostic-only和少量真实STL/PLY/DXF/Data实现，形成未完成的双轨迁移。

Plugins01继续拥有native ABI、load-before-admission与dist空壳父问题；Plugins06拥有30包catalog/profile closure；Runtime04拥有通用asset identity、registry transaction、artifact/load/reload；Runtime08B/08C拥有runtime audio/animation；Editor04拥有导入UI与reimport工作流。本文只拥有具体first-party importer family的source snapshot、外部source dependency、subasset identity、格式语义、资源预算、确定性、内建/插件实现收敛和格式级产品资格。本轮登记 **5项P0、72项P1和16项P2**；只写review与重构计划，不修改production/tests。

## 2. 物理范围与证据边界

### 2.1 Zircon冻结范围

| 范围 | 文件 / 行 / bytes | 当前事实 |
|---|---:|---|
| legacy `asset_importers`，排除shader | 34 / 2,935 / 109,871 | Audio/Texture多为声明聚合；Model有STL/PLY/DXF；Data有TOML/JSON/YAML/XML |
| `gltf_importer` | 10 / 3,171 / 110,844 | 稳定、partial、高优先级真实importer，但动画仍是placeholder |
| `obj_importer` | 7 / 746 / 28,432 | 稳定、partial、mesh-only，丢弃materials |
| `texture_importer` | 43 / 8,732 / 323,386 | 稳定、partial，container验证较深，但当前mip kernel有编译阻断 |
| `audio_importer` | 7 / 820 / 32,381 | 稳定、partial，WAV与codec真实，Opus为diagnostic-only |
| `opus_importer` | 7 / 518 / 21,146 | experimental，高优先级diagnostic-only，没有decoder bridge |
| `ui_document_importer` | 7 / 597 / 23,481 | 稳定、partial，解析`.zui`但无外部reference dependency投影 |
| Runtime/caller/product补充 | 23 / 6,799 / 175,158 | 内建importer、registry、project合并、catalog、App与profile |
| Zircon合计 | 138 / 24,318 / 824,699 | SHA-256 manifest `ef93fc480de91549c85b7a999c108dbafb7abd67d8558d14cd3c9b3b6a47fc26` |

上述fingerprint按相对路径不区分大小写排序，对每个文件计算SHA-256，再对`path|hash`的LF连接串计算SHA-256。它是本报告的currentness边界，不是编译或产品通过证据。

### 2.2 参考冻结范围

| 参考 | 本轮核对的机制 | 不外推的内容 |
|---|---|---|
| Unreal Interchange/AssetImportData | source content hash、多source file、translator/factory分层、texture/audio payload与reimport数据 | 不把UObject、DDC或具体factory数量照搬为Zircon设计 |
| Godot ResourceImporter | import options、generated files、platform variants、texture压缩、WAV loop/trim/normalize、scene animation处理 | Godot纹理streaming本身也标注未完成，不作为完成证据 |
| Bevy Asset/GLTF/Image | reader快照、loader settings、normal/loader dependency、labeled subasset context、异步外部buffer/image加载 | Bevy标签同样可按index生成，不能单凭存在就证明稳定身份 |
| Fyrox GLTF/Texture/Sound | ResourceIo、外部resource request、真实glTF animation/morph channel、import options、长音频streaming buffer | 不复制其scene graph和resource manager内部结构 |
| Unity Graphics | ScriptedImporter的main/subobject发布、TextureImporter平台BC7与normal-map预处理 | 本地镜像不是完整Unity Editor/AssetDatabase，不能用于通用导入生命周期结论 |
| 参考合计 | 25 / 16,172 / 645,781；fingerprint `8875a71310992fca2c2e89e74ae931c3ff441f0ba98293a49c573c9fcfa82e6d` | 路径存在和代码规模均不等于Zircon已实现或性能更优 |

### 2.3 动态证据边界

本轮没有运行Cargo compile/tests。报告是review-only，且编译阻断可由同一函数内的未绑定标识符与参数arity静态确定；未把源码判断写成“cargo check已失败”。也未运行真实Editor import/reimport、跨平台texture cook、恶意container、长音频、NativeDynamic、OS sandbox或性能基准。所有“完成”均只指本报告的静态审查范围完成。

## 3. 当前实际数据流

```text
Project source path
  -> caller fs::read(main source) -> AssetImportContext { source_path, source_bytes, settings }
  -> AssetImporter::default() built-ins
  -> merge plugin importers
       -> available > diagnostic
       -> priority > suffix length > registration slot
  -> chosen handler
       -> some handlers consume source_bytes
       -> glTF/OBJ/manifest handlers reopen source_path/relative files
  -> AssetImportOutcome { root + labeled entries + asset URI dependencies }
       -> no distinct source-file dependency/hash graph
       -> no importer recipe/toolchain/platform artifact identity here
```

主要问题不是缺少另一个parser，而是`source snapshot`、`source dependency`、`asset dependency`、`subasset identity`、`derived artifact`和`product provider`六类事实被压在同一个同步handler返回值附近，却没有共同的导入事务与资格合同。

## 4. 可保留基础

1. `AssetImporterRegistry`的COW generation使已发布reader不会观察半更新matcher index。
2. Registry拒绝同ID和同priority同matcher冲突，并明确full suffix优先于普通extension。
3. `AssetImportOutcome`允许root与多个labeled entry各自携带asset dependency、diagnostic和migration report。
4. Runtime内建glTF使用调用者的`source_bytes`解析主文档，并对required extension、meshopt、WebP、hierarchy cycle和animation sampler做了实质校验。
5. Runtime内建glTF已经生成真实AnimationClip/Skeleton，不应被回退到插件placeholder。
6. 两套glTF均已建立Texture、Material、Mesh、Primitive、Node、Scene、Skin和inverse-bind subasset骨架。
7. Texture container对DDS/KTX1/KTX2/ASTC的header、range、alignment、metadata与部分supercompression有大量负向测试。
8. Texture importer已区分image/container/PSD/cubemap/array/native diagnostic descriptor，并有normal convention、mip与BC5模块。
9. Audio importer从context bytes解码，能识别常见speaker mask并拒绝不完整multichannel frame。
10. Legacy Model的共享mesh helper有index验证，证明仓内已有可复用的局部admission实践。
11. UI importer统一使用`.zui`并拒绝旧suffix，避免继续扩大格式分叉。
12. Manifest由Rust declaration生成，package/runtime/dist metadata投影有结构基础。

## 5. P0：阻断级正确性、安全与产品回退

| ID | 证据 | 影响 | 必须重构 | 验收门 |
|---|---|---|---|---|
| IMP-P0-001 | `texture_importer/runtime/src/mipgen/kernel.rs`中Box函数多参数、Kaiser函数缺参数且fallback arity错误 | 当前tracked stable texture package存在源码级编译阻断，不能形成可链接provider | 修正参数所有权并把两种filter纳入package compile与真实import test | 真实package feature矩阵build；Box/Kaiser测试均执行；禁止用cfg移除失败路径 |
| IMP-P0-002 | 内建glTF v2/priority10生成真实AnimationClip/Skeleton；catalog链接的stable plugin v1/priority120生成`DataAsset` placeholder | 启用第一方插件反而降低功能，animation output kind声明也与真实输出不一致 | 删除双实现或让插件委托唯一canonical glTF implementation；冻结descriptor/version/output schema | 同一fixture在builtin/source/native得到完全一致的typed subasset graph；禁止placeholder进入Ready |
| IMP-P0-003 | glTF/OBJ mesh normal生成以文件index直接索引position，缺少导入边界的index admission | parser接受或损坏的越界index可触发panic/进程终止，而不是typed import failure | 在任何cook/normal/tangent访问前验证index、attribute count和primitive topology，panic boundary只作最后保险 | malformed corpus/property/fuzz覆盖越界、溢出、NaN、空attribute；全部返回typed error且零发布 |
| IMP-P0-004 | plugin glTF重新读取`source_path`和任意relative URI，只检查`exists()`；没有canonical containment或dependency记录 | main snapshot与实际解析可竞态分叉，`../`可越过project root，外部内容可未经准入进入artifact | 建立只读SourceBroker，所有主/外部source按canonical project/mount policy打开、hash、预算并登记 | source在导入中变化、symlink/junction、parent traversal、absolute/scheme URI均有fail-close测试与source receipt |
| IMP-P0-005 | KTX2 zstd/zlib解压`.take(expected_length + 1)`，`expected_length`来自输入且无独立上限 | 小输入可声明巨大输出并驱动OOM/长时阻塞，影响Editor/CI/asset worker可用性 | 统一ImportBudget限制input/output bytes、ratio、dimensions、levels、time与allocation，最好隔离worker | zip-bomb/huge-length/slow decode在预算内终止；RSS/time上限和零partial artifact有自动证据 |

## 6. P1：Importer Authority、Selection 与 Product Closure

| ID | 当前差距 | 需要重构 |
|---|---|---|
| IMP-P1-001 | Runtime内建与插件各维护glTF/OBJ/Texture/UI/Data等重叠实现 | 每个format family只保留一个canonical implementation crate，builtin只装配默认provider |
| IMP-P1-002 | 两套glTF的importer version、output kind与行为不同 | 建立ImporterContractId和OutputSchemaVersion，版本变更绑定migration与golden |
| IMP-P1-003 | `active_importer_registry`对plugin merge错误使用`let _` | merge返回逐importer typed receipt，任一required registration失败阻止Ready |
| IMP-P1-004 | 选择结果只返回handler，未形成持久决策记录 | 记录source、matcher、importer ID/version、plugin generation、priority与fallback reason |
| IMP-P1-005 | required capability主要停留在descriptor，未绑定格式行为资格 | Available必须同时满足provider、capability admission、implementation和qualification receipt |
| IMP-P1-006 | builtin/plugin descriptor与实现手写并可漂移 | 从唯一ImporterSpec生成descriptor、manifest、catalog row、output kinds与test matrix |
| IMP-P1-007 | `stable`可与`partial`、placeholder和compile blocker并存 | maturity gate要求build、behavior、failure、artifact与product E2E全部通过 |
| IMP-P1-008 | SourceTemplate/LibraryEmbed/NativeDynamic没有格式级行为parity | 同一corpus比较entry graph、diagnostic、dependency、artifact hash与failure code |
| IMP-P1-009 | OBJ/Texture/Audio/Opus/UI Document均不在通用runtime catalog | 由Plugins06生成完整provider closure；本文消费resolution receipt而不复制catalog owner |
| IMP-P1-010 | 默认runtime/editor profile没有明确importer policy | profile显式列required/optional format set、fallback、worker/sandbox和target artifact策略 |

## 7. P1：Source Snapshot、Dependency、Sandbox 与 Determinism

| ID | 当前差距 | 需要重构 |
|---|---|---|
| IMP-P1-011 | `AssetImportContext`只有单一路径和主文件bytes | 引入SourceSnapshot、SourceHandle、resolver、content hash、size、mtime/revision与mount identity |
| IMP-P1-012 | plugin glTF主文档解析不消费context bytes | parser只能读取broker提供的immutable snapshot；path仅作logical origin和诊断 |
| IMP-P1-013 | glTF external buffer/image由第三方helper直接读盘 | 所有URI经context resolver打开，并将实际bytes/hash加入source dependency set |
| IMP-P1-014 | relative URI只做join/exists | percent decode、separator、case、symlink/junction、UNC、absolute与scheme统一admission |
| IMP-P1-015 | outcome dependencies只表达AssetUri，不表达源文件依赖 | 区分SourceDependency、AssetDependency、ToolDependency与GeneratedArtifactDependency |
| IMP-P1-016 | cubemap/array manifest直接`std::fs::read` | 通过SourceBroker读取并继承project root、trust、budget、cancel和hash策略 |
| IMP-P1-017 | manifest source reference只写进Texture DTO | 每个face/layer成为有角色和顺序的source dependency，任一变化触发reimport |
| IMP-P1-018 | source可能在initial read与外部read之间变化 | transaction冻结dependency revision；变化则取消并重试，不允许混合generation |
| IMP-P1-019 | glTF/OBJ subasset label依赖数组顺序 | 定义稳定SubassetKey：显式ID/规范name+semantic path+collision ordinal，并提供重映射 |
| IMP-P1-020 | 没有导入确定性receipt | 记录sorted dependency hashes、settings、toolchain、target、CPU feature和output graph hash |

## 8. P1：Model、glTF、OBJ 与 Legacy Geometry

| ID | 当前差距 | 需要重构 |
|---|---|---|
| IMP-P1-021 | plugin glTF animation仍是文本placeholder | 复用Runtime真实clip/skeleton转换，补morph-weight channel或显式拒绝并降级能力 |
| IMP-P1-022 | Runtime glTF支持meshopt/WebP/更多material extension，plugin实现落后 | extension matrix由canonical decoder拥有，unsupported required extension必须fail-close |
| IMP-P1-023 | morph target position/normal/tangent有数据但animation weights未闭合 | 贯通MorphTargetSet、weight track、mesh binding、cook和runtime evaluation |
| IMP-P1-024 | 同mesh多node skin只保留第一个 | skin binding归node/instance，mesh geometry与skin instance解耦 |
| IMP-P1-025 | attribute count、index和joint/weight关联校验不完整 | 在构建vertex/cook前做完整accessor cardinality、range、finite与normalization验证 |
| IMP-P1-026 | 非Triangles primitive一律拒绝 | 明确point/line/strip/fan策略；支持的拓扑规范转换，不支持的形成typed diagnostic |
| IMP-P1-027 | coordinate handedness、up axis、unit与negative scale没有显式recipe | 导入设置和artifact identity记录coordinate conversion，scene/animation/skin统一变换 |
| IMP-P1-028 | plugin primitive未保留完整tangent/color语义 | vertex schema覆盖tangent sign、COLOR_n、TEXCOORD_n、JOINTS_n/WEIGHTS_n或明确限额 |
| IMP-P1-029 | material只映射部分metallic-roughness字段 | 建立glTF extension到canonical material graph/instance的版本化映射和fallback policy |
| IMP-P1-030 | default PBR shader URI硬编码 | 由render profile/material compiler解析qualified shader contract并进入dependency/artifact key |
| IMP-P1-031 | texture color space按通用TextureAsset处理 | base-color/emissive与normal/metallic/roughness/occlusion按slot semantic设置色彩空间与swizzle |
| IMP-P1-032 | camera/light/extras/variants/LOD/collision等scene语义未投影 | 建立支持矩阵和preserve-unknown策略，不能静默丢弃可见源内容 |
| IMP-P1-033 | OBJ丢弃`tobj`返回的materials | 解析MTL、material assignment、texture options与所有外部依赖；缺失材质有typed policy |
| IMP-P1-034 | OBJ object/group/smoothing与subasset identity不足 | 保留group/object/material/smoothing边界，生成稳定mesh sections与reimport映射 |

## 9. P1：Texture Decode、Cook、Platform Artifact 与 Scalability

| ID | 当前差距 | 需要重构 |
|---|---|---|
| IMP-P1-035 | 通用image路径统一转RGBA8 | 保留HDR/float/16-bit/gray/channel metadata，按consumer recipe选择working format |
| IMP-P1-036 | ICC、gamma、EXIF orientation、alpha mode策略不完整 | 建立色彩管理与orientation/alpha canonicalization，并把转换版本放入artifact key |
| IMP-P1-037 | PSD只输出flattened RGBA | 明确flatten-only能力或支持layer/group/mask/text subasset，声明必须与行为一致 |
| IMP-P1-038 | container多为原payload保留，没有target平台派生物矩阵 | cook生成BC/ASTC/ETC/Basis/未压缩目标artifact及platform fallback |
| IMP-P1-039 | requested BC5在无encoder路径时可能保留原DXT5 payload | descriptor必须报告actual format；请求无法满足应fail/degrade并写receipt，禁止静默错配 |
| IMP-P1-040 | Rust transcode只有BC5 | 建立可替换compressor backend、质量档、determinism与license/toolchain identity |
| IMP-P1-041 | KTX2 BasisLZ可通过结构校验但没有universal transcode | 要么接入Basis transcoder并按target产物化，要么标记Unavailable而非交给未知consumer |
| IMP-P1-042 | DDS/KTX字符串format不等于renderer支持 | importer admission查询target format capability，unsupported payload不能发布Ready asset |
| IMP-P1-043 | mip只覆盖RGBA8 D2/cube且filter集合有限 | 支持alpha coverage、normal renormalize、roughness correction、HDR和array/volume规则 |
| IMP-P1-044 | equirect cubemap为串行逐texel CPU全内存转换 | 任务化/并行化并设质量、采样、seam、cancel、time/RSS预算和基准 |
| IMP-P1-045 | cubemap face约定与orientation合同不够明确 | 冻结face order、handedness、rotation、edge seam和golden environment corpus |
| IMP-P1-046 | texture array主要验证尺寸/format | 处理mip count、color space、alpha、usage、sampler与layer semantic一致性 |
| IMP-P1-047 | dimension/mip/layer/decoded bytes没有统一预算 | 解析header后、allocation前执行checked arithmetic与ImportBudget admission |
| IMP-P1-048 | 每level解压限制不等于transaction总预算 | 累计compressed/decompressed/temporary/output bytes与ratio，跨level共享budget |
| IMP-P1-049 | 大图mip/transcode没有job progress/cancel | 接入bounded worker pool、cooperative cancellation、阶段progress和terminal receipt |
| IMP-P1-050 | texture cook recipe未绑定DDC/artifact身份 | key包含source graph、settings、compressor/version、target GPU family、quality和schema |

## 10. P1：Audio、Codec、Streaming 与 Metadata

| ID | 当前差距 | 需要重构 |
|---|---|---|
| IMP-P1-051 | codec导入把完整文件展开为interleaved `Vec<f32>` | 区分short resident clip与long streaming artifact，限制frame/duration/channel/bytes |
| IMP-P1-052 | `DecodeError`被`continue`跳过 | 区分recoverable packet damage与fatal corruption，记录位置/count并由policy决定是否拒绝 |
| IMP-P1-053 | loop、cue/marker、BPM/bar、loudness与gapless metadata丢失 | 定义AudioImportMetadata并贯通authoring、cook和runtime playback |
| IMP-P1-054 | 只选择第一个非空track | 设置中显式选择track/language/role，multi-track container生成稳定subasset或拒绝 |
| IMP-P1-055 | channel layout映射有基础但ambisonics/object audio等缺策略 | 建立layout capability matrix、downmix/upmix recipe与speaker-order golden |
| IMP-P1-056 | 无resample、normalization、codec/bitrate与platform stream cook | 将decode与cook分层，生成平台codec、seek table、chunk和streaming metadata |
| IMP-P1-057 | Audio与Opus包各注册Opus diagnostic，后者仍无decoder | 只有真实backend成功admission时发布一个Opus provider，否则保持Unavailable且无重复matcher |
| IMP-P1-058 | 音频测试集中于tiny WAV和单个Bevy OGG | 增加多codec、多layout、metadata、损坏packet、长流、seek/loop/gapless与budget corpus |

## 11. P1：Data、UI Document 与 Legacy Migration

| ID | 当前差距 | 需要重构 |
|---|---|---|
| IMP-P1-059 | Data importer把结构化格式统一降为通用JSON值 | schema-aware DataAsset保留类型、source map、validation、unknown field与reference语义 |
| IMP-P1-060 | XML转中立JSON会损失namespace/attribute order/mixed content语义 | 要么保留lossless XML DOM/text spans，要么明确只接受受限schema并严格验证 |
| IMP-P1-061 | `.zui`解析出的资源引用未进入outcome dependency | loader返回typed reference set，context解析并写入每个View/Style/Component entry |
| IMP-P1-062 | UI document只有parser版本，没有import schema migration receipt | 绑定source schema、migration chain、diagnostic span与save/reopen roundtrip |
| IMP-P1-063 | legacy Audio/Texture是声明-only，Model/Data又有真实实现 | 为每个legacy provider标注delegate/migrating/deprecated/removed，禁止模糊共存 |
| IMP-P1-064 | legacy/new priority与matcher并存但无迁移计划 | 硬切唯一provider，保留版本化settings/subasset remap和项目升级工具后删除旧路径 |

## 12. P1：Verification、Failure Evidence 与 Operations

| ID | 当前差距 | 需要重构 |
|---|---|---|
| IMP-P1-065 | 没有Khronos glTF Sample Models/Validator等版本化conformance corpus | 记录license/snapshot/expected支持矩阵，必选/可选extension分别验收 |
| IMP-P1-066 | parser负向测试多为手工样本，缺持续fuzz | 对glTF/OBJ/DDS/KTX/PSD/audio/XML/ZUI建立libFuzzer/proptest与crash corpus归档 |
| IMP-P1-067 | 缺root/subasset/dependency的semantic golden | golden记录typed graph与canonical hash，不只比较成功或entry数量 |
| IMP-P1-068 | 缺reimport后rename/reorder/delete subasset引用保持测试 | 用真实项目验证稳定ID、orphan清理、override保留、undo/save/reopen |
| IMP-P1-069 | 跨OS/filesystem/toolchain确定性未验证 | Windows/Linux/macOS对同source/settings比较dependency与artifact digest，解释允许差异 |
| IMP-P1-070 | 没有规模、RSS、decode ratio和取消延迟门 | 建立model/texture/audio分层workload，统计p50/p95/max与budget rejection |
| IMP-P1-071 | package unit test不证明标准产品可导入 | 用真实Client/Editor build feature启动、选择provider、导入、cook、load和render/playback |
| IMP-P1-072 | diagnostics缺source stage、dependency、budget与provider identity | 统一ImportReceipt/Diagnostic记录operation、stage、source span、importer、generation和artifact |

## 13. P2：长期能力

| ID | 能力 | 目标 |
|---|---|---|
| IMP-P2-001 | Import graph inspector | 在Editor查看source、asset、subasset、tool与artifact依赖图 |
| IMP-P2-002 | Provenance diff | 比较两次import的source/settings/toolchain/output差异并定位原因 |
| IMP-P2-003 | Subasset remap assistant | source重构后交互式确认stable ID映射与orphan处理 |
| IMP-P2-004 | Incremental subasset import | 只重建受变更dependency影响的mesh/material/texture/clip |
| IMP-P2-005 | Isolated import worker | 将不可信parser/codec放入限权、限内存、可终止的进程 |
| IMP-P2-006 | Distributed import farm | 以完整recipe和source CAS调度远端确定性cook |
| IMP-P2-007 | GPU texture processing | 在有确定性fallback和同质golden下加速mip/cubemap/compression |
| IMP-P2-008 | Quality profile library | Mobile/Desktop/Cinematic等profile生成可审计recipe而非散落默认值 |
| IMP-P2-009 | Extension negotiation | glTF/vendor extension通过版本化plugin贡献且不破坏core parser安全边界 |
| IMP-P2-010 | Platform qualification farm | GPU family、driver、OS与codec组合形成可追溯artifact acceptance |
| IMP-P2-011 | Import compatibility matrix | 展示格式版本、feature、限制、fallback与最近qualification evidence |
| IMP-P2-012 | Content license/SBOM capture | 外部工具、codec、模型和texture内容的许可证/provenance随artifact记录 |
| IMP-P2-013 | Privacy-safe importer telemetry | 汇总format/stage/failure/budget，不上传原始路径或内容 |
| IMP-P2-014 | Curated sample corpus manager | 固定公开、合成、回归、恶意与性能fixture版本和license |
| IMP-P2-015 | Live dependency watch | source graph变化后合并去抖、取消旧generation并原子发布新结果 |
| IMP-P2-016 | Project migration dashboard | 统计legacy importer/settings/subasset映射并阻止未完成迁移的release |

## 14. 目标合同

### 14.1 `ImportRequest`

至少包含：`operation_id`、`project_id`、`build_set_id`、logical source URI、main `SourceSnapshot`、importer contract/version、settings schema/value、target platform/GPU/audio profile、budget、deadline/cancel token与trust principal。

### 14.2 `SourceBroker`

只通过logical URI解析project/mount source；每次open返回canonical identity、content hash、size、revision和immutable reader。External URI解析必须在打开前完成scheme/containment/symlink/trust检查，并自动登记source dependency。

### 14.3 `ImportProduct`

每个root/subasset具有稳定`SubassetKey`、typed asset、asset dependencies、source dependencies、diagnostics与schema version。Import结束前验证唯一key、依赖闭包、输出kind、预算与reference完整性，在staging area一次性发布。

### 14.4 `DerivedArtifactRecipe`

包含完整source graph digest、settings、importer/toolchain/schema、target、quality、platform capability与determinism policy；输出包含artifact digest、actual format、size、runtime reader version和qualification receipt。Runtime04拥有通用artifact store与load/reload，本文只定义格式侧必须提供的recipe输入。

### 14.5 `ImporterQualificationReceipt`

至少绑定source revision、package/artifact、provider form、test corpus snapshot、supported feature matrix、failure/budget/soak/determinism结果和expiry。没有receipt只能是Experimental/Unavailable，不得因manifest写`stable`而进入shipping Ready。

## 15. 分阶段重构

### M0 · Truth Freeze 与止血

1. 修复并验证Texture mip kernel编译阻断。
2. 禁止catalog glTF插件遮蔽更完整内建实现；在收敛前将其从stable/Ready降级。
3. 给mesh index、KTX2解压和external path加fail-close边界。
4. 输出当前每个extension实际选择的handler/version/status，不再吞merge错误。

### M1 · Canonical Importer 与 Source Graph

1. 建立ImporterSpec与SourceBroker。
2. 将glTF/OBJ/texture manifest所有读取迁移到immutable source snapshot。
3. 区分source/asset/tool/artifact dependency并持久化hash graph。
4. 硬切builtin/plugin重复实现，保留唯一format implementation。

### M2 · Stable Subasset 与 Model Fidelity

1. 冻结SubassetKey和reimport remap协议。
2. 合并Runtime真实glTF animation、extension与material能力。
3. 完成OBJ MTL/material/texture dependency。
4. 补coordinate、skin instance、morph animation与scene semantic合同。

### M3 · Texture/Audio Platform Cook

1. 建立完整texture/audio recipe和target artifact matrix。
2. 接入bounded worker、cancel/progress和RSS/time/decode ratio预算。
3. 关闭requested/actual format错配、Basis/Opus假provider和长音频全驻留。
4. 让runtime reader按actual artifact schema验收，而非相信字符串format。

### M4 · Data/UI、Legacy Cutover 与 Product Closure

1. Data/UI返回lossless schema/reference/source map。
2. 迁移legacy importer settings和stable ID后删除双轨provider。
3. 由Plugins06生成完整catalog/profile closure，Plugins01关闭native parity。
4. Editor04消费统一ImportReceipt完成导入、reimport、冲突和失败UX。

### M5 · Qualification 与竞争性性能

1. 建立conformance/fuzz/golden/reimport/cross-platform/product E2E矩阵。
2. 以相同source、输出质量、target与硬件比较CPU、RSS、I/O、artifact size和runtime quality。
3. 只有正确性、failure、determinism、artifact与产品闭环通过后，才允许声称优于Unreal或其他引擎。

## 16. 验收矩阵

| Gate | 验收内容 |
|---|---|
| G01 | Texture importer所有真实features可编译，Box/Kaiser路径均执行 |
| G02 | 每个extension只有一个canonical Ready provider，merge错误不可丢失 |
| G03 | builtin/source/native同fixture输出同一typed graph或有明确版本迁移 |
| G04 | glTF animation生成真实clip/skeleton，不存在placeholder Ready asset |
| G05 | malformed mesh/index/accessor只返回typed error，不panic、不发布 |
| G06 | 所有外部source经SourceBroker，path escape/symlink/竞态fail-close |
| G07 | source dependency hash变化能精确触发reimport |
| G08 | subasset reorder/rename/delete后的引用与override按policy保留或迁移 |
| G09 | OBJ MTL/material/texture依赖和丢失诊断完整 |
| G10 | texture actual format与runtime capability一致，requested/actual不静默分叉 |
| G11 | decompression、dimension、mip、layer、time与RSS预算在allocation前执行 |
| G12 | 长音频使用streaming artifact，loop/marker/layout/seek metadata可验证 |
| G13 | Opus只有真实decoder backend通过后才Available |
| G14 | Data/XML/ZUI保留所声明的schema/reference语义与source span |
| G15 | legacy/new importer没有未解释的matcher/priority双轨 |
| G16 | conformance、fuzz、semantic golden、reimport与cross-platform determinism进入required CI |
| G17 | 标准Editor profile完成import→cook→load→render/playback→reimport闭环 |
| G18 | ImportReceipt绑定BuildSet、provider、source graph、settings、target和artifact digest |
| G19 | failure/cancel/OOM/worker crash不留下partial registry/artifact或旧新混合generation |
| G20 | 性能报告同时给出质量、correctness、CPU、RSS、I/O、artifact与统计口径 |

## 17. Owner与依赖

| Owner | 本文消费/交付 |
|---|---|
| O00 Capability Truth | maturity/Available必须绑定格式级qualification |
| O01 BuildSet/Artifact | importer/toolchain/target进入recipe与receipt |
| O02 Lifecycle/Operation | bounded job、cancel、worker terminal与shutdown |
| O03 Schema/Identity | ImporterContract、SubassetKey、settings/output migration |
| O04 Source/Artifact | SourceBroker、dependency graph与derived artifact；store归Runtime04 |
| O05 Transaction | staging、atomic publication、failure零partial结果 |
| O07 Budget/Evidence | parse/decode/cook预算、corpus、benchmark与receipt |
| O09 Graphics | texture actual format、material/render reader capability |
| O10 UI | `.zui`typed reference与runtime UI reader |
| O11 Qualification | conformance/fuzz/golden/product E2E与性能声明 |
| O13 Authoring | Editor04 import/reimport/override/conflict UX |
| O14 Currentness | 138-file与25-reference fingerprint漂移触发重审 |
| O15 Security | untrusted source、path/symlink、codec/parser、worker sandbox |

关键依赖顺序为：Plugins01/06与Runtime04公共合同 → SourceBroker/ImporterSpec → canonical format implementations → platform cook/runtime readers → Editor workflow → qualification/performance。不得先补更多format按钮或manifest状态，再回头处理source identity、sandbox和artifact真值。

## 18. 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| 115个first-party importer文件逐文件审查 | review_complete | 2026-08-19 | 七个根约17,519行、649,541 bytes；排除shader importer |
| Runtime/caller/product交叉审查 | review_complete | 2026-08-19 | 合计138文件、24,318行、824,699 bytes；fingerprint `ef93fc480de91549c85b7a999c108dbafb7abd67d8558d14cd3c9b3b6a47fc26` |
| 五参考引擎定向对照 | review_complete | 2026-08-19 | 25文件、16,172行、645,781 bytes；fingerprint `8875a71310992fca2c2e89e74ae931c3ff441f0ba98293a49c573c9fcfa82e6d` |
| P0/P1/P2与目标合同 | review_complete | 2026-08-19 | 5 / 72 / 16；20项验收门 |
| Production重构 | pending | - | 本轮未修改production/tests，未运行Cargo或产品E2E |
