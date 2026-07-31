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

Open state: `PERF-MVP-228已完成6/6静态审查与参考引擎对照；等待Text09实现indexed cache/LRU/key identity、规模counter、current-source Cargo与产品文本trace后回传`。
