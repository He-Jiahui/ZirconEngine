---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: export-overlapping-recursive-digests
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/15
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/export/stages/executor.rs
  - zircon_editor/src/core/export/pipeline.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/prepare.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/staging.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/run.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/view_model.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_projection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_host_projection.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/session_state/polling.rs
reference_sources:
  - dev/godot/editor/export/editor_export_platform.cpp
tests:
  - unchanged warm export bytes-read/hash-count regression
  - overlapping root/child artifact digest de-duplication test
  - changed/deleted/tampered source-output parity matrix
---

# Editor15：export 重叠目录递归 digest 与重复工具探测

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_editor/src/core/export` 8/8 Rust 文件逐文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- 交接原因：文件 inventory、stage fingerprint 与 resume cache 必须由 Editor15 export generation 统一拥有。

## 失败现象与复现证据

每个 pipeline stage 的 `prepare` 都重新序列化 preset、构建命令，并对 CompileHost 输入目录递归 `fs::read` 全文件计算 BLAKE3；`can_reuse` 又对 previous outputs 逐项递归重读。CompileHost 输出包含整个 `ZirconEngine`，PlatformBundle 同时记录 engine root、launcher、runtime library、assets，导致 parent/child 重叠内容在一次 warm resume 中多次读取/哈希。每次 prepare 还重复启动 python/cargo/rustc/node `--version`。

UI host 的 Cargo process runner 还把完整 stdout/stderr 累积为两个 `Vec<u8>`，完成后再各生成 owned `String` 并保存在 invocation/report。长编译日志会把 RSS 与输出字节线性绑定，并与 wizard event/report projection 的后续 clone/serialize 叠加。

native dynamic preparation 每次先删除 `.native-dynamic-staging` 与 `.native-dynamic-build`，再递归复制每个 package 的 assets/resources、plugin manifest 与已有 native artifacts。即使 source、toolchain 与输出均未改变，warm export 仍重新枚举、读取和写入整套静态文件；这部分工作也没有复用 stage digest 已经读取过的 inventory。

export wizard 对每条 stdout/stderr line 先追加到无界 stage buffer，再 clone 整份累积 `ExportWizardJobSnapshot` 作为 `StageOutput` event 并送入无界 `mpsc`。L 行输出的 clone bytes 接近 O(L²)；主线程 poll 无 count/time 配额地 drain 所有 snapshot。随后 view model 再 clone 每个 stage 的完整日志，panel state 为每行创建一个 retained node，host projection 还重建 template/surface/layout/整份 retained tree。report stdout JSON 在同一 projection 内被重组并 parse 两次。

Retained host consumer 还在每次可见 Build/Export pane slow recompute 同步枚举 export 目录、排序 preset 名、逐文件 load/parse，并为每个 preset 调用 `generate_native_aware_export_plan`。稳定面板因此把 preset 数量乘到 plugin/manifest plan 构建上。本轮已把 terminal wizard session 从永久 per-tick poll 中排除，并改为直接 `iter_mut`，只为 changed update clone profile key；manifest/preset/plan generation 仍须由 Editor15 统一持有。

Godot export 的 `FileExportCache` 持久保存 source modified time、MD5 与 saved path；mtime 未变时复用缓存，变化后才重新 MD5，并把 cache 写回 `file_cache`。Zircon 不应只信 mtime，但应采用 metadata 快筛 + content digest 确认 + generation inventory，而非每个 stage/重叠 artifact 独立全树读取。

## 最低共享层根因

每个 stage 和 artifact 独立拥有递归 digest 过程，没有 export-generation file inventory、重叠路径 DAG 或持久 digest/toolchain identity cache。

## 架构修复验收

- 每次 export generation 构建一次 canonical file inventory；同一文件内容最多读取/哈希一次，重叠 root/child artifact 由 Merkle/manifest projection 组合 digest。
- 持久 cache 至少绑定 canonical path、size、mtime/file identity、digest 与 toolchain identity；metadata 变化或不可置信时回退内容校验。
- toolchain identity 每 generation 探测一次；各 stage parameter digest 复用 immutable context。
- process output 使用有界 tail + 流式 artifact/consumer，report 默认只保留 bytes/digest/tail；完整日志不在内存多份常驻。
- native package staging 消费同一 generation inventory，按 changed/deleted/renamed delta 更新临时树；unchanged warm staging 的 copied file/byte count 接近零。
- wizard event 使用 typed delta 或共享 immutable generation snapshot；StageOutput 不携带累积日志，event ingress 有界/coalesced，主线程 poll 有 count/time budget。
- terminal panel 默认只虚拟化/显示有界 tail；完整日志走 artifact/分页 consumer。report structured summary 每 generation parse 一次，retained projection 只 patch changed rows。
- manifest/preset directory identity 与 job/output override generation 共同作为 Build/Export pane cache key；unchanged 可见 pane 的 `read_dir`、preset load/parse 与 native-aware plan build count 为 0，每个 changed preset generation 最多一次。
- unchanged warm export 的内容 bytes-read 接近零；changed/deleted/tampered cases 仍强制重跑，报告/输出 digest 与顺序不变。

## 禁止临时方案

- 不得只信 mtime/size 并放过同时间戳内容篡改。
- 不得把整个源码树或输出树读入内存后一次哈希。
- 不得为每个 stage 保存独立 file cache authority。
- 不得通过降低 poll 频率来保留无界 full-snapshot queue，也不得静默丢弃最终 status/failure/cancel record。

## 修复结果与回传

Open state: `待 Editor15 实现 generation inventory、重叠 digest 去重与持久 warm cache，并回传 bytes-read/hash-count/增量正确性`。
