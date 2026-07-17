# 2026-07-17 runtime time diagnostics 静态审查

## 范围与状态

- 已逐文件读取 `frame_clock.rs`、`time.rs`、`handle/time.rs`、`handle/diagnostics.rs` 与 `diagnostics/store.rs`。
- 静态审查与低风险实现完成；Cargo、分配计数和当前源码 frame trace 尚待完成，因此该切片仍在 `pending.md`。
- 对照 Bevy `bevy_diagnostic/src/frame_time_diagnostics_plugin.rs` 的稳定 diagnostic path 与按帧数值更新模式。

## 已确认瓶颈

一次 `advance_time_by` 写入 frame count、fixed steps、frame time 和 FPS。原路径对每个指标分别：

1. 获取/释放 runtime diagnostic mutex；
2. 把静态 `&str` path 转成拥有 `String` 的 `DiagnosticPath` 后做 `BTreeMap` lookup；
3. 重新分配相同 unit 和 subsystem tags；
4. 再次扫描、去重和排序相同 tags。

因此稳定的四项时间序列每帧产生 4 次锁和多次短字符串分配；性能诊断本身会进入 CPU/allocator trace。

## 已实现优化

- `DiagnosticPath` 实现 `Borrow<str>`，静态序列可用 borrowed path lookup，命中时不重新分配 key。
- `DiagnosticStore::record_static` 首次建立 path/unit/tags；后续 metadata 相同则只更新数值、summary 和 bounded history。metadata 改变时仍回退到原语义。
- 四项 time diagnostics 在一个 diagnostic-store lock 内批量写入。
- 回归 `static_diagnostic_series_reuses_path_and_metadata_allocations` 固定第二次记录沿用 path/unit/tag backing storage。

## 剩余风险与验收

普通 `record_diagnostic` 仍允许动态 path/unit/tags，并会执行通用分配路径。Runtime07 应通过 frame trace 找出高频静态调用者，再显式迁移，而不是把动态 API 偷换成静态语义。

验收需要：聚焦单测、time 测试集、1,000,000 次静态/通用 record 分配与吞吐对照，以及产品帧 diagnostic mutex wait/allocator 样本。未完成前不声明收益。

