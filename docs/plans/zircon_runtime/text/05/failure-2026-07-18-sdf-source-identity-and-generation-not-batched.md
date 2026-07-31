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

Open state: `等待Text05联动Text01/09/Runtime11回传generation-owned source context、bounded batch generator、deterministic offline output与current-source证据`。
