# 核对提示框时序由运行时输入管理器控制，并处理窗口事件与弹窗遮挡。
import unittest
from pathlib import Path


# 仅检查源码文本接线，不执行 Rust 输入派发、计时或渲染行为。
# 由本文件的第三个父目录定位仓库，物理读取位置不随启动目录变化。
REPO_ROOT = Path(__file__).resolve().parents[2]
RUNTIME_MANAGER = REPO_ROOT / (
    "zircon_runtime/src/ui/dispatch/input_manager/manager.rs"
)
RUNTIME_TIMERS = REPO_ROOT / (
    "zircon_runtime/src/ui/dispatch/input_manager/timers.rs"
)
RUNTIME_DISPATCH = REPO_ROOT / "zircon_runtime/src/ui/dispatch/mod.rs"
SURFACE_TIMERS = REPO_ROOT / (
    "zircon_runtime/src/ui/surface/surface/default_interactions/timers.rs"
)
RUNTIME_METADATA_DIRTY = REPO_ROOT / (
    "zircon_runtime/src/ui/surface/property_mutation/metadata_dirty.rs"
)
# 路径常量不在导入时读取；各测试方法会按自己的语句顺序重新读取。
WORKBENCH_TOOLTIP = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/icon_tooltip.rs"
)
WORKBENCH_TOOLTIP_RESOLVER = REPO_ROOT / (
    "zircon_editor/src/ui/runtime/tooltip.rs"
)
WORKBENCH_POINTER_DISPATCH = REPO_ROOT / (
    "zircon_editor/src/ui/retained/dispatch/workbench/pointer.rs"
)
WORKBENCH_WINDOW = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/windows/workbench_window.zui"
)
WORKBENCH_TOOLTIP_METRICS = REPO_ROOT / (
    "zircon_editor/src/ui/retained/contract/paint/"
    "template_tooltips/metrics.rs"
)
WORKBENCH_POINTER_TESTS = REPO_ROOT / (
    "zircon_editor/src/tests/host/callbacks/templates/"
    "workbench_pointer_preview.rs"
)
PLATFORM_INPUT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/"
    "platform_input.rs"
)
POINTER_EVENTS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/"
    "events/pointer.rs"
)
KEYBOARD_EVENTS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/"
    "events/keyboard.rs"
)
WINDOW_EVENTS = REPO_ROOT / (
    "zircon_editor/src/ui/retained/contract/window/event_loop/events.rs"
)
HOST_CALLBACKS = REPO_ROOT / (
    "zircon_editor/src/ui/retained/contract/globals/callbacks/host.rs"
)
HOST_CALLBACK_METHODS = REPO_ROOT / (
    "zircon_editor/src/ui/retained/contract/globals/ui_context.rs"
)
HOST_TOOLTIP = REPO_ROOT / (
    "zircon_editor/src/ui/retained/app/tooltip.rs"
)
NATIVE_TOOLTIP_TARGET = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/native_pointer/"
    "tooltip_target.rs"
)
NATIVE_POINTER_ROOT = REPO_ROOT / (
    "zircon_editor/src/ui/retained/contract/pointer.rs"
)
NATIVE_POINTER_MOVE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/native_pointer/"
    "move_dispatch/entry/body.rs"
)
HOST_CALLBACK_WIRING = REPO_ROOT / (
    "zircon_editor/src/ui/retained/app/callback_wiring/host_shell/runtime.rs"
)
WINDOW_REDRAW = REPO_ROOT / (
    "zircon_editor/src/ui/retained/contract/window/redraw.rs"
)
EVENT_LOOP_LIFECYCLE = REPO_ROOT / (
    "zircon_editor/src/ui/retained/contract/window/event_loop/lifecycle.rs"
)


# unittest 按名称发现独立方法；一个方法的读取错误不等于其他方法未被调度。
class EditorZuiRuntimeTooltipContractTests(unittest.TestCase):
    # 检查输入管理器的外部提示框时序接线，不验证实际到期时间。
    def test_runtime_input_manager_owns_external_tooltip_timing(self):
        source = RUNTIME_MANAGER.read_text(encoding="utf-8")
        # 合并空白的文本仅供一项接线断言，其余断言仍检查原始文本。
        compact = " ".join(source.split())

        self.assertIn("pub fn arm_tooltip_candidate(", source)
        self.assertIn("self.timers .arm_tooltip_expiration(", compact)
        self.assertIn("pub fn dismiss_tooltip(", source)
        self.assertIn("self.timers.clear_tooltip_expirations();", source)
        self.assertIn("self.timers.arm_tooltip_intro(now);", source)
        self.assertIn("self.timers.expire_tooltip_intro(now);", source)
        self.assertIn("self.timers.clear_tooltip_intro();", source)

    # 检查候选更新与后续 tick 的文本分工；所有输入读完后才开始断言。
    def test_workbench_candidate_does_not_open_popup_before_runtime_tick(self):
        # BUG: [CR-ZCA-R074-0001] 首个旧物理路径缺失，这里先抛 FileNotFoundError。
        # 本方法共9次读取，后8次读取及34项断言均被遮蔽；后续缺失路径不会先报错。
        source = WORKBENCH_TOOLTIP.read_text(encoding="utf-8")
        resolver_source = WORKBENCH_TOOLTIP_RESOLVER.read_text(encoding="utf-8")
        window = WORKBENCH_WINDOW.read_text(encoding="utf-8")
        metrics = WORKBENCH_TOOLTIP_METRICS.read_text(encoding="utf-8")
        runtime_timers = RUNTIME_TIMERS.read_text(encoding="utf-8")
        runtime_dispatch = RUNTIME_DISPATCH.read_text(encoding="utf-8")
        surface_timers = SURFACE_TIMERS.read_text(encoding="utf-8")
        metadata_dirty = RUNTIME_METADATA_DIRTY.read_text(encoding="utf-8")
        pointer_tests = WORKBENCH_POINTER_TESTS.read_text(encoding="utf-8")
        # 以下按固定分隔串切片，不是 Rust 语法解析，也不能证明完整函数边界。
        update = source.split(
            "pub(crate) fn update_workbench_icon_tooltip_candidate(", 1
        )[1].split("pub(crate) fn next_workbench_icon_tooltip_delay(", 1)[0]
        tick = source.split("pub(crate) fn tick_workbench_icon_tooltip(", 1)[1]

        self.assertIn(".arm_tooltip_candidate(", update)
        self.assertNotIn("show_workbench_icon_tooltip", update)
        self.assertIn(".manager\n            .tick(", tick)
        self.assertIn("show_workbench_icon_tooltip", tick)
        self.assertIn("apply_workbench_icon_tooltip_extent", source)
        self.assertIn("apply_workbench_icon_tooltip_intro", source)
        self.assertIn("tooltip_intro_progress", source)
        self.assertIn("measure_runtime_text_width", source)
        self.assertIn("node.constraints.width", source)
        self.assertIn("mark_layout_dirty", source)
        current_candidate = source.split("if is_current {", 1)[1].split("}", 1)[0]
        self.assertIn("apply_workbench_icon_tooltip_extent", current_candidate)
        self.assertIn(
            "fn apply_workbench_icon_tooltip_extent(", source
        )
        self.assertIn("Result<bool, BuiltinHostWindowTemplateBridgeError>", source)
        self.assertIn("a visible tooltip should survive narrow-shell reflow", pointer_tests)
        self.assertIn("(96.0..=104.0).contains", pointer_tests)
        self.assertIn("tooltip_wrap_width", source)
        self.assertIn("tooltip_wrap_width = 1000.0", window)
        self.assertIn("layout_min_width = 96.0", window)
        self.assertIn('transition_kind = "fade"', window)
        self.assertIn("transition_duration_ms = 100", window)
        self.assertIn('transition_easing = "linear"', window)
        self.assertIn("TOOLTIP_MAX_WIDTH_IN_ROWS", metrics)
        self.assertIn("metrics.row_height * TOOLTIP_MAX_WIDTH_IN_ROWS", metrics)
        self.assertNotIn("const DEFAULT_TOOLTIP_DELAY_MS", source)
        self.assertIn("DEFAULT_TOOLTIP_DELAY_MS", source)
        self.assertNotIn(".filter(|delay_ms| *delay_ms > 0)", source)
        resolver = resolver_source.split(
            "fn workbench_icon_tooltip_text(", 1
        )[1]
        self.assertNotIn('.get("enabled")', resolver)
        self.assertNotIn('.get("disabled")', resolver)
        self.assertIn("pub const DEFAULT_TOOLTIP_DELAY_MS: u64 = 150;", runtime_timers)
        self.assertIn(
            "pub const DEFAULT_TOOLTIP_INTRO_DURATION_MS: u64 = 100;",
            runtime_timers,
        )
        self.assertIn("DEFAULT_TOOLTIP_DELAY_MS", runtime_dispatch)
        self.assertNotIn("const DEFAULT_TOOLTIP_DELAY_MS", surface_timers)
        self.assertIn("unwrap_or(DEFAULT_TOOLTIP_DELAY_MS)", surface_timers)
        self.assertIn(
            "tooltip_intro_progress_and_status_are_render_only", metadata_dirty
        )

    # 检查原生输入到宿主回调的文本接线，不生成或派发指针事件。
    def test_native_mouse_events_preserve_standard_pointer_metadata(self):
        # BUG: [CR-ZCA-R074-0002] 首次读取仍指向缺失的旧物理路径，这里先抛 FileNotFoundError。
        # 后5次读取及8项断言不会到达；这是本方法自身的失败入口。
        platform = PLATFORM_INPUT.read_text(encoding="utf-8")
        pointer = POINTER_EVENTS.read_text(encoding="utf-8")
        window = WINDOW_EVENTS.read_text(encoding="utf-8")
        callbacks = HOST_CALLBACKS.read_text(encoding="utf-8")
        methods = HOST_CALLBACK_METHODS.read_text(encoding="utf-8")
        keyboard = KEYBOARD_EVENTS.read_text(encoding="utf-8")

        self.assertIn("window.normalized_cursor_move_input()?", platform)
        self.assertIn("window.normalized_pointer_cancel_input(point)?", platform)
        self.assertGreaterEqual(pointer.count("invoke_workbench_pointer_input"), 3)
        self.assertIn("invoke_workbench_pointer_input(pointer, None);", window)
        self.assertIn(
            "Option<Callback2<UiPointerInputEvent, Option<WorkbenchTooltipPointerTarget>>>",
            callbacks,
        )
        self.assertIn(
            "(pointer: UiPointerInputEvent, target: Option<WorkbenchTooltipPointerTarget>)",
            methods,
        )
        self.assertGreaterEqual(keyboard.count("invoke_workbench_input_activity"), 2)
        self.assertIn("on_workbench_input_activity", methods)

    # 检查锚点顺序、目标桥接及计时所有者的文本约束，不实例化窗口。
    def test_native_dock_tabs_use_the_runtime_tooltip_timer_through_a_real_zui_anchor(
        self,
    ):
        # BUG: [CR-ZCA-R074-0003] 首个窗口资源旧路径缺失，这里先抛 FileNotFoundError。
        # 后10次读取及39项断言被遮蔽，与其他测试方法的缺失读取相互独立。
        window = WORKBENCH_WINDOW.read_text(encoding="utf-8")
        target = NATIVE_TOOLTIP_TARGET.read_text(encoding="utf-8")
        native_root = NATIVE_POINTER_ROOT.read_text(encoding="utf-8")
        pointer_move = NATIVE_POINTER_MOVE.read_text(encoding="utf-8")
        pointer_events = POINTER_EVENTS.read_text(encoding="utf-8")
        callbacks = HOST_CALLBACKS.read_text(encoding="utf-8")
        methods = HOST_CALLBACK_METHODS.read_text(encoding="utf-8")
        wiring = HOST_CALLBACK_WIRING.read_text(encoding="utf-8")
        host = HOST_TOOLTIP.read_text(encoding="utf-8")
        bridge = WORKBENCH_TOOLTIP.read_text(encoding="utf-8")
        surface_dispatch = WORKBENCH_POINTER_DISPATCH.read_text(encoding="utf-8")

        # 只取首个分隔区段；嵌套结构和实际资源加载需由其他验证覆盖。
        root_children = window.split("children = [", 1)[1].split("]", 1)[0]
        self.assertIn('{ node = "host_chrome_tooltip_anchor" }', root_children)
        self.assertLess(
            root_children.index('node = "host_chrome_tooltip_anchor"'),
            root_children.index('node = "icon_button_tooltip"'),
        )
        anchor = window.split("[nodes.host_chrome_tooltip_anchor]", 1)[1].split(
            "[nodes.icon_button_tooltip]", 1
        )[0]
        self.assertIn('component = "Container"', anchor)
        self.assertIn('control_id = "WorkbenchHostChromeTooltipAnchor"', anchor)
        self.assertIn("input_interactive = false", anchor)
        self.assertIn("input_clickable = false", anchor)
        self.assertIn("input_hoverable = false", anchor)
        self.assertIn("input_focusable = false", anchor)

        self.assertIn("pub(crate) enum WorkbenchTooltipPointerTarget", target)
        self.assertIn("SurfaceNode(UiNodeId)", target)
        self.assertIn("HostChrome(HostChromeTooltipTarget)", target)
        self.assertIn("tooltip_target_for_chrome_route", target)
        self.assertIn("ChromePointerRoute::DocumentTab", target)
        self.assertIn("ChromePointerRoute::DrawerHeaderTab", target)
        self.assertIn("tab_frames.get(index)?", target)
        self.assertNotIn("iter().nth(index)", target)
        self.assertNotIn("route_top_level_chrome(", target)
        self.assertNotIn("contains(", target)
        self.assertIn("mod tooltip_target;", native_root)
        self.assertIn("route_top_level_chrome(structure, x, y)", pointer_move)
        self.assertIn("tooltip_target_for_chrome_route", pointer_move)
        self.assertIn("WorkbenchTooltipPointerTarget::HostChrome", pointer_move)

        self.assertIn("Option<WorkbenchTooltipPointerTarget>", pointer_events)
        self.assertIn("WorkbenchTooltipPointerTarget", callbacks)
        self.assertIn("WorkbenchTooltipPointerTarget", methods)
        self.assertIn("|pointer, tooltip_target|", wiring)
        self.assertIn(
            "observe_workbench_pointer_input(pointer, tooltip_target)", wiring
        )
        self.assertIn("WorkbenchTooltipPointerTarget", host)
        self.assertIn("WorkbenchTooltipPointerTarget", bridge)
        self.assertIn(
            ".map(WorkbenchTooltipPointerTarget::SurfaceNode)", surface_dispatch
        )
        self.assertIn(
            "update_workbench_icon_tooltip_candidate(tooltip_input, tooltip_target)",
            surface_dispatch,
        )
        self.assertIn("WORKBENCH_HOST_CHROME_TOOLTIP_ANCHOR_CONTROL_ID", bridge)
        self.assertIn(
            "host_chrome_tab_uses_the_runtime_delay_and_real_control_anchor", bridge
        )
        self.assertIn("set_popup_control_anchor", bridge)
        self.assertIn("UiInputManager", bridge)

        native_sources = "\n".join((target, native_root, pointer_move))
        self.assertNotIn("UiInputManager", native_sources)
        self.assertNotIn("Instant", native_sources)
        self.assertNotIn("deadline", native_sources.lower())
        self.assertIn("next_workbench_icon_tooltip_delay", host)

    # 检查独立唤醒槽接线；生产段只取首个测试配置标记之前的原始文本。
    def test_input_timer_has_an_independent_wait_until_slot(self):
        host = HOST_TOOLTIP.read_text(encoding="utf-8")
        redraw = WINDOW_REDRAW.read_text(encoding="utf-8")
        lifecycle = EVENT_LOOP_LIFECYCLE.read_text(encoding="utf-8")

        self.assertIn("pointer.metadata.timestamp", host)
        self.assertIn("set_input_timer_frame_update(deadline)", host)
        self.assertIn("fn take_due_input_timer_frame_wake(", redraw)
        self.assertIn("UiPerfScenario::IdleHover", redraw)
        self.assertIn("take_due_input_timer_frame_wake(now)", lifecycle)
        self.assertIn("input_timer_frame_wake_deadline()", lifecycle)
        production = host.split("#[cfg(test)]", 1)[0]
        self.assertNotIn("std::thread::sleep", production)

    # 计数和包含检查只约束源码标记，不证明弹窗遮挡的运行时效果。
    def test_host_popups_occlude_underlying_workbench_tooltips(self):
        host = HOST_TOOLTIP.read_text(encoding="utf-8")
        methods = HOST_CALLBACK_METHODS.read_text(encoding="utf-8")

        self.assertGreaterEqual(
            host.count("host_popup_occludes_workbench_tooltip()"), 2
        )
        self.assertIn("menu_state.open_menu_index >= 0", methods)
        self.assertIn("host_page_overflow_menu_state.open", methods)


# 直接执行时交给 unittest；也可从本测试目录显式发现并加载。
if __name__ == "__main__":
    unittest.main()
