---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: level-system-runtime-state-frame-snapshot
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/scene/render_extract
tests:
  - cargo test -p zircon_runtime --lib level_system --locked --jobs 1 -- --nocapture --test-threads=1
  - animation/physics/script/render multi-reader scale fixtures
---

# Runtime07：LevelSystem分域frame snapshot交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime scene LevelSystem/render extract 5/5逐Rust文件性能审查，PERF-MVP-469
- 修复责任计划：`docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md`
- 交接原因：Runtime07拥有frame hotpath、锁/clone计数与sealed artifact；Physics03、Plugins04和Runtime13分别消费domain payload。
- 生命周期键：`level-system-runtime-state-frame-snapshot`

## 失败现象与复现证据

`WorldRuntimeState`把physics events、animation pose/playback和script started bindings放进一把Mutex。render每frame先在锁内深clone全部pose，再持World锁完成extract；animation getter clone三张BTreeMap，physics getter clone事件Vec，script started查询每次为`&str`分配String。独立子系统和读者因此互相串行，大payload复制与主线程extract叠加。

## 最低共享层根因

LevelSystem只有mutable domain state和owned getter，没有按domain revision封存的immutable frame publication、borrowed/shared read handle或明确frame seal。组合门面同时承担写owner、snapshot owner和consumer DTO owner。

## 架构修复验收

- physics/animation/script各自拥有短写lane与revision；frame boundary发布只含generation/Arc handles的`LevelFrameStateSnapshot`，不复制正文。
- render一次短锁取snapshot handle，锁外过滤pose/skeleton并与scene generation artifact组合；多camera共享同一sealed payload。
- owner以clear/swap或reused dense storage更新，stable revision不发布/深clone；history与diagnostics借用同一handle。
- script binding state改entity→interned/borrowed key direct lookup，查询不分配String；reload/reset按domain generation原子替换。
- poses/events/bindings 0/1k/100k、threads 1/8/64记录锁wait/hold、clone bytes、builds与p95：跨域互斥=0、stable clone=0、锁hold不随payload增长。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止在现有WorldRuntimeState外再加每consumer cache；sealed artifact必须是唯一frame publication。
- 禁止把深clone移出锁后仍每camera/frame执行；验收统计端到端clone bytes和build count。
- 禁止无generation地跨帧缓存pose/event引用；replace/reload/reset必须让旧handle显式退休。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
