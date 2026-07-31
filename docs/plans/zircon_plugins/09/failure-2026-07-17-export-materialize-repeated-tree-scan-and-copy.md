---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: export-materialize-repeated-tree-scan-and-copy
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/09-export-publishing.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/export_build_plan/materialize/generated.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/native.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/package_lookup.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/copy.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/archive.rs
tests:
  - 1/100/1000 package tree enumeration and manifest-parse benchmark
  - unchanged incremental materialization write-count test
  - deterministic bounded-parallel copy parity test
---

# Plugins09：export materialize 重复树扫描与无条件串行复制

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：export materialize 8/8 Rust 文件逐文件静态审查
- 修复责任计划：`docs/plans/zircon_plugins/09-export-publishing.md`
- 交接原因：package discovery、file manifest、incremental copy 与 archive 都必须消费同一 export generation projection。

## 失败现象与复现证据

`materialize_native_dynamic_packages` 对每个 selected package 调用 `find_native_package_dir`。direct child 未命中时，
该 helper 从 plugin root 递归遍历整棵目录，并为遇到的 `plugin.toml` 逐个读取/TOML parse，直到找到当前 id；
下一个 package 再从 root 重来。P 个 package、T 个树节点接近 O(P×T)。preview 与 zip materialize 各自重复同类查找。

生成文件 materialize 对全部 rows 串行 `create_dir_all + fs::write`，即使内容未变也覆盖；native resource/artifact
copy 同样串行覆盖全部文件，没有 generation file manifest、size/mtime/hash fast path 或有界 I/O 调度。
`native_dynamic_package_export` 还为每个 package 线性搜索 export rows。大 package assets 会同时放大目录 syscall、
manifest parse、写放大和单线程导出时间。

本次审计已把 ZIP package entry 从 `fs::read` 整文件 Vec 改成 `File + std::io::copy` 流式压缩，消除单文件
大小级内存峰值；其余 tree/index/incremental/parallel 问题需要共享设计。

## 最低共享层根因

Export plan 只保存逻辑 package ids 与 generated contents，没有冻结的 package-id→root、relative-file inventory 和
content fingerprint projection。materialize、preview 与 archive 因而各自重新枚举、解析和复制。

## 架构修复验收

- 单次 export generation 枚举 plugin root 一次，建立 package id→canonical real directory 和 native/resource file inventory。
- materialize、preview、zip 复用同一不可变 inventory；symlink policy、首次出现顺序与 duplicate diagnostics 不变。
- unchanged generated/native files 的实际 write/copy count 为 0；变更文件精确失效，删除的 stale output 有显式策略。
- I/O copy 使用有界 worker/队列，报告输出按逻辑 path 确定排序；错误取消不留下“成功”报告。
- 1/100/1000 packages 与 1 KiB/1 MiB/1 GiB payload 记录 enumerate/stat/read/parse/write bytes、wall time、peak RSS。
- ZIP 内容、timestamp/permission determinism、diagnostics 与现有 serial baseline byte-equivalent（压缩字节允许由明确版本契约决定）。

## 禁止临时方案

- 不得为 materialize/preview/archive 各建一份 package cache。
- 不得无界并行 `fs::copy` 或牺牲 deterministic report/order。
- 不得只依赖 mtime 判定内容相同；需要 size/mtime fast path 加可信 fingerprint/manifest 契约。

## 修复结果与回传

Current owner note（2026-07-22）：PERF-MVP-547已完成局部止损：`NativePackageInventory`对一轮materialize只建一次package index，跳过已解析direct package payload并在全部selection解析后停止余树；ZIP写入把preview diagnostics与ordered file entries合并为一次package walk；native/ZIP一次建borrowed export-row index且不clone ABI row；generated writer复用parent mkdir。该改动不等于关闭：preview/materialize/ZIP仍各自建inventory，unchanged file仍覆盖，copy仍串行且无Runtime11统一预算/失败commit。规模counter、Cargo与产品导出未验收，failure保持open。

Open state: `待 Plugins09 建立单次 package/file projection、增量 materialize 与有界 I/O 基准`。
