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

### 2026-09-08 SDF Test Consumer Repair

Stable fixing Session `failure-roll-01a07160-text05`, baseline `601`, continues
this lifecycle as open. Registration request `638f98bb48a1479bb0c9c4df24b8fc77`
retains both existing Text05 failure keys. Managed graphics library jobs
`3db36b30be88485bb06b0a5ff2c6d3e4` and `94560adfdb1a45daa7e2d5785ae6677c`
reported 90 and 67 compiler errors respectively, with zero executed tests.
The latter still includes eleven SDF test-consumer errors; it predates this fix.

The three files below were HEAD-identical before ownership transfer
`19cb4590fbfc4f81bf49a822144173c9`; pre-edit snapshot `3070` preserves their bytes
and archived attribution history. Source `3071`, request
`dcb7a037949d441180e6d25ae64e6479`, freezes:

| Path under `zircon_runtime/src/text/sdf/font_bake/` | SHA-256 |
| --- | --- |
| `tests.rs` | `67914736da037c1275f8b8e6ded1c976eb97d131242953dd8fcca1fc9aef050e` |
| `tests/offline.rs` | `96f56df3e0cda137f29af8a2cfc576425450e2e7d18d04f86a5a5a5707249a3a` |
| `tests/owner_recovery.rs` | `055700a47b27ac1c3722a36b62a256b94ff5794779a7b4ffab02412fcbdccfc4` |

The parent imports the existing shared generation helper for its children and
moves the misplaced observed-generation fixture into the cache-eviction test.
The default resolver uses the existing packaged Runtime font fixture, preserving
the primary-face result, unchanged face count and absent URI-owner mapping.
No private setter is reopened. Offline negative caching now checks the exact
`ProjectAssetUnavailable` error and unchanged parse count across a project
rescan, then requires successful loading after the generation changes. The
unshaped CJK recovery test calls the current lookup-only resolver and retains
its owner-composite face assertions. Production SDF and font admission are
unchanged. Rust 2021 formatting and scoped whitespace checks pass.

The next immutable input derives from `runtime-graphics-text-test-support-3067-20260908`
(digest `c93b37d1c23413b5f1ff16f4705abfbf56ec2511dfb45f89a1b1c85500a839da`)
and overlays only `3071`. Actual SDF tests, scale/worker measurements, independent
review and the original upward acceptance remain required; no fixed return,
commit or WeCom delivery is claimed by this source snapshot.

### Review And Offline Integration Result

The existing task `优化协调器验证效率` reviewed all three `3071` files with
Critical 0, Important 0, Moderate 0. Report:
`.codex/tmp/text-framework-3080-review-20260908-result.txt`.
Snapshot, ObjectStore and current attribution hashes matched before and after
review. Packaged-face identity, exact negative-cache error, generation refresh
and composite-owner recovery assertions remain intact.

Managed job `9e3f8ac2a6564bd48fbeab45874342e3` compiled the Text-only `3073`
input with 22 errors and zero tests. None belongs to the repaired SDF files.
The later input `runtime-text-owner-test-support-3095-20260908` has digest
`4e5f800cd19e5748464ac759b4e1efbfae073ecf1c05d419327a6d6739cbe9e2`.
It preserves `3071` and includes separately attributed Text02/04/07/09 support.
Job `1f242fb1789e44ce87d7a5cacc569d5f` reduced the same Text-only library
configuration to three compiler errors, still zero tests.

Windows managed job `76720416fcfe47a5a76c9e6f8af604f6` then ran this exact
input with `--no-default-features --features font-sdf-build-tool --locked
--test runtime_text_sdf_offline_artifact text_sdf_offline_`, static linkage.
Both existing tests passed: deterministic/decodable offline output for
SDF/MSDF/MTSDF and rejection of a corrupt checksum. No tests were filtered out
or ignored. The 10,955-file source manifest and 354 dependency packages were
verified; queue/sync/check/compile-link/test stages took
15.186/27.573/129.128/183.097/7.258 seconds. Durable evidence:
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime-text-owner-test-support-3095-20260908/results/sdf-offline-integration.json`
and the adjacent `.log`.

This is two-test offline acceptance only. The library unit tests, original
1/100/10k glyph measurements, bounded-worker/reload/cancel matrix and upward
product acceptance remain pending. The operational receipt has not been
relabeled as a formal fixing-Session Cargo run. This lifecycle stays open.

## 2026-09-25 current-source rolling reconciliation (SDF source generation r1)

Successor Session `failure-roll-01a084c8-text05-source-generation-r1` claimed this failure
record, the Text05 plan, and the four exact source owners. No Rust source was edited in this
continuation. The current source probe completed with marker
`TEXT05_SDF_SOURCE_GENERATION_CURRENT_SOURCE_PASS 8 of 8`: the offline build owner creates a
generation-owned `SdfGenerationSourceContext`, borrows one parsed face, submits one
`generate_batch_with_pool` operation, and records batch/duplicate/worker counters; glyphs are
mapped through an ordered `BTreeMap`; offline loading carries source/variation hashes and the
bounded bitmap cache reports evictions; FDSM generation also exposes a face-based path instead
of requiring each caller to reparse a glyph source.

The claimed current hashes are:

| Path | SHA-256 |
|---|---|
| `zircon_runtime/src/text/sdf/fdsm_gen.rs` | `cdb1465ddd1cfdfe72d0068c45ce422ac782a38156e890dc022921cddc8db7a4` |
| `zircon_runtime/src/text/sdf/font_bake/distance_field.rs` | `5df961f14c52ed67217c53605ecab9626e159879e7176d666e2477c2b9842721` |
| `zircon_runtime/src/text/sdf/font_bake/offline_source.rs` | `4c003f053e76a7d6f04839f45fdedb0d622e77729c43e08af790e437743ccec5` |
| `zircon_runtime/src/text/font_sdf_build_tool/bake.rs` | `d81aae8b2f65502e6bb14a45c278097a660a9954b0988d77ca2c64fd10ade9d5` |

All four source paths were already dirty current-source overlays before r1 and retain their
existing attribution; r1 did not absorb or rewrite them. The failure document was also dirty
from its historical receipts, while the Text05 plan remained clean. Scoped `git diff --check`
is clean. The authoritative current-source manifest is coordinator snapshot `3847`.

This is static handoff evidence only. Managed Text05 Cargo, 1/100/10k scale and p50/p95/RSS
measurements, bounded worker/reload/cancel/device-loss/checksum gates, WGPU/RenderDoc/product
pixels, independent review, canonical fixed return, closeout SHA, and WeCom result remain
pending; the historical offline two-test receipt is not reused as whole-lifecycle acceptance.

Independent review receipt (2026-09-25): the reviewer rechecked coordinator snapshot `3847`
and confirmed all four source hashes, the
`TEXT05_SDF_SOURCE_GENERATION_CURRENT_SOURCE_PASS 8 of 8` marker, source-context/face lease/
batch/report contracts, ordered glyph mapping, source/variation hashes, bounded eviction
counters, face-based FDSM path, dirty attribution, and clean scoped diff check. Review result
is Critical `0`, Important `0`, Moderate `0`. This receipt is carried in post-review
coordinator snapshot `3848`.
