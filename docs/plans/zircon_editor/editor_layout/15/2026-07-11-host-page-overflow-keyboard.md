---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/page_overflow.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard/dispatch/actions.rs
  - zircon_editor/src/tests/host/retained_window/host_page_overflow_keyboard.rs
  - docs/zircon_editor/ui/retained_host/host_contract/host_page_overflow_menu.md
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15a-page-tab-strip-overflow.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo check -p zircon_editor --tests --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-editor-overflow-keyboard-0711
  - cargo test -p zircon_editor host_page_overflow_keyboard --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-editor-overflow-keyboard-0711 -- --nocapture
  - direct editor test binary dropdown keyboard exact
  - direct editor test binary popup-menu keyboard exact
  - direct editor test binary popup boundary-key mapping exact
  - direct editor test binary capture_host_page_overflow_keyboard_visual_artifact --exact --ignored --nocapture
  - docs/tests/editor/editor-window-m3-host-page-overflow-keyboard-640x420.png
status: implemented-focused-passed-screenshot-passed
---

# Layout 15 S15.3d：Host Page Overflow 键盘可达

## 完成项目

- 将 procedural host-page overflow list 投影成共享 `PopupKeyboardTarget`,隐藏页索引保持唯一激活身份,标题进入统一前缀搜索。
- 共享 popup model 在没有当前行时让首次 Down/Home 选择第一行、Up/End 选择最后一行,不跳过首项;已有 authored option/menu 路径继续使用同一逻辑。
- hover、Enter、Escape 通过 `UiHostContext` 更新唯一交互状态并复用 `host_page_pointer_clicked`,未增加第二套页面选择模型或页签专用键盘 dispatcher。
- 新增 2 条行为回归和 1 条 ignored screenshot route;截图固定写入 `docs/tests/editor`,不写入任何 target。

## RED → GREEN 证据

首次当前源码 lib-test 构建完成后实际执行为 `0 passed / 2 failed / 1 ignored`;两个失败都发生在第一次方向键的 `request_redraw()`。最低层诊断确认测试 fixture 只把 open state 写入 presentation DTO,随后被 `UiHostContext` 的默认关闭状态覆盖。fixture 按既有 pointer 测试的状态所有权同时更新全局交互状态后,同一 filter 为 `2 passed / 0 failed / 1 ignored / 2996 filtered out`。

该修正只修测试夹具的权威状态写入,没有放宽 production target discovery。完整 `cargo check -p zircon_editor --tests --locked --offline` 在此之前已通过;聚焦 GREEN 的增量构建耗时 19分35秒,只产生既有 warning。

## 回归与视觉证据

- dropdown keyboard exact：`1 passed / 0 failed`。
- popup-menu keyboard exact：`1 passed / 0 failed`。
- Home/End command mapping exact：`1 passed / 0 failed`。
- ignored screenshot exact：`1 passed / 0 failed`,输出 `640×420`,8944 bytes,SHA256 `14EB302814E3CC48E3AD9BFF0C4385E38A07A9153B0B9DA5A327AEDF2A508681`。
- 同名文件扫描：`docs/tests/editor = 1`;repo `target`、`D:\cargo-targets`、`E:\cargo-targets`、`F:\cargo-targets` 均为 0。
- 人工复核：局部 popup 有紧凑三行、1px 暗边框、单行键盘 hover 与无阴影表面;整张 fixture 背景仍大面积为空,因此本记录只验收 overflow list 局部组件,不声明整窗视觉达到参考图。

## 未完成项目

- S15.3 总项继续开放：完整页签收纳、长 overflow list 滚动、更多断点组合与整窗视觉深化尚未完成。
- Layout 15 和当前 `/goal` 保持 active,本记录不代表编辑器布局总体完成。
