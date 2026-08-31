---
status: source-hardcut-static-verified
created_at: 2026-08-25
parent_failure: docs/plans/zircon_editor/editor/09/failure-2026-07-17-asset-pane-projector-repeated-model-scans.md
implementation_status: generation-descriptor-observability-complete
focused_rust_status: blocked-upstream-zr-rhi-wgpu
dynamic_baseline_status: blocked-upstream-rhi
related_code:
  - zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/thumbnail_geometry.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/browser_virtualization.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_nodes.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/projector.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/{transform.rs,draw.rs}
  - zircon_editor/src/ui/retained_host/ui_perf/{counter_catalog.rs,mod.rs}
  - tools/ui-profile-capture.ps1
---

# Asset pane identity-index performance research

## 结论

2026-07-19 的“stable paint identity parse 为零”结论不成立，不能以原静态字符串检查作为验收证据。资产面板已消除 projector 对 `ModelRc::row_data` 的全量扫描，但每个已访问节点仍会在绘制期重新解析 `control_id`：

- Activity projector 先调用 `activity_reference_row_index`，再调用 `AssetContentPaintMetadata::identity` 与 `is_scroll_node`；reference hover 还调用 `contains("RowPanel")`。
- Browser projector 先调用 `browser_source_tree_row_index`、`browser_reference_row_index`，再调用 `identity`；虚拟缩略图绑定在 `apply_browser_slot_item` 中以多个 `control_id.contains(...)` 推断角色。
- `AssetContentPaintMetadata::identity` 重新调用 `parse_activity_content_identity` 或 `parse_browser_content_identity`。因此 list、thumbnail、reference 和 source-tree 的稳定 paint 仍与可见节点数成比例地进行字符串分割、前缀判断与数字解析。

这不是运行时资产注册表问题。资产 generation 已经是 `ui::workbench::asset_content_layout` 的派生 owner，且 View DTO 到 retained-host DTO 保留同一元数据分配；修复必须保留该边界，禁止把 UI 绘制缓存下推到 `zircon_runtime` 或在 painter 增加第二份节点缓存。

## 架构证据

- 现有 `TemplateNodePaintTransform::transform_row` 与 `draw_template_nodes_with_transform` 已将模型行号传入投影器，精确可见行计划也在 metadata 中生成。通用接口具备 O(1) 按行查询所需的输入，缺失的是 generation 发布的按行语义。
- Unreal `SAssetView` 将 `Items`、`FilteredAssetItems` 与 `VisibleItems` 作为独立的 item collection 管理，并使 list/tile view 消费 typed item；这是本修复的主参考：模型 generation 持有 identity 和可见范围，绘制层只消费 typed item。
- Godot `EditorFileSystemDirectory` 是编辑器文件模型的权威来源，`FileSystemDock` 从其 typed 内容建立视图；这确认权威目录模型与派生 UI 表示的边界。
- Fyrox `AssetBrowser` 的 `AssetItem` 持有 identity/selection，刷新仅重建其 UI 项；其模式不适合本项目的虚拟化规模，但证明了 UI 不应在 painter 从字符串反推资产身份。

## 硬切换设计

1. 将 `TemplateNodePaintTransform` 收束为 row-aware 合同：`transform_row(row, node, clip)` 为必需实现，删除 legacy `transform` 默认回退；Console、资产投影器及 pipeline 测试在同一变更中迁移，不能留下无行号的兼容路径。
2. 在 `AssetContentPaintMetadata` 的 generation build 中为每个 node row 写入一条 `AssetContentRowDescriptor`。描述符必须覆盖 fixed、Activity content、Activity references/used-by、Browser list、Browser thumbnail、source tree、references/used-by，并携带 row/index/list kind、是否随各自 scroll 平移及是否承担 hover。
3. `AssetContentPaintMetadata::identity(control_id)`、`is_scroll_node(control_id)`、projector 的 `*_row_index(control_id)` 和 `apply_browser_slot_item` 的 `contains` 角色分支全部删除。解析函数仅可在 generation build 中使用；stable paint 仅调用 `row_descriptor(row)`。
4. Browser thumbnail descriptor 显式表达 Card、InfoBand、SelectionMarker、Visual、NameContinuation、Name、Type、Meta 角色，使 slot binding 填充不依赖字符串。保留现有逻辑 paint generation、selected indices、可见组和 clip 语义。
5. 不新增 painter-owned cache、alias、wrapper 或字符串 fallback；metadata vector 与现有 geometry/visible groups 同生命周期，并由 DTO shared metadata 传递。

## 实现结果（静态）

本次实现完成了上述结构性硬切换，但不将它表述为动态性能验收：

- `asset_content_layout::identity` 在 generation 中一次性分类每个行，并发布稠密 `AssetContentRowDescriptor`；`paint_metadata` 在同一 descriptor 分组遍历中收集 content、header/grid/preview 及三个辅助 viewport 几何，不再重新解析 `control_id`。
- Activity、Browser、source-tree 与两类 reference projector 仅按 `row_descriptor(row)` 消费 role/index/list kind/hover 语义；缩略图 slot 显式区分 Card、InfoBand、SelectionMarker、Visual、NameContinuation、Name、TypeBadge、Type、Meta，删除 painter 的 `contains` 推断。
- `TemplateNodePaintTransform` 已硬切为必需的 `transform_row(row, node, clip)`，迁移 Console、scene overlay 和测试，不保留旧 `transform` fallback 或兼容转发。
- 新增回归覆盖 Browser table/header/preview 与 source-tree/references/used-by viewport 的 descriptor 驱动几何，并强化 Python 合同以拒绝 projector parse/index/`contains` 路径。
- 独立审查指出缩略图虚拟 slot 只替换内容、未重投影 item-specific geometry；现已由 generation 发布扩展名与 type-label 测量值，metadata 保留 materialized card frame，projector 通过共享纯函数重算 Card/Visual/InfoBand/Name/Badge/Meta 框，不新增 painter cache。深滚动长文件名、双行名称和宽类型徽标回归已加入。
- 修正后的独立复审为 `0 Critical / 0 Important`；复审发现的导入排序已由 `rustfmt` 收束。深滚动回归按 production thumbnail role 顺序断言 Card、Visual、Selection、Name、NameContinuation、TypeBadge、Type 与 Meta 的 item-specific geometry。

这证明 stable paint 没有遗留的源码级 identity parser 调用，但不能替代真实运行时的 parse/alloc/CPU/GPU/功耗采样。Windows 受管命令 `cargo test -p zircon_editor --locked --verbose --lib browser_thumbnail_virtual_rebind_reprojects_item_specific_child_geometry` 已产生终态，但在编译 `zr_rhi_wgpu` 时以 14 个既有诊断退出 101，未开始编译 `zircon_editor`；诊断与 `docs/plans/optimize/zircon_runtime/90/failure-2026-08-24-rhi-wgpu-diagnostics-current-source-compile-blocker.md` 一致。动态验证仍以 RHI 恢复后的受管 Windows lane 为准。

## 基线与验证方案

实施前先增加可观测性，而非先猜测优化效果。现有 `UiPerfCounter` 与 `asset_browser_scroll` gate 已记录 logical/materialized/visible item 与 node 数、投影构建数、logical paint chunk build/reuse 和 logical paint item projection。本次已补充 generation 与 retained-host projection 的两个边界计数，并将它们接入 `tools/ui-profile-counter-evidence.ps1` 的 `asset_browser_scroll` gate：稳定滚动要求 generation identity parse 为零，且必须观察到实际 descriptor lookup。它们的动态数值仍受 RHI blocker 限制，不能由静态代码推导为性能结果。

| 指标 | 采样位置 | 目标 |
|---|---|---|
| `asset_content_generation_identity_parse_count` | Activity/Browser composition 在 metadata build 后 | generation 可随总节点增长；stable scroll 必须为 0 |
| `asset_content_descriptor_lookup_count` | asset projector `transform_row` | 与实际 asset-content visited node 数一致，且不超过全模板 `template_node_visit_count` |
| `template_node_visit_count` / `template_node_clone_count` | 既有 draw 计数 | 与可见节点及 damage clip 成比例，不随总目录线性增长 |
| `frame_duration_us`、CPU p50/p95/p99 | `asset_browser_scroll` 与 `idle_hover` profile | 记录改前/改后，而非预先宣称阈值 |
| alloc count/bytes、GPU time、功耗 | Windows ETW/WPR 与 profile capture | 只在真实运行数据可取得时与同级编辑器经验值比较 |

每种操作在 1、1k、10k asset catalog 下至少执行 warm-up 后 3 次 measured run，覆盖 list 与 thumbnail 的 stable paint、连续 scroll、单项 hover、clip edge、selected slot rebinding。需保留像素等价、hit/hover、header/grid/preview 固定语义。动态门禁通过的条件是：stable paint identity parse 为 0；descriptor lookup、visited node、clone 与可见节点受界；无 full rebuild 或新增 paint cache；报告同时给出 p50/p95/p99、alloc 与可用的 GPU/功耗数据。

当前不能生成这些数值：`cargo test -p zircon_editor --lib --locked` 在外部 `zr_rhi_wgpu` 编译错误处停止（14 项诊断，详见 `docs/plans/optimize/zircon_runtime/90/failure-2026-08-24-rhi-wgpu-diagnostics-current-source-compile-blocker.md`）。本次只完成了指标 source、static contract 与 PowerShell gate parser 检查；没有可执行 editor binary 时，不得伪造 CPU、分配、GPU 或功耗结论。

## 执行顺序

1. 在 RHI 恢复后运行未修改前的受控采样，归档 source manifest、trace、hotspot 与 WPR/ETW 数据至 E: profile 根目录。
2. 以失败测试先行覆盖所有 descriptor 角色、row/slot 一致性、scroll/hover/clip 和零 paint-time parser 合同。
3. 实施 row-aware contract 与 generation descriptor 的硬切换，删除旧 parser/contains 路径及静态检查盲点。
4. 运行 Rust、Python contract、像素和 1/1k/10k profile；比较前后数据并仅在瓶颈消失后更新 failure return。

## 产出记录与时间

| 时间 | 状态 | 完成项目与当前门禁 |
|---|---|---|
| 2026-08-25 | `source-hardcut-complete / static-verified / focused-rust-blocked-upstream-rhi / dynamic-baseline-blocked` | 完成 current source、Unreal/Godot/Fyrox reference 后实施 generation-owned dense row descriptor 与 row-aware transform 硬切；metadata 不再以字符串二次求取几何或 viewport，stable paint 删除 identity/index parser 与 thumbnail `contains` 分支。独立审查发现的缩略图 slot geometry 重投影已落地，最终复审 `0 Critical / 0 Important`，覆盖长文件名、双行名称与宽类型徽标。为使后续动态分析可证伪，composition 现上报 `asset_content_generation_identity_parse_count`，projector 以 profile-only 批量计数上报 `asset_content_descriptor_lookup_count`；`asset_browser_scroll` gate 要求 stable scroll 前者为 0、后者大于 0。Python contract `8/8`、PowerShell parser、scoped rustfmt 和 scoped `git diff --check` 均通过。Windows 受管 focused Cargo 终态仍为 exit 101，停在 `zr_rhi_wgpu` 的既有 14 项诊断、未编译 `zircon_editor`；外部 RHI 仍阻断可执行 editor 的 1/1k/10k、CPU、alloc、GPU、功耗和像素证据，未声称任何动态收益。 |
