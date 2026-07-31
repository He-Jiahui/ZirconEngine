---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: dynamic-scene-compiled-spawn-transaction
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/dynamic_scene/scene
  - zircon_runtime/src/scene/dynamic_scene/value
  - zircon_runtime/src/scene/dynamic_scene/spawn_task
  - zircon_runtime/src/scene/world
tests:
  - cargo test -p zircon_runtime --lib dynamic_scene --locked --jobs 1 -- --nocapture --test-threads=1
  - preview/apply parity, failure atomicity and large reflected scene fixtures
---

# Runtime08：dynamic scene compiled spawn transaction交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime dynamic scene非session基础35/35逐Rust文件审查，PERF-MVP-472
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：Runtime08拥有entity/component/type registry、archetype与World mutation transaction；Runtime04消费prepared ticket。
- 生命周期键：`dynamic-scene-compiled-spawn-transaction`

## 失败现象与复现证据

Prepared spawn只做scene schema自检；target remap/compatibility/parent、adapter/field resolution仍在主线程。preview物化remapped values后apply全部重做。spawn先注册types/insert records，再逐field写component/resource，adapter和field metadata按component clone、field lookup O(F²)，失败可留下partial World。capture反向按entity×全TypeRegistry扫描。

## 最低共享层根因

没有绑定`{scene content, target world generation, schema generation}`的compiled mutation plan；preview report与apply各自执行解释器。Reflection暴露String field dispatch而非dense accessor，World也没有一次提交最终signatures/rows/resources的事务入口。

## 架构修复验收

- prepare构建target-generation compiled plan：dense entity remap、resolved type/adapter/field slots、remapped values、final component signatures、resource writes和preview summary共享同一authority。
- preview只借用plan summary；apply验证generation token后按budget一次commit affected rows/resources并单次发布world/query/derived generation。
- failure/cancel丢弃未发布plan，World types/entities/components/resources零partial mutation；generation mismatch显式recompile或stale error。
- reflected multi-field component一次构造/validate/write，不逐field clone整组件或线性找metadata；capture按actual storage/type generation遍历。
- entities/types/fields 1/1k/100k记录probes/clone/moves/main-worker wall：compile≤1、field O(1)、per-field whole component write=0、failure authority不变。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止preview cache和apply cache两份compiled truth；必须共享同一generation plan。
- 禁止先mutate再以补偿命令回滚；publish前authority不可见。
- 禁止缓存raw adapter引用跨schema generation而无token/ownership。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
