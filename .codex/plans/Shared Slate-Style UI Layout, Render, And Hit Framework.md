# Shared Slate-Style UI Layout, Render, And Hit Framework

## Summary

采用“完整 Slate 化”路线，但按里程碑落地。核心原则：`layout` 产出的 arranged geometry 是唯一空间事实；`render extract` 和 `hit grid` 都消费同一份 arranged tree，禁止 toolbar 这类控件再维护手写命中坐标。

参考依据来自 Unreal Slate：`FHittestGrid`、`FArrangedWidget`、`FArrangedChildren`、`EVisibility`。本地落点是：
- 契约：`E:\Git\ZirconEngine\zircon_runtime_interface\src\ui`
- 行为：`E:\Git\ZirconEngine\zircon_runtime\src\ui`
- editor 接入：`E:\Git\ZirconEngine\zircon_editor\src\ui\slint_host`

## Key Interfaces

- 在 `zircon_runtime_interface::ui` 增加共享契约：
  - `UiVisibility`: `Visible`, `Hidden`, `Collapsed`, `HitTestInvisible`, `SelfHitTestInvisible`
  - `UiArrangedNode`: `node_id`, `node_path`, `parent`, `frame`, `clip_frame`, `z_index`, `paint_order`, `visibility`, `input_policy`, `control_id`
  - `UiArrangedTree`: layout 后的稳定 arranged 列表和父子关系
  - `UiHitTestGrid`: spatial cells + sorted hit entries
  - `UiHitPath`: root-to-leaf path + leaf-to-root bubble route
  - `UiSurfaceFrame`: `arranged_tree`, `render_extract`, `hit_grid`, focus/capture/hover state snapshot

- 在 `zircon_runtime::ui` 实现行为：
  - layout pass 生成 `UiArrangedTree`
  - render extract 从 `UiArrangedTree` 生成 `UiRenderCommand`
  - hit grid 从同一 `UiArrangedTree` 建立，按 visibility、enabled、clip、z/paint order 过滤
  - pointer dispatch 使用 `UiHitPath`，支持 hover enter/leave、focus、capture、primary/secondary/middle、scroll、activation phase

- 在 editor native host 中：
  - 每个 host surface 都提交 `UiSurfaceFrame`
  - native pointer route 查询 surface hit grid，不再调用 toolbar 专用坐标函数
  - toolbar action 从 hit node 的 `control_id`/binding 映射得到
  - 旧 toolbar 手写常量命中实现会被删除或收束为通用 hit-grid adapter

## Milestones

1. Shared contracts and visibility model  
   建立 `UiVisibility`、`UiArrangedNode`、`UiArrangedTree`、`UiHitPath`、`UiHitTestGrid` 契约；迁移现有 `state_flags.visible` 调用到明确 visibility 语义。完成后 runtime/editor 都不再用布尔 visible 推导命中策略。

2. Runtime arranged tree and hit grid  
   改造 `UiSurface::compute_layout()`：layout 结束后生成 arranged tree，再由 arranged tree 派生 render extract 和 hit grid。命中支持 clip chain、overlap、z-order、disabled、hit-test-invisible/self-hit-test-invisible、scrollable children、nested surface bounds。

3. Input routing, focus, capture, invalidation  
   补齐 Slate 风格输入状态：hover diff、focus path、pointer capture、activation phase、dirty flags。layout dirty 必须让 arranged/render/hit 同步失效；render-only dirty 不应重算 layout。viewport toolbar 按当前行为使用 `PrimaryPress` 激活，避免 press/release 双触发。

4. Editor native host migration  
   将 root shell、pane surface、template nodes、viewport toolbar 都注册为 `UiSurfaceFrame`。native host 只通过 hit grid 判断命中。toolbar 命中区域必须等于 `.ui.toml` 组件布局后产生的 button frame；新增控件不需要改 Rust 坐标表。

5. Debugging, docs, and hard cutover  
   加 UI hit/debug dump：给定点输出 surface、hit stack、bubble path、clip/visibility 过滤原因。更新 `docs` 下 UI/runtime/editor 文档。删除局部手写 hit-test 旧路径，不保留兼容 shim。

## Test Plan

- Runtime unit tests：visibility 过滤、clip chain、overlap z-order、nested surface、scrollable hidden child、capture/focus/hover path。
- Runtime render-hit coherence：同一个 button frame 在 render command 与 hit grid 中一致；移动/resize 后两者同步变化。
- Editor host tests：toolbar 所有 `.ui.toml` button 都可命中；新增 button 只改模板即可生效；primary press 只触发一次；secondary/middle/release 不触发 toolbar command。
- Export validation：继续使用 `E:\zircon-build\targets` 作为 target dir，运行 editor/runtime 构建和 focused native host tests。
- Docs validation：更新 UI framework、editor host、runtime interface 文档，并记录参考 Unreal 文件和验收命令。

## Assumptions

- 你已选择“完整 Slate 化”，所以不会只做 toolbar 局部修补。
- world-space UI 的射线拾取仍由 scene/editor viewport 层负责；拾取到具体 UI surface 后，2D 命中继续走同一套 `UiHitTestGrid`。
- 当前仓库直接在 `main` 开发，不创建 worktree 或新分支。
