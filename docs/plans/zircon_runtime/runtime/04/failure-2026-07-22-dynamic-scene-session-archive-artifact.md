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

Open state: `实现完成，等待受管验证与二次审查`; no pass is claimed.

- 已实现generation-owned immutable `RuntimeSessionArchiveArtifact`；manifest、statistics、slot index、serialized bytes共享同一payload与generation cache。
- current schema使用typed serde直读写；preview/query借用封口artifact，save写入缓存字节且拒绝旧generation覆盖同一路径的新artifact；World/Level capture不在world锁内serialize。
- 已补充同generation单次capture/normalize/validate/serialize、mutation发布新generation、stale save不覆盖newer path测试，内部JSON roundtrip计数保持0。
- Windows受管compile receipt：ticket `0e933cadb8814993821c52e5cbe70de7`，request `runtime04-dynamic-reload-archive-r5-compile-20260801-ef7edf57baec`，source manifest `a1c42abdc37f3d636c7b66b00a88b2418a178a3a20827de83e6afc4f8079a9d8`；receipt状态为`queued`，不据此声明compile/test通过。
- accepted closeout仅等待受管terminal evidence与全实现后的独立二次审查；Session继续执行其他open failure，不因验证排队停住。

### 2026-08-01 forward repair candidate

- Archive text input now rejects over-limit JSON before decode, and its public payload no longer implements `Deserialize`; only the session-private wire DTO reaches construction. Canonical current scene documents deserialize header-first into typed payloads; payload-before-header is rejected, while the migration bridge remains limited to supported legacy schemas.
- Path intent is assigned before every writer admission and synchronous save. The atomic commit rechecks that ticket against one canonical path state, so a delayed writer from another lane cannot overwrite a later writer or direct save. The regression covers two independent writers plus a later direct save.
- The archive scale guard now streams real 1/64/512 MiB write budgets without allocating a test payload, and derives 1/1k/100k manifest/index/statistics from captured entity and resource data rather than synthetic counters.
- Fresh source-bound receipts are `50eed030fffe421c8ca6cf0723edf258` for `cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1 --color never` and `78c053989d304cb6a1123954287b6bd7` for the dynamic-scene focused lib test. Both are materializing receipts only. Post-repair independent review reports `0 Critical / 0 Important / 0 Minor`; this handoff remains open solely pending terminal evidence.
