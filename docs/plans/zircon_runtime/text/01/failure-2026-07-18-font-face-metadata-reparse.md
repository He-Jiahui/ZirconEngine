---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: font-face-metadata-reparse
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/font/instance.rs
  - zircon_runtime/src/text/font/vertical_metrics.rs
  - zircon_runtime/src/text/font/database.rs
  - zircon_runtime/src/text/shaping/horizontal/backend.rs
  - zircon_runtime/src/text/shaping/vertical.rs
  - zircon_runtime/src/text/raster
---

# Font face metadata在glyph/run路径重复解析

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/font`当前源32/32 Rust文件及shaping/raster调用图
- 修复责任计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 联动责任：Text02消费parsed shaping face，Text04消费stable raster face，Text09限制worker-local metadata副本。
- 交接原因：parsed face、variation axes和metrics的generation owner属于Text01；本记录补全PERF-MVP-240/235跨计划交接，不建立重复性能编号。

## 失败现象与复现证据

`variations_for_face`每次调用都重新`ttf_parser::Face::parse`并collect axes；horizontal variation/language shaping与SDF/raster会反复进入。`vertical_glyph_advance_px`还对每个vertical glyph独立parse face再查vertical metrics。PERF-MVP-240已确认bitmap request侧存在每glyphvariation/Swash face identity重建，本轮证明根因位于共享font metadata层而非单一raster backend。

## 最低共享层根因

FontDatabase长期保存bytes、descriptor与coverage，却没有同generation的parsed face/axis/vertical-decoration metrics artifact；各consumer拿Arc bytes后自行parse并重复投影variation。

## 架构修复验收

- 每个`(FontFaceId, face_index, font generation)`只构建一次可安全共享的face metadata，至少包含axes、units、vertical/decoration metrics与backend stable identity。
- effective variation按`(instance, requested weight)`有界缓存，显式instance与default instance走同一canonical路径；不重复BTreeMap/Vec规范化。
- vertical run按face/instance批量读取metrics，Face parse不按glyph增长；Text02/04直接消费同一artifact。
- Text01 的 1/100/10k glyph 门记录 metadata `Face::parse`/build、axis scan、variation alloc/bytes、metrics lookup、worker duplicate bytes与p50/p95；stable generation 每 face metadata build<=1。FDSM outline `Face::parse`、batch worker 与总 CPU/RSS 计数由 Text05 `sdf-source-identity-and-generation-not-batched` 的独立 1/100/10k 门负责，二者不得互相代替。
- TTC face index、variable/static、system/shared bytes、hot reload、vertical metrics fallback、SDF/native pixels和current-source Cargo通过。

## 禁止临时方案

- 不得在每个worker无界复制完整parsed metadata而没有共享owner和总预算。
- 不得只缓存default instance而让显式variation仍按glyphparse。
- 不得持有借用bytes生命周期之外的`ttf_parser::Face`或绕过font generation失效。

## 修复结果与回传

Open state: `implementation_complete / managed_validation_pending`。Text01 已建立 generation-owned `FontFaceMetadata`，把 metadata SFNT parse 收敛为 `text/font/face_metadata.rs` 单一生产 owner，并由同一 artifact 提供 axes、codepoint→glyph 投影、coverage、units、vertical/decoration metrics 与稳定 source identity。effective variation 使用含显式 instance + requested weight 的 256-entry/256-KiB LRU cache，热消费者读取共享 `Arc<VariationCoords>`；vertical shaping 按唯一 face 复用 `FontVerticalMetrics` 视图。系统 `FontDb` 原始 bytes 与 TTC/nonzero standalone face bytes 使用 per-face `OnceLock<Arc<[u8]>>` 共享，SDF consumer 不再为 glyph-id/axis/metrics 额外 parse 或每 glyph 复制整份字体。database clone 发生字体 mutation 时 detach 派生 cache，避免 generation 分叉后复用旧 face/instance。1/100/10k glyph ignored evidence test 已落代码并输出 metadata build/cache bytes/p50/p95；独立终审为 0 Critical / 0 Important / 0 Minor、Ready。FDSM 轮廓生成仍需借用型 `ttf_parser::Face`，dynamic cache miss 与 offline batch 的 source-context/parsed-outline 复用由仍 open 的 Text05 [`sdf-source-identity-and-generation-not-batched`](../05/failure-2026-07-18-sdf-source-identity-and-generation-not-batched.md) 承接；Text01 不再把该跨计划未完成项误记为 production parse=1。仍待 fresh managed focused/broad Cargo、TTC/variable/system/SDF/native 上行门禁与 fixed return。

## 2026-08-25 Cooked artifact 冷路径测量范围

`FontAsset` 导入的 `parse_font_metadata` 已对每个 SFNT/TTC face 构造 family、轴、metrics 与 cmap coverage；运行时 `FontDatabase::replace_font_asset_blob` 则通过 `font_asset_faces` 调用 `FontFaceMetadata::from_sfnt_bytes`，以建立 codepoint→glyph 映射和 vertical/decoration 消费视图。现有 1/100/10k gate 从已注册 face 开始，只证明热 shaping/raster/SDF 消费者不重复构建 metadata，**不**证明导入后首次 runtime 注册不存在第二次解析。

在改动该结构前，必须以托管 Windows profiling profile 完成以下三相独立计数与 p50/p95：

1. A：`import_font_asset` 的 decode、face parse、cmap 构建、artifact serialize 时间与 RSS；
2. B：从 artifact cache 加载 `FontAsset` 后首次 `replace_font_asset_blob` 的 metadata build、bytes/Arc 共享与注册时间；
3. C：首次 horizontal、vertical 与 SDF glyph 使用的 glyph-map/vertical 访问，以及稳定帧零新增 parse 的时间、分配与 RSS。

矩阵至少覆盖单 face SFNT、TTC nonzero face、variable instance 和 1/100/10k glyph 请求；测试计数必须把 importer parse 与 runtime metadata build 分开，不能把 cache hit 或 SDF/FDSM outline parse 归入本记录。任何候选优化还必须保留 content-hash/version 绑定的 cooked blob、完整 glyph-id 映射和 vertical metrics；仅复用 importer 的 family/coverage 字段会丢失这些运行时语义，不能作为临时捷径。

当前 managed profiling 在编译前被外部 `Cargo.lock` 与 workspace manifest 不一致的 `--locked` 错误阻断，因此本节无基线数据、无功耗数据、无算法收益结论。锁文件恢复后先运行上述三相基线与 Windows power capture，再决定是否持久化额外派生表或采用按需解析；在此之前禁止把该风险标记为已优化或已验收。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
| --- | --- | --- | --- |
| 2026-07-19 03:50 +08:00 | `implementation_complete / managed_validation_pending` | 单一 metadata `Face::parse` owner；metadata `Arc<OnceLock<_>>`；shared variation LRU；run-level vertical metrics view；bounded/scale tests；rustfmt、scoped diff、metadata build count=1。 | Plugins01 captured-source compile 暴露的 Text visibility/type错误已在最低层修正；该外部 job 非 immutable evidence。待 fresh managed Text font focused、lib/upward、ignored scale、review 与 fixed return。 |
| 2026-07-19 04:35 +08:00 | `implementation_complete / managed_validation_pending` | 新增 Windows system face 首次访问 metadata build=1/重复访问不增长回归；font registration mutation 对 match/effective/fallback cache 执行 clone-generation detach；system discovery 批内延迟并在循环后只 detach 一次。 | Rust 1.94.1 rustfmt 与 scoped diff 静态检查通过；Cargo 队列受 blocking controlled action 与前序 reservations 占用，尚未执行测试断言。 |
| 2026-07-19 08:20 +08:00 | `implementation_complete / review_green / managed_validation_pending` | metadata 一次构建新增有序 glyph map；dynamic/offline SDF 删除 wrapper 侧 glyph-id/axis/metrics 重解析；FontDb/TTC bytes 以 winning `Arc` 跨重复请求和 immutable clone 共享；补齐 system/TTC `Arc::ptr_eq` 回归。 | 48 个 leased Rust 文件 rustfmt + scoped diff、9/9 结构断言通过；独立终审 0/0/0 Ready。Render18 长期 queue-1 reservation 阻塞 exact Text check，未声称 Cargo GREEN。 |
| 2026-07-19 16:12 +08:00 | `implementation_complete / cross-plan-outline-batch-open / managed_validation_pending` | 全 Text production 静态扫描确认 metadata parse owner 仅 `font/face_metadata.rs`；另两处 parse 分别为 FDSM glyph outline generator 与 offline cmap selection，不属于 metadata 重建。 | Text05 canonical failure `sdf-source-identity-and-generation-not-batched` 已覆盖 source context、parsed outline、batch worker 与 1/100/10k parse 预算；该 failure 仍 open，Text01 不吸收或伪称完成。 |
| 2026-07-28 01:45 +08:00 | `implementation_complete / managed_broad_runtime_passed / upward_pending` | Managed current-source job `8f1c073d40ce4bee8483c046e6ee6b9b` / run `48f0711c4ca1468d90b7545df7c6e047` executed `cargo +1.94.1 test -p zircon_runtime --lib text::font --locked --jobs 1 --color never -- --test-threads=1`. | Exit 0: `79 passed / 0 failed / 2 ignored / 8922 filtered`, including metadata once-build, effective variation, TTC, system-source, vertical decoration, SDF materialization, and shared-generation coverage. Text05 outline-batch scope and the editor upward return remain open. |
| 2026-07-28 02:42 +08:00 | `Text01_runtime_return_passed / external_editor_return_failed / Text05_open` | Editor job `4eefa547982a4bd896813d9fad698f21` / run `ceff37fc13224768af1c365287f242e5` compiled the current Runtime/Text source before exiting 101. | Its 56 errors are all in editor API/DTO/test owners, not Text01. Text05's independent outline-batch scope also remains open; neither is absorbed or falsely closed here. |
| 2026-08-11 04:20 +08:00 | `implementation_complete / managed_validation_pending` | 静态复核当前 `FontDatabase`、shared generation、metadata 与 effective-instance 路径：`FontFaceMetadata` 仍是唯一 SFNT metadata parse owner；shared snapshot 在读锁内取得 generation，mutation 在同一写锁内先发布 generation；per-face metadata/source/TTC standalone bytes 均由 `Arc<OnceLock<_>>` 共享；face mutation 会 detach face-dependent variation/fallback/match caches。 | 本次未运行 Cargo/WGPU，也未把历史受管 Runtime 通过冒充 current-source 验收。Text05 outline/FDSM 批处理与 editor 上行问题继续由各自 failure 负责；本记录维持 `open`，等待协调器后续 managed receipt。 |
| 2026-08-25 | `implementation_complete / static_checks_passed / managed_validation_pending` | `FontBlobArtifact` 进入 artifact cache 后将统一 manifest wire 升级为 `ZRARTM06` / schema 6，旧 M05 工件在 payload 反序列化前拒绝；冷路径回归首次导入后销毁 `ProjectAssetManager`、删除源 `.ttf`、重新打开项目，从持久化 artifact 取出 cooked blob 并注册可解析的 `FontDatabase` 主 face。 | `rustfmt --check` 与 scoped `git diff --check` 通过，且 `M05` 只保留为旧代际拒绝负例。受管 profiling/Cargo 仍在编译前被外部 `Cargo.lock` 与 workspace manifest 的 `--locked` 不一致阻断；未生成 WGPU PNG、性能或功耗数据，不能作为当前源码验收或里程碑完成证据。 |
