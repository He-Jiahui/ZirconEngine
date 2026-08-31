---
title: Runtime Font Asset、Source、Cook、Database、Face、Fallback、Variation、Color、Resolved Glyph、Cache 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime80
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/font_source.rs
  - zircon_runtime/src/asset/artifact/cache_payload/font.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset
  - zircon_runtime/src/text/font
  - zircon_runtime/src/text/service.rs
  - zircon_runtime/src/text/glyph_artifact.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/text/shaping/cosmic
  - zircon_runtime/src/text/sdf/font_bake
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs
  - zircon_runtime/src/ui/surface/text_artifact.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/runtime_lines.rs
tests:
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt/tests
  - zircon_runtime/src/asset/tests/assets/font.rs
  - zircon_runtime/src/text/font/database/tests
  - zircon_runtime/src/text/font/fallback/tests.rs
  - zircon_runtime/src/text/font/handle_registry/tests.rs
  - zircon_runtime/src/text/font/source_manifest/tests.rs
  - zircon_runtime/src/text/sdf/font_bake/tests
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests/font_assets.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/50-runtime-manager-resolver-named-service-handle-generation-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/70-runtime-scene-text-text2d-text3d-billboard-font-layout-localization-extract-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/79-runtime-ui-renderer-display-list-paint-order-clip-transform-opacity-atlas-text-glyph-batch-wgpu-submit-product-integration-current-source-review.md
  - docs/plans/zircon_runtime/text/01/failure-2026-07-18-font-face-metadata-reparse.md
  - docs/plans/zircon_runtime/text/01/failure-2026-07-18-font-fallback-candidate-rebuild.md
  - docs/plans/zircon_runtime/text/09/failure-2026-07-18-font-handle-per-glyph-global-lock.md
  - docs/plans/zircon_runtime/text/02/failure-2026-08-02-resolved-glyph-artifact-ui-owner-reverse-dependency.md
  - docs/plans/zircon_runtime/text/09/failure-2026-07-18-text-cache-linear-lookup-and-eviction.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontBulkData.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontBulkData.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/CompositeFont.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/FontFace.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/FontFace.cpp
  - dev/godot/scene/resources/font.h
  - dev/godot/scene/resources/font.cpp
  - dev/godot/servers/text/text_server.h
  - dev/godot/servers/text/text_server.cpp
  - dev/godot/modules/text_server_adv/text_server_adv.h
  - dev/godot/modules/text_server_adv/text_server_adv.cpp
  - dev/bevy/crates/bevy_text/src/font.rs
  - dev/bevy/crates/bevy_text/src/font_loader.rs
  - dev/bevy/crates/bevy_text/src/error.rs
  - dev/bevy/crates/bevy_text/src/font_atlas_set.rs
  - dev/Fyrox/fyrox-ui/src/font/mod.rs
  - dev/Fyrox/fyrox-ui/src/font/loader.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Textures/Texture2DAtlas.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 80 · Runtime Font Asset、Source、Cook、Database、Face、Fallback、Variation、Color、Resolved Glyph、Cache 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon 的字体底座已经超过“临时能显示文字”的阶段。当前 `text/font` 有 generation-owned face metadata、共享 source bytes、backend face 映射、composite interval/culture/script 策略、bounded fallback/effective-instance cache、variable coordinate canonicalization、批量 font-handle 注册与解析；native bitmap 路径能区分 alpha/subpixel/color glyph，SDF source context、offline artifact 与 glyph bitmap 也有显式数量/字节预算。Text01/02/09 的 metadata 重解析、fallback candidate rebuild、per-glyph global lock、resolved artifact reverse dependency与线性cache lookup修复已进入当前源码，不应重复推倒。

但字体“资产 -> cook -> runtime byte lease -> face collection -> resolved glyph”仍未闭合。导入器读取并解码源字体，只把 metadata 写进 `FontAsset`；artifact payload不保存字体字节，而项目扫描又把 TTF/OTF/WOFF/WOFF2当作auxiliary。运行时于是从manifest重新canonicalize并打开原始source path，非项目路径还回落 `env!("CARGO_MANIFEST_DIR")`。这使clean package、DLC、远端mount和只读部署无法仅依赖cooked artifact。与此同时共享数据库、布局服务和handle registry仍是process-global，默认启动显式发现主机系统字体；一个project/window/test的mutation会改变同进程其他session的fallback、generation和全套atlas/cache。

本轮不重复登记 Runtime11B 已拥有的两个字体 P0：P0-1 cooked font bytes断链、P0-2真实shaping全失败后仍发布无face的synthetic rasterizable glyph。两项经current source复核均仍开放。本报告新增 **0 项 P0、48 项 Runtime80 独有 P1、12 项 P2 与 48 项资格门**。目标不是继续扩展全局 `FontDatabase`，而是建立 `FontBlobArtifact -> FontCollectionSnapshot -> FontCollectionService -> ResolvedFontFaceLease -> ResolvedGlyphStatus` 的project/session-owned链；只有clean package、双session隔离、恶意字体budget、确定性fallback、真实color/variation和同负载性能门全部通过，才可谈工程级完成度。

2026-08-29 current-source 校准：上段与第 4 节表格保留为 review 基线，其中 P0-1/P0-2 的源码事实已被后续实现取代。P0-2 的 typed shaping correction 见 Runtime11B；P0-1 当前已有 `FontBlobArtifact -> artifact cache payload -> project cooked loader -> FontDatabase Arc bytes` 链，并以编译内嵌 manifest/TTC 建立不依赖系统字体的 2-face runtime 默认包。默认 primary face/CompositeFont/UI family 也已从 project projection 中分离成 engine baseline，解析优先级为 explicit > project > runtime，项目卸载恢复内置 CJK 路由；SDF default resolver不再重开 loose default asset。静态实现不等于 Runtime80 M1/GATE-002 完成：shipping direct-path policy、source-deleted package、多语言 raster、license/toolchain receipt及受管动态证据继续开放。

## 2. 审查边界与物理冻结

### 2.1 Owner 边界

| 领域 | Canonical owner | Runtime80 责任 | 不重复登记 |
|---|---|---|---|
| 通用asset/artifact/store | Runtime04 / Runtime51 / Runtime64 | font blob schema、cook closure、font dependency generation | 通用artifact durability、registry、load state machine |
| Text shaping/Unicode/layout | Runtime11B；下一轮 Runtime81 | font collection输入和resolved face/glyph completeness | BiDi、script shaping、line break、long document |
| Glyph atlas/SDF/GPU submit | Runtime11C / Runtime79 | face/instance/content identity交接与font-derived invalidation | atlas page、upload、batch、WGPU submit |
| Stable handle/service facade | Runtime24 / Runtime50 | font face/collection generation和manager access surface | 通用handle exhaustion、service call guard |
| Scene text/Editor authoring | Runtime70 / Editor23 / Editor33 | 共享font collection consumer合同 | SceneText、font toolkit、localization authoring总owner |

中立 `TextFontFaceHandle`、`TextGlyph` 和status DTO继续放在 `core::framework::text`；生命周期facade应放在 `core::manager`；字体字节、collection、fallback与provider实现归 `zircon_runtime::text`。Editor只能持runtime handle/snapshot，不得再建可变数据库或第二套face identity。

### 2.2 Zircon 物理冻结

指纹算法：相对路径排序；逐文件SHA-256；以 `path<TAB>lowercase-hash` 用LF连接且末尾无LF，再对UTF-8清单做SHA-256。production classifier排除路径段 `/tests/` 和叶文件 `tests.rs`，保留inline tests与默认font manifest。二进制TTC单独冻结，不把二进制误计为源码行。

| 范围 | 文件 / 行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Asset / cook | 11 / 2,441 / 90,071 / 18 | manifest schema、artifact payload、source decode、metadata import、auxiliary scan |
| Font database | 43 / 7,911 / 301,263 / 99 | face metadata、backend、fallback、variation、system policy、shared owner、handle registry |
| Service / shaping handoff | 9 / 2,654 / 99,644 / 25 | neutral DTO projection、generation retry、synthetic fallback、resolved artifact |
| Render / SDF consumers | 25 / 6,921 / 276,221 / 75 | renderer manifest cache、TextRenderState、SDF source/offline/cache/budget |
| UI / Editor surfaces | 8 / 2,280 / 85,788 / 12 | dead UI registry、UI artifact、layout consumer、Editor runtime line consumer |
| 去重合计 | **96 / 22,207 / 852,987 / 229** | 全集 fingerprint `7d318b995d53f3f0b5e9e94b28a594e114f2bb417d23d26fe1705981d8db7266` |
| Production | **68 / 15,789 / 598,749** | production fingerprint `15dfceb551b90f06161271481d015dfaf28f61f0300ba1c78122c4b6e9803548` |

内置 `ZirconDefaultComposite-subset.ttc` 为 103,624 bytes，SHA-256 `bf25507c694c39e9ffd514f8f8ab3b79ced814cd0e96bb60a95c5bb6434936c7`。它只含 Fira Mono 与小型“CJK SC Proof”face；默认清单的日/韩/繁中/Arabic/Hebrew/Emoji覆盖仍主要依赖主机family name。

冻结时 importer、font database/system/matching、Cosmic shaping与Editor runtime line consumer存在其他Session或用户修改；本文没有修改源码，故 `source_recheck_required: true`。实施前必须重取两个源码指纹和TTC hash。

### 2.3 参考物理冻结

参考冻结 **20 个文件、20,837 行、958,439 bytes**，清单 fingerprint `97a6194cbb7bcd0003d7c3d30f39fcf3e58a13fc30552049f8f7638530da912d`。

| 参考 | 文件 / 行 / bytes | 可吸收不变量 | 局限 |
|---|---:|---|---|
| Unreal | 7 / 3,872 / 174,296 | `UFontFace`拥有raw payload；Inline/LazyLoad/Stream cook策略；`.ufont` additional file；bulk lock/memory stat；composite range/culture/history；deferred cache flush | 不复制UObject、宏和FreeType指针模型 |
| Godot | 6 / 14,842 / 686,381 | `FontFile`持bytes/data pointer并按RID管理face/linked variation；显式free/clear；language/script overrides；color/system-fallback/raster settings；TextServer backend隔离 | RID与Object模型不是Zircon公开ABI模板 |
| Bevy | 4 / 298 / 11,506 | `Font` asset直接持 `Blob<u8>`；asset handle alias；removal重建collection并标脏consumer；atlas可统计总bytes | 当前loader同样缺恶意输入budget，不能作为复杂度上限 |
| Fyrox | 2 / 859 / 35,054 | built-in font bytes嵌入资源；fallback依赖可等待并检测cycle；loader经ResourceManager装载 | per-size atlas可无限增长，属于下限而非性能目标 |
| Unity Graphics | 1 / 966 / 51,202 | atlas allocation/hash/update/release/reset/relayout/size estimate | 本地 `dev/Graphics` 没有Unity TextCore/TMP源码，只能旁证atlas生命周期，不能证明Unity字体能力 |

### 2.4 证据限制

本轮逐文件读取上述96个Zircon聚焦文件，并沿 `font manifest -> importer/artifact -> source_manifest -> TextRenderState -> shared database -> neutral glyph artifact -> UI/Editor consumer` 与 `font manifest -> SDF offline/source context` 两条产品链追踪。生产文件没有 `TODO/FIXME/todo!/unimplemented!/panic!` 命中；这只说明问题是架构和合同缺口，不是注释数量问题。

本轮是静态review，没有运行Cargo、Editor、clean package、系统字体隔离、真实WGPU、恶意字体fuzz、跨平台golden、fault、soak或benchmark。229个test attribute大多是unit/source fixture，不能替代cooked product、multi-session和同负载性能资格。

## 3. 当前产品链与可保留底座

### 3.1 当前实际链

1. `import_font_asset`同步读取manifest和完整source bytes，解码WOFF2、遍历collection face、生成metadata，随后只发布 `ImportedAsset::Font(asset)` 与source URI dependency。
2. `ArtifactCacheFontAsset`序列化source字符串、family、face metadata、coverage、variation与render strategy，不包含blob/hash/license/provenance。
3. 项目扫描把字体源后缀视为auxiliary；runtime `load_text_font_source`取得manifest artifact后，再反查project source path、canonicalize root并返回filesystem path。
4. UI renderer和SDF路径分别再次加载manifest，再由 `TextRenderState::replace_font_source/replace_font_asset` 打开source并mutation process-global database。
5. `SharedTextLayoutService`从global generation shape并批量注册handle；renderer/UI/Editor消费neutral `TextGlyph`。当backend全失败时，synthetic glyph仍可能没有face handle但保持 `requires_rasterization=true`。

这条链在开发树可运行，但source、artifact、face、handle、glyph并不共享一个content generation；删除源码或同时reimport时，没有一个closed lease能证明消费者读到同代字节。

### 3.2 应保留成果

- `FontFaceMetadata`一次解析后保留glyph map、coverage、axes、vertical/decoration metrics与source identity，避免hot path重复解析SFNT。
- fallback candidate与effective instance cache已有entry/byte budget、LRU、hit/miss/eviction report；历史Text01修复应继续作为新service内部实现。
- asset owner mapping、backend ID mapping、batch handle registration与immutable registry snapshot已消除逐glyph全局锁；重构应迁移ownership，不退回逐glyph mutation。
- WOFF2 decoder已用 `catch_unwind` 隔离第三方panic，TTC face extraction检查offset/length overflow并重算checksum；应在其外增加输入、输出和CPU预算，而不是删除。
- SDF generation source持 `Arc<[u8]>`、source/variation hash与generation handle；offline artifact/source/glyph cache有128 MiB/64 MiB等预算和eviction report。
- native bitmap/color glyph已区分Alpha/Subpixel/Color storage；Runtime79的GPU atlas/presentation工作可直接消费新的resolved face lease。

## 4. 继承 P0 的 current-source 状态

**本轮无新增P0。** 下列两项仍由 Runtime11B 唯一计数，Runtime80只补充font-foundation证据与关闭条件。

| 既有阻断 | Current-source证据 | 状态 |
|---|---|---|
| Runtime11B P0-1：font artifact不拥有raw bytes | importer读取/解码bytes后只发布metadata `FontAsset`；artifact payload无blob；源字体被scan视为auxiliary；runtime再开source path | **open**；以artifact-owned byte lease关闭，clean package删除source且禁用系统字体仍成功 |
| Runtime11B P0-2：synthetic glyph伪装为可栅格化成功 | current canonical service已无FNV/codepoint glyph ID，且无face handle不会发布`requires_rasterization`；剩余断路是terminal fallback曾把任意request primary的glyph 0当全局缺字策略 | **static implemented / product gate pending**；generation-owned packaged last-resort face已贯通fallback/metric/SDF source，专用全码点字体、typed status与真实产品像素仍开放 |

Text01/02/09的metadata缓存、fallback索引、batch handle、stable cache slot与resolved artifact owner修复在managed product/Editor资格完成前仍可保持其failure记录open，但当前实现成果不作为Runtime80新gap重复计数。

2026-08-29 superseding status：`RFF-P1-001`、`RFF-P1-002`、`RFF-P1-007` 和 `RFF-P1-011` 的核心生产链已推进到 `static_implemented / product_gate_pending`；`RFF-P1-003/009/010/012` 及 session-owned collection、generational lease、color/capability仍开放。内置 baseline 只增加 generation-owned descriptor/index/family 状态，fallback hot path继续复用已编译 `Arc<CompositeFontIndex>`，没有新增逐glyph锁、解析或分配。未完成 1/100/1k/10k、31样本 CPU/RSS/cache/profile/power 与 Unreal 同负载对比，不作性能或功耗结论。

同日 clean-process primary admission 复核发现：仅注册内置 faces/composite 仍不足以服务空 family query，因为旧 `match_face` 在获得 primary 之前只看公共 fallback families。现已按 explicit/project default/runtime primary/runtime family/platform fallback 分层，默认层 mutation 失效匹配缓存，并以 fresh embedded DB 的 Latin+CJK handle-resolution regression锁定。它不替代 `FontCollectionService`/lease，也不解决旧 generation in-flight lifetime；`RFF-GATE-019/021`继续开放。

同日 FontObject current-source 纵向重审确认另一条 P0 级 MVP 断路：UI loader 以 URI 注册 owner，但 canonical shaping 曾把 URI 当 family 查询，资产 CompositeFont、owner-local typeface、line-metric 证书与 shaped cache 均未共享该 identity。当前已按 Unreal `FSlateFontInfo(FontObject, TypefaceFontName)` 硬切为 owner-scoped primary/composite/fallback/layout/cache 链；asset CompositeFont 在 generation 发布时编译，owner attach/remove 进入 render-input generation，物理 face 仍按 source 去重。owner fallback 仅合并自身声明与 base/platform 链，不读取其他 FontAsset fallback 并集。静态回归与格式/边界扫描完成；这不等于 session-owned `FontCollectionService`、旧 generation lease 或 RFF-GATE-013/019/021/047 完成，Cargo、真实多语言 shape/raster、WGPU/PNG、profile/RSS/power 均未执行。状态为 `font_object_owner_scope_static_implemented / managed_product_validation_pending`。

同日最终 SDF consumer 复核：有效 shaped face/instance handle 原已直接驱动 face/glyph bake，符合 Unreal `FShapedGlyphEntry` 到 SDF atlas 的主路径；但无 handle 或 stale/mismatch 恢复仍以 asset primary 调用 `composite=None`，会跳过 owner CompositeFont。当前恢复路径改为同一 request-owner resolver，stale glyph id 不跨 face 使用，空 family 不生成空名称候选；existing face-resolution cache 仍在 glyph-key miss 后才调用该路径。未执行 profile/WGPU，故这里只关闭结构性 resolver 漂移，不宣称耗时、功耗或产品门完成。

Unavailable owner 纵向复核同时修复了 local typeface 泄漏：显式 FontObject 未注册时不再让其 `font_family` 进入全局 family index。registered owner/empty-family 使用 borrowed query；仅 unknown owner + family 在 request 级 clone-and-clear，随后 shaping、metric certificate 与 SDF recovery 共享默认链。该异常恢复成本不进入正常 glyph hot path，仍待 profile 量化。

Registered owner 候选去重的结构复核继续发现来源丢失：旧逻辑无法区分 local typeface 与 external fallback，只要 owner 没有同名 face 就全局搜索。当前用 `OwnerLocalOnly`/`OwnerThenGlobal` scope 随 family 进入单次 O(n) identity dedupe；CompositeFont/asset/base fallback 可显式升级同名候选，裸 request typeface 不可。候选数与 coverage pass 数不增加，动态规模数据仍待 profile。

Last-resort 纵向复核确认旧P0描述已部分过期：canonical service不再生成synthetic glyph ID，但 terminal
`LastResort`仍返回请求primary，custom FontObject的glyph 0因而成为全局缺字图。当前数据库新增独立
generation-owned `runtime_last_resort_face`，packaged bootstrap绑定内嵌Fira Mono；fallback terminal、line-metric
envelope、neutral handle与SDF face-byte lookup共享该face。覆盖正常cluster不增加candidate访问，missing terminal只做
O(1) face读取。源码回归已落但未运行；`RFF-P1-033/036`的专用全码点tofu与typed status、真实像素/性能门仍开放。

## 5. P1 差距

### 5.1 Asset、cook、source 与输入安全

| ID | 差距 | 工程化要求 |
|---|---|---|
| RFF-P1-001 | `FontAsset`与artifact payload只保存source和metadata，不拥有字体blob | 定义versioned `FontBlobArtifact`，包含content hash、byte lease、face/table能力和dependency generation |
| RFF-P1-002 | importer声明source URI dependency，project scan却把字体source排除为auxiliary，artifact闭包语义互相矛盾 | cook graph必须把source编译为artifact payload或可寻址sidecar，发布前验证dependency closure |
| RFF-P1-003 | artifact没有license/provenance/embeddability/subset策略和import toolchain receipt | 记录来源、授权、fsType/策略、subset input、compiler版本与可审计拒绝原因 |
| RFF-P1-004 | `std::fs::read`、collection face遍历、cmap枚举和metadata输出均无输入/face/table/codepoint/输出预算 | 增加字节、face、table、range、CPU、内存和deadline budget，超限返回typed error |
| RFF-P1-005 | WOFF2仅隔离panic，没有压缩比、decoded size、CPU deadline和worker fault domain | decoder置于bounded worker/process，限制输入/输出比、取消、超时和峰值RSS |
| RFF-P1-006 | TTC standalone extraction会复制选中face的所有table，未与共享blob/预算/derived-data identity闭合 | 采用table lease或budgeted derived face artifact，并缓存content identity |
| RFF-P1-007 | runtime从manifest重新打开source path，非项目路径回落runtime asset root或 `CARGO_MANIFEST_DIR` | shipping runtime禁止source-tree fallback，只接artifact/resource lease |
| RFF-P1-008 | `load_text_font_source`用 `Option`折叠URI、artifact、manifest、path containment、missing、permission和decode错误 | 建立 `FontLoadError` 与phase/context，区分Missing、Invalid、Rejected、Pending和Stale |
| RFF-P1-009 | manifest artifact generation与随后canonicalize/read的source文件没有原子关联 | reimport先生成immutable candidate，再以manifest+blob同代事务发布并保留last-good |
| RFF-P1-010 | raw path/direct font registration可绕过artifact schema、provenance、budget和project policy | 将raw source限制为Editor/import provider；shipping contract只接受verified blob handle |
| RFF-P1-011 | 内置default manifest/TTC仍是loose runtime assets，不是startup artifact lease | 把engine default collection纳入版本化内置package并验证hash/许可/locale coverage |
| RFF-P1-012 | font import在调用线程同步read/decode/parse完整输入，没有cancellation、progress、staging或last-good receipt | 接入受控import job、cancellable stages、资源budget和原子candidate publication |

### 5.2 Database、service、identity 与生命周期

| ID | 差距 | 工程化要求 |
|---|---|---|
| RFF-P1-013 | `shared.rs`以 `OnceLock<RwLock<FontDatabase>>`维护process-global mutable DB | 按project/session持有 `FontCollectionService`；进程层只共享只读content-addressed blob |
| RFF-P1-014 | `shared_text_layout_service()`返回static零状态service，不进入CoreRuntime/manager lifecycle | 通过manager facade解析session-qualified service并具备quiesce/shutdown receipt |
| RFF-P1-015 | handle registry和snapshot也由process-global `OnceLock`持有 | registry归属collection generation，handle带collection/session identity并随lease回收 |
| RFF-P1-016 | 每个 `TextRenderState`都可mutation共享DB，renderer同时成为asset publication owner | renderer只消费snapshot/delta；asset/reimport publication由font service唯一负责 |
| RFF-P1-017 | UI/layout/glyph代码大量临时 `SharedTextLayoutSession::new()`，没有project/window/PIE owner | session由document/view/runtime owner长期持有，显式绑定collection、locale和budget |
| RFF-P1-018 | global初始化先建default DB，再强制 `SystemFontPolicy::Discover` | packaged runtime默认Disabled；system provider必须由project/platform policy显式启用 |
| RFF-P1-019 | default manifest读/parse/register失败被静默忽略，随后由系统字体掩盖 | startup发布typed readiness/degraded reason，必要collection缺失时fail asset或真实tofu |
| RFF-P1-020 | shared mutation clone DB before/after，`sync_font_system`和snapshot又clone backend catalog | immutable snapshot + structural sharing + delta publication，消除大catalog clone峰值 |
| RFF-P1-021 | 每线程最多四个locale `FontSystem`各持backend clone；任意global generation变化重建该线程全部entry | service-owned shaping worker pool分离immutable catalog与locale state，预热并受全局预算 |
| RFF-P1-022 | `FontFaceId(u64)`实为单调Vec slot；retire只清bytes并留tombstone，不能安全compact/reuse | generational face handle、lease pin、active/retired/reclaimable状态和安全slot回收 |
| RFF-P1-023 | `InstancedFaceId`只从face ID与variation hash导出，未包含collection/content generation | identity纳入content hash、face index、normalized coords和collection generation |
| RFF-P1-024 | 单一font generation变化会清bitmap source/retry/atlas、SDF face/source/offline/atlas等整域 | 发布per-face/family/policy delta，按依赖定点失效并允许旧generation完成在途frame |

### 5.3 Collection、fallback、variation、color 与glyph handoff

| ID | 差距 | 工程化要求 |
|---|---|---|
| RFF-P1-025 | 默认TTC只提供Latin与少量SC proof，其余CJK/Arabic/Hebrew/Emoji通过主机family name获得 | 提供可裁剪但repository-owned的产品fallback collection或显式platform provider |
| RFF-P1-026 | asset/style主要用family字符串选择face，缺stable collection/family/face source identity | authoring保存stable asset/collection handle，family仅作显示与CSS式请求条件 |
| RFF-P1-027 | `FontCultureTag`自称normalized BCP-47，实际只trim并做ASCII大小写prefix比较 | 使用标准BCP-47 canonicalization、likely-subtag/script/region policy与版本化locale data |
| RFF-P1-028 | `FontScript`是手列少量脚本加 `Other(u32)`，script判定/coverage没有Unicode data version receipt | 绑定Unicode Script/Script_Extensions版本并覆盖Indic、SE Asian、historic与emoji sequence |
| RFF-P1-029 | composite rule只有family/scripts/ranges/cultures vector，没有stable rule ID、priority、conflict或authoring diagnostic | 编译为有ID/优先级/来源的interval/script decision graph并报告shadow/overlap/unreachable rule |
| RFF-P1-030 | system font discovery没有provider/catalog fingerprint、OS source、license或replay identity | `SystemFontProvider`发布catalog generation、face hashes、policy和可复现selection receipt |
| RFF-P1-031 | direct registration的 `FontFaceMetadata::from_sfnt_bytes`解析失败会退为Unknown，而非拒绝invalid face | verified registration必须返回typed parse/backend error，Unknown只用于明确的external opaque provider |
| RFF-P1-032 | fallback/face resolution大量返回 `Option`，无法表达pending dependency、partial cluster coverage、policy reject与budget exhaustion | 定义typed `FontResolveOutcome`，保存候选、chosen face、missing cluster与fallback reason |
| RFF-P1-033 | 缺字没有engine-owned真实tofu face/glyph，最终会进入synthetic ID路径 | 内置可rasterize tofu face并保留missing codepoint diagnostic，不伪造backend glyph ID |
| RFF-P1-034 | artifact metadata不声明COLR/CPAL、CBDT/CBLC、sbix、SVG、variation selector、bitmap strike等能力 | import时生成versioned capability/table summary并由raster route协商 |
| RFF-P1-035 | `render_strategy`是手工allow native/SDF开关，未结合字体capability、scale、platform、color和accessibility给出effective receipt | 统一capability resolver，输出chosen route、fallback、quality和unsupported reason |
| RFF-P1-036 | `TextGlyph`只有optional face/instance和boolean rasterization，没有content/collection identity与resolved status | 增加generation-qualified `ResolvedGlyphStatus`；无完整face的非空白glyph禁止发布到raster队列 |

### 5.4 Cache、product integration、diagnostics 与qualification

| ID | 差距 | 工程化要求 |
|---|---|---|
| RFF-P1-037 | renderer `UiFontAssetCache`是无界HashMap，Ready/Missing/Error都长期驻留且无byte/entry report | 归入service cache，定义entry/byte/negative TTL、LRU、owner teardown与telemetry |
| RFF-P1-038 | decoration metrics按 `(FontFaceId, display_px.to_bits())`无界增长，只在全face invalidation清空 | 量化size并设置entry/byte budget；按face delta删除，报告hit/miss/eviction |
| RFF-P1-039 | handle registry每次publication clone完整face/instance Vec，generation变化全reset | immutable segmented snapshot/delta，稳定slot与bounded publication cost |
| RFF-P1-040 | UI renderer、SDF face cache和offline SDF各自解析manifest/asset identity，形成多个私有cache与错误语义 | font service一次解析并发布shared `ResolvedFontAsset` snapshot |
| RFF-P1-041 | offline SDF虽有source hash/variation hash，却仍先从source-path manifest取得identity；与cooked font artifact没有同代保证 | offline artifact key直接引用FontBlobArtifact content/generation和raster toolchain version |
| RFF-P1-042 | DB bytes、backend clone、TLS FontSystem、SDF source/offline、native bitmap与renderer cache没有统一resident budget | 建立分层memory domains、global/project caps、pressure callbacks与完整resident report |
| RFF-P1-043 | font mutation以process generation触发跨route全清，无法保留未依赖changed face的shape/raster/atlas数据 | dependency-indexed invalidation和old-generation lease，禁止无关cache storm |
| RFF-P1-044 | `UiFontRegistry`曾只有测试consumer，却维护另一套u32 ID/family/fallback/system source真值 | **实现完成，验收待定**：source/test owner 已硬删除；`FontDatabase` 是唯一 owner，不保留 facade 或第二 registry |
| RFF-P1-045 | Editor runtime text只消费global artifact/handle，没有project/PIE collection lease、authoring preview generation或teardown隔离 | Editor document/preview/session显式持collection snapshot并跨PIE隔离 |
| RFF-P1-046 | typed DB错误、fallback diagnostics和cache report分散在私有模块；product readiness没有统一低基数font receipt | 汇总 `FontDiagnosticsReceipt`，关联project/session/collection generation且不记录raw text |
| RFF-P1-047 | 许多测试依赖 `CARGO_MANIFEST_DIR`、loose fixture或当前主机font；没有clean package、双session和catalog replay gate | 建立hermetic artifact fixtures、source-deleted package和multi-session test matrix |
| RFF-P1-048 | 没有当前真实App/Editor跨平台font golden、corrupt-font fault、long reimport soak与同负载Unreal基准 | 资格artifact记录source hash、catalog、locale、backend、阈值和当前源码fingerprint |

## 6. P2 差距

| ID | 差距 | 收敛方向 |
|---|---|---|
| RFF-P2-001 | `FontFamilyName::new`只trim，canonical key规则分散 | 统一显示名、匹配名和stable identity，避免locale/Unicode case临时规则 |
| RFF-P2-002 | `FontCultureTag`字段注释宣称normalized但类型不执行normalization | 修正文档并让构造返回validated canonical tag |
| RFF-P2-003 | dead `UiFontRegistry`的saturating `next_id`到上限后可重复发ID | **随 RFF-P1-044 关闭**：registry 已删除，无 facade 或可重复 ID 路径 |
| RFF-P2-004 | 128项 `SdfFontAssetFaceCache`每次eviction扫描全部recency map | 使用O(1) LRU或复用共享bounded cache |
| RFF-P2-005 | decoration cache没有自身report，无法看出size cardinality和fallback metric占比 | 增加低成本cache report并纳入统一receipt |
| RFF-P2-006 | fallback decoration thickness用小Vec `contains`去重face，长fallback run会反复线性检查 | 使用small-set/indexed unique face list并设置每run candidate budget |
| RFF-P2-007 | metadata fallback为“测试可用”扫描BMP；空cmap TTC还在production metadata插入codepoint 0 | 测试fixture策略移出production parser，invalid/empty cmap显式报告 |
| RFF-P2-008 | default family在Rust数组、font manifest和backend generic family投影间重复 | 建立单一compiled default collection artifact |
| RFF-P2-009 | locale FontSystem容量固定4且不可配置，也没有eviction/rebuild report | 将capacity纳入service profile并暴露low-cardinality metrics |
| RFF-P2-010 | UI cache 原先只在 `cfg(test)`暴露Ready/Missing/Error，production diagnostic无法查询 | **部分实现，验收待定**：`ScreenSpaceUiTextPrepareReport.font_assets` 现提供一帧一次的有界聚合 snapshot（Ready/Missing/Error、source contract/IO/decode 与 registration 分类），不持久化路径或原始错误；仍需按 asset reference 的受控 typed receipt 与 Editor reimport action |
| RFF-P2-011 | render/manifest helpers以相似名称重复包装 `Option`，调用方容易误把配置缺失和加载损坏等价 | 收敛一个typed resolve API与一个compat adapter，随后硬切adapter |
| RFF-P2-012 | font、source、face、family、collection、instance、handle术语在模块和旧计划中混用 | 固定术语/identity表，并让API名称对应生命周期层级 |

## 7. 与参考引擎的差异归纳

| 维度 | Zircon current source | 参考证据 | 必须吸收 |
|---|---|---|---|
| Raw payload/cook | manifest artifact无bytes，runtime重开source | Unreal `UFontFace`/`UFontBulkData`拥有payload并按loading policy cook；Bevy `Font` asset持Blob；Fyrox built-in bytes嵌入 | closed blob artifact、loading/residency policy、memory/accounting |
| Collection/fallback | process-global DB + host-dependent family strings | Unreal composite range/culture/history；Godot explicit fallback/system policy与language/script override | project/session collection snapshot、compiled rule graph、provider receipt |
| Face/variation | cached metadata和variation已有基础，但ID是process slot | Godot RID/linked variation显式创建/free；Unreal face data可被cache安全持有 | generational content identity、lease和targeted invalidation |
| Cache lifecycle | 多owner、global generation全清；局部cache有预算 | Unreal deferred safe-thread flush/memory query；Godot clear/free RID；Unity Graphics release/reset/relayout/size estimate | owner teardown、budget、delta invalidation、pressure和receipt |
| Product qualification | unit fixtures依赖source tree/system font | 参考共同把bytes/resource/cache作为显式对象；Unreal可模拟cooked lazy load | source-deleted package、multi-session、catalog replay、fault/soak/benchmark |

参考实现也有下限：Bevy loader会无界 `read_to_end`，Fyrox atlas允许任意页数，Godot/Unreal存在全cache flush。Zircon不能照抄这些局部成本；应吸收它们清晰的payload、resource、backend和lifecycle边界，再以Zircon自己的budget、immutable generation和content-addressed identity提高工程上限。

## 8. 目标架构

### 8.1 `FontBlobArtifact`

唯一runtime字体字节权威，包含schema、content hash、byte payload/sidecar lease、source provenance/license policy、face directory、table/capability summary、coverage/metrics、subset/cook toolchain、dependency generation和platform chunk信息。source path只存在于Editor reimport record。

### 8.2 `FontCollectionService`

按project/session拥有default families、composite rules、locale policy、system provider和immutable `FontCollectionSnapshot`。进程级只共享按content hash去重的只读blob与安全backend资源。mutation生成delta，旧snapshot由frame/layout lease延迟回收。

### 8.3 Stable resolved identities

`FontCollectionHandle -> FontFamilyHandle -> FontFaceHandle -> FontInstanceHandle`均带collection generation/content identity。`ResolvedFontFaceLease`同时提供verified bytes/backend face/capability；`ResolvedGlyphStatus`只能是Ready、Tofu、Pending、Missing、Invalid或BudgetExceeded，非空白Ready/Tofu必须有可rasterize face。

### 8.4 Provider 与cache domains

系统字体是显式 `SystemFontProvider`，发布catalog fingerprint和selection receipt；不参与deterministic package默认正确性。shape、metrics、native raster、SDF、atlas、renderer cache共享dependency keys但各有entry/byte/time budget和pressure policy，统一进入 `FontDiagnosticsReceipt`。

### 8.5 产品边界

Runtime UI、SceneText、Editor preview、accessibility和plugin API只消费相同collection snapshot与neutral glyph artifact。Editor可请求reimport/preview candidate，但无权直接mutationrenderer DB。headless/server默认不安装系统字体provider或GPU cache。

## 9. 分层重构里程碑

### M0：冻结合同与基线

冻结当前fingerprint、术语、identity、artifact schema、service owner、status和budget；把Runtime11B P0-1/P0-2及Text01/02/09开放记录映射到唯一owner。

### M1：关闭font cook断链

生成 `FontBlobArtifact`，runtime从artifact byte lease建face；移除shipping source-path fallback。完成source-deleted package、DLC、remote/read-only mount和deterministic hash门。

### M2：输入安全与typed load

为TTF/OTF/TTC/WOFF2设置字节/face/table/cmap/decode ratio/CPU/RSS budget、cancellation和fuzz corpus；所有load/parse/policy错误进入typed outcome。

### M3：Session-owned collection service

引入manager facade、project/session snapshot、explicit system provider、structural sharing和delta publication；迁移renderer mutation与ad-hoc layout sessions。

### M4：Generational face/instance lease

替换monotonic tombstone slot和process-only instance hash；实现content/generation handle、in-flight lease、safe compact/reclaim和plugin/session teardown。

### M5：Fallback/capability compiler

编译BCP-47/Unicode-script/range规则、诊断冲突；补engine-owned tofu和repository-owned default collection；生成variation/color/bitmap/SVG capability与effective route receipt。

### M6：Typed glyph completeness

删除synthetic rasterizable success；shape/layout/artifact/UI/SDF/native renderer全链消费 `ResolvedGlyphStatus`，不完整glyph fail closed或显示真实tofu。

### M7：Cache与pressure收敛

`UiFontRegistry` 硬切已完成；继续合并manifest/font cache，并设置UI/decor/TLS/DB/SDF/native全域budget、targeted invalidation、resident bytes和pressure recovery。

### M8：产品与性能资格

完成App/Editor/headless、双project/PIE、跨OS deterministic/system-provider、恶意字体、100k reimport、locale/thread scale、color/variation golden、fault/soak和同负载Unreal比较。

## 10. 资格门

### 10.1 Artifact、cook 与安全门

| Gate | 必须满足 |
|---|---|
| RFF-GATE-001 | artifact含bytes/hash/schema/provenance/license/face capability，manifest和blob generation一致 |
| RFF-GATE-002 | 删除project source和原始font、禁用系统字体后，packaged App仍完成声明字体shape/raster |
| RFF-GATE-003 | WOFF2压缩炸弹在decoded size/ratio/deadline/RSS阈值内typed失败 |
| RFF-GATE-004 | 恶意TTC face/table offset/count和巨大cmap在预算内拒绝，无panic/OOM |
| RFF-GATE-005 | import job支持cancel/deadline/progress，取消后不发布partial artifact |
| RFF-GATE-006 | 相同输入、policy和toolchain得到bit-stable artifact/hash |
| RFF-GATE-007 | 禁止嵌入或不满足license/subset policy的字体在cook阶段有可审计拒绝 |
| RFF-GATE-008 | reimport candidate失败保留last-good；manifest/blob/metadata原子切代 |
| RFF-GATE-009 | DLC、remote CAS、read-only mount和无源码安装均能通过同一byte lease加载 |
| RFF-GATE-010 | shipping binary/source guard证明没有runtime raw project font path入口 |
| RFF-GATE-011 | missing/corrupt/rejected/pending/stale/permission错误可区分并关联asset generation |
| RFF-GATE-012 | TTF/OTF/TTC/WOFF2 fuzz、sanitizer和parser regression corpus纳入required gate |

### 10.2 Owner、identity 与determinism门

| Gate | 必须满足 |
|---|---|
| RFF-GATE-013 | 同进程两个project注册同family不同bytes，selection/layout/glyph互不泄漏 |
| RFF-GATE-014 | PIE hot reload/locale/fallback改变不推进Editor或另一PIE session generation |
| RFF-GATE-015 | session teardown后registry、TLS/worker state、blob lease和resident bytes回到阈值 |
| RFF-GATE-016 | system font provider默认关闭；启用时输出catalog fingerprint和selection receipt |
| RFF-GATE-017 | 相同cooked collection在Windows/Linux/macOS得到相同fallback face/content identity |
| RFF-GATE-018 | 单face reimport只失效依赖它的shape/metrics/raster/atlas entry |
| RFF-GATE-019 | 旧generation在途frame可完成，retire后才回收face/backend/blob |
| RFF-GATE-020 | 100k次reimport/unload后slot、tombstone、RSS和lookup latency有硬上限 |
| RFF-GATE-021 | 16 shaping线程×4 locale的catalog clone/rebuild/RSS/首帧延迟低于阈值 |
| RFF-GATE-022 | 小位宽handle exhaustion/wrap模型拒绝stale handle且不别名新face |
| RFF-GATE-023 | plugin font collection unload不会破坏宿主/其他plugin handle或在途frame |
| RFF-GATE-024 | Core manager lifecycle覆盖register/ready/quiesce/shutdown/reopen并有receipt |

### 10.3 Fallback、capability 与glyph正确性门

| Gate | 必须满足 |
|---|---|
| RFF-GATE-025 | engine-owned collection覆盖Latin/CJK/Arabic/Hebrew/Indic/emoji基准，缺字落真实tofu |
| RFF-GATE-026 | zh-Hans/zh-Hant/ja/ko及script/region变体按canonical BCP-47选择golden face |
| RFF-GATE-027 | Unicode Script/Script_Extensions版本固定，Indic/SE Asian/emoji sequence corpus通过 |
| RFF-GATE-028 | composite overlap/shadow/unreachable规则在import/Editor显示确定性diagnostic |
| RFF-GATE-029 | variable axis clamp/quantize/named instance与weight synthesis跨backend一致 |
| RFF-GATE-030 | TTC多face family/style/index映射在reimport和package后稳定 |
| RFF-GATE-031 | COLR/CPAL、CBDT/CBLC、sbix、SVG与variation selector输出capability/route/fallback golden |
| RFF-GATE-032 | system catalog增删face会改变catalog generation并使replay/cache明确失配 |
| RFF-GATE-033 | invalid face无法注册为active Unknown face，返回typed parse/backend错误 |
| RFF-GATE-034 | 任意非空白 `requires_rasterization` glyph必有完整face lease；synthetic ID为零 |
| RFF-GATE-035 | engine tofu本身有真实face/glyph、可被native/SDF route绘制并保留missing diagnostic |
| RFF-GATE-036 | resolved artifact携collection/content generation，stale snapshot不可解析新handle |

### 10.4 Cache、product、observability 与性能门

| Gate | 必须满足 |
|---|---|
| RFF-GATE-037 | 10k font asset/negative lookup下UI cache entry/bytes/latency受限并可恢复 |
| RFF-GATE-038 | 百万size variation请求下metrics/decor cache受限且命中/驱逐可观测 |
| RFF-GATE-039 | DB/TLS/SDF/native/renderer总resident bytes受project/process cap和pressure policy约束 |
| RFF-GATE-040 | 单face hot reload保留无关atlas page，旧page按fence/lease退休 |
| RFF-GATE-041 | offline SDF artifact与font blob使用同content/generation/toolchain identity |
| RFF-GATE-042 | allocator/decoder/backend OOM fault返回typed outcome，不panic、死锁或发布半状态 |
| RFF-GATE-043 | Runtime UI、SceneText与Editor preview共享同一service和resolved face identity |
| RFF-GATE-044 | headless/server不发现系统字体、不创建GPU atlas且仍能做deterministic layout |
| RFF-GATE-045 | metrics/diagnostics低基数、有sample budget且不记录raw/secure text |
| RFF-GATE-046 | shipped App与Editor在source-deleted环境通过Latin/CJK/RTL/emoji/color/variation视觉golden |
| RFF-GATE-047 | 同font set、locale、text、thread和cache状态对比Unreal，记录CPU/RSS/p95/p99而非单次峰值 |
| RFF-GATE-048 | 所有资格artifact记录当前source/reference fingerprint、OS/catalog/backend、阈值和non-ignored结果 |

## 11. 禁止的临时修补

- 不得把字体source复制到package旁边后继续按路径打开；必须由artifact拥有bytes与generation。
- 不得用“系统字体通常存在”补全默认collection；系统provider必须显式、可审计、可复现。
- 不得保留process-global DB再加project name前缀；可变fallback、registry和generation必须按session隔离。
- 不得只扩大HashMap、TTC、WOFF2或cache常量；输入、resident、CPU与输出均需typed budget和pressure策略。
- 不得把synthetic glyph ID映射到任意方框贴图后宣称P0关闭；必须有真实tofu face和typed completeness。
- 不得新增Editor-only或UI-only font registry；所有产品面消费同一collection service。
- 不得用test attribute数量、source string断言、开发树fixture或单台机器系统字体通过关闭资格门。
- 不得以“比Unreal更快”的不同字体集、不同fallback、不同像素质量或warm/cold状态比较代替同负载基准。

## 12. 本轮产出边界

本文以 current-source 静态 review 和重构路线为主；后续状态只在对应 finding 中记录实现切片，不能替代受管 Cargo 或产品资格。Runtime81继续审查shaping、Unicode、BiDi与line breaking；Runtime82审查editing、IME和secure text。Runtime80实施优先级固定为：先关闭artifact字节与typed failure，再切session-owned collection和generational lease，最后做fallback/capability/cache/product资格；GPU atlas细节继续由Runtime11C/79拥有。

2026-08-29 FontObject owner 路径二次复核：注册事务已产生的有序 face 集原未保留，请求 primary、每个 scoped
family 与 line-metric 查询会从 source keys 重新查表并物化完整 `Vec<FontFaceId>`。当前 owner state 一次发布
generation-local `Arc<[FontFaceId]>`，上述路径共享借用；family candidate 与 coverage 输出规模不变。源码借用回归、
rustfmt、diff-check 已完成；Cargo、allocation profile、p50/p95、RSS/power、真实 WGPU/PNG 仍待受管门，状态为
`owner_face_generation_slice_static_implemented / request_recollection_removed / dynamic_evidence_pending`。

2026-08-29 generation snapshot 前置切片：`FontCollectionSnapshot` 已把 exact generation 与
`Arc<FontDatabase>` 合成单一不可变发布对象；canonical shaping attempt 和 cosmic locale FontSystem 共享该
snapshot，不再先采 generation、后从 global 重选 database。稳定 snapshot acquisition 为 O(1) Arc clone；仍需
mutable database 的 renderer clone 单独暴露 `shared_owned_snapshot_clone` profile span。同 generation 的等价
诊断发布不触发 locale cache rebuild。Arc 复用、旧代跨发布存活和显式旧 snapshot backend binding 回归已写入，
rustfmt/diff/static guards 通过；Cargo 与动态测试未执行。后续非验收切片已将 handle registry、snapshot 与 metrics
归入 `FontCollectionService`，`TextFontFaceHandle` 纳入 collection identity，canonical/artifact projection 和 SDF
consumer 均显式绑定集合；UI 在途 artifact view 同时租用旧代 database Arc 与 registry snapshot，字体发布后仍可完成
既有帧且不会读取新代资源。`RFF-P1-015` 因此为 `static_implemented / managed_validation_pending`；`RFF-P1-013`
仍缺真实 manager owner，`RFF-P1-017` 仍缺 document/window/PIE session 注入，`RFF-P1-022` 的 backend face slot
代际回收仍开放。没有 Cargo/WGPU/PNG/profile/RSS/power 结论。

2026-08-29 owner-ready continuation：裸 generation 已从布局与 artifact 所有权边界移除，统一为
`FontCollectionRevision(collection_id, generation)`。collection-bound `SharedTextLayoutSession` 现贯通
`UiTextMeasureCache`、retained `UiSurface`、physical/logical fragment、plain/rich/secure glyph artifact；artifact
在最后一次 handle registration 后捕获 database 与 resolver publication lease，renderer line acquire 不再通过
进程集合重建租约。foreign collection mutation 不触发 surface，owned mutation 才推进失效且复用同一 session。
`RFF-P1-015` 静态实现因此覆盖 layout-to-raster lease 全链，但仍为 `managed_validation_pending`；
`RFF-P1-013/014/017` 的真实 manager/module/window/PIE 注入、`RFF-P1-022` face slot reclaim 与全部动态/产品/
性能/功耗门保持开放。

2026-08-29 screen-space continuation：`TextRenderState` 的 collection owner 已向上贯通
`ScreenSpaceUiTextSystem` 与 `ScreenSpaceUiRenderer` 显式构造边界；沿用 process task-pool worker budget 不再隐含
选择 process font collection。renderer plan、segment product 与 stale artifact admission 统一比较完整
`FontCollectionRevision`，相同 generation 的 foreign collection 不能复用产品。每个相关 cache identity 只增加一个
collection-id word，比较保持 O(1)，batch admission 保持原 O(text batches)，无新增 shape/coverage/database clone/
per-glyph 工作。真实 Core manager/module/window/PIE owner 仍未把同一 service 注入 Surface 与 renderer，因此
`RFF-P1-013/014/017` 保持开放；Cargo、动态 profile、RSS/功耗与 WGPU/PNG 未执行。状态：
`screen_space_renderer_collection_boundary_static_implemented /
product_manager_injection_open / managed_validation_pending`。
