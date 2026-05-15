#[cfg(feature = "profiling")]
use std::cell::Cell;
#[cfg(feature = "profiling")]
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiPerfScenario {
    Startup,
    IdleHover,
    Click,
    Drag,
    DrawerResize,
    AssetRefresh,
    ViewportImage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiPerfCounter {
    FrameDurationUs,
    SlowPathRebuildCount,
    RenderPathCount,
    PresentationRebuildCount,
    FullPaintCount,
    RegionPaintCount,
    PaintedPixels,
    RedrawFullFrame,
    RedrawRegion,
    DirtyLayout,
    DirtyPresentation,
    DirtyRender,
    DirtyPaintOnly,
    ChromeSnapshotCount,
    WorkbenchModelBuildCount,
    ChromeCommandFullRebuildCount,
    ChromeCommandPatchCount,
    SoftwareFallbackPresentCount,
    GpuUploadBytes,
    GpuDrawCalls,
    GpuVisibleCommands,
    GpuVisibleDrawItems,
    GpuBatchLayers,
    GpuBatchDependencies,
}

#[cfg(feature = "profiling")]
thread_local! {
    static CURRENT_SCENARIO: Cell<UiPerfScenario> = const { Cell::new(UiPerfScenario::Startup) };
}

#[cfg(feature = "profiling")]
pub(crate) struct UiPerfScenarioGuard {
    previous: UiPerfScenario,
}

#[cfg(not(feature = "profiling"))]
pub(crate) struct UiPerfScenarioGuard;

#[cfg(feature = "profiling")]
impl Drop for UiPerfScenarioGuard {
    fn drop(&mut self) {
        CURRENT_SCENARIO.with(|current| current.set(self.previous));
    }
}

#[cfg(feature = "profiling")]
pub(crate) struct UiPerfScenarioTimer {
    scenario: UiPerfScenario,
    start: Instant,
}

#[cfg(not(feature = "profiling"))]
pub(crate) struct UiPerfScenarioTimer;

#[cfg(feature = "profiling")]
impl Drop for UiPerfScenarioTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_micros().min(u64::MAX as u128) as f64;
        record_ui_perf_counter(self.scenario, UiPerfCounter::FrameDurationUs, elapsed);
    }
}

pub(crate) fn enter_ui_perf_scenario(scenario: UiPerfScenario) -> UiPerfScenarioGuard {
    #[cfg(feature = "profiling")]
    {
        let previous = CURRENT_SCENARIO.with(|current| {
            let previous = current.get();
            current.set(scenario);
            previous
        });
        UiPerfScenarioGuard { previous }
    }
    #[cfg(not(feature = "profiling"))]
    {
        let _ = scenario;
        UiPerfScenarioGuard
    }
}

pub(crate) fn time_ui_perf_scenario(scenario: UiPerfScenario) -> UiPerfScenarioTimer {
    #[cfg(feature = "profiling")]
    {
        UiPerfScenarioTimer {
            scenario,
            start: Instant::now(),
        }
    }
    #[cfg(not(feature = "profiling"))]
    {
        let _ = UiPerfCounter::FrameDurationUs;
        let _ = scenario;
        UiPerfScenarioTimer
    }
}

pub(crate) fn current_ui_perf_scenario() -> UiPerfScenario {
    #[cfg(feature = "profiling")]
    {
        CURRENT_SCENARIO.with(|current| current.get())
    }
    #[cfg(not(feature = "profiling"))]
    {
        UiPerfScenario::Startup
    }
}

pub(crate) fn record_current_ui_perf_counter(counter: UiPerfCounter, value: f64) {
    record_ui_perf_counter(current_ui_perf_scenario(), counter, value);
}

pub(crate) fn record_ui_perf_counter(scenario: UiPerfScenario, counter: UiPerfCounter, value: f64) {
    #[cfg(feature = "profiling")]
    {
        zircon_runtime::profile_counter!("editor", counter_name(scenario, counter), value);
    }
    #[cfg(not(feature = "profiling"))]
    {
        let _ = (scenario, counter, value);
    }
}

#[cfg(feature = "profiling")]
macro_rules! counter_name_for_prefix {
    ($counter:expr, $prefix:literal) => {
        match $counter {
            UiPerfCounter::FrameDurationUs => concat!($prefix, ".frame_duration_us"),
            UiPerfCounter::SlowPathRebuildCount => {
                concat!($prefix, ".slow_path_rebuild_count")
            }
            UiPerfCounter::RenderPathCount => concat!($prefix, ".render_path_count"),
            UiPerfCounter::PresentationRebuildCount => {
                concat!($prefix, ".presentation_rebuild_count")
            }
            UiPerfCounter::FullPaintCount => concat!($prefix, ".full_paint_count"),
            UiPerfCounter::RegionPaintCount => concat!($prefix, ".region_paint_count"),
            UiPerfCounter::PaintedPixels => concat!($prefix, ".painted_pixels"),
            UiPerfCounter::RedrawFullFrame => concat!($prefix, ".redraw_full_frame"),
            UiPerfCounter::RedrawRegion => concat!($prefix, ".redraw_region"),
            UiPerfCounter::DirtyLayout => concat!($prefix, ".dirty_layout"),
            UiPerfCounter::DirtyPresentation => concat!($prefix, ".dirty_presentation"),
            UiPerfCounter::DirtyRender => concat!($prefix, ".dirty_render"),
            UiPerfCounter::DirtyPaintOnly => concat!($prefix, ".dirty_paint_only"),
            UiPerfCounter::ChromeSnapshotCount => concat!($prefix, ".chrome_snapshot_count"),
            UiPerfCounter::WorkbenchModelBuildCount => {
                concat!($prefix, ".workbench_model_build_count")
            }
            UiPerfCounter::ChromeCommandFullRebuildCount => {
                concat!($prefix, ".chrome_command_full_rebuild_count")
            }
            UiPerfCounter::ChromeCommandPatchCount => {
                concat!($prefix, ".chrome_command_patch_count")
            }
            UiPerfCounter::SoftwareFallbackPresentCount => {
                concat!($prefix, ".software_fallback_present_count")
            }
            UiPerfCounter::GpuUploadBytes => concat!($prefix, ".gpu_upload_bytes"),
            UiPerfCounter::GpuDrawCalls => concat!($prefix, ".gpu_draw_calls"),
            UiPerfCounter::GpuVisibleCommands => concat!($prefix, ".gpu_visible_commands"),
            UiPerfCounter::GpuVisibleDrawItems => concat!($prefix, ".gpu_visible_draw_items"),
            UiPerfCounter::GpuBatchLayers => concat!($prefix, ".gpu_batch_layers"),
            UiPerfCounter::GpuBatchDependencies => concat!($prefix, ".gpu_batch_dependencies"),
        }
    };
}

#[cfg(feature = "profiling")]
fn counter_name(scenario: UiPerfScenario, counter: UiPerfCounter) -> &'static str {
    match scenario {
        UiPerfScenario::Startup => counter_name_for_prefix!(counter, "ui.startup"),
        UiPerfScenario::IdleHover => counter_name_for_prefix!(counter, "ui.idle_hover"),
        UiPerfScenario::Click => counter_name_for_prefix!(counter, "ui.click"),
        UiPerfScenario::Drag => counter_name_for_prefix!(counter, "ui.drag"),
        UiPerfScenario::DrawerResize => counter_name_for_prefix!(counter, "ui.drawer_resize"),
        UiPerfScenario::AssetRefresh => counter_name_for_prefix!(counter, "ui.asset_refresh"),
        UiPerfScenario::ViewportImage => counter_name_for_prefix!(counter, "ui.viewport_image"),
    }
}
