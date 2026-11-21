# 两个左侧抽屉回归未进入取消手势

**实际失败点。** `zircon_editor` 原生 `--lib` 中的 `root_native_focus_loss_cancels_drawer_resize_without_persisting_transient_extent` 以及 `root_hide_close_cancels_drawer_resize_without_persisting_transient_extent` 都在按下指针前，检查 `left_splitter_frame.width > 0 && height > 0` 时失败。此时没有活动 resize，不能据此断言失焦、主窗关闭或迟到的 Up 处理有缺陷。

**独立复核。** Windows x64/MSVC、Rust 1.94.1、test profile、无默认特性。对两个已编译的 `zircon_editor` libtest 各自用两个独立进程执行 `ui::retained_host::app::tests::projection_geometry::<用例名> --exact --test-threads 1 --show-output`：

| 编译来源 | PE SHA-256 | 失焦 / 关闭 |
|---|---|---|
| 不含 #258 的 `1fde8273c28a5624de2fbbe6f8296feff5126e52`，其夹具、测试宿主和 drawer resize 文件与当时主干相同 | `388bb3289e11f70c4abd3846b13b39d67142a71e4030578248341969051f5777` | 两案均在上述几何断言退出 101，未派发 Down |
| #258 实测源 `402697de6a0140a7bfd8345def265b3d1f0c2185` | `357b453f9b2346b37f6584e27c2cb8173cce705ca4384709f08c6b85407a28b1` | 同一断言，各退出 101；同程序中 100%/150% 的 **Bottom 正常 Up** 两案通过 |

本次没有重编主干，不把这两份既有程序称为新主干产品。代码位于[两项已有测试](../../../zircon_editor/src/ui/retained/app/tests/projection_geometry.rs)与[测试宿主](../../../zircon_editor/src/ui/retained/app/tests/support.rs)：进入 Workbench 与激活、固定具体抽屉实例是分开的操作。下一步先确认生产布局实际拥有的左侧分隔条和何种挂载状态能提供它，再决定修夹具还是投影，不能用伪造矩形或删除断言让用例变绿。

**对照及验收。** UE ToolWidgets 的 `SWidgetDrawer.cpp` 内 `SDrawerOverlay::OnMouseButtonDown` 只在真实 resize handle 被命中时捕获；这不证明 UE 的失焦/关闭等同于 ZE。ZE 保留 retained host、窗口事件桥和布局命令。验收应从真实已发布且非零的目标开始，按下、移动、分别触发失焦/关闭，证明捕获解除、临时宽度消失、未提交尺寸、迟到 Up 不再生效，并保留合适的正常 Up 对照。与 #258 的 Bottom 松手逻辑单位无共用修复。

## 夹具修订与待验范围

已核对[正式 Level Editor 模板](../../../zircon_editor/assets/ui/editor/unreal/level_editor_window.zui)：中部是 Viewport 和右侧 Outliner / Details，底部导入 `BottomDrawerShellRoot`，整套 Unreal 视图模板没有声明 `LeftDrawerShellRoot`。左侧尺寸是可选帧，缺失时发布零矩形；单纯在布局模型中固定一个 Left tab 不会新建该视图控件。独立原生对照 `root_resize_capture_prefers_workbench_left_drawer_shell_extent_over_stale_region_geometry` 在取得左侧 shell 就失败，甚至尚未读取 splitter。

两项取消测试现在固定实际的 Bottom Console，用已发布的 splitter 中心派发 Down 和向上 Move，要求捕获归属根窗口的 Bottom、临时高度改变而持久偏好未变。分别触发失焦与关闭，检查临时值和捕获均清除、无布局命令、公布的实体仍是原 Console 且高度恢复；迟到 Up 不得提交。保留失焦后的重新捕获和关闭后的最终退出检查。不制造假 Left 几何，不改业务布局或取消逻辑，不声称此改动为左侧抽屉提供产品支持。

修订源码已通过 Rust 1.94.1 单文件 `rustfmt --check`、diff 空白检查及真实模板导入/控件核对。**两条修订后的原生取消测试尚未执行，保持 Draft**。容量就绪后，在当前 PR 与最新 `main` 的组合树中对 `zircon_editor --lib` 无默认特性、test profile、Windows x64/MSVC，分别用上文两个名称精确执行，并一并回归 #258 的 Bottom 正常松手对照。已有正常 Up 的绿测不是这两条 Cancel 的证据。
