---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: dynamic-scene-session-archive-artifact
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/dynamic_scene/session/construction
  - zircon_runtime/src/scene/dynamic_scene/session/slot
  - zircon_runtime/src/scene/dynamic_scene/session/slot_capture
tests:
  - cargo test -p zircon_runtime --lib dynamic_scene_session --locked --jobs 1 -- --nocapture --test-threads=1
  - large archive capture and serialization allocation counters
---

# Runtime04：dynamic scene session不可变archive artifact交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：dynamic scene session核心195/563逐Rust文件审查，PERF-MVP-474
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：Runtime04拥有scene asset/import generation和prepared artifact边界；Runtime11负责artifact后的I/O lane。
- 生命周期键：`dynamic-scene-session-archive-artifact`

## 失败现象与复现证据

save深clone完整archive后normalize/sort/validate/pretty serialize；load先parse完整Value，再逐embedded scene Value→String→DynamicScene，slot scene document继续DynamicScene/String/Value往返。Level capture/diff先深clone World再构造完整DynamicScene，摘要查询也反复clone/normalize metadata。

## 最低共享层根因

session没有以project/scene/schema generation为identity的不可变capture/compiled archive artifact；DTO、验证、canonicalization、preview和serialization各自重新拥有payload。

## 架构修复验收

- 一次World/scene capture发布generation-owned immutable scene/archive artifact；summary、validation ticket、slot index和serializable view共享其payload。
- typed serde直接读写archive/scene，不用pretty String或`serde_json::Value`作为内部桥；canonical on-disk order/schema/error语义保持。
- preview/query借用summary/index；save只消费已封口artifact，stable generation不重复capture/normalize/validate。
- slots/entities/payload 1/1k/100k及1/64/512MiB记录World/archive/scene/String/Value clone bytes和各stage次数：每generation capture/normalize/validate/serialize各不超过一次，内部JSON roundtrip=0。
- Runtime11 bounded writer与Runtime08 mutation plan接入同一generation ticket；stale artifact不得覆盖newer path generation。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止只把String换成Vec<u8>而保留Value/DynamicScene重复parse和完整archive clone。
- 禁止scene、session和project save各维护一套独立cache或generation。
- 禁止在World锁内serialize、压缩或写文件。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
