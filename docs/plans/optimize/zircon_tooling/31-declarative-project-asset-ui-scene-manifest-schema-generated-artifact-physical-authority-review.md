---
related_code:
  - .gitignore
  - .github/workflows/ci.yml
  - tools/check_conventions.py
  - tools/zircon_build.py
  - tools/zircon_build_asset_staging.py
  - tools/zircon_build_zui_assets.py
  - tools/zircon_export/plugin_validate_distribution_zui_assets.py
  - tools/zircon_export/plugin_build_asset_pack.py
  - zircon_runtime_interface/src/ui/v2/asset.rs
  - zircon_runtime_interface/src/ui/v2/compiled.rs
  - zircon_runtime_interface/src/ui/template/asset/document.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/cache/cache_key.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/manifest.rs
  - zircon_runtime/src/asset/assets/ui/document_loader.rs
  - zircon_runtime/src/ui/v2/loader.rs
  - zircon_runtime/src/ui/v2/compiler.rs
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/persistent.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/header.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/validate.rs
  - zircon_runtime/src/asset/project/manifest/project_manifest.rs
  - zircon_runtime/src/asset/project/manifest/load.rs
  - zircon_runtime/src/asset/project/manifest/save.rs
  - zircon_runtime/src/asset/assets/project_document/codec.rs
  - zircon_runtime/src/asset/assets/project_document/scene.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/registry/persistence.rs
  - zircon_runtime/src/core/framework/animation/asset/binary.rs
  - zircon_runtime_interface/src/export/preset.rs
  - zircon_runtime_interface/src/serialization/versioned_schema.rs
  - zircon_editor/src/core/settings/io.rs
  - zircon_runtime_interface/src/project/template_pack/embedded.rs
  - examples/vampire/zircon-project.toml
  - examples/woc/zircon-project.toml
  - templates/projects/renderable-empty/zircon-project.toml
  - templates/projects/renderable-empty/.zircon/settings.toml
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/woc/assets/scenes/bootstrap.scene.toml
  - examples/woc/assets/scenes/eastbrook_mvp.scene.toml
  - templates/projects/renderable-empty/assets/scenes/main.scene.toml
  - examples/woc/tools/m8_scene_codegen.mjs
  - examples/woc/contracts/m8_eastbrook_scene.generated.json
  - examples/woc/scripts/woc_game/woc_m4_power_echo_heal_state_tests.zrp
  - examples/woc/tools/m4_power_echo_heal_source_check.mjs
  - examples/vampire/scripts/vampire_game/bin/.zr_cli_manifest
  - examples/woc/scripts/woc_game/bin/.zr_cli_manifest
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/package.rs
  - zircon_editor/assets/ui/theme/editor_material.zui
  - examples/vampire/assets/models/kenney_graveyard/character-vampire.glb.zmeta
  - tools/session_tray/gen/schemas/acl-manifests.json
  - tools/session_tray/gen/schemas/capabilities.json
  - tools/session_tray/gen/schemas/desktop-schema.json
  - tools/session_tray/gen/schemas/windows-schema.json
  - zircon_hub/gen/schemas/acl-manifests.json
  - zircon_hub/gen/schemas/capabilities.json
  - zircon_hub/gen/schemas/desktop-schema.json
  - zircon_hub/gen/schemas/windows-schema.json
tests:
  - tools/tests/test_zircon_build_zui_asset_owner_boundaries.py
  - tools/tests/test_zircon_build_asset_staging_owner_boundaries.py
  - tools/tests/test_plugin_validate_distribution_zui_asset_owner_boundaries.py
  - tools/tests/test_zircon_build_plugin_carriers.py
  - zircon_runtime/src/ui/tests/v2_asset/asset_loading.rs
  - zircon_runtime/src/ui/tests/v2_asset/file_cache.rs
  - zircon_runtime/src/ui/tests/asset_package_validation.rs
  - zircon_runtime/src/ui/tests/asset_dependency_index.rs
  - zircon_runtime/src/asset/tests/project/manifest.rs
  - zircon_runtime/src/asset/tests/project/zmeta/schema_v7.rs
  - zircon_editor/src/core/settings/tests/persistence.rs
  - zircon_editor/src/core/project/tests/template_creation.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
  - docs/plans/optimize/zircon_app/07-renderable-empty-project-template-create-import-render-export-evidence-product-integration-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md
  - docs/plans/optimize/zircon_tooling/17-repository-content-source-set-ignore-generated-vendor-license-distribution-review.md
  - docs/plans/optimize/zircon_tooling/27-version-domain-schema-compatibility-support-window-migration-deprecation-upgrade-downgrade-review.md
reference_engines:
  - dev/UnrealEngine/Templates/TP_ThirdPerson/TP_ThirdPerson.uproject
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/AssetRegistryState.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataCacheKey.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildDefinition.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp
  - dev/godot/scene/resources/resource_format_text.cpp
  - dev/godot/core/io/resource_uid.cpp
  - dev/godot/editor/file_system/editor_file_system.cpp
  - dev/bevy/crates/bevy_asset/src/meta.rs
  - dev/bevy/crates/bevy_asset/src/processor/process.rs
  - dev/bevy/crates/bevy_asset/src/processor/log.rs
  - dev/Fyrox/fyrox-core/src/visitor/mod.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/Runtime UI Resources/RuntimeDebugWindow_PanelSettings.asset
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/Runtime UI Resources/RuntimeDebugWindow_PanelSettings.asset.meta
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 31 · Declarative Project、Asset、UI、Scene、Manifest、Schema 与 Generated Artifact 物理权威审查

## 1. 结论

Zircon不是完全没有工程级持久格式。58个`.zmeta`全部为format v7，reader使用`deny_unknown_fields`并先分类旧版、未来版和字段形状；14个`.zmaterial`与6个`.zshader`全部显式写v2，material拒绝未知顶层字段，shader还按kind校验required/forbidden字段；4个`.zranim`有`ZRANIM01` magic、binary version和kind；唯一`.zpreset`使用严格`$zircon` envelope、SchemaId和VersionedSchema。ZUI侧295个production `.zui`均为v2，295个production ID唯一，1,107个`res://...zui`引用在当前tracked集合中全部可解析；compiled UI又已经把source/compiler/package version、依赖、cache key和artifact fingerprint纳入包报告。这些基础必须保留。

缺口在于这些局部格式没有形成一个可执行的“声明式工程控制面”。仓库没有中央Format Catalog来说明扩展名、真实codec、SchemaId、owner、authoritative/derived角色、reader/writer范围、unknown-field策略、生成器和发布位置；相同产品路径甚至出现名为`settings.toml`、内容却是JSON VersionedSchema envelope的长期合同。局部Rust reader、Python staging validator、Node source checker、外部ZrVM parser和checked-in generated schema各自定义一部分真相，根CI没有对所有tracked声明式文件执行canonical reader、引用闭包、生成currentness和source-to-cooked资格检查。

最明确的现行断点是`examples/woc/scripts/woc_game/woc_m4_power_echo_heal_state_tests.zrp`：仓库其余276个`.zrp`都是具有`name/source/binary/entry`四字段的JSON，该文件却是两行TOML。它自2026-08-05进入HEAD且当前无diff；唯一checker只做`includes()`字符串断言，没有把它交给ZrVM project parser。与此同时38个tracked `.zro`只有28个唯一内容hash，6组重复造成1,922,113 bytes冗余；它们与2个包含本机绝对`E:\Git\ZirconEngine\...`路径的`.zr_cli_manifest`都被`.gitignore`的宽`examples/*`规则命中，形成tracked-but-ignored、部分生成物被当源码保存的混合状态。

Scene authoring同样不是“只是缺一个版本数字”。4个production/example/template `*.scene.toml`都只有`entities`根，没有format/schema/build头。reader为了保留未知组件而大量`#[serde(flatten)] _rest: toml::Table`，但资产引用重映射只手写覆盖camera、mesh、collider、animation、terrain、tilemap和prefab的已知位置；新增含资产引用的组件可以被`_rest`接受却绕过GUID/path迁移。WOC Eastbrook场景是4,466行、268实体的checked-in生成物，其中262个persisted project reference使用zero GUID加path hint，且WOC没有任何`.zmeta`。这不是单纯“文件很长”，而是schema、身份、生成receipt、分区和依赖图尚未成为同一事务。

本篇不接管Runtime04的asset load/residency/artifact、Runtime05的ECS/scene行为、Editor04/23的authoring UX、Tooling03的cook/pack、Tooling05的WOC generator语义、Tooling08的DDC record、Tooling17的SourceSet或Tooling27的版本迁移。本篇是“扩展名到codec/schema/owner/role/validator/graph/generator/publication”的physical authority与全仓declarative conformance canonical owner，登记 **0项P0、52项P1和12项P2**。

## 2. 审查边界、口径与限制

| Evidence | 本轮结果 |
|---|---|
| E1 focused format universe | 317 `.zui`、277 `.zrp`、38 `.zro`、58 `.zmeta`、14 `.zmaterial`、6 `.zshader`、1 `.zpreset`、4 `.zranim`，合计715个tracked路径、11,809,045 bytes；89,821只是物理newline计数，不能解释二进制`.zro/.zranim`语义 |
| E2 broad structured parse | 317个`.zui`全部通过Python 3.11 TOML parser；316个`.json`全部通过JSON parser；241个`.toml`中唯一非TOML是按产品合同存JSON的template `.zircon/settings.toml`；277个`.zrp`中276个是JSON、1个不是 |
| E3 ZUI profile | 295个production v2：205 component、83 view、6 style、1 theme_tokens，共49,211行/2,975,433 bytes；其余22个为test/fixture，其中14个v1、8个v2 |
| E4 ZUI identity/reference | 全317个`asset.id`唯一且无缺失；production文本中的1,107个`.zui`资源引用在当前ID集合中0 unresolved；这是当前snapshot事实，不替代mount/plugin/profile可见性验证 |
| E5 ZUI size | 最大`editor_material.zui` 3,056行/189,129 bytes；其后material lab 1,456行、asset browser 894行、WOC in-world HUD 999行、strict theme 1,416行 |
| E6 project/scene | Vampire/WOC project manifest仍为v1，renderable-empty template为v2；4个scene分别110/1/268/3实体且无顶层版本头；Eastbrook 262个zero-GUID project refs |
| E7 metadata/source formats | 58个`.zmeta`全v7、共553 entries；52个root source digest为16字符、1个为64字符、5个缺失或为空；3个mtime显式为0。14 material和6 shader全显式v2 |
| E8 Zr project/artifact | 276个JSON `.zrp`都只有同一四字段shape；38个`.zro`有28个hash、6个重复组；0个tracked `.zri`、0个tracked AOT C cohort；2个CLI manifest携绝对Windows路径 |
| E9 generated desktop schema | Hub/Tray共8个Tauri JSON schema文件；4个desktop/windows文件byte-identical，2个ACL manifest identical，2个capabilities不同，共4个唯一hash |
| E10 semantic spot reads | 逐读ZUI DTO/两套loader/compiler/file cache/stager、project/scene/material/shader/meta/animation/settings/preset reader、WOC scene/Zr生成入口、ZrVM bridge与所有8个Tauri schemaidentity |
| E11 dynamic/static validation | 本轮只做Git/文件系统清单、structured parse、hash/reference/ignore检查和源码审查；未编译生产代码，未重跑已知Editor、Hub、WOC、plugin阻断，也未运行dirty外部ZrVM workspace |
| Currentness | branch `main`，revision `ae2be3d865a937b9ed368bf965592045346c64e3`；98个frontmatter text输入按`path + LF + normalized UTF-8 content + LF`编码，fingerprint `d5bfa1e4d3c5fbd40af8850bf00beba743b87fbad22acc60fae7e85ee83c10e9`，59,837个normalized content LF、2,599,670 content bytes；外部`E:/Git/zr_vm` HEAD为`8a843bdd7a5aadbbf2deac7242a825cf64c084c8`且worktree dirty，只作只读依赖证据 |

口径说明：

1. `production ZUI`仅排除路径中的tests/fixtures/generated；正式SourceSet仍须由Tooling17和package/mount graph校正。
2. 引用解析只证明当前字符串能命中全仓ID，不证明插件禁用、目标profile、mount order、循环、权限或cook closure正确。
3. `.zro`newline、大小与hash只用于物理内容清单；本篇没有逆向推断其VM语义，也不把二进制重复自动等同运行时性能回归。
4. 外部ZrVM不在本仓修改范围。其schema和loader已经出现version drift，更说明Zircon必须固定消费版本与conformance，而不是把外部目录当前状态当隐式权威。
5. 本篇只写review和重构设计，不修改任何source、asset、generator、test、CI、manifest或生成物。

## 3. 必须保留的工程基础

### 3.1 ZMeta已经有可复用的严格reader骨架

`AssetMetaDocument`和raw DTO拒绝未知字段，先验证`format_version`，再验证tag和entry。旧v6迁移、未来v8拒绝、retired `source_hash`和事务commandlet已有专门测试。目标Format Catalog应注册并调用该reader，而不是用Python重新猜字段。

### 3.2 Material、Shader、Animation和Export Preset不是无schema字节

Material顶层严格，Shader按surface/include/compute/fullscreen profile校验字段，Animation二进制有magic/version/kind，Export Preset有严格SchemaId envelope。后续统一的是catalog、version omission policy、tool projection和全仓gate，不是把所有格式重写为一种JSON。

### 3.3 ZUI的运行期依赖与compiled identity已有良好种子

UI compile key覆盖root、widget/style imports、declared revisions、descriptor registry、component contract和resource dependency revision；package report记录source/compiler/package schema和依赖。当前production ID唯一、引用全解析。应把这些能力接到正式build/cook/catalog，而不是降级成只有`asset.kind`的Python validator。

### 3.4 Project template与settings已经有typed VersionedSchema内容

Renderable Empty manifest使用project format v2，settings内容使用`zircon.editor.settings` v1的strict JSON envelope，Editor round-trip和旧格式拒绝已有测试。问题是物理扩展名与codec不一致、catalog缺失，不是内容完全无版本。

### 3.5 Checked-in generators已有局部`--check`与generated header

Eastbrook generator会读取contract/asset manifest、生成scene和generated JSON并支持`--check`；WOC其他codegen也有类似基础。Tooling05继续拥有算法，Tooling31只要求每种输出进入统一format/role/generator/currentness catalog。

## 4. 当前格式与权威分裂图

| Surface | 当前物理/codec | 当前owner | 已有保护 | 主要断点 |
|---|---|---|---|---|
| `.zui` | TOML v2；v1仅fixture | Runtime Interface DTO + Runtime两套loader + Python stager | kind/profile/compiler/cache tests | unknown fields、version default、validator降级、compiled产物未接shipping closure |
| `zircon-project.toml` | TOML v1/v2 | Runtime project manifest | typed reader/save/tests | active examples仍v1、missing version默认v1、无跨子schema向量 |
| `*.scene.toml` | unversioned TOML | Runtime project document | serde round-trip、手写ref map | 无头、flatten逃逸、ref map不完备、多套scene persistence authority |
| `.zmeta` | strict TOML v7 | Runtime asset project | strict reader/migration/tests | currentness字段可空、digest宽度不一、root/entry重复高churn |
| `.zmaterial/.zshader` | TOML v2 | Runtime asset types | profile/version checks | 缺version时默认current，tool schema projection与全仓gate缺失 |
| `.zranim` | binary magic/version/kind | Runtime animation | typed decode/tests | 未进入中央format/cook/compat catalog |
| `.zpreset` | strict JSON envelope v0 | Runtime Interface export | VersionedSchema | 由Tooling03继续解决字段消费；本篇只登记catalog |
| `.zircon/settings.toml` | 实际JSON envelope v1 | Editor settings | strict VersionedSchema/tests | 扩展名谎报codec，通用TOML工具必然误判 |
| `.zrp` | 276 JSON + 1 TOML异形 | 外部ZrVM + Zircon bridge | 外部parser/schema；局部source checker | 本仓无固定schema/gate，异形文件不进parser，全部缺显式manifestVersion |
| `.zro/.zr_cli_manifest` | 外部ZrVM binary/text cache | 外部ZrVM | magic/hash由外部工具写 | tracked/ignored混合、重复、绝对路径、无BuildSet/provenance/cohort |
| Tauri `gen/schemas/*.json` | generated JSON | Hub/Tray build tooling | JSON可解析 | 8份4个hash、无统一generator receipt/currentness gate |

## 5. P1：Format Catalog、Codec、Schema 与全仓Conformance

### TOOL-FORMAT-P1-001 · 没有中央Declarative Format Catalog

扩展名、codec、SchemaId/version domain、owner、reader、writer、role、generator和consumer散落在Rust/Python/Node/Cargo约定中。建立machine-readable catalog；任一tracked声明式路径必须精确匹配一个format与一个physical role。

### TOOL-FORMAT-P1-002 · 扩展名不能可靠说明实际codec

`templates/.../.zircon/settings.toml`及Editor正式settings路径写JSON `$zircon` envelope。硬切为与codec一致的后缀，或注册明确的compound media type并让所有工具按catalog选择codec；不能继续让通用TOML formatter/parser失败。

### TOOL-FORMAT-P1-003 · Authoritative、Generated、Derived、Cooked、Fixture和User State没有共同角色枚举

`.zro`、`.zr_cli_manifest`、generated scene、Tauri schema、`.zmeta`和template settings各自靠目录/ignore猜角色。定义`PhysicalArtifactRole`及允许进入Git、SourceSet、package、cache和evidence的政策。

### TOOL-FORMAT-P1-004 · 大多数Rust schema没有tool-consumable projection

ZUI、scene、zmeta、material、shader、project和animation的真相主要存在Rust struct/手写parser中，Editor、CLI、LSP和CI无法消费同一descriptor。由Tooling04从canonical schema IR生成JSON Schema或等价descriptor、codec conformance和field IDs。

### TOOL-FORMAT-P1-005 · 同一格式可被多个降级reader独立解释

ZUI至少有两套Rust loader和一套Python kind checker；ZRP又被外部parser与Node字符串检查分别解释。Format Catalog必须只指向canonical validator command/library，轻量工具调用它或消费其generated schema，不重写子集语义。

### TOOL-FORMAT-P1-006 · Unknown-field政策按文件偶然形成

ZMeta/Material/Preset严格，Scene主动flatten，ZUI默认接受未知字段，Shader手写顶层allowlist。每个schema/version必须声明Reject、PreserveOpaque、ExtensionNamespace或Ignore-with-diagnostic；未声明不能发布。

### TOOL-FORMAT-P1-007 · 缺失version时的含义不一致

Project缺version默认为v1，ZUI/Material/Shader缺version默认为current v2，Scene没有version，ZMeta则要求显式v7。Tooling27拥有支持窗口，本篇要求catalog记录omission policy并由lint显式报告，禁止“恰好serde default”成为兼容合同。

### TOOL-FORMAT-P1-008 · 根CI没有全仓structured-document gate

当前`.github/workflows`没有对`.zui/.zmeta/.zmaterial/.zshader/.scene.toml/.zrp/.zpreset/settings`执行canonical parse、schema、reference和role检查。新增秒级sharded `zircon format lint --tracked --profile ci`，先冻结baseline再禁止新增。

### TOOL-FORMAT-P1-009 · 诊断没有统一path/range/code/schema上下文

Rust error、Python字符串、Node throw和外部ZrVM消息无法聚合。定义`DeclarativeDiagnostic { code, severity, format_id, schema, path, byte/line range, field_path, owner, help, safe_fix }`。

### TOOL-FORMAT-P1-010 · 没有producer-reader-writer-compatibility矩阵与receipt

“文件能解析”不能证明由当前工具生成、可由目标runtime读取或能被旧版Editor安全保存。每个BuildSet输出writer version、reader range、schema digest、producer action和round-trip/canonicalization receipt。

## 6. P1：ZUI Source、Compiler、Cache 与Distribution

### TOOL-FORMAT-P1-011 · 两套production ZUI loader复制profile规则

`asset/assets/ui/document_loader.rs`与`ui/v2/loader.rs`分别维护version和component/view/style profile校验，内容近似重复。收敛到Runtime Interface schema validator或一个Runtime adapter，删除第二份行为owner。

### TOOL-FORMAT-P1-012 · ZUI DTO默认接受拼错的结构字段

`UiV2AssetHeader/Document/Root/Node/Component`没有`deny_unknown_fields`。顶层或节点字段拼写错误可能被serde忽略，仍进入compiler。当前295份生产文件能解析不证明负向输入fail closed。

### TOOL-FORMAT-P1-013 · 缺失ZUI version被直接解释为v2

`asset.version`默认2，意味着未标版本的未来/手写文档会冒充current writer。生产文件已全部显式v2，应硬切为required version；v1 fixture通过专用migration reader，不让current reader猜测。

### TOOL-FORMAT-P1-014 · Props、state、layout、slots和tokens以自由TOML Value扩散

大量`BTreeMap<String, toml::Value>`把组件schema验证推迟到后续descriptor/runtime。建立component property schema、typed scalar/resource/binding值、extension namespace和source range；自由值只允许在明确opaque插件payload内。

### TOOL-FORMAT-P1-015 · Distribution validator只检查`asset.kind`

Python stager验证TOML可解析且kind属于四值，不验证id、version、profile、root/node、imports、component contract、resource refs或compile结果。它可能接受Runtime reader必拒绝的包；改为调用canonical validator artifact。

### TOOL-FORMAT-P1-016 · Build没有编译整个shipping ZUI闭包

`zircon_build`复制source并调用浅validator，没有对目标profile的root、imports、descriptor registry、action policy、localization/resource dependency执行全量compile。新增按产品root求闭包的compile stage和逐asset report。

### TOOL-FORMAT-P1-017 · V2 persistent file cache没有production构建owner

`UiV2PrototypeStoreFileCache::with_persistent_cache`在非测试产品调用中没有consumer；现有cache machinery主要由tests覆盖。先定义offline/UI cook builder和Runtime packaged reader，再把cache称为可分发产物。

### TOOL-FORMAT-P1-018 · Stager可选复制任意现存`.zuiart/.zuicache`

`.zircon/ui/compiled_artifacts`或环境覆盖目录只按后缀复制，不要求目标asset closure完整、source/action digest current、BuildSet匹配或每份artifact回读验证。Tooling08提供DDC record，本篇要求distribution只消费validated UI Cook Receipt。

### TOOL-FORMAT-P1-019 · Source与compiled payload的运行时优先级未成为发布合同

staging总是复制source，随后可再复制compiled cache；没有manifest声明shipping是否允许source fallback、cache miss是否现场compile、哪个schema/target可读。按profile明确Editor source、development fallback和shipping compiled-only策略。

### TOOL-FORMAT-P1-020 · Asset ID索引遇到重复时静默取字典序路径

V2 file cache全树扫描`.zui`，duplicate `asset.id`时由`should_replace...`选较小路径。当前无重复是好事实，但未来mount/plugin冲突会静默隐藏一方。索引构建必须返回typed collision，除非catalog声明显式override关系。

### TOOL-FORMAT-P1-021 · 巨型主题/Workbench文档缺少schema分区与source map

`editor_material.zui` 3,056行、material lab 1,456行、strict theme 1,416行，混合大量token/style/component状态。按token namespace、component family和generated projection拆owner，compiler保留跨文件source map与增量dependency，不按行数机械切片。

## 7. P1：Project、Scene、Meta、Material、Shader 与Settings

### TOOL-FORMAT-P1-022 · 两个active example project仍停留在format v1

Vampire和WOC使用v1，当前template使用v2。先用Runtime canonical migrator做dry-run/round-trip并提交migration receipt；不允许样例长期验证旧默认路径、template验证新路径而没有同代产品矩阵。

### TOOL-FORMAT-P1-023 · Project manifest missing-version默认v1仍是隐式兼容入口

这项迁移支持窗口由Tooling27拥有。本篇要求lint把omitted version标为legacy admission并禁止新文件；正式writer永远显式写current，不用缺字段表达版本。

### TOOL-FORMAT-P1-024 · Scene authoring文件没有顶层format/schema头

4个当前scene都只有`entities`。加入SceneDocumentId、format version、saved-by/compatible reader、component schema vector和生成来源；Runtime05继续拥有实体/组件语义。

### TOOL-FORMAT-P1-025 · Scene的`flatten _rest`让拼错字段和未知组件无诊断存活

保留插件扩展不能等于接收任意表。已注册component使用严格typed schema；未知namespace进入显式opaque extension block并记录provider/schema，普通未知字段直接报range diagnostic。

### TOOL-FORMAT-P1-026 · Scene reference remap是手写组件白名单

`map_scene_references`枚举11类已知字段，`_rest`中的新资产引用不会迁移。让reflection/schema IR生成reference walker或由每个component codec报告typed edges，新增ref-bearing field必须自动进入测试和graph。

### TOOL-FORMAT-P1-027 · 仓内存在至少三套scene persistence authority但无catalog

Project authoring TOML、World project JSON v2、DynamicScene VersionedSchema分别服务不同场景，却没有FormatId、转换边和禁止误用规则。登记authoring/runtime snapshot/reflection interchange角色与lossless/lossy转换，不合并成万能DTO。

### TOOL-FORMAT-P1-028 · Eastbrook用zero GUID加path hint充当262个正式引用

路径可移动且不能证明subasset identity；Runtime04还已确认missing subasset fallback风险。generator必须从冻结Asset Registry generation解析真实GUID/subasset/type，并在不能解析时fatal，不能把zero sentinel写入产品scene。

### TOOL-FORMAT-P1-029 · 4,466行generated scene没有分区/实例化source graph

268实体平铺为单文件会放大diff、watch、merge和重生成成本。M8 contract应生成scene partition/prefab/instance graph或中间IR，再由canonical scene writer稳定输出；Runtime05/Editor16/41拥有世界分区行为。

### TOOL-FORMAT-P1-030 · WOC scene/source树没有任何ZMeta身份

全仓58个`.zmeta`中WOC为0，Eastbrook生成物因而没有Asset Registry sidecar、importer/currentness或artifact locator。要么把scene定义为自描述authoritative asset并登记等价identity，要么由import事务生成严格sidecar；不能让example产品走隐式路径特例。

### TOOL-FORMAT-P1-031 · ZMeta currentness关键字段允许空/零默认

严格unknown-field值得保留，但`importer_id/config_hash/source_digest/source_mtime/importer_version`仍可default。当前5份root digest缺失或为空、3份mtime显式为0。按asset role声明required字段；fixture/template seed使用明确`seed/unimported`状态，不伪装ready imported record。

### TOOL-FORMAT-P1-032 · ZMeta digest宽度与算法身份不一致

52份root `source_digest`是16字符，1份是64字符，schema字段不带algorithm/version。改为typed digest `{algorithm, bytes}`或固定256-bit，并让config/dependency/artifact digest同样有domain；迁移由Runtime04/Tooling27执行。

### TOOL-FORMAT-P1-033 · Compound ZMeta重复root与entry依赖造成高churn

58份sidecar含553 entries，三个character模型各89 entries；root和entry反复写相同依赖/locator。规范化为source-unit record + stable subasset table + shared dependency sets，writer保证排序和最小diff，不把生成元数据体积本身当功能完成度。

### TOOL-FORMAT-P1-034 · ZMaterial虽严格却允许缺version冒充v2

14个tracked文件都显式v2，说明可安全硬切required。保留`deny_unknown_fields`、queue/texture/profile验证，移除current默认；旧格式只能通过专用migrator进入。

### TOOL-FORMAT-P1-035 · ZShader手写allowlist仍允许缺version

6个tracked文件都显式v2，reader却在表缺`version`时返回成功并由serde默认2。让统一schema descriptor同时生成profile allowlist和required version，防止手写preflight与DTO漂移。

### TOOL-FORMAT-P1-036 · Settings物理文件名与canonical JSON writer冲突

Editor文档明确JSON envelope是当前唯一owner，但文件名仍为`settings.toml`。执行一次hard-cut rename及project manifest/settings pointer迁移，给旧路径只保留有sunset的read-only诊断，不长期双写或按首字符猜codec。

## 8. P1：Zr Project、Compiled Object 与CLI Manifest

### TOOL-FORMAT-P1-037 · 一个tracked `.zrp`不是ZRP JSON

`woc_m4_power_echo_heal_state_tests.zrp`由两行TOML组成，不能被当前Zr project JSON parser读取。改成canonical JSON manifest并通过真实parser/compiler negative/positive gate；若它其实是plugin selector，必须改后缀和schema，不能占用`.zrp`。

### TOOL-FORMAT-P1-038 · Source checker用字符串包含冒充project validation

`m4_power_echo_heal_source_check.mjs`只断言backend/entry文本存在，不执行JSON parse、ProjectWorkspace open或compile。它可以保留reference projection检查，但project资格必须来自ZrVM command receipt。

### TOOL-FORMAT-P1-039 · ZRP schema权威位于dirty外部兄弟workspace且未固定

Zircon Cargo用`../../../../zr_vm` path dependency，runtime直接调用`ProjectWorkspace::open`。本仓没有vendored schema digest或compat matrix；外部HEAD的JSON schema仍限制`manifestVersion<=1`，而同一HEAD loader/tests已支持v2。锁定ZrVM BuildSet、导入generated schema/catalog并跑parser-schema differential conformance。

### TOOL-FORMAT-P1-040 · 277个ZRP没有repo-wide schema/catalog gate

276个JSON都重复四字段shape，1个异形；没有单一命令遍历并输出typed结果。将ZRP纳入Format Catalog，批量parse、canonicalize、entry/module graph和output containment检查。

### TOOL-FORMAT-P1-041 · 276个JSON manifest全部缺显式manifest/toolchain/assembly identity

它们只写`name/source/binary/entry`，依赖外部loader legacy默认。测试矩阵也应显式声明manifestVersion、assembly kind/version、target/mode和expected artifact profile，避免外部默认变化重解释历史fixture。

### TOOL-FORMAT-P1-042 · 274个binary目录把编译输出放在source project树内

每个测试manifest手工发明`bin-*`路径，SourceSet、cache、evidence和clean规则难以区分。由Build Action分配target/artifact root，manifest只声明logical output profile，不持有具体工作目录。

### TOOL-FORMAT-P1-043 · ZRO与CLI manifest同时tracked且被ignore

`.gitignore:128/130`命中当前tracked outputs，新增或删除可逃过普通review和source枚举。Tooling17决定移出Git或转为具名fixture；无论哪种都必须让content manifest而非ignore定义角色。

### TOOL-FORMAT-P1-044 · 6组ZRO副本浪费1,922,113 bytes并隐藏共享依赖

`rng.zro`和`trace_symbols.zro`各重复4次，targeting/locomotion/lifecycle/roster各重复2次。用CAS/reference manifest或每test Build Receipt引用同一immutable object；不要用复制来表达不同运行场景。

### TOOL-FORMAT-P1-045 · `.zr_cli_manifest`固化开发机绝对路径

两个v3 manifest都记录`E:\Git\ZirconEngine\...`的zro/zri/aot路径，破坏可搬运、隐私和clean checkout验证。只保存project-relative logical ID/content digest；debug path map进入受控provenance sidecar。

### TOOL-FORMAT-P1-046 · Tracked compiled cohort不完整且没有publication receipt

仓库有38 `.zro`，却没有tracked `.zri`或AOT C，CLI manifest又声明这些输出位置。要么所有运行产物都由固定toolchain重建且不入SourceSet，要么具名fixture包含完整profile closure、hash、producer和reader验证；不能保存随机子集。

### TOOL-FORMAT-P1-047 · Zircon没有登记`.zro`的FormatId、decoder owner或runtime admission

实际decode完全委托外部ZrVM，Zircon package bridge只打开project并编译/启动session。Format Catalog需明确external owner、magic/version probe、accepted BuildSet、failure mapping和shipping package位置，不要求Zircon重写VM decoder。

## 9. P1：Generated Schema、Repository Wiring 与End-to-End Qualification

### TOOL-FORMAT-P1-048 · Hub/Tray Tauri schema重复存储且代次关系不明

8个文件只有4个唯一hash：四份desktop/windows完全相同，两份ACL相同，capabilities各自不同。保留产品差异，但由一个generation action产出per-product manifest；共享值进入CAS或单一tool snapshot，产品只引用digest。

### TOOL-FORMAT-P1-049 · Generated schema没有source/tool/currentness receipt

目录名`gen/schemas`和字节相等不能证明由哪个Tauri CLI、配置、lockfile、target和命令生成。每次generation记录tool digest、inputs、outputs和determinism version，并在CI reproduce-and-diff。

### TOOL-FORMAT-P1-050 · “schema”命名文件数量不能替代Schema Catalog

仓库有大量文件名含schema的Python helper/test/Rust模块，真正可供外部工具消费的format descriptor却很少。Catalog按stable SchemaId列definition artifact、codecs和owners，禁止用文件名搜索生成能力清单。

### TOOL-FORMAT-P1-051 · Root required matrix没有声明式资产lane

局部Rust/Python tests覆盖许多正负例，但`.github/workflows/ci.yml`未执行全tracked parse/reference/generator/output-role检查。Tooling01接入format lint、schema conformance、generated currentness和small clean-project round-trip，typed skip只能用于缺失外部toolchain。

### TOOL-FORMAT-P1-052 · 没有clean checkout Authoring→Import→Compile→Cook→Pack闭环

当前能分别找到template、import、UI compiler、cache、cook和pack代码，不能证明同一BuildSet/asset graph贯穿。建立最小Renderable Empty和一个compound/WOC slice：从source解析、GUID/edge、derived artifact、platform cook到runtime packaged load，所有阶段用content-bound receipt串联。

## 10. P2：可用性、可解释性与长期质量

1. **TOOL-FORMAT-P2-001**：提供`zircon format explain <path>`，显示FormatId、codec、schema/version、owner、role、reader/writer、generator、package/cook policy和当前诊断。
2. **TOOL-FORMAT-P2-002**：Editor Problems面板消费统一诊断，支持字段range、schema链接和安全fix preview，不直接覆写未知版本文档。
3. **TOOL-FORMAT-P2-003**：从canonical descriptor生成VS Code/LSP schema association、completion、hover和deprecated字段提示。
4. **TOOL-FORMAT-P2-004**：提供canonical formatter和stable ordering/minimal-diff writer；generated与authoring格式使用不同格式化政策。
5. **TOOL-FORMAT-P2-005**：提供跨asset reference graph查询和可视化，显示hard/soft/generated/import/cook edge与unresolved/cycle原因。
6. **TOOL-FORMAT-P2-006**：报告generated/derived duplicate bytes、churn、CAS复用率和largest owners，但不自动删除canonical source。
7. **TOOL-FORMAT-P2-007**：提供schema compatibility diff，区分reader-safe、writer-breaking、migration-required和unknown-extension变化。
8. **TOOL-FORMAT-P2-008**：为第三方plugin分配schema/format namespace、extension field policy和signed catalog snapshot；远程registry不成为离线启动依赖。
9. **TOOL-FORMAT-P2-009**：建立Windows/Linux/macOS path、case、separator、Unicode和portable receipt矩阵，绝对路径/大小写碰撞在publish前失败。
10. **TOOL-FORMAT-P2-010**：对TOML/JSON/binary header、深度、重复key、超大collection、恶意长度和版本边界执行parser fuzz与differential tests。
11. **TOOL-FORMAT-P2-011**：建立10K/100K asset的parse、index、reference graph、incremental compile、format和diagnostic latency/peak-memory基线。
12. **TOOL-FORMAT-P2-012**：Hub/Editor显示format/schema/generator健康度和migration debt，但只消费ValidationSet，不从文件存在或计数自行推断绿色。

## 11. 参考实现对照

| Reference | 可核对机制 | Zircon应吸收 | 不应照搬/外推 |
|---|---|---|---|
| Unreal | `.uproject`显式FileVersion/module/plugin；AssetRegistry state返回版本并按dependency category查询；DDC BuildDefinition区分function/constants/input builds/files | project/package身份、typed dependency graph、immutable build definition与cook/cache分层 | 不复制UObject/package历史兼容债务，也不因UE体量宣称Zircon性能差 |
| Godot | text scene/resource头写`format`和UID；ext/sub-resource显式建模；UID失效时产生警告并保留path fallback | scene头、稳定identity、外部/内部resource edge、可定位诊断 | 动态Variant和全局singleton不是Rust核心目标；path fallback不能掩盖Zircon subasset type错误 |
| Bevy | AssetMeta绑定loader/process settings；ProcessedInfo含32-byte hash/full hash/dependency hash；processor有write-ahead transaction log | importer/settings/dependency identity、processed currentness、崩溃后重处理 | Bevy处理器不是完整商业cook/release，不能替代Tooling03 |
| Fyrox | Visitor有binary/ascii magic与CURRENT_VERSION；ResourceManager集中loader/registry/watcher/async load | 格式探测、版本reader、集中resource owner | Fyrox typed request文档也承认extension不能严格证明类型，不应作为Zircon exact type目标上限 |
| Unity Graphics | `.asset` YAML引用使用fileID+GUID+type，`.meta`分离fileFormatVersion/GUID/importer；package声明name/version/dependencies | source identity sidecar、subobject ID、package dependency/version | Unity YAML/meta同样可能高churn；Graphics仓只证明可见包结构，不代表闭源Unity全资产管线 |

参考源码显示的共同点不是“都用某种文本格式”，而是source identity、schema/version、dependency、import settings、derived output和package/build identity分别有owner。Zircon已有这些概念的局部实现，目标是收敛合同并证明闭环，不是统一文件语法。

## 12. 目标架构

```text
Repository SourceSet / Product Profile / Plugin Mount Graph
                         |
                         v
              DeclarativeFormatCatalog
       FormatId + codec + SchemaId/version + owner
       role + omission/unknown policy + generator
                         |
          +--------------+---------------+
          |                              |
          v                              v
 Canonical domain reader/writer     Generated schema/LSP
 (Runtime/Editor/ZrVM adapters)      formatter/diagnostics
          |                              |
          +--------------+---------------+
                         v
             DeclarativeDocumentIndex
   stable asset/document IDs + typed reference edges
   source ranges + provider/mount/profile visibility
                         |
                         v
         Validate / Migrate / Generate / Compile Plan
                         |
                         v
      immutable generation + DDC/Cook/Package receipts
                         |
                         v
       clean-checkout conformance / release qualification
```

### 12.1 Format Catalog

每项至少包含`FormatId`、suffix/media type、codec、magic、SchemaId/version domain、minimum/current reader/writer、version omission、unknown/extension policy、canonical reader/writer command、owner crate/tool、physical roles、generator、source/cook/package policy和diagnostic namespace。Catalog是工具路由真相，不替代domain DTO。

### 12.2 Declarative Document Index

索引只消费canonical readers发布的identity、typed edges和source ranges。edge记录hard/soft/import/generated/cook类别、expected type/schema、provider/profile visibility和provenance。Scene reference walker、ZUI imports、ZMeta dependencies、project roots和ZRP module graph都投影到此层，不用文本regex重建正式graph。

### 12.3 Publication与Receipt

Authoritative source由SourceSet冻结；generated source有Generator Receipt；derived data由Tooling08 Action/Record；cook/pack由Tooling03；版本决策由Tooling27。Tooling31只校验每个physical file的format/role/owner以及这些receipt是否闭合并指向同一BuildSet。

### 12.4 Hard cutover原则

错误后缀、重复loader、异形ZRP、zero-GUID生成和tracked runtime cache都不应永久兼容。先提供dry-run inventory和migration plan，再一次切换writer/consumer；旧reader仅在有明确sunset的只读窗口存在，不双写、不首字符猜格式、不继续生成旧路径。

## 13. 实施里程碑

### M0 · 冻结Format/Role基线

- 生成全部tracked声明式路径的FormatId/codec/schema/role/owner候选清单；
- 固定当前parse failures、unknown-version、duplicate ID/hash、tracked-ignore和绝对路径基线；
- required gate先禁止新增，不要求首提交清零历史债务；
- 修正异形ZRP和settings后缀前先产出migration/consumer清单。

### M1 · Catalog、统一诊断与Canonical Validator

- 建DeclarativeFormatCatalog及generated tool projection；
- ZMeta/Material/Shader/Preset等现有reader注册为adapter；
- 定义Diagnostic与DocumentIndex schema；
- `zircon format lint/explain`遍历冻结SourceSet并进入root CI。

### M2 · ZUI hardening与offline compile

- 合并两套loader，version required，unknown/extension policy显式化；
- component property descriptor生成typed validation；
- 产品profile求ZUI闭包、全量compile并发布UI Cook Receipt；
- shipping不再复制无receipt cache，duplicate ID直接失败。

### M3 · Project/Scene/Meta物理迁移

- active examples迁到project v2并留receipt；
- Scene加入version/schema vector，component codec生成reference walker；
- Eastbrook消除zero GUID并输出partition/prefab/generator receipt；
- ZMeta收敛digest/currentness/subasset table；settings执行codec-consistent rename。

### M4 · Zr Project与Artifact治理

- 固定ZrVM BuildSet和schema/parser conformance；
- 277个ZRP走真实parser，binary root由Action分配；
- `.zro/.zr_cli_manifest`从SourceSet迁出或转为完整immutable fixture；
- CAS去重、relative path和完整profile cohort进入Build Receipt。

### M5 · Generate/Import/Cook/Pack闭环

- Tauri/WOC/UI generated outputs进入统一generator manifest；
- DeclarativeDocumentIndex成为Cook Planner的typed dependency输入；
- Renderable Empty和WOC小切片执行clean checkout E2E；
- release只接受同BuildSet的schema、generator、DDC、cook和package receipts。

### M6 · Scale、Fuzz与跨版本资格

- 10K/100K asset图建立性能与内存预算；
- parser/reference/migration/generator corruption与crash矩阵；
- mixed reader/writer、plugin absent、mount collision、platform path矩阵；
- Dashboard只展示validated current generation。

## 14. 验收门

1. 所有tracked声明式路径恰好匹配一个FormatId和一个PhysicalArtifactRole，unknown/duplicate均为0。
2. Catalog中的每个suffix/media type都有canonical codec；扩展名与内容不一致为0。
3. 每个format有唯一domain owner、reader、writer、schema/version policy和diagnostic namespace。
4. 当前writer永远显式写schema/version；缺失version只进入登记的legacy reader并产生诊断。
5. Unknown field按Reject/PreserveOpaque/ExtensionNamespace政策执行，未声明Ignore为0。
6. Canonical reader与generated JSON Schema/LSP validator对正负corpus决策一致。
7. 所有parse/schema错误包含stable code、文件、range、field path和owner。
8. 根required CI遍历冻结SourceSet，不依赖Git ignore或手工文件列表遗漏新文件。
9. 317个ZUI ID保持唯一；重复ID使catalog build失败，不按路径字典序选胜者。
10. 每个shipping ZUI root的import/resource/localization/action-policy闭包完整且可复现。
11. Shipping UI cache每项绑定source/compiler/package/BuildSet digest并回读验证；无receipt复制为0。
12. Shipping profile的source fallback/onsite compile政策显式，release默认不依赖开发机cache。
13. 两套ZUI profile loader收敛为一个canonical行为owner。
14. ZUI未知字段、缺version、空ID、错误root/node、component contract和资源引用负例全部fail closed。
15. Project active examples与template使用current format，旧版只存在具名migration fixture。
16. 所有scene写format/schema/build头并声明component schema vector。
17. 注册component的拼错字段失败；opaque extension必须携provider/schema且不会被核心误解释。
18. 新增ref-bearing component field会自动进入reference walker与graph test，不需要修改中央手写match。
19. Scene authoring、runtime snapshot和reflection interchange拥有不同FormatId及显式转换边。
20. 产品scene中的zero project GUID为0；path hint不能单独获得resolved资格。
21. Eastbrook generation由冻结Asset Registry和contract输入重建，tree diff为0且有receipt。
22. Scene partition/prefab/instance graph保持实体/引用语义，增量改动不重写无关分区。
23. 每个product asset要么有严格ZMeta，要么登记等价自描述identity；隐式路径特例为0。
24. Ready imported ZMeta的importer/config/source/dependency digest和version字段完整；seed/fixture状态显式。
25. 所有digest带algorithm/domain并满足collision policy；16/64字符自由混用为0。
26. ZMeta root/subasset表canonical排序，依赖集合不靠大规模复制表达。
27. ZMaterial/ZShader current reader拒绝缺失version，现有全部current文件round-trip stable。
28. Editor settings后缀与JSON codec一致；旧`.toml`路径只读迁移窗口结束后明确拒绝。
29. 277个ZRP全部通过同一固定ZrVM parser/schema conformance；字符串includes不计资格。
30. ZRP显式声明manifest/assembly/toolchain/output profile；legacy omission只保留fixture。
31. 编译output root由Build Action分配且在workspace policy内，不由277份manifest手写目录。
32. tracked-but-ignored ZRO/CLI manifest为0，或全部转为manifest声明的具名fixture且ignore不定义角色。
33. 重复ZRO value通过CAS引用，同一content不复制进多个fixture/output root。
34. 所有CLI/build manifest使用logical relative ID，无开发机绝对路径、secret或ambient tool path。
35. Zr compiled fixture包含完整profile cohort、producer/tool/schema/hash和reader验证；随机子集为0。
36. `.zro` external decoder的accepted BuildSet、magic/version、failure mapping和package位置进入Catalog。
37. Hub/Tray generated schema由固定tool action重建，outputs/digests与tree一致。
38. Generated schema共享内容不复制或由CAS去重；产品差异由per-product manifest解释。
39. Clean Renderable Empty从source到runtime packaged load全链通过且receipt引用同一BuildSet。
40. WOC representative slice从ZRP/scene/UI source到compiled/cooked package通过，任一旧generation或缺edge会阻断。

## 15. Owner边界

| 事实 | Canonical owner | Tooling31职责 |
|---|---|---|
| Asset type/load/residency/artifact/registry | Runtime04 | 注册format adapter、检查physical role与graph projection，不重写runtime算法 |
| Scene entity/component/prefab/world lifecycle | Runtime05、Editor03/16/41/42 | format/schema/reference-walker合同与文件资格 |
| ZUI runtime/compiler与Editor UI authoring | Runtime11A/11C、Editor23 | Catalog、单validator、offline closure/receipt wiring |
| Import/reimport/catalog/thumbnail workflow | Editor04 | 统一diagnostic/index输入，不拥有UX事务 |
| Cook/Pack/Platform Bundle | Tooling03 | 提供validated DocumentIndex与format receipts |
| WOC codegen semantics | Tooling05、Runtime18 | generator output角色/currentness/catalog，不改内容算法 |
| DDC/CAS/remote cache | Tooling08 | 声明authoritative vs derived并消费Action/Record receipt |
| SourceSet/ignore/generated/vendor | Tooling17 | 每个path的format/role事实，SourceSet政策仍归Tooling17 |
| Version/support/migration | Tooling27、Runtime Interface02 | omission/reader-writer政策登记，迁移语义仍归对应domain |
| Schema IR/codegen | Tooling04 | format catalog消费generated descriptor与compat hash |
| Zr parser/compiler/object codec | 外部ZrVM、Runtime21/Plugin bridge | 固定BuildSet、adapter和conformance，不复制decoder |

## 16. 验证与Currentness

本轮实际执行并观察：

1. Git tracked extension/size/newline inventory，TOML/JSON structured parse和ZUI kind/version/ID/ref清单；
2. ZRO SHA-256重复组、Tauri schema SHA-256相等性、Git ignore解释、CLI manifest绝对路径检查；
3. Project/scene/ZMeta/ZRP结构统计及production reader/tool/CI call-site搜索；
4. Unreal/Godot/Bevy/Fyrox/Unity本地参考实现的格式头、身份、依赖、import/processing和package边界抽查；
5. 外部ZrVM只读HEAD/schema/parser证据；因其worktree dirty且已有既知编译阻断，本轮不运行或修改。

未执行：Cargo编译、Editor/Hub启动、ZrVM compile、UI全量compile、WOC generator、Tauri schema regenerate、cook/pack或clean checkout E2E。静态parse绿色只证明语法层；0 unresolved ZUI refs只证明当前全仓字符串命中。实施前必须重新取branch/source fingerprint、外部ZrVM BuildSet、Format Catalog和产品profile closure。

## 17. Review交接

本报告完成的是逐格式物理审查和重构架构，不是修复完成声明。M0优先级应为：冻结Format/Role inventory，接入canonical parse gate，修异形ZRP与settings codec/extension迁移设计，同时禁止新增tracked-ignore compiled output、zero-GUID scene reference和无receipt generated schema。随后按M2-M5把ZUI、Scene/ZMeta、ZrVM artifacts和cook闭环逐层接线。

在40个验收门成立前，Zircon可以说“若干资产格式已有严格局部reader和测试”，不能说“项目、场景、UI、脚本与生成资产已经拥有工程级统一schema、依赖图、可重现编译和shipping资格”。
