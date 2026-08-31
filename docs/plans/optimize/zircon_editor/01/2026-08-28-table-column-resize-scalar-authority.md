# Table 列宽拖拽标量权威与增量发布计划

日期：2026-08-28

状态：`design_ready`；确定性压力模型已写入；生产 owner 正在变更，尚未实施；managed Rust 与产品 profile 待执行

## 1. 当前结论

Runtime Table 的 component reducer 已能对 `column_width` 事件原地更新
`column_widths[field]` 与首个匹配的 `columns[].width`。但 Surface 默认交互没有复用这条标量
authority。每个 pointer Move 仍执行：

1. 从字符串 drag token 重新解析 `start_width/min_width/field`，并为 field 分配 `String`；
2. 把完整 TOML `column_widths` 转换为 `UiValue::Map`，改一个 entry，再由通用 mutation
   转回完整 TOML，同时重新物化 previous binding value；
3. 把完整 TOML `columns` 转换为嵌套 `UiValue::Array`，线性查找 field，改一个 width，
   再执行相同的完整往返；
4. 对同一逻辑列宽提交两次 property transaction，最后才发出一条 `column_width` 事件。

因此 reducer 内的 in-place 更新没有关闭产品热路。对 C 列表格，输入状态成本仍是每 Move
`O(C * F)`，F 为每列 metadata field 数，并且产生两份聚合值的临时所有权、previous value 和
binding payload。必要的下游 geometry patch 也被这段数据重建成本遮蔽。

相关 current source：

- `zircon_runtime/src/ui/surface/surface/default_interactions/table/mod.rs:127`
- `zircon_runtime/src/ui/surface/surface/default_interactions/table/mutation.rs:12`
- `zircon_runtime/src/ui/surface/surface/default_interactions/table/columns.rs:217`
- `zircon_runtime/src/ui/component/state_reducer/table.rs:189`
- `zircon_runtime/src/ui/surface/property_mutation/metadata_batch.rs:24`

这些生产文件当前包含其他 owner 的活跃修改。本切片不跨写，也不把设计和模型写成已实现。

## 2. Unreal 主参考

本地参考：

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Views/SHeaderRow.cpp:1008`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SHeaderRow.h:237`

`SHeaderRow` 的 sizing grip 在 `OnMouseMove` 中只读取 cursor delta，计算一个 `NewWidth`，然后
调用捕获列对象的 `FColumn::SetWidth`。`FColumn` 的 width attribute 直接驱动对应
`SBox::WidthOverride`；Move 不重建 `Columns` 数组。外部需要持久化时，`OnWidthChanged`
delegate 接收单个 float。可迁移的是所有权合同：drag session 持有 typed target，列宽是单一标量
authority，布局消费该标量，兼容/持久化是下游投影，不是每个原始输入的前置条件。

## 3. 目标算法

### 3.1 Generation-owned schema

Table structure generation 发布一个 `UiCompiledTableColumns`：

- source-order column slots；
- `field -> first slot` 索引，保留当前首个匹配语义；
- 每列 canonical width、min width 与 sizing rule；
- 兼容字段 `field/id/key/name` 只在编译时解析；
- schema/columns 结构变化才重建，普通 width delta 不重建索引。

如果 `column_widths[field]` 存在，它继续覆盖 `columns[].width`。重复 field 仍按 current source
选择首个 array column；hash/index 命中后必须精确校验 live field，不能让冲突改变目标身份。

### 3.2 Typed drag session

Press 时解析一次 target，并保存
`{owner_id, column_slot, start_width, min_width, source_generation}`。Move 不再编码/解析
`table-column-resize:<float>:<float>:<field>` 字符串。generation 不匹配时使用 typed fallback reason
终止或重新解析，不能对陈旧 slot 写入另一列。

### 3.3 Scalar mutation transaction

每个去重后的 Move 只提交一个 `UiTableColumnWidthDelta { owner_id, slot, field, width }`：

1. 比较 canonical width；相等直接返回 no-change，不发 dirty、不发 event；
2. 原地更新一个 width scalar，并推进 table width generation；
3. 生成一次 binding/component update；
4. layout 只访问实际受影响的 header、可见 cells 和后续需要平移的 geometry；
5. damage 是受影响表格区域，不升级为无原因 full surface rebuild。

输入状态 mutation 必须为 `O(1)`；几何成本为 `O(A)`，A 是实际受影响的可见 geometry。列宽
改变可能平移后续可见列，因此不能错误承诺整个 layout 为 `O(1)`，但它不得包含完整业务 rows、
columns payload 的 clone/转换。

### 3.4 Compatibility projection

`column_widths` 与 `columns[].width` 不能继续作为两个同步写入的 canonical authorities。推荐边界：

- runtime canonical：compiled column slot 中的 width；
- component/binding：每个有效 Move 发布一个 scalar delta；
- 旧 aggregate consumer：在 frame cadence 或 release flush 时把累计 latest delta 原地应用到一个
  generation-owned compatibility projection；
- serialization/save：从当前 canonical widths 生成完整 aggregate；
- legacy consumer 若要求拖拽中实时值，消费同一 delta patch，而不是要求 Surface 重新序列化全表。

Release 只 flush 尚未发布的 latest value，不重复应用最后一次 Move。

## 4. 确定性压力模型

默认模型使用 256 列、每列 8 个 metadata entries、2,000 次 Move，并保守计算 current source 中
明确存在的三次完整聚合访问：metadata -> `UiValue`、`UiValue` -> metadata、metadata -> previous
binding value。模型没有估算 allocator、TOML enum dispatch、String payload bytes 或 layout/render。

| 指标 | 当前聚合往返 | 标量 authority 目标 |
| --- | ---: | ---: |
| width-map entry visits | 1,536,000 | cadence/release projection 内计入 |
| columns-array entry visits | 13,824,000 | cadence/release projection 内计入 |
| column match checks | 256,000 | schema generation 一次建立索引 |
| property transactions | 4,000 | 2,000 |
| combined structural work units | 15,620,000 | 12,864 |

默认 structural work ratio 为 1,214.2413 倍。目标数包含一次 schema build、每 Move 三个 scalar
operation、每 Move 一次 transaction，以及一次完整 compatibility flush。它只是算法工作量模型，
不是 CPU、内存、帧率或 input-to-present 加速比。

模型工具：`tools/ui_table_column_resize_scalar_pressure.py`

工件目标：`E:\zircon-profiles\runtime-ui-table-column-resize-scalar-20260828.json`

SHA-256：`ABC787455665BB7386E7ED7323DCA05E7E9CC2AD749D089234230E62064B1910`

## 5. 实施顺序

1. lower test 先证明 scalar delta 原地更新两种读视图、重复 field 首个匹配、same-width no-op；
2. 建立 generation-owned compiled table columns，press 保存 typed slot；
3. 让 Move 只提交 scalar transaction，删除字符串 token decode 和两次 aggregate mutation；
4. layout/render 读取 canonical width generation，记录 affected geometry visit/damage；
5. 在 binding/serialization 边界实现 compatibility delta projection；
6. 删除 Surface 对 `column_widths` 与 `columns` 的双 authority 写入，不保留 legacy fallback 热路；
7. 增加 Editor 产品回归，再执行 managed Runtime/Editor validation 与真实 profile。

## 6. 验收门

- C=1/16/256/1,024，2,000 次 Move；drag-token parse/field `String` allocation 在 press 后为 0；
- 每个有效 Move scalar mutation/transaction 各 1，相同 width 为 0；
- aggregate conversion/serialization 在 raw Move 路径为 0，cadence/release flush 次数有显式上界；
- structure/schema generation 在整个稳定 drag 中不变；
- geometry visits 与受影响可见列/cell 数 A 同阶，与 rows payload 总量无关；
- `columns` 缺失、`column_widths` 缺失、重复 field、min clamp、capture loss、generation change、
  release without final Move 均有 lower regression；
- 收集 main-thread CPU、allocator bytes、RSS/private bytes、input-to-damage 与
  input-to-present p50/p95/p99/max，以及 full rebuild/fallback reason；
- 与同一 current-source manifest 的旧路径 profile 对比，不能用本模型比例代替实测。

在生产迁移、managed Rust、Editor 产品回归和动态指标完成前，本项保持 `design_ready`。

## 7. 本轮静态证据

- pressure model 合同 4/4 GREEN；
- Python compile 与 scoped `git diff --check` GREEN；
- current source guard 确认 Surface 仍有两次 aggregate mutation、`mutation.rs` 仍有两条
  `UiValue::from_toml` 聚合转换，故本计划没有基于旧源码立项；
- `event_routing` 的两条 move-after-dispatch 与 Runtime Interface 的五条 IME pattern 旧诊断，
  当前源码形状均已修复；相关 source guard 4/4，前两文件 rustfmt GREEN，adapter 只剩与这些
  诊断无关的外部 import-order 漂移；
- managed profiling target pool 当前没有 `zircon_editor.exe`，capture runner 又明确只接受
  coordinator-managed `$CARGO_TARGET_DIR/profiling` 产物，因此未使用历史二进制伪造产品数据；
- 未启动 Cargo，未修改 Table/metadata mutation 的活跃生产 owner。
