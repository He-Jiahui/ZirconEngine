---
status: implemented_static
validation: pending
milestone: S15.5
updated_at: 2026-08-13
---

# Sample Grid Theme And Metrics Output Record

`WorkbenchSampleGrid` 的外框、绘图区、网格、零轴、刻度/轴文本、普通/选中样本点与选中标签，已由 `SampleGridPalette` / `SampleGridMetrics` 从共享 `HostMaterialPalette` / `HostControlMetrics` 投影。选中标签宽度继续由 Runtime Text 实际 advance 推导，不使用字符数估算；标签底面消费 neutral floating `popup`，文字消费 primary text，cyan 仅属于选中 Key 与 1px 边框。该角色划分对应 Unreal `PaintAnimationNames` 的黑色 LabelBrush/白字/独立 Key 选中色，不用大面积 selected surface 代替浮层。

对照本地 Unreal `SAnimationBlendSpaceGridWidget` 源码复核后，普通与选中样本均使用 11px `KeySize` diamond，仅以 muted/accent 色区分。参考图中的 Key 是空心菱形，因此外层 11px scanline diamond 内叠 7px `plot_surface` interior，普通/选中共用同一轮廓几何，不再用 3px 小中心点伪装空心。早期记录把只属于 filtered/preview position 的 21px `PreviewSize` 误作 selected halo 依据；该 halo、palette 角色和错误测试已删除。点位 inset 由 11px Key 半径相对计算；紧凑 plot 将 inset 收缩到中心，所有点/标签命令使用父 clip 与 plot 的交集，避免泄漏到轴沟槽。

标签水平居中并限制在 plot 内；纵向按 Unreal `PaintAnimationNames` 优先置于 Key 下方，靠底边空间不足时翻转到上方，两侧均无法容纳完整标签与 4px gap 时隐藏，不强制夹取造成文字/Key 重叠。Host paint command 当前未暴露 Runtime Text writing mode/rotation，因此本切片没有用逐字符堆叠伪造参考图的竖排 Y 轴标题。

轴文字布局继续按用户参考图收敛：top gutter 由容器高度比例和 40/48px 边界推导，X 轴标题通过 Runtime Text 实测宽度在 plot 上方居中，X ticks 位于标题与 plot 之间；只有所有相邻 tick frame 仍保留共享 4px gap 时才整组绘制，极窄 plot 会省略整组而非把标签叠在同一点。Y ticks 通过各自实测宽度共同右对齐到 plot 左缘。Y 轴标题在真正竖排接口接通前仅占用左上辅助槽，与 X 标题按可用宽度互斥；这是 Zircon Host CPU Text 尚无 writing-mode/rotation 的明确临时降级，不宣称 Unreal axis 布局复刻。极窄尺寸会省略不可容纳文本，不引入绝对屏幕坐标或字符堆叠。X 轴文字移到上方后，bottom gutter 收敛为 10/18px 边缘留白，把有限高度返还给 plot。

源码测试覆盖 palette/metrics 投影、label command 度量、selected/ordinary 两组完整 11px Key scanline 等价、紧凑 plot/parent clip 精确交集、顶/中/底标签方向和无可行空间隐藏。目标 Rust 2024 rustfmt、scoped diff、冲突标记和 owner 规模静态门通过；最终独立复核 P0/P1/P2 均无。

本记录不声明受管 Cargo 或新截图通过。当前未生成截图；后续视觉验证图只允许写入 `docs/tests/editor`，不得写入 `target`。里程碑保持 `implemented_static / validation_pending`，等待协调器 current-source 终态。
