---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: text-cache-linear-lookup-and-eviction
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/cache
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/surface/surface.rs
---

# Text cache线性查找与逐出放大

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/cache/**`当前源6/6 Rust文件
- 修复责任计划：`docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md`
- 责任切片：PF-M1/PF-M4。
- 交接原因：缓存数据结构、整形复用、计数预算与文本规模验收由Text09拥有；performance audit不在owner活跃改动期间并发重写四类生产缓存。

## 失败现象与复现证据

`TextFrameDedup`、`TextMeasureCache`、`TextLayoutCache`与`ShapedRunCache`都以`Vec`保存entries。默认persistent容量为4096/2048/1024项（shaped另有8 MiB上限）；get/contains/update线性扫描，trim对每个逐出项全表找oldest再`Vec::remove`。`ShapedRunCacheKey::from_request`还在每次请求分配normalized font family/language `String`。这些cache已接入`UiTextMeasureCache`、`UiSurface`、layout session与parallel shape pool，是F4编辑器文本生产路径。

现有容量2/4的单测能保护语义，却不能揭示resident entries增长后的probe、entry move和key allocation。当前受管Cargo在执行测试前被资产模块编译错误阻塞，故动态/规模结果仍为pending；静态证据见`docs/plans/performance/01/2026-07-18-text-cache-static-review.md`。

## 最低共享层根因

统一缓存契约只定义了key、容量、帧戳、碰撞校验与LRU语义，没有定义lookup/eviction复杂度。实现把LRU元数据与entry identity都绑定到可移动Vec下标，导致正常命中O(N)，逐出又叠加O(N) oldest scan与O(N)搬移。shaped cache的exact-text collision与Auto/Mixed direction alias需要候选bucket内二次比较，但不要求扫描全表。

## 架构修复验收

- 四类缓存建立hash bucket索引；persistent cache用stable slot/arena保存entry，frame dedup可按帧整体清空。exact text碰撞校验、layout width validity range与Auto/Mixed direction alias必须保留。
- LRU touch为O(1)，逐出为O(1)或带stale-generation queue的amortized O(1)；禁止在容量边缘对每个逐出项全表`min_by_key`并移动尾部entries。
- shaped key的font family/language改用shared/interned identity，或由style/font generation预计算；稳定热查询owned key bytes=0。不得只换更快hasher而继续每请求分配String。
- report增加或测试侧暴露lookup probes/visited entries、eviction scans/moved entries与key allocation bytes；时间只作观测，确定性计数作为gate。
- 规模矩阵至少覆盖16/256/1024/2048/4096 entries的hit/miss/update/evict；稳定命中average probes近常数，insert/evict不随resident N扫描/搬移。
- 现有collision、direction alias、width interval、同帧dedup、双上限LRU、frame report测试全部等价；补`render_perf_text_scroll_list_reuses_cache`与典型workbench/Console产品trace。

## 禁止临时方案

- 不得用裸`text_hash`直接取值或删除exact text比较，制造可控碰撞错误。
- 不得为降低实现复杂度删除Auto/Mixed方向复用、有效宽度区间或双容量上限。
- 不得在cache外增加全局mutex/后台队列掩盖线性算法；UI主线程lookup本身必须有界。
- 不得仅降低默认容量来压低扫描时间；这会提高昂贵shape/layout miss并破坏Text09命中率目标。

## 修复结果与回传

2026-08-01 implementation state: `open / resolving_failure / non_validation_implementation_complete / managed_validation_pending`。

- `TextFrameDedup`、`TextMeasureCache`、`TextLayoutCache` 与 `ShapedRunCache` 已共用 stable-slot hash index；persistent caches 由 linked LRU 以常数次 link 更新完成 touch 和 oldest eviction，不再扫描全表或移动尾部 entries。frame dedup 保留整帧 clear 语义。
- measure/layout 的 exact key 与 width validity range 保持不变；shaped cache 保留 exact source collision compare、canonical feature slice 与 Auto/Mixed direction alias bucket，hash 只负责缩小候选集，不替代语义比较。
- shaped hot lookup 直接借用 request/style 的 family、language 与 canonical feature slice；只有 miss 物化新 entry 时才分配 owned family、normalized language 和 feature `Arc`，稳定 hit 的 owned key bytes 为 0。
- report 已暴露 lookup probes、visited entries、LRU touch、eviction scan/move、owned key bytes、entry/byte eviction。16/256/1024/2048/4096 resident entries 的 hit/miss/update/evict 确定性测试已落代码，并要求 lookup probes 保持 bucket-local、eviction scan/move 为 0。
- 当前只声明非验收实现与静态调用链核验完成；managed current-source Cargo、ignored timing exporter、`render_perf_text_scroll_list_reuses_cache` 产品 trace 和典型 workbench/Console 证据仍待 coordinator wakeup，成功回执前保持 open。

2026-08-10 forward-review correction state:
`open / resolving_failure / implementation_complete / second_review_complete / managed_validation_pending`。

- `IndexedTextCache` 的实际 oldest eviction 现在返回 `TextCacheEvictionWork`：linked LRU head
  每次只检查 1 个 victim candidate，stable entry move 为 0。measure/layout/shaped report 从每次
  实际逐出结果累计 `eviction_scan_count` 与 `entry_move_count`，不再依赖从未更新的默认零字段。
- 16/256/1024/2048/4096 resident matrix 在每档都执行 exact hit、absent miss、exact update 与
  capacity eviction；frame dedup 同档验证 hit/miss/update 后按帧整体 clear。三个 persistent cache
  均要求 lookup candidates 保持 bucket-local、每次逐出只检查一个 LRU head、stable entry move 为 0。
- frame/measure/layout 的闲置 `get_or_insert_with` helper 改为 borrowed lookup，只有 miss closure
  才物化 `Arc<str>`。生产 UI 的跨帧 measure/layout persistent hit 同时复用 cache entry 中已有的
  `Arc<str>` 给 frame dedup，避免在确认 persistent hit 前复制文本。
- scoped Rustfmt parse/check、`git diff --check` 与文件预算检查已通过；本段尚未取得 managed Cargo
  或产品 trace，因此不把 failure 标为 fixed，也不主张任何 wall-time、功耗或截图结果。
- 独立只读二次审查基于当前工作树复核了 linked-LRU victim work、五档 hit/miss/update/evict
  matrix、borrowed helper 与生产 UI `Arc<str>` 复用链路，未发现 P0/P1/P2；该结论不替代
  coordinator 管理的 Cargo、产品 trace 或 framebuffer 验证。

2026-08-11 forward-review correction: `rich_cache.rs` has a separate hard-admission gap. New
`RichTextArtifactCell`s are inserted through `IndexedTextCache::insert_untracked`, so they are
intentionally absent from the completed-entry LRU until `record_compiled`; however
`lookup_or_insert` admits every unique in-flight markup before checking either entry or byte
budget. A concurrent burst can therefore retain arbitrary pending markup cells, while the
existing capacity-one regression blesses that behavior by requiring the first pending cell to
remain indexed after a second unique pending request. This is P1 cache-boundary debt, not a
reason to evict a pending single-flight cell or roll back the stable-slot cache.

- Required forward fix: before indexing a new unique cell, evict only completed LRU entries to
  reserve its initial markup bytes and entry slot. If that cannot fit because only pending cells
  remain or the markup alone exceeds the budget, return a nonresident cell to the current caller
  without inserting it. Exact same-key requests still share an admitted pending cell; rejected
  keys deliberately do not create a second unbounded pending registry.
- `record_compiled` must reserve the final artifact byte delta before touching the cell. If even
  an otherwise empty cache cannot contain the completed artifact, remove the cache entry while
  retaining the caller's `Arc`; no cache report may exceed either configured bound. Add explicit
  admission-bypass telemetry to the cache report/frame sampler.
- Test contracts: capacity one with two unique pending cells keeps the first same-key pointer but
  reports one resident entry and one bypass; a cell whose compiled artifact exceeds `max_bytes`
  remains usable by its caller yet leaves zero resident entries/bytes. Managed Cargo/WGPU remain
  pending and this record stays `open`.

2026-08-11 forward-repair implementation state:
`open / resolving_failure / non_validation_implementation_complete / second_review_complete /
managed_validation_pending`.

- `CompiledRichTextCache::reserve_for` now reserves both the pending entry and initial markup
  bytes before `insert_untracked`. It evicts only completed linked-LRU entries; if pending cells
  or the request's own size prevent admission, the caller receives a detached cell and
  `admission_bypass_count` increases without changing resident totals.
- Compilation now produces a caller-local `Arc<CompiledRichText>`, reserves or removes its cache
  entry using the final estimated bytes, and only then publishes through `OnceLock`. This closes
  the former publish-before-accounting interval. A successfully accounted cell alone enters the
  completed LRU; an oversized result remains usable by the active caller but is not retained.
- Oversized markup/final artifacts fail fast before evicting healthy completed entries. Five
  deterministic regressions cover pending entry pressure, initial byte pressure, final byte
  pressure, and healthy-LRU preservation; the frame sampler also exposes bypass deltas.
- A follow-up boundary review found that incremental eviction could still remove completed LRU
  entries before discovering that non-evictable pending bytes made admission impossible. The
  cache now maintains completed resident entry/byte aggregates and performs an O(1) feasibility
  preflight before mutating the LRU. A mixed completed-plus-pending regression requires failed
  admission to leave both the eviction count and the retained completed cell unchanged.
- The same review found that `CompiledRichText::estimated_bytes()` omitted the owned family string
  carried by inline icon runs. Style and icon families now contribute their allocated string
  capacity, and an equal-structure 4 KiB icon-family regression requires the estimate delta to
  cover those bytes before cache admission.
- Exact-file repository-edition `rustfmt --edition 2021 --check`, scoped `git diff --check`, and a workspace tracked
  Rust literal scan pass. No Cargo, WGPU, performance trace, screenshot, or acceptance claim was
  produced in this non-validation slice; the failure remains `open`.
- A final source review verified that pending cells are hash-indexed but deliberately absent from
  the linked LRU, completed cells enter recency only after byte accounting, compilation never holds
  the global cache mutex, and failed admission cannot mutate healthy completed residency. The
  parser-generation key and exact markup collision check remain unchanged. No actionable P0/P1/P2
  remains in this repair; managed type/behavior/scale validation is still pending.
