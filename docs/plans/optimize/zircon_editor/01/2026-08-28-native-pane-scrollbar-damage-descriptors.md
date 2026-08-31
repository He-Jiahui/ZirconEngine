# Native pane scrollbar generation descriptors 与 damage 准备裁剪

日期：2026-08-28

状态：`production_static_candidate`；生产迁移、源码合同与压力模型通过；lower Rust 回归已写入；managed Cargo 与动态验证待执行

## 1. 已确认问题

Assets 与 Asset Browser 每次 pane paint 都分别调用 tree、content、References、Used By
滚动条路径。每条路径独立读取 typed metadata、平移 viewport、读取主题并计算 track/thumb；primitive
层才应用 clip。`draw_pane` 只在整个 pane 与 damage 不相交时返回，而 native content 接收到的仍是
pane content clip，因此 damage 只命中一个子视口时，其余滚动条仍完成准备工作。

这不是全模型扫描问题：M2 已把 tree count、content extent 和 reference group count 收敛到
`AssetContentPaintMetadata`。当前问题是同一代际 authority 被多个 paint route 重复消费，且 damage
裁剪发生得太晚。

## 2. 参考引擎约束

本地 Unreal Slate 源码显示：`SScrollBar` 保留 track/thumb 子项，`SetState` 只在 offset/fraction
变化时更新；`SScrollBox` 把 culling rect 传入 panel paint；FastUpdate widget proxy 只对明确的
repaint/volatile 标志重建 paint。Slint 与 Fyrox 的 scroll view 同样把 max/page/value 作为少量状态，
尺寸或 value 变化才使布局失效。

可迁移的约束是：pane generation 拥有 typed scrollbar descriptors；damage 在样式与几何准备前
筛选；scroll/hover 只作为动态 thumb 状态。不可直接迁移 Unreal 的对象生命周期或缓存容器。

## 3. M3a 实施结构

1. `AssetContentPaintMetadata` 在固定容量 4 的内联存储中生成样式无关 typed kind 索引；
   viewport 与 extent/count 从该 metadata 的既有字段和 row groups 解析，不复制第二份 authority，
   generation 也不为该索引单独分配堆内存。
2. pane paint 对每个资产 surface 只读取一次 metadata，并遍历其有界描述符切片；不分叉成 4 个
   独立 metadata route。
3. native pane 入口先求 `pane clip ∩ frame paint damage`；空交集在行选择和描述符读取前返回。
   pane layers 向上报告的是 O(1) 可判定的“逻辑内容是否存在”，不是“本次 damage 是否画到像素”；
   因此局部重绘不会误触发 fallback，完整空 pane 仍会显示 `No actors` / `No assets`。
4. 单个滚动条在读取主题、计算 track/thumb 前先做 viewport/damage 相交测试。
5. 主题度量不进入 workbench metadata。最终像素 track/thumb 仍由 host paint owner 计算，避免第二份
   跨层缓存及 style generation 失配。

当前实现已按上述结构迁移：两个资产 surface 都调用统一描述符入口；每次 paint 只读取一次
metadata；descriptor viewport 与 effective frame damage 相交后，才读取动态 interaction、换算
content extent、读取主题并计算几何。通用 `draw_vertical_scrollbar` 自身保留第二道 damage/fit
防线，Hierarchy 和其他调用者同样受益。

M3a 不宣称完成 retained track command。该工作必须与后续 shared prepared render list 的
style/layout generation 一起完成，不能在 painter 内临时再造缓存。

## 4. 复杂度与验证计划

描述符数 `K <= 4`，damage 命中数为 `I`。生成期发布为 O(K)；paint 选择为 O(K) cheap rect
probes + O(I) style/geometry preparation。默认模型取 4,000 次 pane paint、每 pane 4 个描述符、
每次 damage 命中 1 个描述符、2 次 metadata generation：metadata lookup 从 16,000 降为 4,000，
style read 与 geometry evaluation 从 16,000 降为 4,000。该 4x 是确定性结构计数，不是 CPU、
RSS、延迟或 GPU 结论；描述符索引本身为内联容量 4，结构上新增 0 次 heap allocation。

测试阶段包括：描述符生成/顺序/extent lower tests，effective clip 与单子视口像素拒绝回归，静态
hot-path guard，压力模型；受管通道恢复后再执行 focused Rust tests 与真实产品 workload，采集
main-thread CPU、allocation/RSS、input-to-damage、input-to-present p50/p95/p99/max，并核对像素。

动态验证通过前，本切片保持候选状态，不提交为完成里程碑。

## 5. 静态证据

- M1-M3a 合并 Python 合同：24/24 GREEN；
- M3a 源码合同先 4/4 RED，生产迁移后 4/4 GREEN；
- touched Rust `rustfmt --check --config skip_children=true,reorder_imports=false` GREEN；
- scoped `git diff --check` GREEN；
- lower Rust 回归覆盖 descriptor kind/order/extent、effective native clip、damage 空交集仍维持
  native-pane handled 合同、单子视口 damage 像素拒绝、独立 References/Used By scroll state；
  尚未执行，不能声明通过；
- 独立代码复审发现的 empty-state/fallback Important 已按逻辑内容存在性修复并补 lower 回归；
  最终增量复审无 Critical/Important/Minor；唯一 pressure 参数边界 Minor 已修复并覆盖；
- 未启动 raw Cargo，未使用历史 editor 二进制。

压力工件：`E:\zircon-profiles\editor-native-pane-scrollbar-damage-20260828.json`

SHA-256：`6E5412A867CB26BC06611A503CCFF77C28C4BF99B41424E01B086A940D93C9C6`

source HEAD：`a2d8d811c4a3a1fc1db6f5375c491e7e4502533f`，另有本报告对应的明确 working-tree candidate。
