---
related_code:
  - zircon_editor/src/ui/retained_host/host_page_pointer/constants.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/build_host_page_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/host_page_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/handle_click.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/handle_overflow_click.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/host_page_pointer_route.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/shell_chrome.rs
  - zircon_editor/src/ui/retained_host/app/workbench_pointer/shell_chrome/host_page.rs
  - zircon_editor/src/ui/retained_host/host_contract/host_page_overflow_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/page_overflow.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/page_overflow_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/page_overflow.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/dispatch/actions.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers/overlay/page_overflow.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls/tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_tab.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_tab_strip.zui
design_references:
  - docs/ui-and-layout/editor-workbench-designs/main-tabs-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/tab-overflow-window-spec.png
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Styling/StarshipCoreStyle.cpp
tests:
  - cargo test -p zircon_editor host_page_overflow_keyboard --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-editor-overflow-keyboard-0711 -- --nocapture
  - direct editor test binary dropdown/menu/boundary-key exact regressions
  - direct editor test binary capture_host_page_overflow_keyboard_visual_artifact --exact --ignored --nocapture
  - docs/tests/editor/editor-window-m3-host-page-overflow-keyboard-640x420.png
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
status: in_progress
---
# 15a 页签条溢出与几何单源(S15.3 深化)

> 本文是 `15` 计划 **S15.3** 切片的实现就绪深化,专治截图里"主页面页签被压成 `Sce / Eff / Abil`"。它把页签条几何收敛为**单一事实源**(描画与命中共用),并补"超宽溢出收纳"语义。父计划见 `15-component-standardization-from-primitives.md`;承载语义与 `07` 的 Chrome 页签合并相邻,本文只做**几何 + 溢出 + 命中一致 + 标签省略**,不重做 `07` 的窗口化/吸附。

## 1. 根因追踪(按代码核实)

主页面页签条 = `host_page_pointer` owner。关键常量(`host_page_pointer/constants.rs`):

| 常量 | 值 | 含义 |
| --- | --- | --- |
| `STRIP_X` | 8.0 | 页签条左右留白 |
| `STRIP_Y` | 1.0 | 顶部偏移 |
| `TAB_MIN_WIDTH` | 108.0 | 单页签最小宽 |
| `TAB_HEIGHT` | 30.0 | 页签高 |
| `TAB_GAP` | 4.0 | 页签间隙 |

故障链:

1. `build_host_page_pointer_layout.rs` 计算 `estimated_width = 2·STRIP_X + N·TAB_MIN_WIDTH + (N-1)·TAB_GAP`,**但最终 `strip_width` 取的是共享壳帧宽**(`shared_strip_frame.width` 或 `shared_shell_frame.width`),即页签条被钉死成壳宽,**不随页签数量增长**。
2. 该 builder **只产出 strip_frame + items(仅 page_id)**,不产出每个页签的 `x/width`。每个页签的实际宽度在**描画侧**按"strip 宽等分给 N 页"得出 → 当 `N·TAB_MIN_WIDTH > strip_width` 时,等分宽**跌破 `TAB_MIN_WIDTH`**(900px 壳 ÷ 11 页 ≈ 72px,再减内边距/间隙后文本可用 ~20px)。
3. 文本进 `fontdue`,`layout_text_run`(S15.2 前)按 `max_width` 折行/硬裁字形 → `Scene Editor` 退化成 `Sce`。
4. `handle_click.rs` 命中测试用 `tab_width.max(TAB_MIN_WIDTH)` **单边钳到 108**,于是**命中几何与描画几何不一致**(描画 72,命中 108),窄页签点击会偏。
5. `handle_click` 返回 `Result<HostPagePointerDispatch, String>`——裸 `Result<_, String>`,属 `engine-code-review-findings` 的 `E1` 残留债。
6. 另有 `TAB_MIN_WIDTH=108`(页签条)与 `workbench_tab.zui` 的 `width.min=88`(组件资产)两处**最小宽不一致**。

> 结论:这是**几何分配缺溢出语义 + 描画/命中不共源**的问题,不是单纯文本问题。S15.2 的省略让退化更可读(`Scene…`),但根治要在 S15.3 给页签条几何加"min 钳制 + 溢出收纳",并让描画与命中**共用同一份几何**。
>
> 另一条**测量侧根因**归 `17`:页签宽估算(`estimated_width`/等分)与文本折裁都建立在 `font_size*0.5` 等宽近似上(`ui/text/layout_engine.rs`),与绘制端真实字形度量不一致——同样会让"够不够放下标签"的判断失准。`17` 的"测量=绘制真实字形度量"是 `Sce` 类截断的测量侧根治,与本文的几何单源互补。

## 2. 当前 owner 与职责

| owner | 现职责 | S15.3 改动 |
| --- | --- | --- |
| `host_page_pointer/constants.rs` | 页签度量常量 | 最小宽收敛(与 `15` 度量单源/资产一致),新增溢出按钮宽常量 |
| `host_page_pointer/build_host_page_pointer_layout.rs` | 产出 strip_frame + items | 产出**每页签 `x/width` + 可见/溢出划分**(几何单源) |
| `host_page_pointer/host_page_pointer_layout.rs` | layout 数据结构 | 新增 `tabs: Vec<TabSlot>`(含 frame、是否溢出)、`overflow: Option<OverflowSlot>` |
| `host_page_pointer/handle_click.rs` | 命中测试(自钳 108) | 改读几何单源的 per-tab frame,**删自钳**;返回类型改 typed error |
| `template_segmented_controls/tabs.rs` | 页签描画 | 标签走 `ellipsize_single_line`;读几何单源的 per-tab frame |
| 溢出弹层 | 无 | 复用菜单指针路径产出 tab-overflow 弹层(见 `tab-overflow-window-spec.png`) |

## 3. 目标架构

### 3.1 几何单源(描画 = 命中)
新增"页签条几何分配"纯函数(落 `host_page_pointer` 下的 owner 叶子,如 `tab_strip_geometry.rs`),输入 `(strip_width, page_count, active_index, metrics)`,输出每页签 `x/width` 与可见/溢出划分。描画(`template_segmented_controls/tabs.rs`)与命中(`handle_click.rs`)**都调用它**,消除现有 72/108 不一致。

### 3.2 分配算法(伪码)
```text
fn allocate_tabs(strip_w, n, active, M=metrics):
    avail = strip_w - 2*STRIP_X
    pref  = M.tab_pref_width            // 内容自适应,夹在 [TAB_MIN, TAB_MAX]
    // 1) 能否在 >= TAB_MIN 下放下全部?
    need_all = n*TAB_MIN + (n-1)*TAB_GAP
    if need_all <= avail:
        w = min(TAB_MAX, (avail-(n-1)*TAB_GAP)/n).max(TAB_MIN)
        return 全部可见, 每个宽 w
    // 2) 放不下 -> 预留溢出按钮,放尽量多且每个 >= TAB_MIN
    avail2 = avail - OVERFLOW_W - TAB_GAP
    k = floor((avail2 + TAB_GAP) / (TAB_MIN + TAB_GAP))   // 可见数
    k = max(k, 1)
    // 3) active 必须可见:若 active >= k,把 active 提到可见集末位
    visible = first k pages, 但保证 active ∈ visible(必要时换入)
    overflow = 其余页
    每个可见页宽 = clamp((avail2-(k-1)*TAB_GAP)/k, TAB_MIN, TAB_MAX)
    return visible(每个>=TAB_MIN) + overflow_button
```
要点:**绝不**把页签压到 `< TAB_MIN`;超宽改"少放 + 溢出";`active` 页恒在可见集;可见标签仍可能略宽于内容时由 `ellipsize` 收尾(`Scene…`)。

### 3.3 溢出弹层
末尾 `⋯`(溢出按钮)点击 → 经菜单指针路径弹出 tab-overflow 列表(每行一页,选中即激活并换入可见集),1px 边框、无阴影、行内边距 `8/3`(遵 `15` 复合契约)。键盘可达。

### 3.4 标签省略
可见页签标签统一走 `paint_text/draw/ellipsis.rs::ellipsize_single_line`(S15.2 已落地),`Scene Editor` 窄档降级 `Scene…` 而非 `Sce`。

## 4. 数据结构草案
```rust
// host_page_pointer_layout.rs
pub(crate) struct HostPagePointerLayout {
    pub strip_frame: UiFrame,
    pub tabs: Vec<HostPageTabSlot>,          // 仅可见页,含已分配 frame
    pub overflow: Option<HostPageOverflowSlot>,
}
pub(crate) struct HostPageTabSlot { pub page_index: usize, pub page_id: String, pub frame: UiFrame }
pub(crate) struct HostPageOverflowSlot { pub frame: UiFrame, pub hidden_page_indices: Vec<usize> }
```

## 5. 结构/债务纪律(遵优先文档)
- **E1 收口(顺手)**:`handle_click` 及相邻 `Result<_, String>` 改 `thiserror` 枚举 `HostPageTabError` + `pub type Result<T>`;不新增裸 `Result<_, String>`。(`§9.1` 关联 `engine-code-review-findings` F5-F7)
- **R1.4/R4.3**:几何分配落新 owner 叶子(`tab_strip_geometry.rs`),纯函数 + 内联测试 ≤150 行;`build_host_page_pointer_layout.rs` 不超长。
- **最小宽收敛**:统一 `TAB_MIN_WIDTH` 与 `workbench_tab.zui width.min`(择一值并双向对齐,建议页签条 108、子页签 88,文档注明语义差异),避免两处漂移。
- 新 owner 同变更迁移调用方、删旧自钳逻辑,不留双轨。

## 6. 测试矩阵
| 测试 | 断言 |
| --- | --- |
| `all_tabs_fit_when_strip_wide` | N 小 + 宽 strip → 全可见,无溢出,每宽 ∈ [MIN, MAX] |
| `tabs_clamp_to_min_and_overflow_when_narrow` | N 大 + 窄 strip → 可见数符合 floor 公式,每可见宽 ≥ MIN,出现 overflow |
| `active_tab_always_visible` | active 在尾部 → 被换入可见集 |
| `paint_and_hit_geometry_match` | 命中用的 per-tab frame == 描画用的 frame(同一分配函数) |
| `label_ellipsizes_not_hard_clips` | 窄可见页签标签以 `…` 结尾(走 S15.2) |
| `handle_click_returns_typed_error` | 越界/无效 index → `HostPageTabError`,非裸 String |

## 7. 验收
- `editor-window-m3-workbench-900x620` 截图:页签为全称或 `Scene…`,**不再 `Sce`**;超出部分进 `⋯`。
- `editor-window-m3-svg-icon-scale-small-640x420`:窄档可见页签 ≥ MIN,溢出按钮出现。
- 人工:点击窄页签命中正确(描画=命中)。
- 命令:`cargo test -p zircon_editor --lib host_page_pointer`(几何/命中)+ `capture_m3_gui_acceptance_visual_artifacts --ignored` 刷新 `docs/tests/editor/`(不进 target)。

## 8. 实现顺序
1. 新增 `tab_strip_geometry.rs` 纯函数 + 单测(RED→GREEN)。
2. `build_host_page_pointer_layout.rs` 改用它产出 `tabs/overflow`。
3. 描画 `template_segmented_controls/tabs.rs` 与命中 `handle_click.rs` 改读同源 frame;删自钳;标签接 `ellipsize`。
4. `handle_click` 等 `Result<_,String>` → typed error(E1 顺手)。
5. 溢出弹层接菜单指针路径。
6. 跑测试 + 截图验收,写状态。

## 9. 边界
- 不做 `07` 的窗口化/吸附/Chrome 合并(那是 07);本文只到"页签条几何 + 溢出收纳 + 命中一致 + 标签省略"。
- 不改页签的 `.zui` 视觉族色彩(走 `15`/`01` 色板单源)。

## 10. 状态与产出记录
| 日期 | 切片 | 状态 | 产出/证据 | 后续项 |
| --- | --- | --- | --- | --- |
| 2026-06-25 | 15a/S15.3 深化立项 | planned | 核实根因:`host_page_pointer` 几何只给 strip_frame,描画侧等分 strip 宽跌破 `TAB_MIN_WIDTH=108` → `fontdue` 硬裁成 `Sce`;`handle_click` 自钳 108 致描画/命中不一致且返回裸 `Result<_,String>`(E1);`TAB_MIN_WIDTH=108` 与 `workbench_tab.zui min=88` 漂移。给出几何单源 + 溢出算法 + typed error 收口 + 测试矩阵。 | 按 §8 顺序实现;父计划 `15` 的 S15.3 勾稽随之更新。 |
| 2026-06-25 | 15a/S15.3a host 页签度量与点击一致首段 | implemented-focused-passed-screenshot-passed | 新增 `ui/workbench/page_tabs/metrics.rs` 作为主页面页签 min/max/gap/overflow/preferred 宽度事实源,并让 `host_page_pointer` 几何、layout 构建和 `chrome_template_projection` 消费同一 owner;`host_page_pointer/error.rs` 将本路径裸 string error 收口到 typed error;点击处理改用实际 callback frame,与描画 slot 保持同坐标系。验证:`fallback_page_chrome_keeps_medium_width_tabs_readable_before_overflow`、`shared_host_page_pointer_bridge_routes_tabs_from_shared_hit_test`、`root_host_page_pointer_click_uses_shared_projection_tab_slot` 均 1/1 通过;整窗截图 harness 1/1 通过并刷新 `docs/tests/editor`。 | 该行只关闭几何/typed error 首段;完整页签 overflow popup 已由后续 S15.3b 行关闭。实际 900x620 顶部 `Sc...` 根因同时来自 Workbench 模块工具栏,已在父计划 S15.4h/S15.4i 单独处理。 |
| 2026-06-25 | 15a/S15.3b host 页签 overflow 弹层/隐藏页选择列表 | implemented-focused-passed-screenshot-passed-build-passed | 新增 `host_page_pointer/handle_overflow_click.rs` 和 `HostPagePointerRoute::Overflow`,让共享 pointer bridge 能从 overflow 按钮进入隐藏页路由;新增 `host_contract/host_page_overflow_menu.rs`、`data/host_interaction/page_overflow.rs`、`native_pointer/button_dispatch/page_overflow_menu.rs` 与 `paint_workbench_renderer/scene_layers/overlay/page_overflow.rs`,把弹层几何、open/hover state、native 点击/外部关闭和软件绘制拆到小 owner。`shell_chrome/host_page.rs` 只负责打开/关闭 popup 与复用真实 `host_page_pointer_clicked` 激活隐藏页。验证:`cargo build -p zircon_editor --locked --jobs 1 --message-format short --color never` 通过(仅既有 warning);`cargo fmt -p zircon_editor --check`;`cargo test -p zircon_editor overflow --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture` 15 passed/2 ignored;`capture_host_page_overflow_menu_visual_artifact --ignored` 1/1,生成 `docs/tests/editor/editor-window-m3-host-page-overflow-420x260.png`,构建产物使用 `D:\cargo-targets\zircon-editor-components-0625`,截图未写入 repo `target`。 | host 页签 popup/选择列表闭环;仍需把 640/900/1260 `LayoutTier` 强制策略接入页签 overflow、token 化 popup anchor,并刷新完整 640/1260 窗口截图。 |
| 2026-06-25 | 15a/S15.3c host 页签 LayoutTier overflow 联动 | implemented-focused-passed-screenshot-passed-build-passed | `ui/workbench/page_tabs/metrics.rs` 继续作为主页签度量 owner,新增 project-path 预留宽度、Narrow tier 可见页签 cap 与 overflow popup 宽度 token;`chrome_template_projection.rs` 和 `host_page_pointer/tab_strip_geometry.rs` 共用 `main_page_project_path_width(...)` 与 `main_page_tab_visible_cap_for_width(...)`,因此 640 窄档先保留 2 个可读主页面 tab 并收纳其余页,active tab 必须可见,1260 宽档在能容纳时不再强行显示 overflow;`host_page_overflow_menu.rs` 删除本地 popup width,改消费 `MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH`。验证:`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never` 通过;`fallback_page_chrome_narrow_tier_caps_visible_tabs_before_project_path`、`fallback_page_chrome_wide_tier_does_not_force_overflow_when_tabs_fit`、`narrow_tier_caps_visible_tabs_before_overflow` 均 1/1 通过;`capture_m3_gui_acceptance_visual_artifacts --ignored` 1/1 并刷新 `docs/tests/editor` 下完整 M3 截图;最终 `cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never` 通过。更宽 `page_chrome` harness 编译超时未产出测试结果,不计通过。 | 页签 overflow 的断点联动首段关闭;popup anchor 余量 token 化、键盘/hover 深化和跨窗口更多组合验收仍 pending。 |
| 2026-07-11 | 15a/S15.3d host 页签 overflow 键盘可达 | implemented-focused-passed-screenshot-passed | procedural host-page popup 复用共享 `PopupKeyboardTarget`:Down/Up 首次选择与循环、前缀搜索、Enter 激活和 Escape 关闭均通过 `UiHostContext` 回到既有页面回调,未增加页签专用按键分支。首次真实 RED 为 0/2,定位到测试 fixture 只写 presentation DTO、未写全局交互状态;按既有 pointer fixture 的状态所有权修正后 GREEN 为 2 passed/0 failed/1 ignored。共享 dropdown/menu/Home-End 回归各 1/1。ignored capture 1/1,生成 `docs/tests/editor/editor-window-m3-host-page-overflow-keyboard-640x420.png`(8944 bytes,SHA256 `14EB302814E3CC48E3AD9BFF0C4385E38A07A9153B0B9DA5A327AEDF2A508681`);repo `target` 与 D/E/F cargo target 同名图均 0。 | 键盘路径关闭;S15.3 仍保留完整页签收纳、长列表滚动和整窗视觉深化。 |
| 2026-07-31 | 15a/S15.3e host 页签 overflow 长列表自适应与热路径收敛 | implemented_static / validation_pending | popup 使用 Runtime Text 投影阶段缓存的最宽隐藏标题,运行时几何不再逐项重复测量;自然宽在需要滚动时与 renderer 共用 `scrollbar_thickness + gap` 预算。纵向布局按 anchor 上下真实可用空间翻转/限高,内容视口、滚轮限幅、键盘 reveal、可见行闭开区间、绘制 clip、hover 与点击命中共用同一几何;scrollbar gutter 不再穿透激活页面,滚轮通过显式候选 offset 重算 stationary hover,不再 clone 整份 presentation。偏移 shell 使用真实 `x + width` 边界,不足一个共享 edge inset 的幽灵视口直接拒绝;Runtime Text 行高保留 fractional token。独立二次审查提出滚动条宽度预算、O(N) 文本测量、presentation clone、gutter 穿透与 1px 视口五项问题,均已前向修复;`rustfmt --check`、`git diff --check`、旧重复 popup contains/row-hit 路径零残留及审查 finding 静态门通过。新增/扩展 focused 源回归覆盖 long-list scroll/keyboard reveal/pointer hover/scroll retarget/visible range/scrollbar gutter/non-finite offset/offset shell/cache width;本轮未执行 Cargo,这些测试不声明运行通过。 | 保持本计划 `in_progress`;等待受管当前源码验证后刷新 `docs/tests/editor` 下长列表与 640/900/1260 整窗图,不使用旧二进制、不写 repo `target`。 |
