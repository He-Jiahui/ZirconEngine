---
related_code:
  - zircon_editor/src/ui/workbench/window_registry/editor_window_registry.rs
  - zircon_editor/src/ui/workbench/window_registry/window_instance.rs
  - zircon_editor/src/ui/workbench/window_registry/window_kind.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_view_instance.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_dock_position.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_binding.rs
  - zircon_editor/src/ui/workbench/layout/activity_window_layout.rs
design_references:
  - docs/ui-and-layout/editor-workbench-designs/main-tabs-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/drawer-expanded-state-spec.png
  - docs/ui-and-layout/editor-workbench-designs/split-editor-state-spec.png
plan_sources:
  - docs/plans/zircon_editor/editor_layout/03-jetbrains-docking-workbench.md
  - docs/plans/zircon_editor/editor_layout/04-layout-presets-and-persistence.md
status: planned
---
# 07 窗口化、Chrome 式页签与可吸附抽屉架构

## 1. 目标

把工作台的承载模型做成**"每种使用目的 = 一个可窗口化的编辑器视图"**:多个视图合并到一个窗口时,顶部用 **Chrome 浏览器式页签栏**管理(每个页签 = 一个目的视图,页签内部是该视图的完整界面,类似浏览器网页内部结构);抽屉里的界面同样**注册进 window 内部**,且像 JetBrains 一样可**独立成单独页面或被转移/吸附到其它位置**。本计划在既有 window registry 之上补**页签合并语义 + 抽屉↔独立窗口的拆/合/吸附流转**,不重写注册中心。

## 2. 现状(按代码核实)

### 2.1 已存在的设施

| 能力 | 落点 | 证据 |
| --- | --- | --- |
| 窗口注册中心 | `editor_window_registry.rs` | `register_window`/`register_drawer_view`/`register_drawer_window`/`bind_drawer` 已具拆/合骨架 |
| 窗口实例 + 抽屉槽 | `window_instance.rs` | `WindowInstance{drawer_views: BTreeMap<DrawerDockPosition, Vec<ViewInstanceId>>, selected_drawer}` |
| 窗口类型 | `window_kind.rs` | `WindowKind::{Ordinary, DrawerCapable, DrawerWindow}` |
| 抽屉视图实例 | `drawer_view_instance.rs` | `DrawerViewInstance{instance_id, descriptor_id, owner_window, dock_position}` |
| 五槽停靠位 | `drawer_dock_position.rs` | LeftTop/LeftBottom/Bottom/RightTop/RightBottom(与 index §1.1 抽屉区一致) |
| 重绑流转 | `bind_drawer` | 已能把 drawer 从旧 owner 摘下、重设 dock_position、绑到新 window |

### 2.2 真实缺口

- 缺 Chrome 式**页签栏合并语义**:多个目的视图合并进一个窗口、页签序、激活页签、拖拽重排、拖出新建窗口。
- 缺**抽屉↔独立窗口**的完整流转动作集(`detach`/`reattach`/`float`/`absorb`),现有 `bind_drawer` 只覆盖重绑 dock。
- 缺**目的视图(PurposeView)**这一注册概念:把"每种使用目的"声明为可窗口化单元。

## 3. 设计

### 3.1 目的视图(PurposeView)= 可窗口化单元

每种使用目的(Scene/Material/Asset Browser/Diagnostics/插件视图…)注册为一个 `PurposeView`,可作为:(a)窗口内的一个 Chrome 页签,(b)抽屉内的一个停靠视图,(c)独立浮动窗口。同一 `descriptor_id` 在三种宿主形态间流转,内容不变,仅宿主变。

### 3.2 Chrome 式页签栏(窗口内合并)

- 一个 `WindowInstance` 顶部持有一条**文档页签栏**:`tabs: Vec<ViewInstanceId>` + `active_tab`。
- 页签 = 目的视图;页签内部 = 该视图完整界面(对应 05 的页面模板),类比浏览器"标签页里是整张网页"。
- 动作:`add_tab`/`activate_tab`/`reorder_tab`/`close_tab`/`tear_off_tab`(拖出页签 → 新建窗口,类比 Chrome 拖出标签)。
- 页签栏与抽屉是**两层**:页签管 center 文档,抽屉管四角/底部停靠;两者都注册在同一 window 内。

### 3.3 抽屉的拆/合/吸附(JetBrains 式)

在 `bind_drawer` 之上补完整流转动作,均为注册中心原子操作:

| 动作 | 语义 | 复用/新增 |
| --- | --- | --- |
| `detach_drawer_to_window` | 抽屉视图独立成单独浮动窗口(`DrawerWindow`) | 复用 `register_drawer_window` + 摘除原 owner |
| `reattach_drawer` | 独立窗口吸附回某窗口的某 dock 槽 | 复用 `bind_drawer` |
| `move_drawer` | 抽屉在同/异窗口的 dock 槽间转移吸附 | 复用 `bind_drawer` |
| `promote_drawer_to_tab` | 抽屉视图升级为 center 页签 | 新增:摘 drawer → add_tab |
| `demote_tab_to_drawer` | center 页签降级为抽屉停靠 | 新增:close_tab → register_drawer_view |

吸附时校验 dock 槽职责(index §1.1):四角抽屉强制职责匹配,center 页签自由。

### 3.4 注册进 window 内部(统一事实)

页签视图与抽屉视图都注册在 `EditorWindowRegistry` 内,window 是承载唯一事实源;布局预设(04)与持久化恢复的是"哪些视图在哪个 window、是页签还是抽屉、在哪个 dock"。

## 4. 接口与数据结构草案(Rust)

```rust
// window_instance.rs 扩展
pub struct WindowInstance {
    // ...既有字段...
    pub tabs: Vec<ViewInstanceId>,        // 新增:Chrome 式 center 页签序
    pub active_tab: Option<ViewInstanceId>,
}
// editor_window_registry.rs 新增动作
impl EditorWindowRegistry {
    pub fn add_tab(&mut self, window: ActivityWindowId, view: ViewInstanceId) -> Result<(), String>;
    pub fn reorder_tab(&mut self, window: ActivityWindowId, from: usize, to: usize) -> Result<(), String>;
    pub fn tear_off_tab(&mut self, window: ActivityWindowId, view: ViewInstanceId) -> Result<ActivityWindowId, String>; // 返回新窗口
    pub fn detach_drawer_to_window(&mut self, drawer: ViewInstanceId) -> Result<ActivityWindowId, String>;
    pub fn promote_drawer_to_tab(&mut self, drawer: ViewInstanceId, target_window: ActivityWindowId) -> Result<(), String>;
    pub fn demote_tab_to_drawer(&mut self, view: ViewInstanceId, dock: DrawerDockPosition) -> Result<(), String>;
}
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 修改 | `window_instance.rs` | 加 `tabs`/`active_tab` |
| 修改 | `editor_window_registry.rs` | 加页签 + 拆/合/吸附动作 |
| 新增 | `window_registry/purpose_view.rs` | 目的视图注册概念 |
| 新增 | `window_registry/tab_strip.rs` | Chrome 式页签栏语义 |

## 6. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
| -- | --- | --- | --- | --- |
| S1 | 目的视图 + Chrome 页签合并(add/activate/reorder/close) | purpose_view.rs / tab_strip.rs / window_instance.rs | `cargo test -p zircon_editor --lib --locked` | — |
| S2 | tear_off + 抽屉拆/合/吸附 + 页签↔抽屉升降级 | editor_window_registry.rs | `cargo test -p zircon_editor --test registry --locked` | — |

## 7. 测试矩阵

- 多视图合并进一窗口,页签序/激活/重排正确。
- 拖出页签 → 新窗口,原窗口移除该页签。
- 抽屉 detach → 独立窗口,reattach → 吸附回正确 dock 槽。
- 页签↔抽屉升降级后,视图仍注册在 window 内、内容不变。
- 抽屉吸附四角时职责校验生效(index §1.1)。

## 8. 风险与对策

- 风险:页签/抽屉双层与 04 持久化档案不兼容。对策:档案记录每视图的(window, host_form: Tab|Drawer, dock|tab_index),版本化校验。
- 风险:tear_off 产生孤儿窗口。对策:窗口空(无页签无抽屉)时自动回收。

## 9. 完成定义

每种目的视图可窗口化;窗口内 Chrome 页签合并;抽屉可独立成窗口/转移吸附;全部注册在 window 内、可被 04 持久化。

## 10. 边界约束

不实现 OS 级原生多窗口(那在 `native_window_hosting` 插件 + `editor_ui/08`);本计划只做注册中心的承载语义;dock 职责按 index §1.1。

## 11. 参考实现对照(dev/ 源码锚点)

- `dev/UnrealEngine/.../Slate/Public/Framework/Docking`:tab manager / dock area / tear-off。
- `dev/Fyrox/fyrox-ui/src/dock`:docking tile 拆合。
- `dev/godot/editor`:编辑器 dock slot 转移参考。

## 12. 状态与产出记录

planned。后续项:S1 目的视图 + Chrome 页签合并。
