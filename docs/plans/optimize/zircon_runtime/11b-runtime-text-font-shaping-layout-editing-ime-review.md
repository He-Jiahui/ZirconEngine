---
related_code:
  - zircon_runtime/src/text
  - zircon_runtime/src/ui/text
  - zircon_runtime/src/ui/surface/input
  - zircon_runtime/src/ui/surface/render
  - zircon_runtime/src/asset/importer/ingest/import_font_asset
  - zircon_runtime/src/asset/artifact/cache_payload/font.rs
  - zircon_runtime/src/asset/project/manager/collect_files.rs
  - zircon_runtime_interface/src/ui/text.rs
  - zircon_runtime_interface/src/ui/surface
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
  - docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md
  - docs/plans/zircon_runtime/text/08-ime-and-text-input.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts
  - dev/bevy/crates/bevy_text
  - dev/Fyrox/fyrox-ui/src
  - dev/godot/servers/text
  - dev/godot/modules/text_server_adv
  - dev/godot/scene/gui
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 11B · Runtime Text、Font、Shaping、Layout、Editing 与 IME 工程化差距

## 1. 结论

Zircon 的文本系统不是临时拼出的单一 `draw_text`。当前源码已经包含字体 manifest 与 face/variation metadata、Rustybuzz 和 Cosmic Text shaping、Unicode BiDi、grapheme 分段、横排与竖排、`vert/vrt2` 与 vertical metrics、彩色字形、native bitmap/SDF atlas、resolved glyph artifact、rich text、layout/hit-test、selection/composition geometry，以及带 clause、surrounding text 和 cursor area 的 IME DTO。plain document measure cache、rich parse cache、atlas budget 和多类局部上限也确实存在。这些真实基础应保留。

但产品边界有三项 P0，使内部算法无法组成工程级文本服务。第一，字体导入 artifact 只保存 manifest/metadata，不保存可独立加载的字体字节；项目扫描又把 `.ttf/.otf/.woff/.woff2` 排除为 source auxiliary，runtime 最终重新打开项目源目录中的原始字体。clean package、远程内容、只读容器或删除 source tree 后没有闭合的 cooked font byte 链。第二，所有真实 shaping 路径失败时，canonical API 不返回错误，而是为每个 grapheme 生成猜测 advance、合成 glyph ID 且没有 font/face handle 的“成功”run；后续 artifact 又把它标成需要 rasterization。布局和命中测试因此可能成功，GPU 却不可能从权威字体生成对应字形。第三，`secure` 文本字段仍把原文写入 render command 并允许 copy/cut，所谓安全策略只是在 focus 时禁用 IME；密码既会明文显示/复制，又阻断中文、日文、韩文等组合输入。

规模和生命周期也未达到 Unreal/Slate 级别。字体数据库是 process-global `OnceLock<RwLock<...>>`，启动时默认发现系统字体并静默尝试源码树 manifest；每次 mutation clone 整库，generation 变化后使多个缓存整域失效，thread-local Cosmic `FontSystem` 又按线程复制数据库。文本布局虽然有 retained session 和 bounded cache，但大文档 viewport fast path 只覆盖 plain、horizontal、nowrap、clip、non-editable；wrapped editor、rich text、vertical text、preedit 和绝大多数编辑场景仍全量 layout。编辑状态没有 undo/redo transaction，公开 selection/composition action 只保证 UTF-8 char boundary，不保证 grapheme；上下键按源 hard line 导航，RTL 左右键按逻辑顺序移动。

历史审查中的 64 KiB shaping cap 曾把一个逻辑 hard line 切成多个 `HardLine`，每段被计入行高；跨分块的 Arabic/Indic context、ligature 和 grapheme 也可能被切断。该 source-line 语义问题已由后续 P1-9 current-source re-audit 更正：64 KiB 现只是不接入生产决策的分类阈值，不能再被用作布局边界。富文本 cache 驻留是有界的，但 parser 入口没有 input/token/node/depth/time budget；HTML/BBCode/Markdown 子集会从每个未闭合标记重新扫描剩余字符串，构造近似 O(n²) 的拒绝服务输入。解析结果没有 structured diagnostics，“Markdown”实际只实现 `**`、`*` 和反引号三个极小语法。

本轮登记 3 项 P0、29 项 P1、8 项 P2。重构顺序必须先关闭字体 cook 和 secure input，再把 shaping 失败变为 typed outcome；随后建立 session-owned font/text service、语义保持的长文本分块、paragraph/document 增量模型、编辑事务和视觉导航，最后完善富文本预算、真实平台 IME 与性能门禁。11C 将单独审查 glyph upload、atlas、SDF、UI GPU batch、clip 与 submit，不在本文用 CPU 结构测试替代 GPU 结论。

## 2. 审查边界与证据

### 2.1 当前源码范围

| 集合 | 文件 / 物理行 | 本轮证据 |
|---|---:|---|
| `zircon_runtime/src/text` 全部 | 284 / 62,681 | E3：font、shaping、layout、raster、SDF、atlas、artifact、cache、rich text 与 service |
| 其中 production path | 210 / 40,180 | fingerprint `7761821f150e57d0a1c288301f2988e99b2ef3b07c33333c0e70d61c08fa306e` |
| `zircon_runtime/src/ui/text` 全部 | 64 / 13,247 | E3：UI shared shaper、layout engine、viewport、geometry、grapheme、rich text |
| 其中 production path | 33 / 8,307 | fingerprint `849ddf990ae0723f3f544f6c7dba60c4a90cb78803f711e3` |
| text/IME interface focused set | 27 / 6,276 | E3：editable state、selection/caret/composition、IME event/host request、render/layout DTO |
| combined focused set | 375 / 82,204 | E2-E3：1,029 个 test attributes、10 个 ignored；不以数量代替产品闭环 |

fingerprint 算法与 09H/11A 一致：路径排序、逐文件 SHA-256，再对 `path<TAB>hash<LF>` 清单取 SHA-256。当前两个 text production path 未出现在 `git status` 的修改列表，但 UI/input/render、asset 与 graphics consumer 有其他 Session 修改，因此实施前仍需重新取指纹并复核跨域调用。

本轮按 owner chain 读取：font source/import/artifact/project collect -> font database/shared owner -> shaping backend/normalization/BiDi/hard-line split -> glyph artifact/raster/SDF -> text service -> UI layout/viewport/geometry -> editable reducer/keyboard/clipboard/IME -> render extraction。Rich text 另从 parser identity/cache 追到 HTML、BBCode、Markdown、table/list 和 custom decorator。11C 承接 GPU atlas texture、upload、batch 与 final paint，不在本文重复判断。

### 2.2 参考源码边界

- Unreal `FontBulkData.h/.cpp` 把 raw font data 作为可序列化、可锁定、可统计、可压缩的 bulk data；`FontCache`、HarfBuzz shaper 和 composite font 将 face selection、glyph cache、fallback 与生命周期放在明确 owner 下。这里用它判断 cooked font closure、cache owner 和 failure contract，不照搬 UE 类型层级。
- Godot `TextServer` 允许以 `PackedByteArray`/pointer 设置字体数据，并提供 shaped text、glyph、line break 与 variation API；`TextEdit`/`LineEdit` 拥有 undo stack、IME 与 secret character/secret mode。这里用它判断产品能力下限。
- Bevy `font_loader.rs` 从资产字节构建 `Font`，`error.rs` 提供 typed `TextError`；`text_edit.rs` 覆盖 grapheme edit、visual line、preedit/commit 与 clipboard command。Bevy 的能力面较小，但 Rust 资源边界和错误语义有直接参考价值。
- Fyrox `FormattedText` 与 `TextBox` 提供较简单的 Rust retained text/edit baseline；它不是本文的工程上限，只用于识别 Zircon 是否连较小引擎的基本编辑事务/输入闭环都未达到。
- 仓内 Unity Graphics 不包含 Unity 主文本引擎或 TextCore/TMP 的权威实现，只有样例资源。本文不根据这些资源猜测 Unity 闭源文本行为；11C 只引用 Graphics 仓中可直接证明的 GPU resource/atlas 设计。

### 2.3 明确未做

本轮没有运行 Cargo、Editor、WGPU、真实系统字体枚举、clean package、TSF/IMM32/IBus、屏幕键盘或性能采样；也没有把 10 个 ignored exporter/manual test 当验收。静态源码可以证明 owner、数据流和缺失分支，不能证明视觉质量、fallback 字体覆盖、真实候选窗位置或百万行编辑器延迟。

历史 `docs/plans/zircon_runtime/text` 有 01–09 九组计划和 25 个 `failure-*.md`。本文不删除这些记录，而是以 current source 重新判断它们的完成状态。尤其 Text08 第 61 行声明 `Ctrl+Z/Y/X/C/V/A` 已存在，但 current source 没有任何 undo/redo action 或 history；该完成声明必须重开，不能继续作为验收证据。

## 3. 可保留的真实基础

### 3.1 Shaping、BiDi 与竖排不是占位算法

横排有 Rustybuzz direct path 与 Cosmic Text fallback，BiDi 使用 Unicode BiDi 数据；竖排读取 vertical metrics 并支持 `vert/vrt2`。run、cluster、source byte range、direction、script/language 和 glyph positioning 已形成可扩展 DTO。后续应修复 failure/status、chunking 和 owner，不应退回逐字符宽度估算。

### 3.2 字体 face、variation、color glyph 与 raster pipeline 已有实质实现

font model 保留 face index、coverage、variation axis/instance；raster 路径含 COLR/CPAL、bitmap strike 与 RGBA native atlas，SDF 路径也有 bake/cache 结构。不能把问题简化成“没有可变字体或彩色字体”；真正 P0 是这些能力依赖的原始字节没有进入 closed artifact。

### 3.3 UI layout 共用 runtime text service

`zircon_runtime/src/ui/text` 没有再手写一套逐字符 shaping backend，而是通过 `SharedTextLayoutSession`/shared shaper 消费 text service。geometry、selection、composition 与 hit-test 能复用 resolved layout artifact。重构应把 session 变成长期 owner，而不是再增加第三套 UI 专用 font/shaper。

### 3.4 IME 中立合同已覆盖关键字段

preedit clause、commit/cancel、delete surrounding、surrounding text window、cursor area、enable/disable 和 composition range 已进入 runtime-neutral DTO；正常非安全字段能够把 resolved layout geometry 用于候选窗。缺口是产品安全策略、fallback geometry、平台实机和 lifecycle，不是完全没有 IME 模型。

### 3.5 缓存并非全部无界

Rich parse cache 有 256 项/8 MiB 驻留预算，plain document measure cache 与 atlas/SDF 也有明确局部上限；table/list 设置了列数和局部嵌套限制。后续应保留 bounded residency 和 single-flight 思路，同时补 admission、owner、telemetry、CPU time 与输入复杂度预算。

## 4. P0：先关闭打包、渲染正确性与安全输入

### P0-1：字体 artifact 不拥有原始字体字节，打包链不闭合

`asset/importer/ingest/import_font_asset/mod.rs` 会读取字体源字节并解析 metadata，但 `AssetImportOutcome` 最终只持有 `ImportedAsset::Font(FontAsset)`。`asset/artifact/cache_payload/font.rs` 的 `ArtifactCacheFontAsset` 保存 source 字符串、family、face metadata、coverage 与 variation 信息，没有 binary font blob。与此同时 `asset/project/manager/collect_files.rs` 把 `.ttf/.otf/.woff/.woff2` 排除为 source auxiliary。

运行时 `text/font/source_manifest.rs` 先从 asset manager 取得 manifest artifact，然后重新定位 manifest 的 source，canonicalize 项目 asset root 下的原始文件并返回 filesystem path；非项目路径还会回落 `env!("CARGO_MANIFEST_DIR")` 或 runtime source asset。artifact store 中没有与 font manifest 同 generation 的 auxiliary byte payload。开发树和系统字体会掩盖这个断链，clean package、DLC、远端 mount、只读部署或删除 source 后将无法可靠构建 face。

必须定义 cooked font artifact：至少包含经过授权/子集化策略处理的原始字体 bytes、face/table index、content hash、license/provenance、variation/color capability、schema version 与 dependency generation。runtime 只从 content-addressed blob/lease 建 face；source path 仅供 Editor reimport。验收必须在复制出的 package 中删除项目源码并禁用系统字体后运行多语言 shaping/raster。

2026-08-29 current-source 修正：上述“artifact payload 没有 binary font blob、project runtime 重开原始 source”的实现事实已经过期。Importer 当前生成 versioned `FontBlobArtifact`，以 BLAKE3 content hash 校验并由 artifact cache payload 持有；project `load_text_font_source` 要求 cooked payload，只产生 `cooked-font/<asset-uri>` 逻辑 source key，UI/SDF 注册走 `replace_font_asset_blob`。共享 runtime 默认字体也从编译入 binary 的 manifest + TTC blob 启动，不依赖 checkout source 或系统字体。此次进一步补齐 engine-owned default composite：runtime baseline 与 project override 是两个独立 owner，fallback 固定按 explicit > project > runtime 选择，项目 default 删除后恢复内置 Fira Mono/zh-Hans CJK，而不是宿主 generic family；SDF 在没有项目 default owner 时直接消费同 generation 的 runtime primary face，不重开 loose manifest/source。这关闭 P0-1 的核心字节断链和默认复合字体所有权的静态实现缺口，但 clean package、shipping direct-path policy、license/provenance/subset/toolchain receipt、跨平台真实 shaping/raster 与受管 Cargo/WGPU 尚未验收；P0-1 状态为 `implementation_complete_static_checked / product_qualification_pending`，不得据此关闭第 14 节门禁 1。

2026-08-29 clean-process default-face 补充：默认 neutral request 不声明 family，旧 primary match 又没有读取 engine default，导致系统字体 Disabled 时 CompositeFont 尚未参与就失败。当前 primary admission 固定为 explicit -> project default -> runtime primary -> runtime family -> platform/asset fallback；default state mutation 同时失效 face-match/fallback cache。fresh embedded database 的 `A界` source regression要求 Latin/CJK rasterizable glyph handle均可解析到真实 face。该项是 P0-1/P0-2 之间的 MVP 接缝修正，状态 `static_implemented / managed_shape_raster_validation_pending`。

### P0-2：真实 shaping 全失败时仍发布不可 rasterize 的“成功”glyph

`text/shaping/cosmic.rs` 的 canonical `shape_text` 返回 `ShapedGlyphRun` 而不是 `Result`。direct/cosmic 路径都失败后，`fallback.rs` 为每个 grapheme 生成 FNV 风格 synthetic glyph ID，并以 0.33/0.56/0.85/1.0 em 等猜测 advance 排版；这些 glyph 的 `font_id` 和 `font_instance_id` 均为 `None`。

`text/service.rs` 仍把非空白 glyph 标成 `requires_rasterization=true`，`glyph_artifact.rs` 可以登记 `(None, None)` handle 并发布 artifact；`sdf/font_bake.rs` 对无 face key 明确无法解析 face。于是 layout、caret 和 hit-test 得到稳定坐标，renderer 却没有权威 face/glyph 可画。单测如果只断言 run 非空或宽度有限，会产生 false green。

canonical shaping 必须返回 typed outcome：ready glyph run、pending font dependency、missing glyph with tofu policy、invalid font/data、unsupported feature、budget exceeded。缺字可映射到真实 fallback face 或可 rasterize 的 engine-owned tofu glyph，绝不能伪造“像 glyph ID 的数”。所有 layout/artifact/render publication 必须携带 font generation 与 completeness；错误路径要可观测且可在测试中强制触发。

2026-08-26 current-source修正：上述“FNV synthetic glyph仍作为成功发布”的源码结论已经过期。当前`text/shaping/cosmic.rs::shape_text`返回typed `TextShapingOutcome`；无可用字体返回`FontUnavailable`，backend产出非空白/非virtual glyph却无法映射实际face时返回`FallbackExhausted`。`service.rs::project_glyph`只有在实际font face存在且glyph不是空白/tab/virtual时才置`requires_rasterization`。回归`service_reports_font_unavailable_instead_of_publishing_synthetic_glyphs`与`projection_never_requests_rasterization_without_a_font_face`已锁定该合同。P0-2实现状态改为`implementation_complete_static_checked / managed_validation_pending`；engine-owned tofu作为产品降级能力仍可后续增加，但不得恢复伪glyph成功路径，也不得把静态检查冒充WGPU验收。

### P0-3：`secure` 字段明文渲染和复制，同时禁用 IME

`ui/surface/input/text_state.rs` 只从 metadata 识别 `secure`。focus 路径对 secure field 禁用 IME，直接 Enable 还会报错；但 `surface/render/resolve.rs::resolve_visible_value_text` 返回原始 editable value，`render/text_fields.rs` 又把该字符串写入 `UiRenderCommand.text` 和 layout。`keyboard_clipboard.rs` 的 copy/cut 也会把 selection 原文写入 clipboard，没有 secure policy 检查。

因此当前 password field 既不保密，又无法用组合输入法输入国际字符。安全边界必须从“关掉 IME”改为端到端 policy：render/layout 使用可配置 secret character 与 grapheme-count-preserving mask；clipboard copy/cut 默认拒绝；surrounding text、accessibility value、diagnostics、capture、serialization、crash/log 与 plugin event 不得泄漏；IME 仍可在平台允许的 secure input scope 内提交文本。原文只存在于最小 owner 的受控 state，公开 render command 不携带 secret。

2026-08-27 current-source修正：mask/glyph artifact、accessibility value/selection/action redaction与clipboard copy/cut deny已存在，本轮又为TextField catalog声明typed `input_kind` enum，并让WOC `input_kind=password`、`type=password`和secure aliases进入唯一internal `UiSecureTextPolicy`；冲突、畸形secure alias与未知input kind均fail-closed。render不再维护第二套classifier。P0-3仍开放：通用Change/Commit component event与binding report携带原文，secure focus仍禁用IME，Runtime Interface没有versioned secure policy/host session，真实WOC/Cargo/IME/capture未验证。状态为`classifier_bypass_closed / render_a11y_clipboard_frontend_converged / secure_event_projection_open / secure_ime_session_open / managed_validation_pending`，不得把前置收敛表述为端到端安全输入完成。

## 5. P1：字体资源、所有权与确定性

### P1-1：字体数据库是 process-global，而不是 session/project-owned service

`text/font/shared.rs` 使用 `OnceLock<SharedFontDatabase>` 和 `RwLock<FontDatabase>`。一个 renderer、Editor project 或 test 对 global database 的 mutation 会影响同进程其他 session/world/window。它没有 project mount、Play-in-Editor isolation、plugin lease、session teardown 或 deterministic snapshot owner。

应由 `TextRuntimeService` 按 engine/device 共享 immutable byte blobs，按 project/session 拥有 font collection generation；view/layout 持短期 snapshot/lease。跨 session 只共享内容寻址的只读数据和 GPU-safe cache，不共享可变 fallback 顺序与注册表。

2026-08-29 non-validation implementation：`TextModule` 现由 Core Services 注册 `FontCollectionService`，
动态 Runtime UI、Graphics renderer 与 layout surface 通过同一 collection handle 注入；跨 runtime 仍保持
collection identity 隔离。动态 UI 首次布局前先汇总并 admission 默认/显式 font assets，渲染器 fallback
shape 只在 prepare 阶段消费该 collection。window/PIE、project hot-reload 的完整 asset lifecycle
仍待接入，managed Cargo/WGPU/PNG/profile/RSS/power 未执行，因此 P1-1 状态更新为
`core_service_and_first_layout_admission_static_implemented / managed_validation_pending`。

2026-08-29 owner-lifecycle re-review：renderer 字体缓存当前不会按 project/session scope 释放不再引用的
asset owner；旧 owner 可能跨 project switch 驻留并参与全局 fallback。不能以动态 surface 的依赖集合
直接清库，因为 HUD、菜单与其他 consumer 可能共享同一 collection。P1-1 的剩余结构项明确为
`FontCollectionService` owner-scope claim/release transaction、单次 generation publish 与 bounded residency；
在 project switch/hot reload/fallback isolation/profile 门禁通过前保持 open。

### P1-2：默认启动混合系统字体与源码树 fallback，错误全部静默

`FontDatabase::with_default_fallbacks()` 默认 `SystemFontPolicy::Discover`，随后尝试 `assets/fonts/default.font.toml`，路径来自 `env!("CARGO_MANIFEST_DIR")`。manifest read/parse/register 错误通过 `Option`/忽略分支消失。不同 OS、容器、CI、语言包和用户已安装字体会改变 fallback 与像素结果。

必须区分 Editor discovery、development fallback 和 packaged runtime policy。项目 cook 应冻结 fallback collection/content hashes；启动失败报告 missing/corrupt font dependency。系统字体若作为平台 feature，必须显式 opt-in、可审计并进入 replay/cache key，不能成为默认正确性来源。

### P1-3：字体 mutation clone 整库并触发粗粒度全域失效

global mutation 在 write lock 下 clone before/after database，比对 render inputs；generation 改变后 `TextRenderState` 使 native source cache/retry/atlas/SDF face 等整域 discard/invalidate。大字体集合或 hot reload 会造成停顿、内存峰值和无关 glyph 抖动。

应使用 immutable generation + structural sharing，mutation 产生按 family/face/variation/content hash 的 delta。shaping cache、raster cache、atlas entry 和 paragraph layout 依据实际依赖定点失效；旧 generation 由 lease 延迟回收，不能在正在提交的 frame 中原地清空。

### P1-4：Cosmic `FontSystem` 以 thread-local 复制数据库

`shaping/cosmic/font_system_cache.rs` 每线程缓存最多四个 locale 的 `FontSystem`；每个实例克隆 font DB，global generation 变化时重建该线程所有 cached systems。worker 数、locale 数和字体库大小共同放大内存与 rebuild 时间，且缓存 owner 不随 session 关闭。

应建立可度量的 shaping worker pool/arena，把 immutable font snapshot 与 locale-specific state 分离；明确最大并发、admission、eviction 和 rebuild budget。generation 切换要预热并原子发布，而不是在任意用户输入线程首触发整库复制。

### P1-5：注销 face 留下单调 tombstone，identity/lifetime 未闭合

`FontDatabase` 的 retired slot 留在 `Vec` 中，bytes 被清空但 ID/slot 单调增长。长时间 Editor reimport、theme/locale 切换或 plugin reload 会积累 tombstone；外部 handle 没有统一 generation，难以区分已退休和新 face。

应使用 generational face handle 与 content-addressed blob，registry slot 可在 lease 归零后安全复用或 compact；所有 cache key 必须包含 generation/content identity。调试 API 应区分 active、retired-pinned 和 reclaimable bytes。

### P1-6：字体 source load 把具体错误压成 `Option`（实现推进，受管验收待定）

`load_text_font_source`/project helper 对 URI、artifact、manifest、canonicalize、越界 source、文件不存在和权限失败都返回 `None`。上层无法区分“未配置字体”和“配置损坏/安全校验拒绝”，fallback 会继续掩盖内容错误。

2026-08-24 已先硬切 manifest/source 边界：`load_text_font_source`、UI manifest adapter 与 `TextRenderState` 分别返回 `FontLoadError` 或数据库 `Result`，不再将失败压为 `Option`。`FontLoadError` 区分 project URI/asset、manifest parse、空或绝对 source、allowed-root/source canonicalize 和越界；IO cause 以 `NotFound`/`PermissionDenied`/`Other` 的低基数分类保存。直接字体字节读取和解码失败分别映射为稳定原因，避免混入通用 registration failure。UI `Ready / Missing / Error` cache 在 resource-backed error entry 中保留该分类，SDF face cache 与 offline manifest cache 也保存 `Result` 负记录，而不是以 `None` 覆盖；`ScreenSpaceUiTextPrepareReport` 以一帧一次的聚合 cache snapshot 提供 source contract/IO/decode/registration 诊断。回归覆盖越界、绝对 source、TOML parse、missing source、直接读取错误映射，以及 Error -> Ready 清理同一失败分类。

仍需补齐 artifact generation/source-cook phase 的跨进程 receipt、Editor reimport action、明确的 runtime fail/last-good/tofu policy与shipping direct-path禁用策略；project cooked artifact-only loader及asset UUID投影已进入当前源码。因此 P1-6 仍是 `implementation_in_progress / managed_validation_pending`，不得把静态实现当作 Cargo 或产品资格完成。

### P1-7：字体 ingest 未建立 engine-level byte/table/decompression budget

已检查的 font importer 会把 source 读入内存并交给解码/metadata 构建，但没有找到统一的最大文件字节、face count、table count/size、glyph count、variation/color table、解压膨胀或解析时间预算。第三方字体属于不可信内容，不能只依赖底层库“不崩溃”。

import/cook 必须在进入 parser 前检查源大小，并对 collection face、table directory、解压字节、glyph/color layer、variation axis 与 CPU time设预算。超限返回稳定 diagnostic；fuzzer/corpus 覆盖截断、重叠、循环 offset、超大 WOFF/彩色表和 collection。

2026-08-24 P1-7A admission budget 已实现，状态为 `implementation_in_progress / managed_validation_pending`。`asset/assets/font_source/budget.rs` 是唯一策略 owner：64 MiB source、128 MiB decoded、64 TTC faces、每 face 256 tables、64 MiB per-table、128 MiB materialized standalone face、64 variation axes 和 256 named instances 都以具名 crate policy 定义；这些值尚无项目字体语料的 p50/p95 校准，因为当前 shared worktree 不含非 `dev/` 的字体二进制。Importer 在 `fs::read` 前检查 source 长度；统一 decoder 在 WOFF2 decode 前检查 header 声明长度、decode 后检查实际长度；metadata 和 standalone TTC extraction 在枚举前检查目录和 `fvar` 基数，并在任何 extraction `Vec` 扩容前限制 table copy 总量；`FontDatabase` 同样执行 pre-read 与 post-decode guard。UI/SDF cache 保留 budget failure 的 typed category，prepare report 只聚合其次数，不保留路径或原始 payload。现有静态边界回归覆盖 WOFF2 声明展开、TTC face、table directory、`fvar` axes、TTC duplicate-table materialization 和 UI mapping。

2026-08-24 P1-7B 结构审查：`parse_sfnt::cmap_coverage` 仍用 `BTreeSet<u32>` 对每个 Unicode subtable 的每个命中码点去重，再转 ranges。该选择与 Text01 的 cmap bitset 目标不一致；满覆盖 face 会使导入边界持有百万级树节点，且多个 subtable 的重复码点重复比较。Unreal Slate 把 character map 消费封装在 face/font-cache owner，不向 layout 传播临时逐码点集合；Zircon 应在 importer owner 以固定 0x110000-bit Unicode-scalar bitmap 去重，随后只按已置位 bit 流式压缩到既有 `FontAssetCodepointRange`，不改 artifact schema、fallback API 或 layout owner。该设计将临时 coverage storage 固定为 136 KiB/face，并把集合维护从 `O(C log C)` 降为 `O(C)`；但交替码点仍能产生大量 serialized ranges，因此同一 budget owner 还必须限制每 face 的 range fan-out，并以 face-indexed typed error 拒绝超限输入。这些是数据结构与内存上界结论，不是未经执行的 p50/p95 或功耗结果。实际导入 corpus timing、peak allocation、third-party fuzz corpus 与 managed Cargo 仍待后续验证。

2026-08-24 P1-7B implementation update：`BTreeSet` 已从 `parse_sfnt` 移除，`UnicodeScalarCoverage` 以固定 Unicode-scalar bitmap 去重并仅遍历 set bits 压缩为原有 ranges/count artifact；`FontSourceBudget` 以 65,536 ranges/face 限制序列化 fan-out，在第 `limit + 1` 个 range 返回 `CmapRangeCount { face_index, observed_at_least }`，上层保留为 `FontMetadataParseError` 而非静默截断。跨 word range、重复码点和 range 超限的回归已写入。当前状态仍是 `implementation_in_progress / managed_validation_pending`：尚未获得真实 corpus 的 timing/peak-allocation 数据，glyph semantic cardinality、color-layer、parse CPU budget、fuzzer、cook receipt 和所有产品验收仍 open。

仍未实现的 P1-7 项必须保持 open：cmap/glyph 与 color-layer cardinality、wall-clock parse budget、第三方损坏 corpus/fuzzer、阈值实测校准、cook receipt，以及 managed Cargo/WGPU 产品验收。本切片不声称性能、功耗或截图结论。

### P1-8：`UiFontRegistry` 第二真值（实现完成，受管验收待定）

原 `ui/text/font_registry.rs` 曾定义独立 record/ID/fallback family registry，生产 consumer 未使用它且会复制 `FontDatabase` fallback 名称。2026-08-24 已将该 source/test owner 物理删除：FontAsset 生命周期、default family、fallback identity 与 backend lineage 只由 `text/font::FontDatabase` 管理，没有 UI facade、compat re-export 或第二套 ID。

静态 retired-path 扫描、格式与 diff 检查已经覆盖该硬切；Cargo、产品帧与 M2 session-owned collection 的其余资格仍待受管验证。禁止重新引入 UI registry 作为后续工作绕过。

## 6. P1：Shaping、Unicode 与长文本语义

### P1-9：64 KiB shaping cap 把一个逻辑行变成多个布局行

`text/hard_line.rs` 的 `TEXT_SHAPING_RUN_MAX_BYTES=64*1024` 会把无换行文本物理切成多个 `HardLine`；`hard_line_count` 和 layout line height 把每段视为新行。一个超长聊天消息、代码行或生成文本会凭空增加高度。若单个 pathological grapheme 超过 cap，代码退到 UTF-8 boundary，仍可能切断 grapheme；Arabic/Indic context、ligature 和 cluster 也可能跨块失真。

预算分块必须与逻辑 paragraph/line identity 分离。使用 context overlap 或 backend streaming shaping，并以 source/cluster map 去重边界 glyph；layout 仍只有真实 newline 才产生 line。超出硬预算时返回 typed partial/budget result，不能偷偷改变文档语义。

2026-08-24 current-source re-audit：上述“64 KiB cap 产生 `HardLine`”的前提已经不成立。`text/hard_line.rs` 只将实际 source separator 投影为 `HardLine`，一条超过 64 KiB 的无换行输入仍保持单一 source line；`TextShapingWorkBudget` 仅表达调度分类阈值，当前没有生产调用据此切分、排队或降级请求，调用点仅位于边界回归。故 P1-9 的逻辑行/cluster 语义缺陷为 `implementation_complete / managed_validation_pending`，不得按这段历史描述重新引入 budget chunking。它不等同于 typed deferred work-unit scheduler：该调度合同、长行的实测 profiling、31-sample p50/p95、功耗结论与 WGPU 产品验证仍为 open，当前没有性能数据或优化收益结论。

### P1-10：Normalization/source mapping 明确关闭

初始审查时 `text/shaping/normalize.rs` 使用含混的 `ShapingTextView::v1_disabled`，没有 normalization buffer 与 normalized-to-source mapping。组合/分解序列、font expectation、caret/selection 和 shaping cache key 的合同当时没有明确版本。

需要决策并记录 normalization policy，而不是默认“原样通常可用”。若保持不规范化，要建立 canonical-equivalence 测试与 backend 一致性；若规范化，必须保存双向 byte/grapheme/cluster mapping，使 hit-test、IME、selection 和 accessibility 仍引用原始 source。

2026-08-24 current-source policy closure：V1 已明确采用 `source-preserving`，不执行 NFC/NFD，也不以 canonical equivalence 合并输入；`ShapingTextView::source_preserving`、direct RustyBuzz 与 Cosmic fallback 都消费同一原始 UTF-8，published glyph/source range 仍是原文 byte offset。Shaped-cache key 对原始文本 hash，组合与分解形式不会 alias；回归同时锁定 identity range 和 cache separation。该选择对齐 Unreal Slate 直接将原始 UTF 序列交给 HarfBuzz、并持久 `SourceIndex`/source range 的模式。故 V1 policy 为 `implementation_complete / managed_validation_pending`；启用 NFC/NFD 仍是 V2，前置条件是版本化双向 byte/grapheme/cluster map、backend parity golden 与 CPU/memory budget，不能在缺少这些合同的情况下局部打开。

### P1-11：direct 与 Cosmic fallback 的错误合同不对称

horizontal direct Rustybuzz validation 失败可转 Cosmic；vertical path 没有等价 fallback，最终仍落 synthetic run。backend failure、font missing、unsupported vertical feature 和 malformed data 被压成同一“有 glyph 的结果”。

建立统一 `ShapeOutcome` 与 backend diagnostics，明确 retry/fallback 顺序、vertical capability、font generation 和 completeness。fallback 只可选择真实 backend/face 或 tofu，不得切换到猜测布局。测试要强制让各 backend 独立失败并验证相同上层语义。

2026-08-24 current-source re-audit：旧的 synthetic-run 结果已被硬切。`shape_text(...)` 返回 `Result<ShapedGlyphRun, TextLayoutError>`；horizontal direct 失败只可尝试真实 Cosmic 后端，vertical direct 失败返回 `ShapingFailed`，而最终 projection 会以 `FontUnavailable`、`FallbackExhausted` 或 `ShapingFailed` 拒绝没有实际 raster face 的非虚拟 glyph。该 fail-closed 边界为 `implementation_complete / managed_validation_pending`。剩余 contract work 不能误报完成：horizontal/vertical direct helper 仍以 `Option` 合并具体 failure，`TextLayoutError` 尚无 backend/face/range/cause/retryability diagnostic，且内部 convenience provider 仍会把失败转成空 run；这些应在统一 `ShapeOutcome` 设计中一次收敛，而不是再次引入猜测 glyph 或不兼容 facade。

### P1-12：BiDi invariant 失败时静默回退逻辑顺序

BiDi 主路径使用真实 Unicode BiDi 算法，但当 range 不属于 paragraph 或内部映射不一致时，会返回 logical-order/fallback level，而非传播 invariant error。这会在 RTL isolate、嵌套 embedding 或错误 chunk range 时生成看似合理却错误的视觉顺序。

内部 range/paragraph mismatch 应是 typed invariant failure 并附 paragraph/run identity；内容导致的 unsupported case才可选择可见 fallback。debug/CI 必须 fail fast，release 可发布 tofu/diagnostic，但不能伪装为正确 BiDi。

2026-08-24 current-source re-audit 已完成结构改造，状态为 `implementation_complete / managed_validation_pending`。`BidiParagraph::level_for_range` 不再截断越界 offset；`line_order` 不再构造 logical-order fallback，而是以 `BidiInvariantError` 表达 source/line/glyph range、paragraph、resolved level、visual projection 与 advance-cardinality 违例。direct horizontal/vertical 把该错误传给 canonical `TextLayoutError::BidiInvariant`，Cosmic glyph/hard-line projection 同样 fail closed；UI visual-order 与 rich horizontal/VerticalRl adapter 统一返回 `Result`，顶层只发布 overflow-clipped 空安全 layout，并在 `TextLayoutFallbackReport` 记录低基数计数，不存原文或 raw range。回归锁定 level/line/glyph invalid input、投影原子性与计数分类；限定 `rustfmt`/`git diff --check` 通过。

本轮不是性能优化：静态复核确认新增工作仅为每个 cluster/line 的 O(1) range/UTF-8 boundary check，不分配集合、不增加 BiDi analysis、二次 shape 或 backend call。性能报告计划已固定为 coordinator 在同 font/fallback/cache 状态下，以 `text_direct_shape_*`、`text.shape_batch.*`、`text.layout` scope 和 `ZR_UI_LAYOUT_PROFILE` 采集 cold/warm 的 1/100/1k/10k grapheme LTR/RTL/isolate/rich/VerticalRl p50/p95/p99、RSS、backend-call 与 power trace；在没有受管数据前不得声称瓶颈消失、功耗接近 Unreal 或算法最优。Cargo、focused test、真实 WGPU framebuffer 和新的真实 PNG 都尚未执行；产品 fallback/tofu、paragraph/run identity receipt、official BidiTest corpus 和唯一 canonical visual-cluster artifact 仍为 open。

### P1-13：缺字、fallback face 与 script coverage 没有端到端 completeness receipt

Font metadata 有 coverage，shaping run 有 script/language，然而 layout publication 没有稳定表达每个 cluster 是 primary、fallback、tofu、pending 或 synthetic。日志/telemetry 也无法按 asset/locale 聚合缺字，内容团队只能从截图发现方框或空白。

每个 resolved run/artifact 应携带 chosen face/content generation 与 missing-cluster summary；高频细节用 bounded counters/hashed sample，Editor 提供 font coverage audit。cook 可对声明 locale 做静态 coverage gate，runtime 仍保留动态 fallback。

2026-08-24 P1-13 architecture checkpoint：owner 固定为 `zircon_runtime::text::shaping`，以 folder-owned internal `TextShapingOutcome` 的 `Ready(Arc<ShapedGlyphRun>)`、`Deferred(TextLayoutError)`、`Failed(TextLayoutError)` 区分可消费 artifact、generation retry 和不可发布的失败。MVP 不扩大 public DTO：`core::framework::text::TextLayoutError` 继续只是稳定低基数 code；第二阶段才在 framework contract 引入不含 raw text 的 phase/backend/face/range/retryability receipt，供 service、renderer 和 Editor 一致消费。

2026-08-24 P1-13 M1 Ready-only cache admission：状态为 `m1_implementation_complete / managed_validation_pending`。`SharedTextLayoutSession` 与 parallel pool 现均消费 `TextShapingOutcome`；仅 `Ready` 能调用 `ShapedRunCache::insert_ready`，并复用同一个 `Arc<ShapedGlyphRun>`，不复制 glyph storage。`Deferred`/`Failed` 只在本次请求返回显式 empty fallback：session 不分配 owned cache key，parallel report 分离 `generation_deferred_count` 与 `failed_count`，profiling 记录 `text.shape_batch.failed`。回归直接锁定 session 和 parallel finish path 的 failed outcome 不增加 cache insert，并锁定 generation change 的 deferred 分类。该 M1 不将空 `ShapedGlyphRun` 伪装为 cached Ready，但不误报完整管线完成：`TextShapeRunProvider`/direct convenience helper、layout/UI safe publication、backend/face/coverage receipt、tofu policy 与 public framework contract 仍未迁移，必须按后续 M2--M4 完成。

2026-08-24 P1-13 M2 provider-boundary audit：`TextShapeRunProvider` 有三处生产 consumer（`measure.rs`、`line_break/mod.rs`、`boundary_correction.rs`），但它们将结果立即压缩为 width、chunk 或边界修正；七个测试 provider 也直接实现旧 `Arc<ShapedGlyphRun>` 签名。仅把 trait 改为 outcome 而不同时提升这三条 layout contract，会在算法中重新把 `Failed` 变成零宽/空 chunk，属于错误的包装式重构。M2 因此保持 `design_review_complete / implementation_deferred`：下一次实现必须以“layout analysis outcome + safe publication”同一切片同时迁移 provider、measure、line-break、boundary correction 与 UI owner，不建立第二个 `Arc`-return compatibility trait。该决定没有形成性能或 Cargo/WGPU 结论。

架构证据：Unreal Slate 的 `Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h` 将 `FShapedGlyphSequence` 的 source range、glyph sequence、测量/枚举失败和 loading faces 留在同一 font/shaping owner；Slint `internal/core/textlayout.rs` 先构建一次 `ShapeBuffer`，再让 line layout 消费含 byte offset 的 glyph clusters，并以 `FixedTestFont` 覆盖 line/ellipsis/cursor source offset。Zircon 故意不复制 Unreal 的非空 sequence facade：现有 generation retry/cache 是 Rust runtime 的独立约束，必须用 typed outcome 防止 Ready cache 混入失败。

实施顺序和测试矩阵：M1 已完成 outcome owner、session/parallel Ready-only cache admission；M2 迁 direct/provider，M3 迁 layout/UI safe publication，M4 扩展 framework receipt 和 screen-space renderer。回归覆盖 direct/Cosmic/vertical failure、font generation defer/retry、BiDi invariant、invalid font/language、cache miss/hit/repeated failure、parallel duplicate、empty input、LTR/RTL/rich/VerticalRl 和 source range；31-sample cold/warm 1/100/1k/10k corpus记录 p50/p95/p99、RSS、backend calls、cache churn与power。当前 `text_direct_shape_*`、`text.shape_batch.*`、`text.layout` scope 与 `ZR_UI_LAYOUT_PROFILE` 已能提供采样骨架；没有 coordinator receipt 前不声称性能、功耗、Unreal 对比或 WGPU 截图通过。

## 7. P1：Layout、Viewport 与文档规模

### P1-14：便利 layout/measure 路径频繁创建短命 session

retained `SharedTextLayoutSession` 与 bounded cache真实存在，但若调用 convenience `layout_text`/measure API，仍会新建 session，无法跨帧复用 paragraph、font snapshot、shape run 和 line break。调用方很容易绕过真正的 retained fast path。

产品 surface、Editor document、world-space label 和 accessibility 应显式持有 text session/document handle；one-shot API 仅用于工具/小字符串并标注成本。telemetry 要区分 cache owner 和 bypass caller。

2026-08-30 current-source audit：该 finding 的“产品频繁构造”未在当前仓内复现。`UiSurface` 与
dynamic HUD/menu fallback 已持有 Core collection-bound `UiTextMeasureCache`/session；HUD/menu 的
default cache 构造仅在测试 helper。`compute_layout_tree`、standalone extract、`layout_text`、measure
wrapper 与 `shape_text_line` 保持显式 one-shot compatibility 边界，仓内没有 retained product-frame
调用点。对齐 Unreal `FSlateFontServices -> FSlateFontMeasure -> per-font cache` 的关键是产品 owner
retained lifetime，而不是把工具入口改成 TLS/global cache。因此当前状态为
`current_source_product_hot_path_not_reproduced / optimization_profile_gated`；只有出现真实产品 callsite
并取得 construction/cold-warm/allocation/RSS/backend/power profile 后才迁 explicit owner。

### P1-15：Viewport virtualization 只覆盖极窄的 plain-text 情形

`ui/text/layout_engine/viewport.rs` 的 fast path要求 Plain、HorizontalTb、Wrap::None、Overflow::Clip、有 document key、无 preedit；surface render 还只对 `editable.is_none()` 传 viewport。wrapped text、TextEdit、rich text、vertical text和composition仍全量 shape/layout。

百万行编辑器、日志、终端和本地化文档不能以此称为 virtualized。需要 paragraph index、visible line range、overscan、incremental line metrics、fold/decoration layer和后台预热；编辑/IME 只重排受影响 paragraph 与后续有限传播。viewport 与 accessibility/search 需共享文档模型而非复制全文。

### P1-16：rich/vertical intrinsic measure 以源字节数构造巨大方形 frame

`ui/text/layout_engine.rs` 对 rich/vertical intrinsic path 使用 `(parsed.text().len()+1)*em` 作为方形 extent，只在接近 `sqrt(f32::MAX)` 时截断，再执行完整 layout。大输入会制造极大坐标、昂贵布局和下游空间索引风险；UTF-8 byte length也不是视觉 advance 上界。

intrinsic measure 应按 paragraph/chunk 有界累积，受最大 glyph/line/extent/CPU budget约束；超预算返回 partial/intrinsic-lower-bound 或 explicit overflow，不得通过天文 frame 诱导算法完成。

2026-08-24 P1-16 pre-implementation audit：现实现把 `parsed.text().len() * em` 作为 rich/VerticalRl intrinsic 的正方形 `UiFrame`，将 UTF-8 byte length 当成 glyph advance 上界，并把大坐标传给 clipping、alignment 和 vertical column placement。Text03 已确定 `available_wrap_extent(f32::INFINITY)` 是无界主轴的正式合同，故不新增臆测上限：horizontal rich intrinsic 使用无界 width/height 且测量强制 logical start alignment；VerticalRl 使用无界 height，交叉轴 width 仅由 canonical hard-line count 与既有 column advance 推导为有限值，因为 `vertical_layout.rs` 对无穷 width 的 finite sanitization 会得到零。该修复是布局语义/数值安全收敛，不将其包装为性能优化；复杂 block/table 实际列数仍应由后续 retained paragraph artifact 给出。

性能测量计划先固定而不填结果：coordinator 在 Windows managed matrix 中以 1/100/1k/10k Latin、CJK、RTL、rich inline/block、VerticalRl hard-line corpus 比较变更前后 `text.layout`、`text.shape_batch.*`、`ZR_UI_LAYOUT_PROFILE` 的 p50/p95/p99、backend calls、allocated bytes/RSS、cache churn、WGPU timestamp 与同场景 power trace。任何内部分析确认的热点才允许单独选择优化；在受管 capture 前不得声称该语义修复消除了 CPU/GPU/功耗瓶颈或达到 Unreal 经验值。P1-16 当前为 `correctness_implementation_complete / static_review_complete / managed_validation_pending`：`intrinsic_measurement_frame` 及其 horizontal/VerticalRl contract regression 已落地，限定 Rust 2024 formatting、diff whitespace 和 retired byte-extent scan 已通过；Cargo、profiling、power、WGPU 和 PNG 仍未执行。

### P1-17：编辑器没有持久 paragraph/document model，修改复制整串状态

editable value、caret、selection、composition 以完整 `String` 和 metadata/component state重写。插入/删除需要分配和移动后缀，max-length 又会扫描 grapheme；没有 rope/piece table、paragraph generation、incremental shape invalidation或 saved-version identity。

建立 session-owned text document：chunked UTF-8 storage、paragraph/line index、grapheme cache、edit generation、selection/composition anchor、dirty span和layout dependency。UI component只保存 document handle与presentation state，序列化/提交通过受控 snapshot/transaction。

2026-08-28 current-source 前向状态：manager-owned retained document、exact range transaction 与 delta
history 已落地，但 Surface metadata/component/binding 仍复制完整正文，因此 P1-17 只完成 document 侧、
不能关闭。`editable_text/profile.rs` 已在正文 state 物化、proposed property clone、成功 projection、
committed/state-only/composition 分类和可选 component payload 处加入固定名、无正文 counter，并为
property prepare/commit 加固定 span；未开启 profiling feature 时为 no-op。E盘 current-source profile
harness `5/5`，Rustfmt/whitespace/隐私静态检查通过。managed 1 KiB/64 KiB/1 MiB/10 MiB
allocation/RSS/p50/p95/p99/power 与 matched Unreal baseline 尚未执行，故 document-handle/model-edit
硬切尚未开始；状态为 `retained_document_side_implemented_unvalidated /
surface_full_source_projection_open / content_free_profile_instrumentation_implemented_unvalidated /
direct_profile_path_5_of_5 / managed_baseline_pending`。

## 8. P1：Editing、Clipboard 与 IME

### P1-18：没有 undo/redo transaction 或 history

`UiTextEditAction` 只有 Insert、Backspace、Delete、MoveCaret、SetSelection、Set/Commit/CancelComposition；keyboard action代码没有 Undo/Redo，仓内 current source 也没有编辑 history。Text08 对 `Ctrl+Z/Y` 的完成声明与源码冲突。

需要 typed edit transaction，记录 replacement range、before/after selection、composition、timestamp/source、coalescing group 和 document generation；支持 undo/redo branching、IME composition grouping、paste/cut atomicity、max history bytes和 saved-state marker。password history须遵守 secret storage/zeroization policy。

### P1-19：公开 selection/composition action 只 clamp 到 UTF-8 char boundary

Backspace/Delete 使用 grapheme boundary，但 `SetSelection`、composition 和部分 movement只校正 UTF-8 char boundary。调用方可以把 caret/selection 放进 combining sequence 或 emoji ZWJ cluster，随后 replacement拆散用户感知字符。

编辑不变量应统一为 grapheme boundary，并允许 shaping cluster/visual affinity作为几何层附加信息。所有外部 byte offset先验证 document generation和boundary；无效 action返回 diagnostic，不要静默改到难以预测的位置。

### P1-20：上下键按源 hard line，左右键在 BiDi 中按逻辑顺序

`ui/text/grapheme.rs` 的 Up/Down按源码 newline和grapheme column移动，不看 wrapped visual lines，也不保留 preferred x；Left/Right调用 previous/next logical grapheme，不按 resolved BiDi visual order。竖排方向、home/end、page movement同样没有完整视觉模型。

navigation必须消费 resolved layout generation，保存 caret affinity和preferred inline coordinate；支持 visual grapheme left/right、wrapped line up/down、logical document command、line/document start/end、page movement和vertical writing。layout过期时应明确延迟/重算，不能回落源列并声称视觉正确。

### P1-21：word movement 仅以 Unicode split + alphanumeric 启发式定义

word navigation用 `split_word_bound_indices`，只要 segment含任意 alphanumeric就视作word。代码标识符、CJK、apostrophe、locale punctuation、emoji、路径和平台约定会产生与用户预期不符的 Ctrl+Arrow/selection。

建立可替换的 word-boundary policy：Unicode default、Editor/code、locale/platform profile；命令层区分 word/subword/token。测试覆盖 CJK、Arabic、combining、emoji、snake/camel case和标点。

### P1-22：IME cursor/composition geometry 在 layout 缺失时使用宽度猜测

resolved layout可提供准确 geometry，但缺失或 generation不匹配时会用 `font_size*0.6`、grapheme column与估算 wrap构造 cursor area。比例字体、kerning、BiDi、竖排、复杂 script和rich decoration下候选窗会明显错位。

IME host request必须绑定 document/layout generation；在有效 geometry未就绪时触发同步小范围layout或发布 last-good + pending标记，而不是按等宽猜测。平台 adapter要对屏幕坐标、DPI、窗口移动、subviewport和composition clause做实机测试。

### P1-23：IME surrounding-text/secure 生命周期没有统一 privacy contract

普通字段可提供最多约256 grapheme surrounding window，这是合理的功能基础；但 secure字段通过彻底禁用IME规避泄漏，normal/secure之间没有平台 capability、redaction范围、plugin可见性、focus loss清理和host receipt合同。

定义 `TextInputScope`/privacy policy，区分普通、密码、PIN、email、number等；host只获取最小必要 surrounding range和opaque selection offset，secure scope按平台API提交而不暴露全文。focus/window/session teardown必须撤销composition和清除host cache。

### P1-24：`NumberField` 只是逐字符白名单，不是数值编辑模型

`text_constraints.rs` 对 Number 允许任意 ASCII digit以及 `. - +` 出现在任意位置，没有 locale decimal/group separator、exponent、sign grammar、intermediate state、commit parse、range/step或typed value transaction。`--1..+` 也可能经过字符过滤。

NumberField应有独立 numeric editor model：locale-aware lexer、合法中间状态、commit/cancel、min/max/step/precision/unit、wheel/stepper、typed binding error和undo transaction。文本呈现与数值真值不能靠任意字符串长期并存。

## 9. P1：Rich Text、Parser 与内容预算

### P1-25：Parser入口没有统一 input/token/node/depth/time budget

Rich cache自身有驻留上限，table/list也有局部限制；但 parse入口未限制输入字节、token、node、attribute、style span、inline tag depth和decorator CPU。HTML/BBCode `active_tags` 可无限增长。缓存预算只限制结果驻留，不能阻止单次恶意输入耗尽CPU/内存。

建立 `RichParseBudget` 并贯穿 tokenizer/parser/decorator：input bytes、tokens、nodes、spans、attributes、nesting、decoded text、CPU/deadline。超限返回 partial/failed typed diagnostic，且不能把超大失败结果塞进cache反复解析。

### P1-26：未闭合标记可触发重复后向扫描，格式名称又超过实装语义

HTML tokenizer在每个 `<` 后扫描匹配 `>`，BBCode在每个 `[` 后找 `]`，Markdown在每个 marker后搜索closing marker；大量未闭合标记可近似 O(n²)。Markdown只实现 bold/italic/inline-code极小子集，没有 escape、nesting、link、paragraph、list、heading或CommonMark一致性。

改用单调游标 tokenizer/stack，保证O(n)或有预算上界；对每种格式声明 versioned subset和feature matrix。若不实现CommonMark，不应只用“Markdown”暗示兼容；公开名改为明确 subset或引入经验证的 parser库并在引擎层加预算。

### P1-27：Malformed/unknown markup 没有 structured diagnostics

parser返回 `RichParseResult`，不是 `Result`/diagnostic set；未知标签、属性错误、未闭合结构、截断和budget状态会被字面化、忽略或静默结束。Editor无法定位问题，runtime也无法区分作者错误与受限降级。

结果应包含 source range、format/version、severity、code、recovery action和completeness。Editor显示诊断并保留last-good preview；runtime使用bounded fallback text。diagnostic cardinality必须限额并聚合重复错误。

### P1-28：Custom decorator 在解析热路径运行但没有隔离与成本合同

custom decorator可扩展rich content，但没有执行次数、输入/输出节点、递归调用、panic隔离、线程模型或deadline合同。插件代码可在UI layout/parse路径阻塞frame，或生成远超输入规模的结构。

decorator应在声明 capability下运行，接收预算和只读token slice，返回bounded typed nodes；panic/error隔离，结果按decorator version/content hash缓存。非确定或主线程专用decorator不得进入cook/runtime关键路径。

### P1-29：Text01/02/03/07/08/09 的完成状态与 current source 未重新对账

现有计划记录了大量真实实现，但部分结论把“结构/API存在”升级成“工程闭环完成”。Text01未关闭 cooked bytes和global owner；Text02未关闭synthetic fallback与long-line语义；Text03未覆盖wrapped editable virtualization/visual navigation；Text07未覆盖入口预算和线性复杂度；Text08误报Ctrl+Z/Y并把secure/平台IME视作次要；Text09没有以session isolation、package与真实负载验收global cache。

这些计划必须按本文 finding重新打开，不应新建平行 truth。每个历史failure handoff保留归档，但 owner plan首页要列 current-source fingerprint、reopened finding、acceptance artifact和旧结论失效原因。

## 10. P2：完整性、诊断与维护性

### P2-1：`UiTextShaperStack` 目前只有一个 backend wrapper

类型名暗示多shaper stack，实际只有一个 `UiSharedTextShaper`。若没有近期多backend需求，应收敛命名；若保留扩展点，需定义选择、fallback、capability和diagnostic，而不是只增加一层转发。

### P2-2：多处字体/布局启发式没有统一配置与统计

fallback advance、cache限额、surrounding window、locale cache count、line cap等常量分散。部分应是安全硬上限，部分应由device/project profile配置。建立 typed budget schema、effective value dump和命中/拒绝计数。

### P2-3：每线程四 locale cache 的 eviction 无产品可见性

locale多于四个会替换thread-local state，但没有命中率、rebuild time或thrash诊断。多语言Editor/聊天产品可能出现周期性尖峰。加入bounded telemetry并由worker owner统一管理。

### P2-4：Rich source range 使用 `u32` 并在超大输入上饱和

超过4 GiB的byte range会saturate到`u32::MAX`，多个节点别名。正常UI不应接受如此大输入，因此应在parser入口早拒绝并给budget diagnostic，而不是让range静默饱和。

### P2-5：CPU cache size 主要是估算值，缺少真实owner/峰值统计

部分cache以文档字节或结构估值核算，不含allocator overhead、共享blob、backend内部对象和GPU counterpart。保留快速估算用于admission，但增加实际allocation/entry/generation/eviction/peak观测，并与11C的GPU atlas bytes关联。

### P2-6：Ignored/manual test 没有 current-source artifact binding

10个ignored test多为managed scale/exporter/manual evidence，没有统一记录源码fingerprint、OS/font set、driver、locale、输入corpus与阈值。它们可用于诊断，不能作为产品完成gate。关键性能与package测试必须非ignored并产出machine-readable结果。

### P2-7：文本诊断没有统一进入runtime diagnostics/profiling schema

font fallback、missing glyph、shape backend retry、layout budget、parser recovery、IME stale geometry和cache churn分散或不可见。应定义低基数metric与bounded sample，关联session/document/font generation，不记录secret/raw text。

### P2-8：文档术语未区分source line、hard segment、visual line与paragraph

当前代码和计划中“line/hard line/run/chunk”容易互换，导致64 KiB segment被误认为真实换行、viewport line被误认为paragraph。重构前先固定术语和identity，避免接口在实现中继续泄漏临时分块。

## 11. 既有 Text01–09 计划的重开映射

| 既有计划 | 必须重开的 current-source 差距 | 本文 owner |
|---|---|---|
| Text01 Font Resource/Database | cooked font bytes、系统字体确定性、session owner、typed load error、ingest budget、retired face lifetime | P0-1、P1-1～8 |
| Text02 Shaping/BiDi | synthetic glyph、typed outcome、64 KiB语义、normalization mapping、BiDi invariant、coverage receipt | P0-2、P1-9～13 |
| Text03 Layout/Hit Test | retained document、wrapped/editable/rich/vertical viewport、intrinsic budget、visual navigation | P1-14～17、P1-20 |
| Text04/05/06 Raster/SDF/Atlas | CPU glyph completeness与generation；GPU细节交11C | P0-2、P1-3、P1-13 |
| Text07 Rich Text | parser budget、线性复杂度、versioned subset、diagnostics、decorator isolation | P1-25～28 |
| Text08 IME/Input | secure policy、undo/redo、grapheme invariant、visual navigation、IME geometry/privacy、NumberField | P0-3、P1-18～24 |
| Text09 Performance/Observability | session isolation、real memory、cache churn、百万行、package和平台门禁 | P1-1～4、P1-14～17、P2-2～7 |

## 12. 目标架构

### 12.1 唯一 `TextRuntimeService`

按 engine/device 共享只读 content-addressed font blob与GPU资源；按 project/session拥有font collection、fallback policy、locale和generation。document/layout/shaping/raster cache通过typed handle和lease引用generation。Editor、runtime UI、world-space text、accessibility和plugin API只适配这一服务。

### 12.2 Closed font artifact 与异步依赖状态

Editor source manifest -> importer validation/subset/license policy -> cooked font blob + face metadata -> artifact registry -> runtime byte lease -> face/shaper/raster。每一步有schema/content hash、budget、typed error和last-good策略。系统字体是显式platform provider，不是隐藏fallback。

### 12.3 Document/paragraph 增量模型

`TextDocument`拥有chunked storage、paragraph index、grapheme/word boundaries、edit transaction/history、composition和saved generation。`TextLayoutSession`按paragraph和style/font generation缓存shape/line break，viewport只物化visible lines + overscan；accessibility/search可查询逻辑文档而不强制全量GPU layout。

### 12.4 Typed completeness 从 shaping 贯穿 render

每个 run/artifact表达 ready/pending/tofu/error/budget，保留chosen face与source cluster mapping。render只接受可rasterize glyph或engine tofu；缺失dependency可显示last-good/tofu并发低基数diagnostic，不能发布synthetic成功。

### 12.5 安全输入是跨层 policy

`TextInputScope`驱动mask、clipboard、IME surrounding、accessibility、capture/log和plugin visibility。secret原文不进入render command、diagnostic或通用snapshot；IME通过平台secure scope正常提交。undo/history、serialization和crash路径都遵守同一策略。

## 13. 分层重构里程碑

### M0：冻结合同与基线

记录current-source fingerprint，定义service owner、font artifact schema、shape outcome、document identity、secure scope和术语。把Text01/02/03/07/08/09 reopened finding写回owner计划；建立clean package、missing font、password、long-line、million-line和parser corpus基线。

### M1：关闭字体 cook 与确定性加载

把font bytes纳入artifact/cook，runtime禁止从project source重新打开；引入typed load error、budget和explicit system font policy。完成package-without-source gate。

### M2：Session-owned font/text service

替换process-global可变DB，使用immutable generation/lease与granular delta；`UiFontRegistry` 硬切已完成，继续把Cosmic worker/cache纳入service lifecycle。支持双session隔离与deterministic fallback。

### M3：Typed shaping 与真实缺字策略

移除synthetic rasterizable glyph；统一direct/Cosmic/vertical error contract，引入engine tofu、pending dependency和completeness receipt。贯通layout/artifact/11C renderer。

### M4：语义保持的Unicode/长文本管线

修复64 KiB chunk不产生假行，建立context overlap/cluster mapping；决定normalization策略，强化BiDi invariant和script/fallback coverage。加入Arabic/Indic/emoji/RTL corpus。

### M5：Document/paragraph 增量存储与layout

引入chunked document、paragraph generation、incremental shape/line-break和bounded intrinsic measure。扩展viewport到wrapped editable/rich/vertical/preedit，并提供后台预热/取消。

### M6：编辑事务与视觉导航

实现undo/redo、coalescing、saved state、grapheme invariant、visual BiDi/wrapped/vertical navigation、preferred coordinate和可配置word/subword policy。

### M7：安全文本与国际IME

端到端mask/clipboard/surrounding/a11y/log policy，移除“secure=禁用IME”；验证Windows/macOS/Linux真实IME、候选窗、DPI/窗口移动和focus teardown。

### M8：Rich parser hardening

线性tokenizer、统一budget、versioned subset、structured diagnostics和decorator sandbox/cost contract。对恶意未闭合marker、深嵌套和膨胀decorator做fuzz/benchmark。

### M9：NumberField 与typed input scope

建立locale-aware numeric edit/commit model、范围/step/unit、binding与undo；把email/url/search/PIN/password等scope映射到平台input hints和privacy policy。

### M10：性能、观测与11C联调

加入font/shaping/layout/parser/IME/cache低基数指标，关联GPU glyph/atlas residency；在百万行、多locale、多session、hot reload、VRAM pressure和device loss下做稳定门禁。

### M11：硬切旧API与关闭历史计划

删除source-path runtime font load、global mutable font DB、synthetic glyph、bool/Option错误旁路、dead UI registry和旧editable state写法。只有current-source非ignored gate与artifact齐全后才把Text01–09重新标记完成。

## 14. 验收门禁

1. **Cook/package**：项目source目录和原始font文件移除、系统字体禁用后，package仍能加载声明字体并完成Latin/CJK/Arabic/Indic/emoji shaping与raster；artifact hash在相同输入上稳定。
2. **Session isolation**：同进程两个project/session注册同family不同font，fallback/layout/glyph cache互不泄漏；一个session hot reload/teardown不清空另一个session。
3. **Typed failure**：强制missing/corrupt/unsupported font和backend failure，结果只能是typed error/pending/真实tofu；任何`font_id=None`的非空白glyph不得进入raster queue。
4. **Long line correctness**：超过64 KiB的Arabic/Indic/ligature文本与未分块oracle比较，不增加visual line，不切grapheme，cluster/source mapping一致；超预算有明确状态。
5. **Secure field**：render command、glyph artifact、clipboard、accessibility snapshot、capture、diagnostic和plugin event均无原文；password仍可用中日韩IME输入，focus/window teardown后host不保留surrounding text。
6. **Editing**：undo/redo覆盖typing coalescing、paste/cut、IME composition、selection replacement与redo branch；公开action永远保持grapheme boundary。RTL/wrapped/vertical视觉导航有golden geometry。
7. **Million-line**：100万行plain/wrapped editor在固定viewport下首帧、滚动、caret edit、search decoration的CPU、峰值内存和shaped paragraph数有硬阈值；near-caret edit不触发全文shape/layout。
8. **IME platform**：Windows TSF/IMM32、macOS text input、Linux IBus/Fcitx至少各一条自动/受控实机链，覆盖比例字体、BiDi、竖排、DPI、窗口移动、composition clause和candidate area。
9. **Rich parser**：100k未闭合`<`/`[`/`*`输入在线性/预算时间内结束，内存有上限并报告diagnostic；深层tag和decorator膨胀不能阻塞frame或绕过budget。
10. **Observability**：missing glyph、font load、shape retry/failure、layout budget、parser recovery、IME stale geometry和cache churn有bounded metric；任何metric/log不得包含secret/raw text。
11. **Current-source evidence**：关键门禁非ignored，artifact记录源码fingerprint、OS/font set/locale、backend、输入hash和阈值；旧PNG或只断言结构存在的测试不能关闭finding。

## 15. 禁止的临时修补

- 不得把font source复制到package旁边继续让runtime按路径打开；必须成为artifact-owned bytes与generation。
- 不得保留synthetic glyph ID并让renderer“尽量跳过”；typed completeness必须从shape贯穿render。
- 不得通过禁用IME声称password安全，也不得只在最终像素阶段盖黑框；原文不能进入通用render/clipboard/a11y链。
- 不得提高64 KiB常量或million-line benchmark机器配置来掩盖全文layout；需要paragraph/viewport架构。
- 不得只加Ctrl+Z键映射而没有edit transaction/history/coalescing和IME grouping。
- 不得仅扩大rich cache解决恶意parser成本；单次解析复杂度和输出规模必须受预算。
- 不得新增第三套font registry、text shaper或editable state来绕开global owner。
- 不得用ignored exporter、源码字符串断言或开发机系统字体通过来关闭package/平台finding。

## 16. 本轮产出边界

本文以 current-source 静态 review 和分层重构计划为主；后续状态仅在对应 finding 中记录已完成的结构硬切，不能替代 Cargo 或产品验收。下一轮11C会从`UiRenderCommand`/resolved glyph artifact继续追踪GPU upload、atlas/SDF、batch、clip、bind group、render graph和submission；11B的P0-2只有在11C确认renderer拒绝不完整glyph并有真实tofu后才算端到端关闭。

2026-08-29 lifecycle follow-up：Text Core 已实现 `RuntimeFontAssetClaimScope` 聚合认领账本，动态 UI 在首个
layout 前认领完整字体依赖，renderer 在 collection refresh 前 reconciliation；最后一个 scope 释放批量
退休 unclaimed owners，并裁剪 renderer 本地 ready/missing/error cache。此项仅关闭旧的 renderer-only
residency 设计缺陷；session/project switch、hot reload、fallback isolation、Cargo/WGPU、31-sample
allocation/RSS/power 与 11C 真实 glyph/tofu 产品门仍开放。`implementation_status` 继续保持
`pending`。release + changed/new asset admission 已收敛为单一 collection transaction；不得将源码复杂度
描述扩大为性能或产品完成结论。该 transaction 的 managed residency、fallback isolation、Cargo/WGPU 与
profile 证据仍开放。

## 2026-08-29 Publication clone-boundary follow-up

The review now separates three database-copy boundaries: the outer immutable-generation clone,
the owner-local registration staging clone, and an owned database clone returned only to legacy
mutable renderer consumers. The first is the snapshot lifetime contract; the second is a candidate
structural hotspot; the third is avoidable for claim/admission and receipt-only callers. A managed
Windows profile must record fixed spans/counters for all three boundaries across 0/1/8/64 dependency
admission, equivalent refresh, replacement, last-owner release, and release-plus-admission. Include
31 cold/warm samples, p50/p95/p99 CPU, allocations, RSS, generation publications, and package power,
then compare the same workload with the Unreal font-cache update/flush ownership model. The profile
is required before changing owner registration error-atomicity or claiming an optimization.

The non-validation slice may add a collection-owned published `Arc<FontDatabase>` result and migrate
receipt-only callers, eliminating their post-publication full clone while preserving exact snapshot
lineage. Owner staging remains profile-gated until an in-place transactional API can prove rollback,
cache invalidation, and owner/source deduplication equivalence.

Status: `clone_boundary_profile_plan_written / receipt_arc_migration_static_implemented /
owner_staging_api_profile_gated / managed_validation_pending`.

## 2026-08-29 Renderer/SDF font-loader convergence follow-up

The single-asset renderer load/resolve path and standalone admit/retire wrappers have been removed;
production and focused cache regressions now converge on the batch admission transaction and a real
collection claim scope. A second structural defect was found in SDF raster preparation: cache miss
could parse a runtime font manifest and mutate only the renderer-owned database after shaping had
already selected its collection snapshot. This was not an offline-bake requirement; offline `.zsdf`
artifact lookup has a separate manifest/bitmap cache and does not require runtime face registration.

The production SDF face cache is therefore lookup-only and reports an unadmitted mapping instead of
performing source I/O or registration. This removes duplicate loader work and prevents a raster-only
font face from diverging from shaping/layout. Seven now-unreachable SDF source/decode/budget/registration
metrics were deleted rather than published as permanent zeroes; their detailed diagnostics remain at the
batch admission owner. No claim is made about CPU, allocation, RSS, package power,
or Unreal parity: the managed Windows lane returned `cargo_reuse_pool_busy` before Cargo execution.
Static contract tests are 9/9 and scoped format/diff checks pass. `implementation_status` remains
`pending` until managed Cargo, fault injection, real Native/SDF WGPU evidence, and the prescribed
31-sample performance/power matrix complete.

## 2026-08-29 Cosmic snapshot-bound locale cache

The Cosmic thread-local locale cache now starts lazily from the caller's exact
`FontCollectionSnapshot`. Its previous zero-argument initialization read the process-shared font
database before refreshing to the renderer/session collection, adding an avoidable full-database
copy and making isolation depend on a post-initialization repair. The cache remains the single owner
of locale entries, eviction, and generation refresh; no second font registry or shaping path was
introduced. Parallel paragraph prewarm likewise has no process-default production entry: the default
wrapper is test-only, production callers carry their session collection, and the unused default finish
wrapper is deleted. Rustfmt, the static ownership guard, and scoped diff checks pass. Cargo, multilingual
shape/raster, WGPU/PNG, and the required 31-sample CPU/allocation/RSS/power comparison remain
pending; `implementation_status` remains `pending`.

## 2026-08-30 Range-invariant fail-closed follow-up

The M2 layout data path now treats source-range projection as an invariant rather than a best-effort
filter. Cosmic line/glyph projection rejects invalid checked ranges before alternate hard-line
normalization; `line_break` rejects invalid hard-line/glyph ranges with typed `LayoutFailed` or
`BidiInvariant`; UI glyph-wrap and corrected-range materialization reject metrics that cannot be sliced
from the canonical source. Legal zero-width virtual anchors remain supported. This prevents a malformed
backend result from becoming a partial but publishable line and keeps the existing owner-level
`Deferred`/`Failed` policy intact. Static suite 18/18, Python compile and scoped diff checks pass.
Managed Cargo remains pending; two direct Cargo attempts failed during E-drive dependency target writes
before `zircon_runtime` source compilation. Real multilingual shaping/raster, WGPU/PNG and the prescribed
CPU/allocation/RSS/power matrix remain pending; `implementation_status` remains `pending`.

## 2026-08-30 Dynamic fallback UI collection owner follow-up

Current-source review found a second collection identity in the dynamic runtime: project surfaces
received the Core-owned collection, while the no-project HUD/menu fallback extract cache used the
process-default `UiTextMeasureCache` and global generation probe. The session now resolves the Core
font service once and injects the same `Arc` into both owners. Fallback extract reuse is invalidated by
the generation of its own layout session; production has no default cache constructor or global key.

This is a correctness and ownership repair, not a measured optimization. Static suite 18/18, rustfmt
and scoped diff checks pass; an independent-collection generation/rebuild Rust regression is written
but not run. Multi-Core isolation, font-publication rebuild behavior, Cargo, real
project/no-project WGPU pixels, 31-sample latency/allocation/RSS and package power remain pending;
`implementation_status` remains `pending`.

## 2026-08-30 Rich representation and projection final static slice

The Runtime Text rich owner now bounds runs, paragraphs, tables, cells, retained projection indices,
and BBCode block/table nesting before the corresponding vectors grow. Default representation limits
are 131,072/16,384/4,096/65,536/262,144 and default block/table depths are 32/8. Depth overflow is a
typed failure; silent suppression and saturated `u16` table-depth identities are removed. Grapheme
alignment moved to a 100-line parser child, leaving parser root/builder at 715/162 lines.

The request-local table interval owner has also completed its isolated post profile. At 4,096
runs/paragraphs/tables/cells, the old 50,331,648-comparison path measured p50/p95/p99
60,544/85,779/123,556 us; the final canonical-order interval path entered 215,046 nodes and measured
3,337/4,467/5,611 us over the same 31-sample E-drive lane. The p50 improvement is 18.14x and the
256-to-4,096 p50 growth is 22.70x instead of 260.97x. First-sample working-set delta increased from
208,896 to 360,448 bytes, so only the isolated quadratic rescan is considered removed.

Exact-tag decorator dispatch now uses one parser-local HashMap instead of scanning all unrelated
providers per token. The 4,096-dispatch, 31-sample lanes measured old/new p50
517/140, 7,381/142, and 116,314/139 us at 16/256/4,096 decorators; the largest lane improves
836.79x. This follows Zircon's unique normalized-tag contract while preserving the parser/widget-owned
decorator lifetime seen in Unreal. Callback panic/deadline/cancel and provider lease/revoke are not
part of the lookup correction. A following static admission slice catches callback unwind as a tagged
parse error, limits accepted decorator metadata to 64 KiB per call by default, and limits cumulative
retained run metadata to 32 MiB before publication. Its Rust behavior regressions are written but
unrun; deadline/cancel/private temporary allocation and provider lease/revoke remain open.

Static Runtime Text contracts pass 38/38 with targeted Rust 2024 formatting and source checks. Two
E-drive Cargo attempts produced no result within 90/120 seconds and their owned processes were
stopped. Real table layout, Cargo, allocation/RSS, package power, WGPU framebuffer, a new PNG under
`docs/tests/runtime/text`, and matched Unreal-load evidence remain pending;
`implementation_status` remains `pending`.

## 2026-08-30 Rich table projection interval owner

Compiled rich-table projection previously rescanned all runs, paragraphs, and tables for each cell.
The isolated 31-sample E-drive release baseline reached 50,331,648 comparisons and p50/p95/p99
60,544/85,779/123,556 us at 4,096 objects, with only 8,192 output indices. A request-local balanced
interval index now prunes by subtree `max_end`, applies table depth/containment after candidate
collection, and is dropped after construction. The UI boundary consumes its checked source-order
indices without another sort/dedup pass. This follows the same explicit range ownership used by the
rest of Runtime Text and is not a measured end-to-end performance claim. Static Runtime Text tests
pass 35/35; managed Cargo, real table layout/WGPU/PNG, allocation/RSS/power, and Unreal matched-load
validation remain pending; `implementation_status` remains `pending`.

## 2026-08-30 Rich source representation admission follow-up

P2-4 is now closed at the production source boundary. `RichParseBudget` separately limits source and
visible output, caps both effective limits to the indexed `u32` representation, and rejects source
before the process cache copies it. Parser output append and emoji expansion share the output budget;
public parser APIs return `RichTextParseError` with actual/effective byte counts. The default 32 MiB
limit follows the existing retained text-document scale rather than the 8 MiB compiled-cache policy,
so cache bypass and parser admission remain distinct owners.

`CompiledRichText` construction is fallible and all byte/count/projection conversions use one checked
index owner. Failed single-flight compilation is delivered as one terminal result to current waiters
and removed from cache residency. UI layout maps capacity rejection to stable diagnostic
`ZR-TEXT-LAYOUT-012` and the existing failure-layout path. Static contracts pass 29/29; the current
Cargo fingerprint has no owned text-side errors but the workspace is not clean. Performance, power,
WGPU and Unreal-matched product claims remain prohibited until the managed matrix runs.

The next admission slice bounds the shared HTML/BBCode active-tag stack at a request-configurable
depth (default 128) and returns `ActiveTagDepthBudgetExceeded` before vector growth. The prior
10,000-depth default-success corpus now verifies typed rejection; the 5,000-depth close-index release
benchmark opts into a larger request budget so it remains algorithm evidence rather than product
authorization. Paragraph list-prefix projection also propagates unrepresentable compact indices as
typed layout failure. Tokenizer admission now also caps recognized tokens (default 65,536), encoded
bytes per token (64 KiB), and per-token attribute count/bytes (64/16 KiB). HTML and BBCode check
token size before tag-name allocation and check cumulative attribute work before key/value string
materialization; BBCode `tag=value` is covered by the same value budget. Markdown paired delimiters
consume two token units before style dispatch. Static contracts pass 35/35. Delta-style
clone/allocation work remains explicitly profile-gated; node/span/general block and table depth,
time/deadline/cancellation, consumed-budget receipts, and decorator isolation remain open. The latest
Cargo check stopped at an unrelated interface session export before Runtime Text.

## 2026-08-30 One-shot compatibility provider snapshot boundary

Current-source review found that the remaining `DirectTextShapeRunProvider` compatibility object
was zero-sized and reacquired the process font collection inside every shape request. A single
standalone measurement could therefore observe different font generations if publication occurred
between its line/metric requests, unlike Unreal `FSlateFontMeasure`, which retains one concrete
`FSlateFontCache` for measurement and character-offset queries.

The provider now captures one immutable `FontCollectionSnapshot` at construction, reports that
snapshot revision, and routes both horizontal and vertical requests through the canonical
`shape_text_with_diagnostics_in_font_collection` entrypoint. Retained Runtime paths continue to use
`SharedTextLayoutSession` or an explicit collection-bound provider; this only hardens the declared
Editor/standalone compatibility boundary. A regression publishes a new generation after provider
construction and verifies that all returned glyph face handles still belong to the original
collection.

This is an ownership/correctness repair, not a shaping or performance optimization. Static source
contracts and Python tests are updated to 20/20; the Rust behavior regression is written but remains
unrun because managed Cargo is unavailable. WGPU/PNG, IME/pointer product input, profile/RSS/power
and matched Unreal validation remain pending; `implementation_status` remains `pending`.

## 2026-08-30 Surface input font-measure owner follow-up

Current-source tracing found that Core collection injection stopped one layer too early for input
geometry. A retained Surface laid out text with its injected collection, while missing-artifact caret,
selection, IME-rectangle, and pointer hit-test recovery instantiated the process-default direct shape
provider. This is a correctness split, not evidence that the shaping algorithm itself is slow.

The correction adds an immutable collection-bound direct provider and carries the Surface measure
cache snapshot through those recovery calls. Because the neutral layout DTO has no collection revision,
source reshaping is gated by the Surface's observed generation; a mismatch consumes published
artifact/glyph advances until normal layout recomputation. The design is grounded in Unreal
`SlateCore/Private/Fonts/FontMeasure.cpp:48-54` and `Public/Fonts/FontMeasure.h:201,239`, where
`FSlateFontMeasure` is created with and retains the exact `FSlateFontCache` used for measurement and
offset queries. Editor/standalone compatibility remains process-owned; Runtime input does not recover
global font state after its owner is selected.

No algorithmic or performance claim is made. Static ownership tests pass 19/19 and the injected
collection face-identity Rust regression is written but unrun. Managed Cargo, real IME/pointer input,
WGPU pixels, and the 31-sample latency/allocation/RSS/power matrix remain pending;
`implementation_status` remains `pending`.

## 2026-08-30 Rich table projection interval owner

Compiled rich-table projection previously rescanned all runs, paragraphs, and tables for each cell.
The isolated 31-sample E-drive release baseline reached 50,331,648 comparisons and p50/p95/p99
60,544/85,779/123,556 us at 4,096 objects, with only 8,192 output indices. A request-local balanced
interval index now prunes by subtree `max_end`, applies table depth/containment after candidate
collection, and is dropped after construction. The UI boundary consumes its checked source-order
indices without another sort/dedup pass. Representation count caps now bound run, paragraph, table,
cell and retained projection-index growth before their owning vectors expand. This follows the same
explicit range ownership used by the rest of Runtime Text and is not a measured end-to-end performance
claim. Static Runtime Text tests pass 35/35; managed Cargo, real table layout/WGPU/PNG,
allocation/RSS/power, and Unreal matched-load validation remain pending; `implementation_status`
remains `pending`.

## 2026-08-30 Rich compiled cluster-owner correction

The rich compiler retained a document-sized `(u32, u32)` grapheme vector with no production
consumer, duplicating cluster ownership already held by shaping/layout. Local Unreal source confirms
that rich markup parsing publishes stripped text and line/run ranges rather than a parser-owned
grapheme index. Before code changes, an E-drive 31-sample release microprofile measured the exact old
expression: 1/8/32 MiB ASCII produced 8/64/256 MiB vector payload and p50
65,236/736,093/3,074,179 us; the 32 MiB first working-set delta was 269,508,608 bytes.

The field, segmentation pass, identity/accounting work, and accessor are now removed as a hard
cutover. The isolated owner has exact post-cutover payload zero and no replacement stage. Static
Runtime Text contracts pass 34/34, with targeted format/source/diff guards also passing. This is not
end-to-end shaping/layout/render or power acceptance: managed Cargo is still blocked before Runtime
Text, and WGPU/PNG, allocation/RSS, package power, and matched Unreal-load validation remain pending;
`implementation_status` remains `pending`.

## 2026-08-30 Rich immutable artifact owner convergence

Current-source consumer tracing found no production caller of public `RichTextParser::parse()`:
Runtime UI/layout already retains `Arc<CompiledRichText>`. The obsolete entry cloned the complete
parsed payload after canonical compile/cache lookup, separating runs/paragraphs/tables and dynamic
metadata from the source and parser generation owned by the compiled artifact. Local Unreal's parser
and rich marshaller hand parsed output directly into layout-run construction rather than publishing a
second partial deep-copy owner.

An E-drive release baseline measured the exact clone before removal over 31 samples. The
4,096/32,768/131,072-run lanes performed 12,355/98,819/395,267 allocations, requested
1,014,784/8,118,272/32,473,088 bytes, and measured p50 2,454/22,059/111,366 us. Production now has one
materialization API, `compile() -> Arc<CompiledRichText>`; parsed consumers borrow from the retained
parent and owned clone helpers are test-only. The removed production stage has exact post cost zero.
The current reproducible Runtime Text static suite passes 34/34, but managed Cargo, external downstream compatibility,
end-to-end layout/render, WGPU/PNG, allocation/RSS/power, and matched Unreal-load evidence remain
pending; `implementation_status` remains `pending`.

## 2026-08-30 Rich parser cache-identity exhaustion correction

Parser identity and decorator/emoji generations previously wrapped and could alias an older
compiled-rich cache key. Registration also mutated provider state before its generation advance.
The owner now uses a nonzero optional parser identity allocated by atomic
`fetch_update + checked_add`; exhausted parsers fail typed before cache work. Provider and emoji
generations are checked before registry mutation and fail without publication after `u64::MAX`.

This follows the ownership property of Unreal's widget-retained parser/decorator marshaller rather
than copying pointer identity into Zircon's still-global cache. It is a correctness fix, not measured
shaping/layout optimization. The current reproducible Runtime Text static suite passes 35/35 and
format/source guards pass; Rust boundary tests are written but managed Cargo remains pending.
RuntimeRichTextService, provider leases/revoke, targeted retirement, WGPU/PNG, RSS/power, and matched
Unreal validation remain open; `implementation_status` remains `pending`.

## 2026-08-30 Rich compile owner convergence

Production rich parsing no longer crosses a process-global parser/cache/report boundary. Each
`RichTextParser` owns its bounded compiled cache, and the retained Surface text session explicitly
supplies that owner to layout, measurement, prewarm, retained-document, render preparation, and
profiling. Independent sessions no longer share rich artifact residency or clear lifecycle; the
static parser survives only in cfg-gated corpus helpers. This matches Unreal's retained
widget/marshaller ownership direction without inventing an app singleton.

The combined Runtime Text static suite passes 36/36. Owner-local Rust behavior coverage is written
but unrun because managed Cargo was bypassed; no product latency/RSS/contention/power claim is made.
RRT-P1-010/014/016, WGPU/PNG, multi-Surface profiling, and matched Unreal validation remain open, so
`implementation_status` remains `pending`.
