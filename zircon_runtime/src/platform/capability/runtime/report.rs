use crate::core::framework::platform::{PlatformHostLifecycleState, PlatformHostSnapshot};

use super::{PlatformRuntimeCapabilityStatus, PlatformRuntimeHostRequirement};
use crate::platform::capability::{
    CapabilityStatus, EventLoopPolicy, MonitorBackend, PlatformCapabilityReport, WindowBackend,
    WindowEventBackend, WindowLifecycleBackend, WindowMetricsBackend,
};

/// Runtime capability projection for the platform-host path. The planning
/// catalog remains inspectable, but only this type is appropriate for product
/// admission because it carries the current host owner and observations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformRuntimeCapabilityReport {
    enabled: bool,
    planning: PlatformCapabilityReport,
    host: PlatformHostSnapshot,
}

impl PlatformRuntimeCapabilityReport {
    pub(crate) fn new(
        enabled: bool,
        planning: PlatformCapabilityReport,
        host: PlatformHostSnapshot,
    ) -> Self {
        Self {
            enabled,
            planning,
            host,
        }
    }

    /// 返回未经过宿主观测的静态目录，适合离线规划和诊断展示。
    pub fn planning(&self) -> &PlatformCapabilityReport {
        &self.planning
    }

    /// 返回生成本报告时捕获的宿主快照，供调用方核对 provider 与 generation。
    pub fn host(&self) -> &PlatformHostSnapshot {
        &self.host
    }

    /// 将窗口目录项按 Windowing 要求投影为运行时状态。
    pub fn window_backend(&self) -> PlatformRuntimeCapabilityStatus<WindowBackend> {
        self.project(
            self.planning.window_backend,
            PlatformRuntimeHostRequirement::Windowing,
        )
    }

    pub fn monitor_inventory(&self) -> PlatformRuntimeCapabilityStatus<MonitorBackend> {
        self.project(
            self.planning.monitor_inventory,
            PlatformRuntimeHostRequirement::DisplayTopology,
        )
    }

    pub fn window_events(&self) -> PlatformRuntimeCapabilityStatus<WindowEventBackend> {
        self.project(
            self.planning.window_events,
            PlatformRuntimeHostRequirement::WindowingAndEventLoop,
        )
    }

    pub fn window_lifecycle(&self) -> PlatformRuntimeCapabilityStatus<WindowLifecycleBackend> {
        self.project(
            self.planning.window_lifecycle,
            PlatformRuntimeHostRequirement::WindowingAndEventLoop,
        )
    }

    pub fn window_metrics(&self) -> PlatformRuntimeCapabilityStatus<WindowMetricsBackend> {
        self.project(
            self.planning.window_metrics,
            PlatformRuntimeHostRequirement::WindowingAndEventLoop,
        )
    }

    pub fn event_loop_policy(&self) -> PlatformRuntimeCapabilityStatus<EventLoopPolicy> {
        self.project(
            CapabilityStatus::Supported(self.planning.event_loop_policy),
            PlatformRuntimeHostRequirement::EventLoop,
        )
    }

    // 先应用 enabled/feature/unavailable 结论，再检查宿主生命周期、provider 身份和
    // observed_capabilities；只有 Ready 且观测满足 requirement 才能向 admission 放行。
    fn project<T>(
        &self,
        planning: CapabilityStatus<T>,
        requirement: PlatformRuntimeHostRequirement,
    ) -> PlatformRuntimeCapabilityStatus<T> {
        if !self.enabled {
            return PlatformRuntimeCapabilityStatus::Disabled;
        }
        let value = match planning {
            CapabilityStatus::Supported(value) => value,
            CapabilityStatus::FeatureDisabled { feature } => {
                return PlatformRuntimeCapabilityStatus::FeatureDisabled { feature };
            }
            CapabilityStatus::Unavailable { reason } => {
                return PlatformRuntimeCapabilityStatus::Unavailable { reason };
            }
        };

        let lifecycle = self.host.lifecycle();
        let provider = self.host.instance();
        let generation = self.host.generation();
        match lifecycle {
            PlatformHostLifecycleState::Ready => {
                let Some(provider) = provider else {
                    return PlatformRuntimeCapabilityStatus::HostUnavailable {
                        lifecycle,
                        provider: None,
                        generation,
                    };
                };
                let observed = self
                    .host
                    .evidence()
                    .map(|evidence| evidence.observed_capabilities());
                if observed.is_some_and(|observed| requirement.is_observed_by(observed)) {
                    PlatformRuntimeCapabilityStatus::Ready {
                        value,
                        provider,
                        generation,
                    }
                } else {
                    PlatformRuntimeCapabilityStatus::NotObserved {
                        value,
                        requirement,
                        provider,
                        generation,
                    }
                }
            }
            PlatformHostLifecycleState::Degraded => match provider {
                Some(provider) => PlatformRuntimeCapabilityStatus::Degraded {
                    value,
                    provider,
                    generation,
                },
                None => PlatformRuntimeCapabilityStatus::HostUnavailable {
                    lifecycle,
                    provider: None,
                    generation,
                },
            },
            _ => PlatformRuntimeCapabilityStatus::HostUnavailable {
                lifecycle,
                provider,
                generation,
            },
        }
    }
}
