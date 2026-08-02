---
related_code:
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/facade/impls.rs
  - zircon_runtime/src/asset/management.rs
  - zircon_runtime/src/asset/watch/mod.rs
  - zircon_runtime/src/asset/watch/fold_events.rs
  - zircon_runtime/src/ui/template/asset/prototype_store.rs
  - zircon_runtime/src/ui/template/asset/prototype_file_cache.rs
  - zircon_runtime/src/ui/template/asset/invalidation/graph.rs
  - zircon_runtime/src/ui/template/asset/invalidation/fingerprint.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/collect.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/resolve.rs
  - zircon_runtime/src/ui/template/asset/localization
  - zircon_runtime/src/ui/v2/loader.rs
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_editor/assets/ui/editor/components
  - tools/zircon_build.py
plan_sources:
  - .codex/plans/Zircon UI .zui 组件资产与 Unreal 风格入口重构计划.md
  - .codex/plans/M16 UI Compiled Artifact And Package Validation Implementation Plan.md
  - .codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
status: in_progress
---

# 05 UI 资产管理收束

## 1. 目标

让 UI 资产（`.zui` UI 文档，按 `asset.kind` 承载 component / view / style / theme_tokens profile，以及 theme、字体、图标、UI 用图片/材质）成为 runtime 资产大模块（facade / management / importer / watch）的一等公民：统一加载、依赖追踪、热重载、包验证、引用解析。`.ui.toml` / `.v2.ui.toml` 后缀已退役，不作为当前页面模板或布局描述入口；补齐归档 M15（resource refs 全链）。本计划与 runtime asset 大模块协同推进，editor 侧只消费。

## 2. 现状（按代码核实修正）

### 2.1 已存在的设施

| 能力 | 落点 | 证据 |
|------|------|------|
| UI 资产类型（双轨已注册） | `zircon_runtime/src/asset/assets/ui.rs` | `UiLayoutAsset`（:13）、`UiWidgetAsset`（:19）、`UiStyleAsset`（:25）、`UiV2ViewAsset`（:31）、`UiV2ComponentAsset`（:37，含 `from_zui_str` :114）、`UiV2StyleAsset`（:43） |
| 引用收集 | 同上 | `ui_asset_references`（:135）、`ui_v2_asset_references`（:160）→ `AssetReference` |
| 依赖失效骨架 | `zircon_runtime/src/ui/template/asset/invalidation/` | `UiInvalidationGraph::classify`（graph.rs:11/:14）、fingerprint.rs、diagnostic.rs |
| resource ref 路径级解析 | `.../resource_ref/` | collect.rs、`UiResourcePathResolver`（resolve.rs:9，res/asset/project root 三级）、`validate_resource_dependency_files`（:44） |
| 原型仓库 | `.../prototype_store.rs` | `UiPrototypeStore`{insert/insert_alias/get/component_prototype}（:7–:43）、`UiPrototypeStoreBuilder`（:70） |
| 源加载缓存 | `.../prototype_file_cache.rs` | `UiPrototypeStoreFileCache::load_flat_store/try_load_flat_store`（:18–:51）——**缓存的是源文件读入，非编译产物落盘** |
| watcher 全模块 | `zircon_runtime/src/asset/watch/` | asset_watcher、fold_events（事件合并）、map_notify_event、is_meta_sidecar、watch_loop 共 17 文件 |
| 字体/纹理样板 | `zircon_runtime/src/asset/assets/{font.rs, texture/}` + `importer/ingest/import_font_asset.rs` | 新类型注册可照此样板 |
| staged build 资产合并 | `tools/zircon_build.py` | `stage_engine_assets`（:778）合并 `zircon_editor/assets` + `zircon_runtime/assets` → `ZirconEngine/assets`（:32–:33） |
| 本地化表 | `.../localization/` | M14 已落地 |

### 2.2 真实缺口

1. **资产类型已存在但载体硬切未闭环**：`UiThemeAsset`、`UiIconAsset` 已在统一 UI asset 模块落地；theme importer 仍需完全经过 `.zui` `theme_tokens` profile 与 `asset.kind` 校验，避免裸 TOML 入口重新成为第二载体。
2. **依赖索引已存在但查询与规模合同未闭环**：`UiAssetDependencyIndex` 已提供依赖数据面；editor resource browser、watch 级联的有界扫描，以及与 node 治理脚本的一致性仍需验收。
3. **消费级 resolver 已有落点但产品覆盖未闭环**：resource resolver、resolution report 与热重载计划已存在；icon atlas slot、GPU handle、font face 与 theme 的实际消费、占位资源和可查询诊断仍需端到端证据。
4. **热重载链已有实现但全类型闭环未达成**：watch、依赖索引与 invalidation 路径已存在；仍需证明 `.zui`/theme/icon/font 变更只重编译并 damage 受影响节点，不发生全图扫描或无关子树重建。
5. **persistent compiled store 已存在但生命周期未收束**：`UiCompiledArtifactStore` 与 v2 file cache 已接线；仍需解决重复解析/owned artifact、无界 stale artifact、generation 所有权和 staged build payload 验收。

## 3. 设计

### 3.1 资产类型与引用图

- 类型收口：复核并收束现有 `UiThemeAsset`（包 `UiThemeDocument`，04 M1 类型）与 `UiIconAsset`（SVG 源 → tessellation 或 SDF 位图，栅格策略衔接 03）的 importer/facade/消费链；既有 `UiV2ComponentAsset`/`UiV2ViewAsset`/`FontAsset`/`TextureAsset` 维持。
- 权威依赖索引归 runtime：编译期把 `imports`（组件→组件）、`resource_refs`（组件→icon/font/texture/theme）、`localization_refs` 写入 prototype store 旁的 `UiAssetDependencyIndex`；watch 事件沿索引做级联指纹失效（叶子改动只重编译受影响者）。
- node 治理脚本继续作 CI 把关，但 runtime 索引是运行时事实来源；CI 增加双跑一致性对比。

### 3.2 引用解析与消费（M15 补课）

- `UiResourceResolver`（消费级，区别于现有路径级 `UiResourcePathResolver`）：模板中 `icon = "icons/scene.svg"` 等引用 → asset handle → GPU 资源（atlas slot / texture view / font_id）；解析失败给出占位资源 + `UiResourceResolveDiagnostic`（不 panic、不静默空白）。
- editor resource browser 数据面：依赖索引提供双向查询（供计划 09 批次 3 的 Reference Finder / Asset Dependency 模块）。

### 3.3 热重载与缓存

- 热重载链定稿：watch 事件（fold_events 合并）→ `UiInvalidationGraph::classify` → 指纹级联失效 → 重编译 → theme 走 restyle（04 M6）/ 结构变更重建受影响 surface 子树 → damage。`.zui`、template、theme、icon、font 全类型支持；editor 实机改文件即时生效是验收硬指标。
- persistent compile cache：`UiCompiledArtifactStore` 把编译产物 + 指纹落盘到工作区缓存目录；冷启动校验指纹（含 schema 版本 + 编译器版本）直接加载；`tools/zircon_build.py` 把 compiled artifact 打进 `ZirconEngine/assets`（衔接 M16 包验证）。

### 3.4 与 runtime 大模块的 gating

- 本计划假定 asset 大模块的句柄/生命周期/事件契约稳定；若 Bevy-Style Asset Stack 计划调整 handle 语义，本计划同步适配，UI 侧不自建第二套句柄。
- 材质类 UI 引用（UI 特效材质）等待渲染计划 `docs/plans/zircon_runtime/render/08-material-shader-permutation.md` 的资产接口，先以 TextureAsset 路径兜底。
- （2026-07-02 评审收口）字体资产 schema 演进（TTC/WOFF2/变量字体）归 `docs/plans/zircon_runtime/text/01` FR-M2，本计划只消费其产物；`UiResourceResolver::Font` 走 text/01 的 `FontFaceId` 契约。

## 4. 接口与数据结构草案

```rust
// 新增 zircon_runtime/src/asset/assets/ui_theme.rs（样板：assets/font.rs）
// （2026-07-02 评审收口，U6）theme 文档 = asset.kind = theme_tokens 的 `.zui` profile（生产已有 editor_tokens.zui），
// 复用 `.zui` importer 分派做物化/facade 注册，不引入第二载体；
// 原 `from_toml_str` 独立 TOML 载体草案作废。
pub struct UiThemeAssetDocument {
    pub theme: UiThemeDocument,                 // 04 M1 类型（interface style/theme.rs）
}
impl UiThemeAssetDocument {
    // （作废草案）pub fn from_toml_str(document: &str) -> Result<Self, UiAssetDocumentError>;
    pub fn from_zui_theme_tokens(document: &str) -> Result<Self, UiAssetDocumentError>;   // 经 .zui profile 分派
}

// 现有 zircon_runtime/src/asset/assets/ui.rs 中的 UiIconAsset
pub struct UiIconAsset {
    pub source: UiIconSource,                   // Svg { text: String } | Bitmap { ... }
    pub default_size: f32,
    pub semantic_id: String,                    // "icons/scene" 等稳定 id
}

// 现有 zircon_runtime/src/ui/template/asset/dependency_index.rs
pub struct UiAssetDependencyIndex {
    /* imports / resource_refs / localization_refs 正反向表，编译期填充 */
}
impl UiAssetDependencyIndex {
    pub fn record_compiled(&mut self, asset_id: &str, refs: &[AssetReference]);    // 现有 AssetReference
    pub fn dependents_of(&self, asset_id: &str) -> impl Iterator<Item = &str>;     // 谁引用我
    pub fn references_of(&self, asset_id: &str) -> &[AssetReference];              // 我引用谁
    pub fn cascade_invalidation_targets(&self, changed: &str) -> Vec<String>;      // watch 级联用
}

// 新增 zircon_runtime/src/ui/template/asset/resource_ref/resolver.rs（消费级）
pub struct UiResourceResolver { /* 依赖索引 + asset facade 句柄缓存 + 占位资源 */ }
pub enum UiResolvedUiResource {
    Icon { atlas_slot: UiIconAtlasSlot },       // 新增（M4 图标通道）
    Texture { handle: /* asset facade 句柄类型 */ },
    Font { face_id: FontFaceId },               // （2026-07-02 评审收口）改用 text/01 契约类型 FontFaceId（原 UiFontId 草案作废）
    Theme { fingerprint: u64 },
    Placeholder { diagnostic_index: usize },    // 解析失败兜底
}
pub struct UiResourceResolveDiagnostic { pub reference: AssetReference, pub reason: String }
impl UiResourceResolver {
    pub fn resolve(&mut self, reference: &AssetReference) -> UiResolvedUiResource;
    pub fn diagnostics(&self) -> &[UiResourceResolveDiagnostic];
}

// 现有 zircon_runtime/src/ui/template/asset/compiler/cache/persistent.rs
pub struct UiCompiledArtifactStore { root: PathBuf /* 工作区缓存目录约定 */ }
pub struct UiCompiledArtifactKey { pub asset_id: String, pub fingerprint: u64, pub schema_version: u32, pub compiler_version: u32 }
impl UiCompiledArtifactStore {
    pub fn load(&self, key: &UiCompiledArtifactKey) -> Option<CompiledUiDocument>;  // 指纹失配即 None
    pub fn store(&self, key: &UiCompiledArtifactKey, doc: &CompiledUiDocument) -> std::io::Result<()>;
}
```

## 5. 模块与文件落点

**已有并待收束**：`zircon_runtime/src/asset/assets/ui.rs`、`zircon_runtime/src/ui/template/asset/dependency_index.rs`、`zircon_runtime/src/ui/template/asset/resource_ref/resolver.rs`、`zircon_runtime/src/ui/template/asset/compiler/cache/persistent.rs`。**仍待新增或完成产品接线**：`zircon_runtime/src/ui/icon_atlas/{mod.rs, raster.rs, atlas.rs}`（仅在 M4 复用 graphics/text 图集评估否定后）、`zircon_editor/assets/ui/editor/icons/`（默认图标包）。

**修改**：

| 路径 | 改什么 |
|------|--------|
| `zircon_runtime/src/asset/assets/mod.rs` | 注册两个新类型（薄声明） |
| `zircon_runtime/src/asset/facade/impls.rs`、`management.rs` | 新类型 load/handle 接线 |
| `zircon_runtime/src/asset/watch/`（消费侧） | UI 资产事件 → dependency_index 级联 |
| `zircon_runtime/src/ui/template/asset/{loader.rs, prototype_store.rs}` | 编译期填充 dependency_index |
| `zircon_runtime/src/ui/v2/{loader.rs, file_cache.rs}` | 冷启动先查 UiCompiledArtifactStore |
| `tools/zircon_build.py` | `stage_engine_assets`（:778）附带 compiled artifact 目录 |

**删除（硬切换义务）**：node 脚本作为 `.zui` import 图「事实来源」的地位（脚本保留为 CI 把关，文档与流程更新指向 runtime 索引）；resolve 路径级与消费级职责重叠部分在 M3 收口（`UiResourcePathResolver` 保留为 resolver 内部组件）。

## 6. 热重载链时序

```
文件变更 → asset watch（notify → map_notify_event → fold_events 合并）
  → asset_uri_for_path 定位资产 → UiAssetDependencyIndex.cascade_invalidation_targets
  → invalidation/fingerprint 级联失效 → 受影响资产重编译（查/写 persistent store）
  → 按类型分流：theme → UiThemeRegistry restyle（04 M6，不重建树）
               .zui/template → 重建受影响 surface 子树
               icon/font/texture → 资源句柄替换 + 引用节点 damage
  → damage 标记 → 下帧重绘
```

## 7. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
|---|------|---------|---------|--------|
| M1.S1 | 收束现有 `UiThemeAsset` + `UiIconAsset` 类型与注册（照 font.rs 样板：assets + facade + importer ingest）。（2026-07-02 评审收口）theme 侧要求 `asset.kind = theme_tokens` 物化/facade 注册，**复用 `.zui` importer 分派**，不引入第二载体；裸 `from_toml_str` 入口不得成为公开载体。M1 **不含字体切片**（原 03 弱依赖改指向 text/01 FR-M2） | assets/ui.rs、facade/impls.rs、importer/ | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter asset` | 裸 TOML 载体入口硬切 |
| M1.S2 | 收束现有 `UiAssetDependencyIndex`：编译期收集三类引用写入（复用 ui_v2_asset_references + resource_ref/collect） | dependency_index.rs、loader.rs | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter dependency_index` | 无删除 |
| M1.S3 | 双向查询测试 + 与 node 脚本结论一致性对比（CI 双跑） | 测试 + CI 脚本 | 同上 + node 脚本 | node 脚本降级为把关 |
| M2.S1 | watch → classify → 级联指纹失效贯通（fold_events 接 dependency_index） | watch 消费侧、invalidation/ | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter invalidation` | 无删除 |
| M2.S2 | 全类型热重载：`.zui`/template/theme/icon/font 改文件 → 即时生效 | 各类型分流路径 | 实机 + `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter hot_reload` | 无删除 |
| M2.S3 | 级联范围测试：叶子改动只重编译受影响者（编译计数断言） | 测试 | 同上 | 无删除 |
| M3.S1 | `UiResourceResolver` 消费级解析（icon/texture/font/theme 四类） | resource_ref/resolver.rs | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter resource_resolver` | resolve 职责收口 |
| M3.S2 | 占位资源 + 诊断：缺失引用不 panic、不空白、诊断可查询 | resolver.rs | 同上 | 无删除 |
| M3.S3 | resource browser 数据面：双向查询暴露给 editor（供 09 批次 3） | editor 消费接口 | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests` | 无删除 |
| M4.S1 | （2026-07-02 评审收口）先评估**复用 graphics/text 图集设施**（R8/Rgba8 格式分组分页、脏矩形上传、页 LRU，text/04/05），避免重复建设；评估通过则图标通道挂其图集，仅在结论否定时才自建 icon_atlas。其余：SVG 解析（限定子集）→ 栅格（tessellation/SDF，衔接 03 raster 策略）→ atlas slot | ui/icon_atlas/（或 graphics/text 图集复用点） | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter icon_atlas` | 无删除 |
| M4.S2 | 默认图标包 + 图标渲染对拍 + asset 测试 | editor assets/icons/ | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests` + 实机 | 模板内联 icon 路径切资产引用 |
| M5.S1 | 收束现有 `UiCompiledArtifactStore` 落盘生命周期（指纹含 schema/编译器版本，失配即弃） | cache/persistent.rs | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter persistent_cache` | 无删除 |
| M5.S2 | 冷启动接缓存：v2 loader 先查 store；冷启动时间对比基准记录 | v2/loader.rs、file_cache.rs | 同上 + 启动计时 | 无删除 |
| M5.S3 | staged build 集成：compiled artifact 进 payload + M16 包验证通过 | tools/zircon_build.py | `python tools/zircon_build.py --targets editor --out <tmp> --mode debug` | 无删除 |

## 8. 测试矩阵（代表性用例）

- **M1**：`ui_theme_tokens_zui_profile_round_trips`、`ui_icon_asset_registers_in_facade`、`dependency_index_resolves_bidirectional_refs`、`runtime_index_matches_node_script_graph`
- **M2**：`leaf_zui_change_invalidates_only_dependents`、`theme_file_touch_triggers_restyle_not_rebuild`、`icon_change_damages_referencing_nodes_only`
- **M3**：`missing_icon_ref_yields_placeholder_with_diagnostic`、`resolver_caches_handle_per_reference`、`browser_query_lists_dependents_of_texture`
- **M4**：`svg_icon_rasterizes_into_atlas_slot`、`icon_render_extract_matches_native_painter`
- **M5**：`compiled_artifact_store_round_trips_with_fingerprint`、`stale_schema_version_misses_cache`、`cold_start_uses_persistent_cache`、`staged_build_payload_contains_compiled_artifacts`

落点：runtime 模块内 `#[cfg(test)]` + `zircon_runtime/src/asset/tests/` 既有目录；staged build 验证走脚本实跑。

## 9. 风险与对策

| 风险 | 对策 / 探测信号 |
|------|----------------|
| runtime 索引与 node 脚本结论漂移 | M1.S3 CI 双跑一致性测试；不一致即红 |
| 热重载风暴（IDE 保存触发多文件事件） | fold_events 已合并；级联目标集合去重 + 单帧上限，超限退化为整面重建并记录诊断 |
| persistent cache 用了过期产物 | 指纹含 schema 版本 + 编译器版本 + 源指纹；任何失配即弃用重编译 |
| SVG 解析依赖选型（usvg 等）引入大依赖 | 限定 SVG 子集（path/fill/stroke）；依赖评审进 M4.S1 切片 |
| asset 大模块 handle 语义演进（Bevy-Style 计划） | gating 条款：handle 语义变更时本计划同步适配，UI 不自建句柄 |

## 10. 里程碑级依赖表

| 里程碑 | 前置 | 被依赖 |
|--------|------|--------|
| M1 | 04 M1（UiThemeDocument 类型） | 05 M2/M3、06 全部（DoD 第 1 条）。（2026-07-02 评审收口）M1 不含字体切片；原「03 M2（字体注册，弱）」被依赖项改指向 text/01 FR-M2 |
| M2 | 05 M1 | 04 M6（主题热重载）、07 M3（motion 资产热重载） |
| M3 | 05 M1 | 06 M1（Icon 组件）、09 批次 3（Reference Finder） |
| M4 | 05 M3、03 M1（栅格策略） | 06 M1（Icon/IconButton DoD） |
| M5 | 05 M1 | E3 发布链（staged build 含 compiled artifact） |

## 11. 完成定义

- 六类 UI 资产全部经 asset facade 加载、可热重载（实机改文件即时生效）。
- 依赖索引双向查询可用且与 CI 脚本一致；缺失引用产生占位 + 诊断。
- 图标走资产通道渲染（对拍通过）；冷启动命中 persistent cache（时间对比有记录）。
- `python tools/zircon_build.py` 产物含 compiled artifact 且包验证通过。
- 验收命令组：`.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter asset`、`.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests`、staged build 实跑。

## 12. 边界约束

- `.zui` 单组件 profile 约束不放松（禁 `view`/`style`/多组件表）；新增资产字段走 schema 版本迁移。
- 资产 IO 全部经 asset facade；UI 模块不直接读文件系统（watch 也经 asset watch）。
- 不为 UI 资产新建独立缓存目录方案，复用工作区资产缓存约定。
- 占位资源必须视觉可辨（如品红方块 + 诊断角标），禁止静默空白。

## 13. 参考实现对照（dev/ 源码锚点）

实现切片前先读对应锚点，不确定的行为语义以参考实现为准（在 PR 说明中注明出处）；禁止凭印象实现、禁止引用未核实路径。

| 设计点 | 主参考 | 次参考 | 参考什么 |
|--------|--------|--------|---------|
| 资产 handle/事件/热重载架构 | `dev/bevy/crates/bevy_asset/src` | — | AssetServer 的 handle 生命周期、AssetEvent 分发、热重载与依赖加载（Bevy-Style Asset Stack 计划同源参照） |
| UI 资产加载器形态 | `dev/Fyrox/fyrox-ui/src/loader.rs` | — | UI 专属资产（字体/纹理）经引擎资源管线加载的接口边界 |
| atlas/纹理资源管理 | `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Textures` | `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Brushes` | Slate 的 atlas slot 分配与 brush 引用解析（UiResourceResolver 的消费级对照） |
| 图标素材 | `dev/ionicons.designerpack` | `docs/ui-and-layout/ai-workbench-style/component-prototype`（内联 SVG 现状） | 默认图标包的素材来源与 SVG 子集范围评估 |

## 14. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本计划仍处于 `in_progress`；UI asset 类型、索引、resolver 与 compiled store 已有实现基础，产品接线、规模合同和运行时验收仍待闭环。

- 迁入产出记录：[UI asset output/performance handoffs](05/2026-08-01-ui-asset-output-performance-handoffs.md)
- open / 待修复：[UI asset v2 projection drift](05/failure-2026-07-11-ui-asset-v2-projection-drift.md)
- open / 待修复：[template projection deep-copy and cache generation](05/failure-2026-07-17-template-projection-deep-copy-and-cache-generation.md)
- open / 待修复：[runtime UI template authoring linear index and style copy](05/failure-2026-07-18-runtime-ui-template-authoring-linear-index-and-style-copy.md)
- open / 待修复：[runtime UI template hot-reload resolver full scan](05/failure-2026-07-18-runtime-ui-template-hot-reload-resolver-full-scan.md)
- open / 待修复：[runtime UI v2 file-cache fs poll and stale artifacts](05/failure-2026-07-18-runtime-ui-v2-file-cache-fs-poll-and-unbounded-stale-artifacts.md)
- open / 待修复：[runtime UI v2 instancing and compile full projection](05/failure-2026-07-18-runtime-ui-v2-instancing-and-compile-full-projection.md)
- open / 待修复：[runtime UI v2 persistent-cache reparse and owned artifacts](05/failure-2026-07-18-runtime-ui-v2-persistent-cache-reparse-and-owned-artifacts.md)
- fixed / 已修复：[runtime interface UI resource-ref hard cutover](../../zircon_plugins/06/fixed-2026-07-13-runtime-interface-ui-resource-ref-hard-cutover.md)
