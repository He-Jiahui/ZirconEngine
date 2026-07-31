---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: dynamic-scene-session-indexed-transaction
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/dynamic_scene/session/slot_store
  - zircon_runtime/src/scene/dynamic_scene/session/query
  - zircon_runtime/src/scene/dynamic_scene/session/merge
  - zircon_runtime/src/scene/dynamic_scene/session/retention
  - zircon_runtime/src/scene/dynamic_scene/session/selected_capture
  - zircon_runtime/src/scene/dynamic_scene/session/selected_mutation
tests:
  - cargo test -p zircon_runtime --lib dynamic_scene_session --locked --jobs 1 -- --nocapture --test-threads=1
  - 100k slot merge, selection, preview and prune counters
---

# Runtime08：dynamic scene session索引化事务交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：dynamic scene session核心195/563逐Rust文件审查，PERF-MVP-476
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：Runtime08拥有scene/world identity、dense storage和原子mutation边界；Runtime04提供generation artifact。
- 生命周期键：`dynamic-scene-session-indexed-transaction`

## 失败现象与复现证据

slot/manifest查找为线性扫描，每次push/upsert/rename全量sort；selection先clone/normalize/sort完整manifest，再按id二次查找。preview重复验证所有embedded scenes并clone报告payload，commit再次查找/验证。merge逐incoming重复contains并逐项push/sort；capture+retention preview深clone整个archive，tag protection还反复构造slot-id集合并线性membership。

## 最低共享层根因

archive只有公开Vec排序约定，没有canonical slot-id index、generation validation ticket、borrowed selection handle或batch mutation plan；preview/commit只能各自重演算法。

## 架构修复验收

- 单一canonical slot storage同时提供slot-id index、stable order和updated/tag secondary index；mutation增量维护，禁止平行真相漂移。
- selection返回generation-bound borrowed handle/index及borrowed summary；不构造完整owned manifest，commit验证generation后直接命中。
- merge/import/prune/capture-retention先生成轻量mutation plan，批量查重/验证一次、sort/publish一次；失败丢弃plan，authority零变化。
- preview report只复制对外必要小字段，不clone scene/archive payload；`ensure_supported`结果以generation ticket复用。
- slots/incoming/tags 1/1k/100k记录lookup probes、validation passes、sort comparisons、payload/metadata clone bytes：lookup O(1)或O(logN)、batch sort≤1、preview payload clone=0、failure partial writes=0。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止仅把某个linear find替换为binary search，却允许公开未排序Vec绕过canonical invariant。
- 禁止为preview深clonearchive来换取rollback，复用PERF-MVP-467 affected-row transaction原则。
- 禁止每项mutation调用全量sort/validate或维护未绑定generation的旁路index。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
