---
related_code:
  - zircon_runtime/src/text/font_sdf_build_tool
  - zircon_runtime/src/text/atlas/shelf_allocator.rs
  - zircon_runtime/src/text/sdf/fdsm_gen.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
reference_sources:
  - dev/Fyrox/fyrox-ui/src/font/mod.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCache.cpp
tests:
  - shelf allocation failure and earlier-page reuse source-level RED to GREEN guards passed
  - Rust 1.94.1 module-wrapper shelf and pack tests 4/4 passed
  - rustfmt check and scoped diff check passed
  - current-source Windows font-sdf-build-tool Cargo tests pending
  - full-cmap CPU/page/RSS benchmark pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime text font SDF build tool逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/text/font_sdf_build_tool`当前源6/6个Rust文件已逐文件阅读，覆盖font decode/standalone face、cmap selection、glyph mapping、SDF/MSDF/MTSDF generation、atlas packing、artifact encode/inspect、typed request/error。共享`atlas/shelf_allocator.rs`按实际调用图复审。

## PERF-MVP-248：offline pack只尝试最后一页

原packer对每个glyph只调用`allocators.last_mut()`；最后页放不下时即使较早页仍有空间也新建完整page。共享shelf allocator的失败尝试还会先推进cursor/shelf再返回None，使后续更小glyph不能复用原shelf空隙。离线全cmap遇到大小混合glyph时会增加page count、artifact bytes、encode I/O和GPU residency。

本轮先加入“失败allocation保持current shelf”和“较早page可复用时不新建page”两个测试，再让allocator只在成功后提交cursor，并让packer扫描所有已有页。源码RED确认旧`last_mut`和失败前mutation存在，GREEN确认existing-page scan与transactional state；Rust 1.94.1 module-wrapper 4/4、rustfmt/diff通过。Fyrox font atlas同样遍历已有page的rect packer，只有全部失败才新建page。

## PERF-MVP-250回链：全cmap generation没有batch owner

build tool先用BTreeSet/BTreeMap去重codepoint/glyph，然后在单线程for-loop逐glyph调用`generate_distance_field_glyph`。该API每glyph重新parse `ttf_parser::Face`、应用variation、构建outline/shape/image并执行FDSM error/sign correction；相同face metadata不能复用，也没有TaskPool、并发配额、取消、进度或峰值内存预算。该根因与runtime动态SDF/offline source共享，统一交接Text05并联动Text01/09/Runtime11，不为工具另造平行实现。

## 验收

Text05收到`failure-2026-07-18-sdf-source-identity-and-generation-not-batched.md`。当前直接packing修复需通过feature `font-sdf-build-tool`聚焦Cargo测试；随后用Fira/variable/CJK全cmap记录glyph count、Face parse、worker并发、FDSM CPU p50/p95、peak RSS、page count/occupancy、artifact bytes与encode I/O。输出必须byte-deterministic，glyph顺序、page index、checksum、TTC/variation/SDF/MSDF/MTSDF parity不变。动态与规模完成前6/6仍留pending。
