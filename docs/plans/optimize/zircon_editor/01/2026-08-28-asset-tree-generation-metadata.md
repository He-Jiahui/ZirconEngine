# Asset 树 generation 元数据与 paint 热路收敛

日期：2026-08-28

状态：`production_static_candidate`；源码合同与压力模型通过；lower Rust 回归已写入；managed Cargo 与产品 profile 待执行

## 1. 问题与边界

Activity 与 Browser 的 Asset tree scrollbar 在每次 pane paint 中遍历完整
`ModelRc<TemplatePaneNodeData>`，通过 control-id 叶名重新计算行数。Activity hover overlay 又从头
扫描到第 k 个匹配行以恢复 frame。布局 generation 已经构建并附加
`AssetContentPaintMetadata`，因此这些扫描是重复派生，而不是必要的可见行工作。

本切片只收敛 tree count 与 Activity hover frame authority：

- 不改变 content/reference virtualization；
- 不改变 Activity tree 当前作为 fixed paint rows 的行为；
- 不建立 renderer 私有的第二份缓存；
- Browser 直接复用既有 `browser_source_tree_groups`；
- 不把静态模型结果表述为动态 CPU、内存、延迟或 GPU 证据。

## 2. 目标数据流

`describe_asset_content_row` 在 generation pass 中一次性分类 control id。Activity tree panel 发布
`ActivityTreeRow` descriptor；`AssetContentPaintMetadata` 按 source order 保存其稳定 template
row address。Browser 的 logical row count 继续由 source-tree group 数量提供。

paint 消费路径变为：

1. pane model 读取同一份 typed metadata；
2. scrollbar 通过 surface-aware `asset_tree_row_count()` O(1) 取行数；
3. Activity hover 通过 `activity_tree_node_row(index)` O(1) 定位当前 model row；
4. 从 live row 读取 frame，再做 body origin 与 scroll 的标量变换；contiguous model 为 O(1)，
   persistent row overlay 最坏为 O(log N)；
5. paint 路径不再解析 control id，也不遍历 template nodes。

这与本地 Unreal 参考中的 retained item source 约束一致：Scene Outliner、Content Browser 和
Slate list view 从 source-owned items/index 生成可见行，paint/interaction 不回扫 widget/template
全集恢复逻辑身份。参考源见同系列主审查：
`docs/plans/performance/01/2026-08-22-editor-native-pane-viewport-metadata-scrollbar-architecture-review.md`。

## 3. 实施结果

- `controls.rs` 统一拥有 Activity tree row control id；
- `identity.rs` 发布 `ActivityTreeRow`，只在 generation 解析叶名；
- `paint_metadata.rs` 发布 Activity row addresses，并以 Browser group count 作为 Browser authority；
- `scrollbar/asset.rs` 删除完整模型 filter 与 suffix matcher；
- `scrollbar.rs` 删除 Activity/Browser row-control 参数和重复常量；
- `assets/frame.rs` 删除 kth-row 全模型扫描；
- hover overlay 两层删除 `row_control_id` 透传。

Activity tree descriptor 在本切片中仍落入 `fixed_node_rows`。下层回归显式保护这一点，避免在
tree virtualization 尚未成为独立 owner 前改变 paint selection。

metadata 不缓存 Activity rect。Composition cache 允许同 generation 的 frame row patch 并复用
metadata，因此缓存 rect 会在 resize/layout patch 后陈旧；稳定 row address 让定位保持 O(1)，并
始终从当前 overlay row 读取 live geometry。`PersistentRowPatchMap` 是按 model index width 的 trie，
所以 overlay row 解析最坏为 O(log N)，而不是错误宣称严格 O(1)。

该 O(log N) 合同已明确接受：严格缓存 rect 会破坏 geometry patch 正确性；为维持 rect 正确而让
每个 Asset frame patch 强制 O(N) metadata rebuild，会倒退窗口 resize 的增量路径；为通用
`ModelRc` 增加 dense O(1) patch lookup 则会扩大共享存储的内存与更新范围。10,000 rows 的最坏
15 次 trie node visits 是有界 indexed work，核心验收门是 paint/hover 不再扫描 N 个 template nodes。

## 4. 复杂度证据

默认确定性模型：10,000 template nodes；Activity/Browser 各 2,000 次 pane paint；Activity
hover paint 1,000 次；metadata generation 1 次。

| 结构工作 | 旧路径 | 目标路径 |
| --- | ---: | ---: |
| Activity count node visits | 20,000,000 | 0（query O(1)） |
| Browser count node visits | 20,000,000 | 0（query O(1)） |
| Activity hover node visits | 10,000,000 | 0（indexed query） |
| generation node visits | 0（未计既有 generation） | 10,000 |
| count/hover-index queries | 隐含在扫描中 | 5,000 |
| worst-case overlay trie node visits | 隐含在扫描中 | 15,000 |
| combined structural work units | 50,000,000 | 30,000 |

结构工作比例为 1666.6667x，消除 49,970,000 个模型工作单元。该模型按 10,000 rows 的
14-level overlay trie 保守计入 14 个 branch 与 1 个 leaf；contiguous model 实际没有该遍历。模型没有估算 allocator、
layout、render、GPU、RSS、input-to-present 或 wall-clock latency。

工具：`tools/editor_asset_tree_metadata_pressure.py`

工件：`E:\zircon-profiles\editor-asset-tree-metadata-scrollbar-20260828.json`

SHA-256：`44302E4A8DACA3B0D28F9B32BC3EA702609DAE03E87390C32F826A01FC461A12`

source HEAD：`a2d8d811c4a3a1fc1db6f5375c491e7e4502533f`，另有明确列出的 working-tree candidate。

## 5. 验证状态

- 源码合同先 4/4 RED，生产迁移后 4/4 GREEN；
- 压力模型 4/4 GREEN；
- touched Rust 使用 `rustfmt --check --config skip_children=true,reorder_imports=false` GREEN；
- scoped `git diff --check` GREEN；
- lower Rust 回归已覆盖 Activity source-order row addresses/count、metadata-preserving frame
  patch、fixed paint rows 保留、Browser logical group count；尚未执行，不能声明通过；
- 独立只读 code review 发现并要求修正 overlay row lookup 的复杂度声明；模型与报告已从严格
  O(1) 修正为 contiguous O(1) / overlay 最坏 O(log N)；
- 未启动 raw Cargo，也未使用历史 editor 二进制；
- managed current-source Editor profile、CPU/内存/p50/p95/p99、WPR 与 RenderDoc 均待执行。

## 6. 动态验收门

受管通道可用后，先运行 `zircon_editor` 下层 focused tests，再运行真实 Activity/Browser 产品路径：

- 0/1/1k/10k/100k tree rows；
- hover none/stable/move，scroll stable/change；
- tree pane damage 与 unrelated subview damage；
- 计数 template node visits/control-id parses 必须为 0；
- metadata generation visits 与 generation 数成正比；
- count/frame query 数与实际 paint/hover 数一致；
- 收集 main-thread CPU、allocation/RSS、input-to-damage 与 input-to-present p50/p95/p99/max；
- 同一 source manifest、同一 workload、同一可执行文件配置下比较前后，并验证像素/文本 parity。

动态门通过前，本项不提交为完成里程碑，也不发送量化企业微信完成通知。
