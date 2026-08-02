---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: shaping-quadratic-metadata-and-backend-projection
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/hard_line.rs
  - zircon_runtime/src/text/shaping
  - zircon_runtime/src/text/layout/line_break/mod.rs
  - zircon_runtime/src/text/layout/measure.rs
  - zircon_runtime/src/text/layout/rich.rs
  - zircon_runtime/src/ui/text/layout_engine/wrapping.rs
  - zircon_runtime/src/text/service.rs
  - zircon_runtime/src/text/font/database.rs
---

# Shaping二次扫描、重复backend与worker状态放大

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/shaping/**`当前源18/18 Rust文件
- 修复责任计划：`docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md`
- 联动责任：face/cache/worker联动Text01/Text09。
- 交接原因：BIDI/script/linebreak/backend与glyph projection由Text02拥有；shared font bytes和worker总预算分别需要Text01/Text09协作，不能在性能计划复制owner。

## 失败现象与复现证据

PERF-MVP-234：UAX14 opportunity、script segment与line start原来分别按glyph/line全表重扫；前三项已直接改为partition/precomputed index。horizontal/vertical backend projection仍按每个backend glyph扫描全部boundary与source glyph，并collect overlap Vec，最坏O(G²)+O(G)临时分配。BIDI line order重复计算reordered levels，fallback spans按grapheme临时codepoint Vec/family String。

PERF-MVP-235：service与cosmic各构建一次BidiInfo；cosmic Advanced shape后，language/variable horizontal及vertical upright segments再次RustyBuzz shape，并逐段重建face/variations/features/buffer。thread-local cache每worker持最多4 locale FontSystem，generation变化由caller同步重建。静态证据见`docs/plans/performance/01/2026-07-18-text-shaping-static-review.md`。

## 最低共享层根因

shaping pipeline没有一份贯穿itemization、BIDI、script、fallback、backend与projection的indexed paragraph context。各阶段各自从String/ranges重建查找结构；为补cosmic未暴露的language/variation/vertical细节，又在已shape glyph上执行第二backend。FontDatabase暴露bytes而非generation-owned parsed face，使segment backend重复构造资源。

## 架构修复验收

- 保留已落`partition_point` line-break/script与precomputed line starts；源码门禁不得回退全表fold/find/prefix scan。
- paragraph context一次构建BIDI levels、line starts、break opportunities、script/fallback segments，并以cursor/index供各glyph阶段消费。
- horizontal/vertical backend projection用sorted cluster boundaries与two-pointer/interval cursor；每backend glyph只访问重叠source cluster，禁止filter+collect全source Vec。
- BIDI base/levels/line order共用一个analysis，line visual/logical结果从同一次reordered levels派生。
- 选择唯一shape backend：language/variation/vertical进入一次shape；禁止cosmic Advanced输出后再整segment RustyBuzz shape。若保留cosmic layout，只消费其未被二次替换的结果。
- Text01提供generation-owned shared bytes/parsed face/instance；同face多segment copied bytes=0。Text09限制per-worker locale systems总数/bytes，generation refresh有counter并移出敏感caller路径。
- 1/100/1k/10k Latin/CJK/RTL/vertical记录metadata/projection visits、overlap alloc、BIDI/backend calls、face bytes、FontSystem bytes与refresh time；复杂度近O(G log N)或O(G)。
- locl/variable/kerning/features、fallback face/instance、TTB/BTT、source/visual range、UAX9/14与产品像素全部等价。

## 禁止临时方案

- 不得只给二次扫描预留Vec capacity；访问复杂度与per-glyph collect必须一起消除。
- 不得关闭language/variation/vertical二次backend而丢失locl/vmtx精度；应收敛为一次正确shape。
- 不得让每worker无限增加locale FontSystem或以更多线程掩盖重复shape。
- 不得缓存borrowed RustyBuzz Face跨越font generation；shared parsed face必须携generation/face identity并可失效。

## 修复结果与回传

2026-08-01 implementation state: `open / resolving_failure / text02_non_validation_implementation_complete / secondary_review_complete / secondary_review_findings_forward_fixed / final_static_recheck_complete / managed_validation_pending / cross_owner_metrics_pending`。

- PERF-MVP-234/235不再保留projection阶段：`itemize.rs`一次生成hard-line、grapheme、BIDI level、script、fallback face/instance与vertical orientation segment；`horizontal/direct.rs`和`vertical/direct.rs`直接从一次RustyBuzz结果构建最终glyph。旧horizontal/vertical projection和临时overlap index均已硬删除，因此不存在cosmic glyph全表扫描、overlap Vec或二次segment替换入口。
- horizontal一次应用language/locl、canonical features、kerning与effective face instance；vertical upright一次应用TTB/BTT、vertical GSUB/GPOS、language/features与effective instance，sideways使用同一RustyBuzz horizontal backend。cosmic只在direct整请求不能建立时独立回退，不消费其输出再调用第二backend。
- Text02输出严格保持逻辑source cluster顺序；RTL/BTT backend结果由共享cluster owner恢复逻辑序并保留同cluster多glyph内部顺序，Text03继续在断行后唯一执行L1/L2。cosmic-only回退也按source cluster稳定归一，不把视觉序泄漏到`ShapedGlyphRun`。
- fallback primary coverage直接遍历原文，不再复制全文codepoints；cluster codepoint Vec跨grapheme复用，连续同face/instance span在分配family String前合并。
- `text/hard_line.rs`的shared descriptor显式保存content与CRLF/Unicode separator range；horizontal、vertical、cosmic、synthetic fallback、Text03 measure/rich/line-break和UI source segmentation复用同一owner。separator统一投影为零advance mandatory virtual glyph，line/source range连续覆盖完整输入；mandatory boundary禁止kinsoku跨行合并。
- cosmic fallback按backend raw LF paragraph起点解释`run.line_i`，再用单次共享hard-line数据分配CR/VT/FF/NEL/LS/PS；baseline使用`line_y-line_top`行内值。`LayoutGlyph.font_id`映射的实际face是唯一权威，映射缺失时face/instance fail closed，不回猜预选span。
- Vertical Mixed把Unicode VO `Tr`作为独立shape orientation：同一script/language/instance/user features先执行TTB/BTT，只有启用与强制关闭`vert`/`vrt2`的cluster输出确实不同才保持upright；`locl`、variation selector或用户feature改变glyph不会误判vertical substitution，未替代时采用`Cw90` fallback。普通`R`仍走horizontal backend；horizontal direct按每个实际segment face聚合缩放ascent/descent/line-gap，固定0.8em仅用于缺失元数据/空行。
- horizontal/vertical direct在恢复逻辑单调cluster序后用单cursor确定cluster end，删除per-segment offset Vec/sort/binary search；`BidiParagraph::line_order`从一次`reordered_levels`同时派生logical levels与visual indices。
- service生产shape不再先解析Auto direction再交给cosmic；resolved direction来自canonical shaped run。native fallback span查询也保留requested direction，不执行未消费的前置Bidi分析。
- 二次审查已前向修复RTL projection错误模板选择、parallel source Arc未复用、feature O(F^2)/digest-only碰撞身份、Rust 2024 let-chain对2021 workspace的编译阻断，以及native fallback重复Bidi。
- 独立二次审查最终无P0/P1；提出的两个P2随后已补齐为保守cache resident estimate与完整run serde/range roundtrip，不留作managed validation替代实现。

Text02唯一shape backend的非验收实现已完成。实现完成后的定向二次审查为P0=0、P1=4、P2=4，8项均已按上述最低owner前向修复；focused Rust 2021 format、whitespace、单一hard-line owner、零cluster sort与单次BIDI reorder静态复核通过。真实Windows WGPU产品帧harness已准备为`docs/tests/runtime/text/runtime_text_mvp_foundation_product_framebuffer_20260801.png`，但尚未运行，不把路径切换记为像素证据。failure仍保持`open`：Text01 generation-owned parsed-face/face-byte指标、Text09 FontSystem bytes/refresh counter、managed Cargo、1/100/1k/10k backend-call数据和真实产品像素尚未形成验收证据；本轮没有等待queued/running ticket，也没有生成策略文字截图。

2026-08-01 post-fix review再次确认P0=0/P1=0：`Tr` provenance 改为同 buffer、direction、script、language 与全部非竖排 feature 相同、仅关闭 `vert`/`vrt2` 的 cluster glyph-sequence 差分；第二次 shape 只在 TransformOrRotate segment 启用，cluster map/set 为线性辅助，不再扫描 GSUB lookup 输出集合。

2026-08-01 产品证据基础设施前向修复：`runtime_text_multilingual_product_framebuffer.rs` 只有在连续两帧的 raster worker 均为 `pending=0`、`failed=0`、`missing_image=0`、`visible_placeholder=0`、`upload_requeued=0`、`upload_failed=0` 时才允许 capture；其中 pending 映射 durable `pending_worker_count`，failed 合并请求失败、完成错误和被拒绝的无效位图，missing_image 表示无 source raster，visible_placeholder 只在实际 `TransparentPlaceholder` handoff 时计数，upload 两项覆盖 GPU atlas 图集未写入的路径。六项均从 native atlas report 贯穿到 `RenderStats`。达到帧数上限或任一状态非零都会在 capture 前失败，不能生成未完成的 PNG。输出路径保持为 `docs/tests/runtime/text/runtime_text_mvp_foundation_product_framebuffer_20260801.png`，该文件尚未生成，failure 继续保持 `open`。

2026-08-02 SDF/MSDF capture-path 审计：distance-field atlas 在本帧同步提交 `queue.write_texture`，不持有 native raster worker 或 renderer-upload queue 的未完成状态；SDF 生成的 `GenerationPending`、预算延后和 generation failure 会在同一 prepare 轮经 `apply_sdf_atlas_fallbacks_with_cpu_runs` 转入 native/overlay，继而由上述六项可见 native raster 条件覆盖。故不重复增加 SDF 等待计数，也不把已正常呈现的 native fallback 误作截图不稳定；真实 Windows WGPU 产品帧仍待 coordinator wakeup 后受管执行，failure 保持 `open`。
