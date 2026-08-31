# RichTable Track Metrics 几何权威与增量复杂度收口

日期：2026-08-28

状态：生产静态候选已实现；lower Rust 回归已写入但未执行 managed Cargo；产品性能验收待 current-source Editor 构建

## 1. 问题结论

`zircon_runtime/src/ui/text/layout_engine/rich_table` 原先对同一组 column/row extents 建立了互不一致的几何：

- `track_origins(&column_extents)` 只累计 track extent，没有加入 `column_gap`；
- `track_span_extent(..., column_gap)` 和 table total extent 会加入 gap；
- provisional cell、resolved box、final cell 与 measured table extent 因此可能从不同坐标权威计算；
- 每个 cell 的 provisional、box 和 final 阶段会重复遍历 colspan/rowspan slice 求和，复杂度随 span 总长度增长。

这同时是 correctness 和 performance 问题，不能通过缓存某一个调用点解决。正确边界应当是 sizing 完成后发布一次不可变 track metrics，所有 placement 阶段只读同一份 origin/span/total authority。

## 2. Unreal 主参考

本地参考：

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Layout/SGridPanel.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Layout/SGridPanel.h`

`SGridPanel::OnArrangeChildren` 在 arrange 前先计算最终 column/row sizes，随后对两组数组执行 `ComputePartialSums`。child placement 直接读取起点 offset，并用 `partial[end] - partial[start]` 取得 row/column span 大小。Header 还明确说明数组保留尾部 faux value，以便最后一个 cell 仍能使用相同差分公式。

可迁移原则不是复制 Unreal 容器，而是保持相同所有权：track sizing 一次形成位置产品，arrange 中不得按 child span 重复扫描 track slice。

## 3. 实现

`cell_layout.rs` 新增唯一 `TrackMetrics`：

- owns `extents: Vec<f32>`；
- 一次构建 `prefix_extent_with_gap: Vec<f32>`；
- `origin(index)`、`span_extent(start, span)`、`total_extent()` 均从同一 prefix 读取；
- empty、zero-span、越界 start/end 和超大 saturating span 都安全返回；
- 非有限/负 gap 在构建边界收敛为 0，避免传播无效坐标。

prefix 在每个 track 后累计 `extent + gap`。因此：

- `origin(i) = prefix[i]`；
- `span(start,end) = prefix[end] - prefix[start] - gap`；
- `total = prefix[len] - gap`；
- 非空 span 查询为 `O(1)`。

`layout.rs` 在 column sizing 后构建一次 `TrackMetrics::new(column_extents, column_gap)`，在 row sizing 后构建一次 `TrackMetrics::new(row_extents, 0.0)`。provisional content frame、resolved background/border box、final content frame、clip translation和 table measured extent 全部消费这两份 metrics；旧 `track_origins`、`track_span_extent` 和末尾独立 `iter().sum()` 已删除。

本次只在现有 external shaping outcome 改动之上替换几何读取，没有改写 `TextLayoutOutcome`、Deferred/Failed 传播、column shrink solver 或 source slicing。

## 4. 复杂度与内存

确定性模型：10,000 cells、256 columns、1,000 rows、平均 colspan 32、平均 rowspan 4；最坏情况按所有 cell 都有 box style，column span 在 provisional/box/final 三阶段查询，row span 在 box/final 两阶段查询。

| 指标 | 重复 span 求和 | Prefix metrics |
|---|---:|---:|
| span 查询 | 50,000 | 50,000 |
| span track visits/work | 1,040,000 | 50,000 |
| combined model work | 1,092,512 | 101,258 |
| 复杂度 | `O(C + R + cells * (colspan + rowspan))` | `O(C + R + cells)` |

模型减少 991,254 个 track work units，combined ratio 约 10.789389 倍，span work ratio 20.8 倍。模型只计算算法访问，不是产品 CPU 加速比。

旧 `extents + origins` 与新 `extents + prefix` 的 f32 payload 基本等量；默认模型只因 column/row 各多一个 prefix terminal 增加 8 bytes，不随 cell 或 span 数增长。Vec header、allocator、text shaping、clipping和 GPU 成本不在模型内。

模型产物：`E:\zircon-profiles\runtime-rich-table-track-metrics-20260828.json`

SHA-256：`EFB65DBB75C6A798B2828934CDD59CB7A44F8DBBA11ABDB96829222A04176D06`

## 5. 验证

已完成：

- TDD 静态合同先 2/2 RED，生产实现后 2/2 GREEN；
- pressure model 与参数错误合同 2/2 GREEN；
- 相邻 RichTable shrink sizing 合同 2/2 GREEN；
- 合计 focused Python 6/6 GREEN；
- `rustfmt --edition 2021` 成功解析并格式化 `cell_layout.rs` 与 `layout.rs`；
- scoped Python compile、diff/whitespace check 待最终统一执行。

已写入三条 lower Rust 回归：gap-aware origins/span/total，empty/clamped/zero/overflow span，以及同一 logical metrics 在 horizontal-tb/vertical-rl 下的物理 frame 映射。由于当前没有授权的 managed Cargo lane，本轮不运行 raw Cargo，也不把静态候选写成已编译。

## 6. 产品验收门

current-source Editor 构建可用后必须覆盖：

1. 非零 column gap 下第二列及以后 origin、单列 cell、跨列 cell、box frame 和 final text frame 一致；
2. horizontal-tb 与 vertical-rl 使用同一 logical metrics，只有 `TableAxes::physical_frame` 负责物理轴映射；
3. empty track、最后一轨、clamped span、rowspan 与 clip 语义不回归；
4. 10,000-cell 大 span 压力中，prefix build 为每表每轴一次，span query 数与 cell phase 同阶，fallback 为 0；
5. 采集 layout CPU、allocator bytes、working set/private bytes 和输入到呈现 p50/p95/p99；
6. 与修复前产品基线对比，而不是用本报告的工作量比例冒充实测加速。

只有 lower Rust、current-source Editor 产品场景和内存/时延门全部通过后，本项才能从 `static_candidate` 提升为动态完成。
