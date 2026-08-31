use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{DiagnosticReadbackBudget, GpuMemoryBudget, RenderDeviceLimits, SubmissionLimits};

/// Process-local identity assigned when a concrete backend creates a device.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(u64);

impl DeviceId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Monotonic lifetime generation for device-owned RHI objects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceGeneration(u64);

impl DeviceGeneration {
    pub const fn initial() -> Self {
        Self(1)
    }

    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Backend family used for stable adapter selection and persisted diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RenderBackendKind {
    Dx12,
    Vulkan,
    Metal,
    Gl,
    BrowserWebGpu,
    Other,
}

/// Hardware class used by selection policy, not by rendering hot paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RenderAdapterClass {
    Discrete,
    Integrated,
    Virtual,
    Cpu,
    Other,
}

/// Optional device features with an explicit RHI-level meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum RenderDeviceFeature {
    HdrR11G11B10UfloatRenderTarget,
    IndirectFirstInstance,
    MultiDrawIndirectCount,
    BindlessMaterialArrays,
    GpuTimestamp,
    PipelineStatistics,
    Subgroups,
}

impl RenderDeviceFeature {
    pub const COUNT: usize = Self::Subgroups as usize + 1;

    pub const ALL: [Self; Self::COUNT] = [
        Self::HdrR11G11B10UfloatRenderTarget,
        Self::IndirectFirstInstance,
        Self::MultiDrawIndirectCount,
        Self::BindlessMaterialArrays,
        Self::GpuTimestamp,
        Self::PipelineStatistics,
        Self::Subgroups,
    ];

    const fn index(self) -> usize {
        self as usize
    }
}

/// Fixed feature set used by cold-path adapter selection and device negotiation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderDeviceFeatureSet {
    enabled: [bool; RenderDeviceFeature::COUNT],
}

impl Default for RenderDeviceFeatureSet {
    fn default() -> Self {
        Self {
            enabled: [false; RenderDeviceFeature::COUNT],
        }
    }
}

impl RenderDeviceFeatureSet {
    pub const fn contains(&self, feature: RenderDeviceFeature) -> bool {
        self.enabled[feature.index()]
    }

    pub const fn is_empty(&self) -> bool {
        let mut index = 0;
        while index < RenderDeviceFeature::COUNT {
            if self.enabled[index] {
                return false;
            }
            index += 1;
        }
        true
    }

    pub fn insert(&mut self, feature: RenderDeviceFeature) {
        self.enabled[feature.index()] = true;
    }

    pub fn remove(&mut self, feature: RenderDeviceFeature) {
        self.enabled[feature.index()] = false;
    }

    pub fn iter(&self) -> impl Iterator<Item = RenderDeviceFeature> + '_ {
        RenderDeviceFeature::ALL
            .into_iter()
            .filter(|feature| self.contains(*feature))
    }
}

/// Serializable immutable facts collected before a concrete device is created.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderAdapterFacts {
    pub backend: RenderBackendKind,
    pub name: String,
    pub vendor_id: u32,
    pub device_id: u32,
    pub driver_version: String,
    pub adapter_class: RenderAdapterClass,
    pub dedicated_memory_bytes: Option<u64>,
    pub supported_features: RenderDeviceFeatureSet,
}

impl RenderAdapterFacts {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        backend: RenderBackendKind,
        name: impl Into<String>,
        vendor_id: u32,
        device_id: u32,
        driver_version: impl Into<String>,
        adapter_class: RenderAdapterClass,
        dedicated_memory_bytes: Option<u64>,
        supported_features: RenderDeviceFeatureSet,
    ) -> Self {
        Self {
            backend,
            name: name.into(),
            vendor_id,
            device_id,
            driver_version: driver_version.into(),
            adapter_class,
            dedicated_memory_bytes,
            supported_features,
        }
    }

    fn stable_cmp(&self, other: &Self) -> Ordering {
        (
            self.backend,
            self.vendor_id,
            self.device_id,
            self.driver_version.as_str(),
            self.name.as_str(),
            self.adapter_class,
        )
            .cmp(&(
                other.backend,
                other.vendor_id,
                other.device_id,
                other.driver_version.as_str(),
                other.name.as_str(),
                other.adapter_class,
            ))
    }
}

/// Declarative selector for explicit adapter override and deny-list policy.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderAdapterSelector {
    pub backend: Option<RenderBackendKind>,
    pub vendor_id: Option<u32>,
    pub device_id: Option<u32>,
}

impl RenderAdapterSelector {
    pub fn matches(&self, adapter: &RenderAdapterFacts) -> bool {
        self.backend
            .is_none_or(|backend| adapter.backend == backend)
            && self
                .vendor_id
                .is_none_or(|vendor_id| adapter.vendor_id == vendor_id)
            && self
                .device_id
                .is_none_or(|device_id| adapter.device_id == device_id)
    }
}

/// Cold-path policy for deterministic adapter selection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterSelectionPolicy {
    pub preferred_backend: Option<RenderBackendKind>,
    pub prefer_discrete: bool,
    pub allow_software: bool,
    pub adapter_override: Option<RenderAdapterSelector>,
    pub denied_adapters: Vec<RenderAdapterSelector>,
}

impl Default for AdapterSelectionPolicy {
    fn default() -> Self {
        Self {
            preferred_backend: None,
            prefer_discrete: true,
            allow_software: false,
            adapter_override: None,
            denied_adapters: Vec::new(),
        }
    }
}

impl AdapterSelectionPolicy {
    pub fn with_preferred_backend(mut self, backend: RenderBackendKind) -> Self {
        self.preferred_backend = Some(backend);
        self
    }

    pub fn with_adapter_override(mut self, selector: RenderAdapterSelector) -> Self {
        self.adapter_override = Some(selector);
        self
    }

    pub fn deny_adapter(mut self, selector: RenderAdapterSelector) -> Self {
        self.denied_adapters.push(selector);
        self
    }
}

/// Immutable adapter catalog that selects from facts rather than backend enumeration order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderAdapterCatalog {
    adapters: Vec<RenderAdapterFacts>,
}

impl RenderAdapterCatalog {
    pub fn new(adapters: Vec<RenderAdapterFacts>) -> Self {
        Self { adapters }
    }

    pub fn adapters(&self) -> &[RenderAdapterFacts] {
        &self.adapters
    }

    pub fn select(
        &self,
        policy: &AdapterSelectionPolicy,
    ) -> Result<AdapterSelectionReceipt, AdapterSelectionError> {
        let mut candidates = self.adapters.clone();
        candidates.sort_by(RenderAdapterFacts::stable_cmp);

        let mut eligible = Vec::new();
        let mut rejected = Vec::new();
        for adapter in candidates {
            let reason = if policy
                .adapter_override
                .as_ref()
                .is_some_and(|selector| !selector.matches(&adapter))
            {
                Some(RejectedAdapterReason::OverrideMismatch)
            } else if policy
                .denied_adapters
                .iter()
                .any(|selector| selector.matches(&adapter))
            {
                Some(RejectedAdapterReason::Denied)
            } else if !policy.allow_software && adapter.adapter_class == RenderAdapterClass::Cpu {
                Some(RejectedAdapterReason::SoftwareNotAllowed)
            } else {
                None
            };

            if let Some(reason) = reason {
                rejected.push(RejectedAdapter { adapter, reason });
            } else {
                eligible.push(adapter);
            }
        }
        eligible.sort_by(|left, right| adapter_preference_cmp(left, right, policy));

        let Some(selected) = eligible.first().cloned() else {
            return Err(AdapterSelectionError::NoEligibleAdapter { rejected });
        };
        rejected.extend(eligible.into_iter().skip(1).map(|adapter| RejectedAdapter {
            adapter,
            reason: RejectedAdapterReason::LowerPriority,
        }));

        Ok(AdapterSelectionReceipt { selected, rejected })
    }
}

fn adapter_preference_cmp(
    left: &RenderAdapterFacts,
    right: &RenderAdapterFacts,
    policy: &AdapterSelectionPolicy,
) -> Ordering {
    let left_backend = u8::from(policy.preferred_backend != Some(left.backend));
    let right_backend = u8::from(policy.preferred_backend != Some(right.backend));
    let left_class = adapter_class_rank(left.adapter_class, policy.prefer_discrete);
    let right_class = adapter_class_rank(right.adapter_class, policy.prefer_discrete);

    (left_backend, left_class)
        .cmp(&(right_backend, right_class))
        .then_with(|| left.stable_cmp(right))
}

const fn adapter_class_rank(adapter_class: RenderAdapterClass, prefer_discrete: bool) -> u8 {
    if !prefer_discrete {
        return 0;
    }

    match adapter_class {
        RenderAdapterClass::Discrete => 0,
        RenderAdapterClass::Integrated => 1,
        RenderAdapterClass::Virtual => 2,
        RenderAdapterClass::Other => 3,
        RenderAdapterClass::Cpu => 4,
    }
}

/// Stable outcome of choosing one adapter from a catalog.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterSelectionReceipt {
    selected: RenderAdapterFacts,
    rejected: Vec<RejectedAdapter>,
}

impl AdapterSelectionReceipt {
    pub const fn selected(&self) -> &RenderAdapterFacts {
        &self.selected
    }

    pub fn rejected(&self) -> &[RejectedAdapter] {
        &self.rejected
    }
}

/// Reason one adapter was not selected, retained in startup diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RejectedAdapterReason {
    OverrideMismatch,
    Denied,
    SoftwareNotAllowed,
    LowerPriority,
}

/// Candidate and reason preserved in an adapter selection receipt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectedAdapter {
    pub adapter: RenderAdapterFacts,
    pub reason: RejectedAdapterReason,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum AdapterSelectionError {
    #[error("no adapter satisfied the declared selection policy")]
    NoEligibleAdapter { rejected: Vec<RejectedAdapter> },
}

/// Device creation policy. The MVP baseline requests no optional backend features.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderDeviceRequestPolicy {
    required_features: RenderDeviceFeatureSet,
    optional_features: RenderDeviceFeatureSet,
}

impl RenderDeviceRequestPolicy {
    pub fn mvp_baseline() -> Self {
        Self::default()
    }

    pub fn with_required_feature(mut self, feature: RenderDeviceFeature) -> Self {
        self.required_features.insert(feature);
        self.optional_features.remove(feature);
        self
    }

    pub fn with_optional_feature(mut self, feature: RenderDeviceFeature) -> Self {
        if !self.required_features.contains(feature) {
            self.optional_features.insert(feature);
        }
        self
    }

    pub fn negotiate(
        &self,
        supported_features: &RenderDeviceFeatureSet,
    ) -> Result<RenderDeviceFeatureNegotiation, RenderDeviceNegotiationError> {
        for feature in self.required_features.iter() {
            if !supported_features.contains(feature) {
                return Err(RenderDeviceNegotiationError::RequiredFeatureUnavailable { feature });
            }
        }

        let mut requested_features = self.required_features.clone();
        let mut unavailable_features = RenderDeviceFeatureSet::default();
        for feature in self.optional_features.iter() {
            if supported_features.contains(feature) {
                requested_features.insert(feature);
            } else {
                unavailable_features.insert(feature);
            }
        }

        Ok(RenderDeviceFeatureNegotiation {
            requested_features,
            unavailable_features,
        })
    }
}

/// Result of applying one request policy to adapter facts before calling a backend API.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderDeviceFeatureNegotiation {
    requested_features: RenderDeviceFeatureSet,
    unavailable_features: RenderDeviceFeatureSet,
}

impl RenderDeviceFeatureNegotiation {
    pub const fn requested_features(&self) -> &RenderDeviceFeatureSet {
        &self.requested_features
    }

    pub const fn unavailable_features(&self) -> &RenderDeviceFeatureSet {
        &self.unavailable_features
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RenderDeviceNegotiationError {
    #[error("required render device feature `{feature:?}` is unavailable")]
    RequiredFeatureUnavailable { feature: RenderDeviceFeature },
}

/// Typed device-request failure with enough context to select a safe fallback or diagnostics.
#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("render device request failed for {adapter:?}: {backend_detail}")]
pub struct RenderDeviceRequestFailure {
    adapter: RenderAdapterFacts,
    feature_negotiation: RenderDeviceFeatureNegotiation,
    requested_limits: RenderDeviceLimits,
    backend_detail: String,
}

impl RenderDeviceRequestFailure {
    pub fn new(
        adapter: RenderAdapterFacts,
        feature_negotiation: RenderDeviceFeatureNegotiation,
        requested_limits: RenderDeviceLimits,
        backend_detail: impl Into<String>,
    ) -> Self {
        Self {
            adapter,
            feature_negotiation,
            requested_limits,
            backend_detail: backend_detail.into(),
        }
    }

    pub const fn adapter(&self) -> &RenderAdapterFacts {
        &self.adapter
    }

    pub const fn feature_negotiation(&self) -> &RenderDeviceFeatureNegotiation {
        &self.feature_negotiation
    }

    pub const fn requested_limits(&self) -> &RenderDeviceLimits {
        &self.requested_limits
    }

    pub fn backend_detail(&self) -> &str {
        &self.backend_detail
    }
}

/// Logical command classes backed by one or more physical queues on a device.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderDeviceQueueTopology {
    pub physical_queue_count: u8,
    pub supports_graphics_commands: bool,
    pub supports_compute_commands: bool,
    pub supports_copy_commands: bool,
    pub supports_async_compute: bool,
    pub supports_async_copy: bool,
}

impl RenderDeviceQueueTopology {
    /// WGPU's graphics, compute, and copy commands serialize through one physical queue.
    pub const fn single_serialized_queue() -> Self {
        Self {
            physical_queue_count: 1,
            supports_graphics_commands: true,
            supports_compute_commands: true,
            supports_copy_commands: true,
            supports_async_compute: false,
            supports_async_copy: false,
        }
    }
}

/// Immutable device-qualified profile consumed by later resource and submission milestones.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderDeviceProfile {
    device_id: DeviceId,
    generation: DeviceGeneration,
    adapter: RenderAdapterFacts,
    feature_negotiation: RenderDeviceFeatureNegotiation,
    device_limits: RenderDeviceLimits,
    queue_topology: RenderDeviceQueueTopology,
    memory_budget: GpuMemoryBudget,
    submission_limits: SubmissionLimits,
    diagnostic_readback_budget: DiagnosticReadbackBudget,
}

impl RenderDeviceProfile {
    pub fn new(
        device_id: DeviceId,
        generation: DeviceGeneration,
        adapter: RenderAdapterFacts,
        feature_negotiation: RenderDeviceFeatureNegotiation,
        device_limits: RenderDeviceLimits,
        queue_topology: RenderDeviceQueueTopology,
        memory_budget: GpuMemoryBudget,
        submission_limits: SubmissionLimits,
        diagnostic_readback_budget: DiagnosticReadbackBudget,
    ) -> Self {
        Self {
            device_id,
            generation,
            adapter,
            feature_negotiation,
            device_limits,
            queue_topology,
            memory_budget,
            submission_limits,
            diagnostic_readback_budget,
        }
    }

    pub const fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub const fn generation(&self) -> DeviceGeneration {
        self.generation
    }

    pub const fn adapter(&self) -> &RenderAdapterFacts {
        &self.adapter
    }

    pub const fn requested_features(&self) -> &RenderDeviceFeatureSet {
        self.feature_negotiation.requested_features()
    }

    pub const fn unavailable_features(&self) -> &RenderDeviceFeatureSet {
        self.feature_negotiation.unavailable_features()
    }

    pub const fn device_limits(&self) -> &RenderDeviceLimits {
        &self.device_limits
    }

    pub const fn queue_topology(&self) -> &RenderDeviceQueueTopology {
        &self.queue_topology
    }

    pub const fn memory_budget(&self) -> GpuMemoryBudget {
        self.memory_budget
    }

    pub const fn submission_limits(&self) -> SubmissionLimits {
        self.submission_limits
    }

    pub const fn diagnostic_readback_budget(&self) -> DiagnosticReadbackBudget {
        self.diagnostic_readback_budget
    }
}
