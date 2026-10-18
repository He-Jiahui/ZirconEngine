from __future__ import annotations

import re
import subprocess
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
BROWSER_SOURCE = REPO_ROOT / (
    "zircon_runtime/src/plugin/export_build_plan/platform_host_files/browser.rs"
)
MOBILE_SOURCE = REPO_ROOT / (
    "zircon_runtime/src/plugin/export_build_plan/platform_host_files/mobile.rs"
)


class ExportHostInputCoalescingContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.browser = BROWSER_SOURCE.read_text(encoding="utf-8")
        cls.mobile = MOBILE_SOURCE.read_text(encoding="utf-8")
        cls.android = cls.mobile.split("fn android_activity_template", 1)[1].split(
            "fn android_runtime_binding_template", 1
        )[0]
        cls.ios = cls.mobile.split("fn ios_host_template", 1)[1]

    def test_browser_moves_and_metrics_flush_once_per_animation_frame(self) -> None:
        required = (
            "const zirconExportPendingPointerMoves = new Map();",
            "requestAnimationFrame((now) =>",
            "zirconExportMeasureMainThread(() => zirconExportFlushFrameInput(now))",
            "event.getCoalescedEvents?.()",
            "zirconExportQueuePointerMove(event)",
            "zirconExportQueueViewportMetrics()",
            "canvas.addEventListener('pointerdown'",
            "canvas.setPointerCapture?.(event.pointerId)",
            "window.addEventListener('pointerup'",
            "window.addEventListener('pointercancel'",
            "zirconExportCancelActivePointers()",
            'style=\\"touch-action: none\\"',
        )
        for anchor in required:
            self.assertIn(anchor, self.browser)

        self.assertNotIn(
            "canvas.addEventListener('pointermove', (event) => "
            "zirconExportDispatchPointer(event.pointerId, 'moved'",
            self.browser,
        )
        self.assertNotIn(
            "window.addEventListener('resize', zirconExportDispatchViewportMetrics)",
            self.browser,
        )
        self.assertNotIn("canvas.addEventListener('pointerup'", self.browser)
        self.assertNotIn("canvas.addEventListener('pointercancel'", self.browser)

    def test_android_moves_and_metrics_use_one_view_frame_callback(self) -> None:
        required = (
            "private val pendingTouchMoves = linkedMapOf<Long, PendingTouchMove>()",
            "private val frameInputCallback = Runnable",
            "frameView.postOnAnimation(frameInputCallback)",
            "frameView.removeCallbacks(frameInputCallback)",
            "MotionEvent.ACTION_MOVE -> queueTouchMoves(event)",
            "val moveIterator = pendingTouchMoves.entries.iterator()",
            "dispatchPendingTouchMove(pending, now)",
            "queueViewportMetrics(width, height, resources.displayMetrics.density)",
            "MotionEvent.ACTION_CANCEL -> dispatchCancelledTouches(event)",
            "measureInputMainThread",
        )
        for anchor in required:
            self.assertIn(anchor, self.mobile)

    def test_android_pause_is_a_synchronous_input_lifecycle_boundary(self) -> None:
        required = (
            "private const val ZIRCON_LIFECYCLE_SUSPENDED = 8",
            "override fun onPause()",
            "measureInputMainThread { suspendInput() }",
            "private fun suspendInput()",
            "lastTouchPositions.entries.sortedBy { it.key }",
            "ZirconRuntime.dispatchTouch(pointerId, ZIRCON_TOUCH_CANCELLED",
            "ZirconRuntime.dispatchLifecycle(ZIRCON_LIFECYCLE_SUSPENDED)",
            "private fun resumeInput()",
            "private fun queueCurrentViewportMetrics()",
            "if (!frameView.isLaidOut || frameView.isLayoutRequested)",
            "if (!isInputActive || width <= 0 || height <= 0)",
        )
        for anchor in required:
            self.assertIn(anchor, self.android)

        pause = self.android.split("override fun onPause()", 1)[1].split(
            "override fun", 1
        )[0]
        self.assertLess(pause.index("suspendInput()"), pause.index("super.onPause()"))
        suspend = self.android.split("private fun suspendInput()", 1)[1].split(
            "private fun resumeInput()", 1
        )[0]
        self.assertLess(suspend.index("flushFrameInput()"), suspend.index("isInputActive = false"))
        self.assertLess(
            suspend.index("ZIRCON_TOUCH_CANCELLED"),
            suspend.index("ZIRCON_LIFECYCLE_SUSPENDED"),
        )
        resume = self.android.split("private fun resumeInput()", 1)[1].split(
            "private inline fun measureInputAbi", 1
        )[0]
        self.assertLess(
            resume.index("ZIRCON_LIFECYCLE_RESUMED"),
            resume.index("queueCurrentViewportMetrics()"),
        )
        current_viewport = self.android.split(
            "private fun queueCurrentViewportMetrics()", 1
        )[1].split("private fun queueViewportMetrics", 1)[0]
        self.assertLess(
            current_viewport.index(
                "if (!frameView.isLaidOut || frameView.isLayoutRequested)"
            ),
            current_viewport.index("queueViewportMetrics("),
        )

    def test_ios_moves_and_metrics_use_a_paused_display_link(self) -> None:
        required = (
            "private var pendingTouchMoves: [ObjectIdentifier: PendingTouchMove] = [:]",
            "private var pendingTouchOrder: [ObjectIdentifier] = []",
            "private var nextTouchId: UInt64 = 1",
            "private final class ZirconDisplayLinkTarget: NSObject",
            "@objc func tick(_ displayLink: CADisplayLink)",
            "target: displayLinkTarget",
            "selector: #selector(ZirconDisplayLinkTarget.tick(_:))",
            "displayLink?.isPaused = false",
            "UIApplication.willResignActiveNotification",
            "UIApplication.didBecomeActiveNotification",
            "suspendInput()",
            "orderedTouches(touches)",
            "override func touchesBegan",
            "override func touchesEnded",
            "override func touchesCancelled",
            "queueViewportMetrics()",
            "isMultipleTouchEnabled = true",
            "private func configureInput()",
        )
        for anchor in required:
            self.assertIn(anchor, self.mobile)
        self.assertEqual(self.ios.count("configureInput()"), 3)
        self.assertNotIn("UInt64(touch.hash)", self.mobile)
        self.assertNotIn("pendingTouchMoves.keys.first", self.mobile)

    def test_every_host_tracks_the_shared_input_pressure_counters(self) -> None:
        counters = (
            "inputEventsReceived",
            "inputEventsCoalesced",
            "inputEventsDispatched",
            "inputRawDeltaX",
            "inputRawDeltaY",
            "maxInputQueueAge",
            "inputAbiWallTime",
            "inputMainThreadWallTime",
        )
        for counter in counters:
            self.assertIn(counter, self.browser)
            self.assertGreaterEqual(
                self.mobile.count(counter),
                2,
                f"Android and iOS must both publish {counter}",
            )

    def test_generated_browser_scheduler_bounds_125_500_1000_hz_dispatch(self) -> None:
        function = self.browser.split(
            "fn browser_runtime_host_script_template", 1
        )[1]
        raw_chunks = re.findall(r'r#"(.*?)"#', function, flags=re.DOTALL)
        self.assertEqual(len(raw_chunks), 3)
        generated = (
            raw_chunks[0]
            + "const adapter = null;\n"
            + raw_chunks[1]
            + "wasm"
            + raw_chunks[2]
        )
        harness = _browser_harness_prefix() + generated + _browser_harness_assertions()
        completed = subprocess.run(
            ["node", "--input-type=module"],
            input=harness,
            text=True,
            capture_output=True,
            cwd=REPO_ROOT,
            check=False,
        )
        self.assertEqual(
            completed.returncode,
            0,
            f"generated browser scheduler failed:\n{completed.stdout}\n{completed.stderr}",
        )
        self.assertIn("PLUGIN09_BROWSER_INPUT_COALESCING_GREEN", completed.stdout)

    def test_mobile_reference_scheduler_bounds_multi_pointer_dispatch(self) -> None:
        for frequency in (125, 500, 1000):
            dispatches, raw_delta = _simulate_mobile_frame_scheduler(
                frequency=frequency,
                pointer_ids=(11, 29),
                frames=60,
            )
            self.assertEqual(dispatches, 120)
            self.assertEqual(raw_delta, (frequency * 2, frequency * 4))


def _browser_harness_prefix() -> str:
    return r"""
const canvasListeners = new Map();
const windowListeners = new Map();
const pendingFrames = new Map();
const eventLog = [];
const capturedPointers = new Set();
let nextFrameId = 1;

const mockCanvas = {
    addEventListener(name, callback) { canvasListeners.set(name, callback); },
    getBoundingClientRect() { return { width: 1280, height: 720 }; },
    setPointerCapture(pointerId) { capturedPointers.add(pointerId); },
    hasPointerCapture(pointerId) { return capturedPointers.has(pointerId); },
    releasePointerCapture(pointerId) { capturedPointers.delete(pointerId); },
};
globalThis.document = { querySelector() { return mockCanvas; } };
globalThis.location = { href: 'https://example.invalid/game/' };
Object.defineProperty(globalThis, 'navigator', {
    value: {},
    configurable: true,
});
globalThis.requestAnimationFrame = (callback) => {
    const id = nextFrameId++;
    pendingFrames.set(id, callback);
    return id;
};
globalThis.cancelAnimationFrame = (id) => pendingFrames.delete(id);
globalThis.fetch = async () => ({
    json: async () => ({
        wasmModule: './runtime.wasm',
        allowedAssetRoot: './assets/',
    }),
    arrayBuffer: async () => new ArrayBuffer(0),
    body: null,
});
Object.defineProperty(WebAssembly, 'instantiateStreaming', {
    configurable: true,
    value: async () => ({
        instance: {
            exports: {
                zircon_export_start() { return true; },
                zircon_export_shutdown() { eventLog.push(['shutdown']); return true; },
                zircon_export_handle_lifecycle(state) { eventLog.push(['lifecycle', state]); return true; },
                zircon_export_handle_touch(pointerId, phase, x, y) {
                    eventLog.push(['touch', Number(pointerId), phase, x, y]);
                    return true;
                },
                zircon_export_handle_keyboard(action) { eventLog.push(['keyboard', action]); return true; },
                zircon_export_handle_viewport_metrics(width, height, scale) {
                    eventLog.push(['viewport', width, height, scale]);
                    return true;
                },
            },
        },
    }),
});
globalThis.window = {
    devicePixelRatio: 1,
    zirconRuntime: {
        handleTouch() {},
        handlePointerMotion(sample) { eventLog.push(['motion', sample.deltaX, sample.deltaY]); },
        handleKeyboard() {},
        handleViewportMetrics() {},
        handleLifecycle() {},
    },
    addEventListener(name, callback) {
        const callbacks = windowListeners.get(name) ?? [];
        callbacks.push(callback);
        windowListeners.set(name, callbacks);
    },
};
"""


def _browser_harness_assertions() -> str:
    return r"""
function assert(condition, message) {
    if (!condition) throw new Error(message);
}
function dispatchWindow(name, event = {}) {
    for (const callback of windowListeners.get(name) ?? []) callback(event);
}
function flushAnimationFrame() {
    const callbacks = Array.from(pendingFrames.values());
    pendingFrames.clear();
    for (const callback of callbacks) callback(performance.now());
}
function resetTelemetry() {
    eventLog.length = 0;
    for (const key of Object.keys(window.zirconExportHost.inputTelemetry)) {
        window.zirconExportHost.inputTelemetry[key] = 0;
    }
}
function pointerMove(pointerId, ordinal) {
    canvasListeners.get('pointermove')({
        pointerId,
        clientX: ordinal,
        clientY: ordinal * 2,
        movementX: 1,
        movementY: 2,
        getCoalescedEvents: () => [],
    });
}

for (const frequency of [125, 500, 1000]) {
    resetTelemetry();
    let emitted = 0;
    for (let frame = 0; frame < 60; frame += 1) {
        const frameTarget = Math.round(((frame + 1) * frequency) / 60);
        while (emitted < frameTarget) pointerMove(7, ++emitted);
        flushAnimationFrame();
    }
    const telemetry = window.zirconExportHost.inputTelemetry;
    const touches = eventLog.filter((entry) => entry[0] === 'touch');
    assert(emitted === frequency, `expected ${frequency} samples, got ${emitted}`);
    assert(touches.length === 60, `${frequency} Hz dispatched ${touches.length} moves`);
    assert(telemetry.inputEventsReceived === frequency, 'received counter drift');
    assert(telemetry.inputEventsCoalesced === frequency - 60, 'coalesced counter drift');
    assert(telemetry.inputEventsDispatched === 60, 'dispatch counter drift');
    assert(telemetry.inputRawDeltaX === frequency, 'raw delta X was not accumulated');
    assert(telemetry.inputRawDeltaY === frequency * 2, 'raw delta Y was not accumulated');
}

resetTelemetry();
canvasListeners.get('pointerdown')({ pointerId: 11, clientX: 0, clientY: 0 });
assert(capturedPointers.has(11), 'pointerdown must explicitly capture the pointer');
for (let index = 1; index <= 10; index += 1) pointerMove(11, index);
dispatchWindow('pointerup', { pointerId: 11, clientX: 10, clientY: 20 });
const touchPhases = eventLog.filter((entry) => entry[0] === 'touch').map((entry) => entry[2]);
assert(JSON.stringify(touchPhases) === JSON.stringify([1, 2, 3]), 'captured move must precede window pointer-up');
assert(pendingFrames.size === 0, 'edge drain must cancel the queued animation frame');
assert(!capturedPointers.has(11), 'pointer-up must release explicit capture');

resetTelemetry();
pointerMove(13, 1);
dispatchWindow('keydown', { code: 'KeyW', key: 'w' });
assert(eventLog[0][0] === 'touch' && eventLog[1][0] === 'motion', 'move must drain before key');
assert(eventLog[2][0] === 'keyboard', 'keyboard edge ordering drifted');

resetTelemetry();
for (let index = 0; index < 100; index += 1) dispatchWindow('resize');
flushAnimationFrame();
assert(eventLog.filter((entry) => entry[0] === 'viewport').length === 1, 'resize was not frame-coalesced');
assert(window.zirconExportHost.inputTelemetry.inputEventsCoalesced === 99, 'resize coalescing counter drift');

resetTelemetry();
canvasListeners.get('pointerdown')({ pointerId: 17, clientX: 1, clientY: 2 });
pointerMove(17, 2);
dispatchWindow('pagehide', { persisted: true });
const lifecycleTail = eventLog.filter((entry) => entry[0] === 'touch' || entry[0] === 'lifecycle');
assert(lifecycleTail.at(-2)[0] === 'touch' && lifecycleTail.at(-2)[2] === 4, 'pagehide must cancel active pointers');
assert(lifecycleTail.at(-1)[0] === 'lifecycle' && lifecycleTail.at(-1)[1] === 8, 'cancel must precede suspended lifecycle');
assert(!capturedPointers.has(17), 'lifecycle boundary must release pointer capture');
assert(Number.isFinite(window.zirconExportHost.inputTelemetry.inputMainThreadWallTime), 'main-thread wall telemetry missing');

console.log('PLUGIN09_BROWSER_INPUT_COALESCING_GREEN');
"""


def _simulate_mobile_frame_scheduler(
    *, frequency: int, pointer_ids: tuple[int, ...], frames: int
) -> tuple[int, tuple[int, int]]:
    pending: dict[int, tuple[int, int]] = {}
    dispatched = 0
    raw_x = 0
    raw_y = 0
    emitted = 0
    for frame in range(frames):
        frame_target = round(((frame + 1) * frequency) / frames)
        while emitted < frame_target:
            emitted += 1
            for pointer_id in pointer_ids:
                delta_x, delta_y = pending.get(pointer_id, (0, 0))
                pending[pointer_id] = (delta_x + 1, delta_y + 2)
        for pointer_id in pointer_ids:
            delta_x, delta_y = pending.pop(pointer_id)
            raw_x += delta_x
            raw_y += delta_y
            dispatched += 1
    return dispatched, (raw_x, raw_y)


if __name__ == "__main__":
    unittest.main()
