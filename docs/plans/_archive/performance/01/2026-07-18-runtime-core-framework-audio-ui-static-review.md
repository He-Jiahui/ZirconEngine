---
related_code:
  - zircon_runtime/src/core/framework/audio
  - zircon_runtime/src/core/framework/ui
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_plugins/sound/runtime/src
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/02-sound.md
reference_sources:
  - dev/godot/servers/audio/audio_server.h
  - dev/godot/servers/audio/audio_server.cpp
tests:
  - three of three current framework audio Rust files reviewed
  - one of one current framework UI Rust files reviewed
  - source-guard RED to GREEN for allocation-free named validation and single-pass uniqueness
  - rustfmt and scoped git diff check passed
  - current-source Cargo and sound asset/plugin regression pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime audio与UI framework leaf逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`core/framework/audio/**`当前3/3个Rust文件（含本轮新增源码门禁）和`core/framework/ui/**`当前1/1个Rust文件，并回查SoundAsset、audio importer与sound plugin消费者。UI leaf仅是活跃Frameworks05迁移新增的`UI_MODULE_NAME`常量，不含运行时行为；本轮只读、不修改外部owner。

## PERF-MVP-336：channel layout验证重复分配与二次算法

原`is_valid_contract_layout`为判断named layout先调用`from_name`构造`String + Vec`，随后`is_canonical_named_layout`又构造一次相同owned layout；一次合法named验证产生两轮对象物化。自定义layout的`has_unique_speakers`还对每个speaker扫描后缀，最坏O(C²)，尽管契约只有8个speaker变体。

本轮按TDD先让两个source guards RED，再加入borrowed static named metadata并把唯一性改为单遍8-bit mask；named validation不再调用owned `from_name`，唯一性访问降为O(C)，公开构造、serde与错误判词不变。Godot同类speaker mode以固定enum/switch映射channel count，也不会为查询固定布局临时构造owned name/vector。rustfmt、source guards与scoped diff check通过；当前Cargo运行早于该切片，不能记为本修复动态通过。

## 验收要求

按named/custom、speakers 0/1/8/1k/65k、valid/early-duplicate/late-duplicate执行1/1k/1M次验证，记录String/Vec alloc、speaker comparisons、CPU p50/p95/p99：named validation alloc=0，valid custom comparisons=C，duplicate不超过首次重复位置，算法不得恢复suffix contains。补跑framework tests、SoundAsset WAV named/side layout、audio importer与sound plugin config/device mapping；serde/constructor/name/channel/speaker parity、current-source Cargo与产品插件加载trace全部完成前，audio/UI leaf留在`pending.md`。
