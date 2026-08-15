---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: sdf-source-identity-and-generation-not-batched
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/sdf/fdsm_gen.rs
  - zircon_runtime/src/text/sdf/font_bake/distance_field.rs
  - zircon_runtime/src/text/sdf/font_bake/offline_source.rs
  - zircon_runtime/src/text/font_sdf_build_tool/bake.rs
---

# SDF source identity与generation没有批处理owner

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/sdf`24/24与`font_sdf_build_tool`6/6 Rust文件
- 修复责任计划：`docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md`
- 联动责任：Text01提供parsed face/variation metadata，Text09/Runtime11提供有界cache、worker、取消与shutdown预算；回链PERF-MVP-240/235/246。
- 交接原因：runtime/offline必须共用Text05 batch generator，不能分别局部缓存同一font source。

## 失败现象与复现证据

PERF-MVP-250：动态glyph先在distance-field wrapper parse face，generator再parse一次。offline glyph每次materialize standalone bytes、hash整份font、解析variation/face、同步读artifact并复制rect pixels；原来还重复manifest load与instance handle resolve，本轮已直接各降为一次。build tool对全cmap glyph单线程逐项重复face/FDSM setup。

## 最低共享层根因

没有`(asset, face, instance, bake params, font/asset generation)`级source context；manifest、bytes/hash、parsed face/axes、artifact、FDSM worker请求都从per-glyph函数重新推导。

## 架构修复验收

- generation-owned source context一次解析manifest、standalone bytes/source hash、face/axes/variation和offline artifact；所有glyph只引用stable handle。
- dynamic glyph选择与FDSM generation共用同一parsed face，不允许wrapper+generator双parse。
- runtime miss按unique face/instance/bake params成批提交有界worker queue，支持dedup、cancel、age、byte/CPU budget和shutdown；主线程只commit完成结果。
- offline build tool复用同一batch generator并受全局TaskPool预算，输出按glyph identity稳定排序，任意worker数byte-identical。
- 1/100/10k glyph记录manifest parse、font bytes materialize/hash、Face/axis parse、artifact stat/read/decode、pixel copy、worker depth/age/RSS与p50/p95；每generation/identity重工作<=1。
- missing/stale artifact安全回退dynamic；TTC/variation/system font、SDF/MSDF/MTSDF、reload/cancel、checksum和current-source Cargo通过。

## 禁止临时方案

- 不得为每worker无界复制完整font/artifact并绕过总memory budget。
- 不得在主线程等待整批FDSM完成或用无限worker越过Runtime11调度预算。
- 不得去掉source/variation checksum验证来减少hash；应缓存可信identity并在generation变化精确重算。

## 修复结果与回传

2026-08-01 implementation state: `open / resolving_failure / non_validation_implementation_complete / secondary_review_complete / managed_validation_pending`。

- generation-owned `SdfGenerationSourceContext` 现在以自引用 parsed face 持有稳定 font bytes/face/variation/source hash，runtime dynamic 与 offline build 共用 batch generator；runtime 同一 source/variation 只 parse/hash 一次，offline 多 worker 输出仍按 glyph identity 稳定排序。
- runtime miss 已接入全局 `TaskPool` 的 bounded scheduler：batch/glyph/source bytes/completion depth/completion bytes 都有 admission budget，主线程只 drain/commit；reload 会 cancel。二次审查发现 completion backpressure/worker panic 会让 `pending_keys` 永久停在 `GenerationPending`，现通过单锁 active-work 对账在下一帧清理并重试，不扩张 completion pixel queue。
- source context cache 增加 64 context/128 MiB unique source bytes 上限与 LRU；离线 manifest/artifact/glyph bitmap 增加 128 manifests、32 artifact identities/128 MiB、4096 bitmaps/64 MiB 上限及 negative cache；runtime baked glyph 增加 4096 entries/64 MiB 上限。resident bytes、eviction、oldest idle age、stat/read/decode/copy、batch depth/age 均进入 bake/scheduler report。
- `SdfAtlasGlyphKey` 的 font/family/language 已硬切为 run-owned `Arc<str>`，每个 text batch 只规范化/分配一次，glyph key clone 不再逐字形深拷贝 String；未保留 String compatibility key。
- compiled atlas 只在无 pending/retry 且 exact plan 稳定时复用；`GenerationPending` / `GenerationBudgetDeferred` 明确绕过 cache，因此 cache 不会冻结 scheduler 前向进展。font generation 变化同时清空 parsed source、resident glyph/page 与 compiled artifact。
- current production owners 为 `generation_scheduler.rs` 447 行、`generation_source.rs` 219、`font_bake.rs` 724、`source_context.rs` 202、`offline_source.rs` 364、`glyph_cache.rs` 124、`prepared_atlas.rs` 108，均低于 800 行 warning；production panic/unwrap/expect/dead-code allow、旧 per-glyph source parse 和独立 failure probe 扫描为 0。

当前不标记 fixed：Text05 尚无 managed validation receipt，本轮未直接执行 Cargo；1/100/10k 的 current-source compile、p50/p95/RSS、TTC/variation/system-font/reload/cancel/checksum 组合门仍待 coordinator 后续唤醒执行。没有轮询 queued/running 状态，也没有把旧结果冒充当前验收。

## 2026-08-11 静态复核

状态维持 `open / resolving_failure / non_validation_implementation_complete / secondary_review_complete / managed_validation_pending`。

- `SdfGenerationSourceCache` 继续以 `(FontFaceId, variation_hash)` 缓存 generation-owned self-referential parsed-face context；同一 face 的 standalone bytes 与 source hash 只物化一次，context 以稳定 handle 参与动态和异步批分组。
- `SdfFontBakeCache` 在观察到共享字体 generation 变化时先 cancel 异步 work，再清理 source context、glyph、atlas、offline artifact 与派生 face cache；旧 generation 不会提交到新代。
- 本次只做源码与静态契约复核，未运行 Cargo、WGPU、截图或性能矩阵；1/100/10k、p50/p95/RSS 与组合环境门仍只接受后续协调器 managed receipt。
