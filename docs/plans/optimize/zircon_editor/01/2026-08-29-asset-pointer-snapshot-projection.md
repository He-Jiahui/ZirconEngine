# Asset pointer snapshot 窄投影

日期：2026-08-29

状态：`production_static_candidate`；源码合同、rustfmt、确定性压力模型通过；managed Cargo 与产品 profile 待执行

## 1. 发现

retained asset pointer 的事件频率路径只需要两类已经发布的 authority：

- `visible_assets` generation，用于列表命中、拖拽和 context menu；
- `selection`，用于 reference/used-by 拖拽和 Browser detail scroll 的当前详情。

发布指针状态时此前直接 `Arc::new(snapshot.clone())`，因此每次快照同步还会复制与 pointer 无关的 project path、folder tree、visible folder rows、search/import 字段和菜单句柄。事件热路本身已经只做 `Arc` clone；本切片只缩小 publication payload，不在事件回调内建立第二份查询缓存。

## 2. 实施

`AssetWorkspaceSnapshot::pointer_projection()` 生成同类型的最小快照：保留代次/视图标量、`visible_assets` generation 和 `selection`，其余字段使用默认值。`sync_asset_pointer_layout` 在唯一发布边界调用该方法；现有 content/reference/detail pointer consumer 不需要分叉类型或改变调用合同。

这个边界仍遵守 generation ownership：visible asset 的 chunk/index `Arc` 只增引用计数，selection 仍由 detail 语义明确拥有；没有把完整 pane snapshot 或 renderer 私有缓存带入 pointer bridge。

## 3. 静态证据

- `tools/tests/test_editor_asset_pointer_snapshot_projection_contract.py`：2/2 GREEN；确认发布边界调用 `pointer_projection()` 且 projection 只保留两个 pointer authority 字段；
- `tools/tests/test_editor_asset_pointer_snapshot_pressure.py`：3/3 GREEN；
- `rustfmt --edition 2021 --check --config skip_children=true,reorder_imports=false`：两个 touched Rust path GREEN；
- `git diff --check`：GREEN（仅保留既有 LF/CRLF 警告）。

确定性模型默认 2,000 folder-tree rows、128 visible-folder rows、10,000 visible assets、1,000 stable publications：

| 工作 | 原路径 | 窄投影 |
| --- | ---: | ---: |
| publication structure units | 12,149,000 | 10,016,000 |
| removed unrelated units | - | 2,133,000 |
| structural ratio | 1.0x | 1.2129592652x |

模型不测 allocator bytes、CPU 时间、RSS、GPU、input-to-present 或 wall-clock latency。工件：`E:\zircon-profiles\editor-asset-pointer-snapshot-pressure-20260829.json`；SHA-256：`74C32996202F1A8A099B30F407FEF1F2EECF691916E62AA049F2D2CF5AC4612A`。

## 4. 验收门

managed lane 恢复后，先跑 Editor retained asset pointer lower tests，再跑 content/reference/detail 产品路径，验证：

- pointer event route、drag payload、context menu 和 detail scroll 的字段 parity；
- projection publication 不复制 folder tree、visible folders 或 project metadata；
- stable publication 的 `AssetPointerSnapshotCloneCount` 保持一笔，但 snapshot payload clone bytes 与全量路径相比下降；
- catalog/selection/resource generation 改变时新 projection 可见，旧代次不被复用；
- CPU/RSS/input-to-present p50/p95/p99 需绑定同一 current-source manifest 后再报告。

当前未启动 Cargo，未运行历史 Editor 二进制，不宣称动态性能达标。
