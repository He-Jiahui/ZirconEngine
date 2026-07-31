use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::ToolId;

pub const DEFAULT_MAX_QUEUE_PER_RESOURCE: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ExclusiveResource {
    ViewportInput,
    ModalSurface,
    SceneModeSlot,
}

impl ExclusiveResource {
    const ALL: [Self; 3] = [Self::ViewportInput, Self::ModalSurface, Self::SceneModeSlot];
}

/// Immutable, nonempty, canonically sorted resources acquired as one scheduler lease.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToolResourceSet(Vec<ExclusiveResource>);

impl ToolResourceSet {
    pub fn new<I>(resources: I) -> Result<Self, ToolResourceSetError>
    where
        I: IntoIterator<Item = ExclusiveResource>,
    {
        let resources = resources.into_iter().collect::<BTreeSet<_>>();
        if resources.is_empty() {
            return Err(ToolResourceSetError::Empty);
        }
        Ok(Self(resources.into_iter().collect()))
    }

    pub fn as_slice(&self) -> &[ExclusiveResource] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Serialize for ToolResourceSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ToolResourceSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let resources = Vec::<ExclusiveResource>::deserialize(deserializer)?;
        Self::new(resources).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolResourceSetError {
    Empty,
}

impl std::fmt::Display for ToolResourceSetError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(formatter, "a tool resource set cannot be empty"),
        }
    }
}

impl std::error::Error for ToolResourceSetError {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcquireDenial {
    QueueFull { max_queued: usize },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcquireOutcome {
    Acquired,
    AlreadyHeld,
    Queued {
        position: usize,
    },
    AlreadyQueued {
        position: usize,
    },
    Denied {
        holder: ToolId,
        reason: AcquireDenial,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcquireSetOutcome {
    Acquired,
    AlreadyHeld,
    Queued { position: usize },
    AlreadyQueued { position: usize },
    Denied { reason: AcquireDenial },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReleaseOutcome {
    Released { activated: Option<ToolId> },
    NotHeld,
    NotHolder { holder: ToolId },
    SetHeld { resources: ToolResourceSet },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReleaseSetOutcome {
    Released {
        activated: Option<ToolId>,
    },
    NotHeld,
    NotHolder {
        resource: ExclusiveResource,
        holder: ToolId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WithdrawOutcome {
    Withdrawn { previous_position: usize },
    NotQueued,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WithdrawSetOutcome {
    Withdrawn {
        previous_position: usize,
        activated: Option<ToolId>,
    },
    NotQueued,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReleaseAllOutcome {
    pub released_resources: Vec<ExclusiveResource>,
    pub withdrawn_resources: Vec<ExclusiveResource>,
    pub activated_tools: Vec<(ExclusiveResource, ToolId)>,
    pub released_resource_sets: Vec<ToolResourceSet>,
    pub withdrawn_resource_sets: Vec<ToolResourceSet>,
    pub activated_resource_sets: Vec<(ToolResourceSet, ToolId)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolLifecycleEvent {
    Activated {
        tool: ToolId,
        resource: ExclusiveResource,
    },
    Deactivated {
        tool: ToolId,
        resource: ExclusiveResource,
    },
    Queued {
        tool: ToolId,
        resource: ExclusiveResource,
        position: usize,
    },
    Withdrawn {
        tool: ToolId,
        resource: ExclusiveResource,
        previous_position: usize,
    },
    Denied {
        tool: ToolId,
        resource: ExclusiveResource,
        holder: ToolId,
        reason: AcquireDenial,
    },
    SetActivated {
        tool: ToolId,
        resources: ToolResourceSet,
    },
    SetDeactivated {
        tool: ToolId,
        resources: ToolResourceSet,
    },
    SetQueued {
        tool: ToolId,
        resources: ToolResourceSet,
        position: usize,
    },
    SetWithdrawn {
        tool: ToolId,
        resources: ToolResourceSet,
        previous_position: usize,
    },
    SetDenied {
        tool: ToolId,
        resources: ToolResourceSet,
        reason: AcquireDenial,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use = "tool lifecycle events must be published before exposing the new scheduler state"]
pub struct ToolScheduleReport<O> {
    outcome: O,
    events: Vec<ToolLifecycleEvent>,
}

impl<O> ToolScheduleReport<O> {
    fn new(outcome: O, events: Vec<ToolLifecycleEvent>) -> Self {
        Self { outcome, events }
    }

    pub fn outcome(&self) -> &O {
        &self.outcome
    }

    pub fn events(&self) -> &[ToolLifecycleEvent] {
        &self.events
    }

    pub fn into_parts(self) -> (O, Vec<ToolLifecycleEvent>) {
        (self.outcome, self.events)
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct ResourceState {
    holder: Option<ToolId>,
    queue: VecDeque<ToolId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResourceSetRequest {
    tool: ToolId,
    resources: ToolResourceSet,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ToolScheduler {
    resources: BTreeMap<ExclusiveResource, ResourceState>,
    set_queue: VecDeque<ResourceSetRequest>,
    active_sets: BTreeMap<ToolId, ToolResourceSet>,
    max_queue_per_resource: usize,
}

impl ToolScheduler {
    pub fn new(max_queue_per_resource: usize) -> Self {
        Self {
            resources: BTreeMap::new(),
            set_queue: VecDeque::new(),
            active_sets: BTreeMap::new(),
            max_queue_per_resource,
        }
    }

    pub fn max_queue_per_resource(&self) -> usize {
        self.max_queue_per_resource
    }

    pub fn holder(&self, resource: ExclusiveResource) -> Option<&ToolId> {
        self.resources
            .get(&resource)
            .and_then(|state| state.holder.as_ref())
    }

    pub fn queued_tools(&self, resource: ExclusiveResource) -> impl Iterator<Item = &ToolId> {
        self.resources
            .get(&resource)
            .into_iter()
            .flat_map(|state| state.queue.iter())
    }

    pub fn acquire(
        &mut self,
        tool: ToolId,
        resource: ExclusiveResource,
    ) -> ToolScheduleReport<AcquireOutcome> {
        let set_queue_holder = self.set_queue.front().map(|request| request.tool.clone());
        let state = self.resources.entry(resource).or_default();
        if state.holder.as_ref() == Some(&tool) {
            return ToolScheduleReport::new(AcquireOutcome::AlreadyHeld, Vec::new());
        }

        if let Some(index) = state.queue.iter().position(|queued| queued == &tool) {
            return ToolScheduleReport::new(
                AcquireOutcome::AlreadyQueued {
                    position: index + 1,
                },
                Vec::new(),
            );
        }

        if state.holder.is_none() && set_queue_holder.is_none() {
            state.holder = Some(tool.clone());
            return ToolScheduleReport::new(
                AcquireOutcome::Acquired,
                vec![ToolLifecycleEvent::Activated { tool, resource }],
            );
        }

        if state.queue.len() >= self.max_queue_per_resource {
            let reason = AcquireDenial::QueueFull {
                max_queued: self.max_queue_per_resource,
            };
            if let Some(holder) = state.holder.clone().or(set_queue_holder) {
                return ToolScheduleReport::new(
                    AcquireOutcome::Denied {
                        holder: holder.clone(),
                        reason: reason.clone(),
                    },
                    vec![ToolLifecycleEvent::Denied {
                        tool,
                        resource,
                        holder,
                        reason,
                    }],
                );
            }
        }

        state.queue.push_back(tool.clone());
        let position = state.queue.len();
        ToolScheduleReport::new(
            AcquireOutcome::Queued { position },
            vec![ToolLifecycleEvent::Queued {
                tool,
                resource,
                position,
            }],
        )
    }

    /// Acquires every resource in one canonical set or queues the full set with no partial hold.
    pub fn acquire_set(
        &mut self,
        tool: ToolId,
        resources: ToolResourceSet,
    ) -> ToolScheduleReport<AcquireSetOutcome> {
        if self.active_sets.get(&tool) == Some(&resources) {
            return ToolScheduleReport::new(AcquireSetOutcome::AlreadyHeld, Vec::new());
        }
        if let Some(index) = self
            .set_queue
            .iter()
            .position(|request| request.tool == tool && request.resources == resources)
        {
            return ToolScheduleReport::new(
                AcquireSetOutcome::AlreadyQueued {
                    position: index + 1,
                },
                Vec::new(),
            );
        }

        if self.set_queue.is_empty() && self.resources_are_available(&resources) {
            self.activate_set(tool.clone(), resources.clone());
            return ToolScheduleReport::new(
                AcquireSetOutcome::Acquired,
                vec![ToolLifecycleEvent::SetActivated { tool, resources }],
            );
        }

        if self.set_queue.len() >= self.max_queue_per_resource {
            let reason = AcquireDenial::QueueFull {
                max_queued: self.max_queue_per_resource,
            };
            return ToolScheduleReport::new(
                AcquireSetOutcome::Denied {
                    reason: reason.clone(),
                },
                vec![ToolLifecycleEvent::SetDenied {
                    tool,
                    resources,
                    reason,
                }],
            );
        }

        self.set_queue.push_back(ResourceSetRequest {
            tool: tool.clone(),
            resources: resources.clone(),
        });
        let position = self.set_queue.len();
        ToolScheduleReport::new(
            AcquireSetOutcome::Queued { position },
            vec![ToolLifecycleEvent::SetQueued {
                tool,
                resources,
                position,
            }],
        )
    }

    pub fn release(
        &mut self,
        tool: &ToolId,
        resource: ExclusiveResource,
    ) -> ToolScheduleReport<ReleaseOutcome> {
        if let Some(resources) = self.active_sets.get(tool) {
            if resources.as_slice().contains(&resource) {
                return ToolScheduleReport::new(
                    ReleaseOutcome::SetHeld {
                        resources: resources.clone(),
                    },
                    Vec::new(),
                );
            }
        }
        let mut events = Vec::new();
        {
            let Some(state) = self.resources.get_mut(&resource) else {
                return ToolScheduleReport::new(ReleaseOutcome::NotHeld, events);
            };
            let Some(holder) = state.holder.as_ref() else {
                return ToolScheduleReport::new(ReleaseOutcome::NotHeld, events);
            };
            if holder != tool {
                return ToolScheduleReport::new(
                    ReleaseOutcome::NotHolder {
                        holder: holder.clone(),
                    },
                    events,
                );
            }
            state.holder = None;
        }
        events.push(ToolLifecycleEvent::Deactivated {
            tool: tool.clone(),
            resource,
        });

        if self.promote_set_head(&mut events).is_none() {
            let activated = self.promote_single_resource(resource, &mut events);
            self.remove_empty_resource_state(resource);
            return ToolScheduleReport::new(ReleaseOutcome::Released { activated }, events);
        }

        ToolScheduleReport::new(ReleaseOutcome::Released { activated: None }, events)
    }

    pub fn release_set(
        &mut self,
        tool: &ToolId,
        resources: &ToolResourceSet,
    ) -> ToolScheduleReport<ReleaseSetOutcome> {
        let Some(active_resources) = self.active_sets.get(tool) else {
            return ToolScheduleReport::new(ReleaseSetOutcome::NotHeld, Vec::new());
        };
        if active_resources != resources {
            return ToolScheduleReport::new(ReleaseSetOutcome::NotHeld, Vec::new());
        }
        for resource in resources.as_slice() {
            match self.holder(*resource) {
                Some(holder) if holder == tool => {}
                Some(holder) => {
                    return ToolScheduleReport::new(
                        ReleaseSetOutcome::NotHolder {
                            resource: *resource,
                            holder: holder.clone(),
                        },
                        Vec::new(),
                    );
                }
                None => return ToolScheduleReport::new(ReleaseSetOutcome::NotHeld, Vec::new()),
            }
        }

        self.active_sets.remove(tool);
        for resource in resources.as_slice() {
            if let Some(state) = self.resources.get_mut(resource) {
                state.holder = None;
            }
        }
        let mut events = vec![ToolLifecycleEvent::SetDeactivated {
            tool: tool.clone(),
            resources: resources.clone(),
        }];
        let activated = self.promote_set_head(&mut events).map(|(tool, _)| tool);
        if activated.is_none() && self.set_queue.is_empty() {
            self.promote_waiting_single_resources(&mut events);
        }
        for resource in resources.as_slice() {
            self.remove_empty_resource_state(*resource);
        }
        ToolScheduleReport::new(ReleaseSetOutcome::Released { activated }, events)
    }

    pub fn withdraw(
        &mut self,
        tool: &ToolId,
        resource: ExclusiveResource,
    ) -> ToolScheduleReport<WithdrawOutcome> {
        let Some(state) = self.resources.get_mut(&resource) else {
            return ToolScheduleReport::new(WithdrawOutcome::NotQueued, Vec::new());
        };
        let Some(index) = state.queue.iter().position(|queued| queued == tool) else {
            return ToolScheduleReport::new(WithdrawOutcome::NotQueued, Vec::new());
        };
        let previous_position = index + 1;
        if state.queue.remove(index).is_none() {
            return ToolScheduleReport::new(WithdrawOutcome::NotQueued, Vec::new());
        }

        ToolScheduleReport::new(
            WithdrawOutcome::Withdrawn { previous_position },
            vec![ToolLifecycleEvent::Withdrawn {
                tool: tool.clone(),
                resource,
                previous_position,
            }],
        )
    }

    pub fn withdraw_set(
        &mut self,
        tool: &ToolId,
        resources: &ToolResourceSet,
    ) -> ToolScheduleReport<WithdrawSetOutcome> {
        let Some(index) = self
            .set_queue
            .iter()
            .position(|request| &request.tool == tool && &request.resources == resources)
        else {
            return ToolScheduleReport::new(WithdrawSetOutcome::NotQueued, Vec::new());
        };
        let previous_position = index + 1;
        if self.set_queue.remove(index).is_none() {
            return ToolScheduleReport::new(WithdrawSetOutcome::NotQueued, Vec::new());
        }
        let mut events = vec![ToolLifecycleEvent::SetWithdrawn {
            tool: tool.clone(),
            resources: resources.clone(),
            previous_position,
        }];
        let activated = self.promote_set_head(&mut events).map(|(tool, _)| tool);
        if activated.is_none() && self.set_queue.is_empty() {
            self.promote_waiting_single_resources(&mut events);
        }
        ToolScheduleReport::new(
            WithdrawSetOutcome::Withdrawn {
                previous_position,
                activated,
            },
            events,
        )
    }

    pub fn release_all(&mut self, tool: &ToolId) -> ToolScheduleReport<ReleaseAllOutcome> {
        let mut outcome = ReleaseAllOutcome::default();
        let mut events = Vec::new();

        let queued_sets = self
            .set_queue
            .iter()
            .filter(|request| &request.tool == tool)
            .map(|request| request.resources.clone())
            .collect::<Vec<_>>();
        for resources in queued_sets {
            let report = self.withdraw_set(tool, &resources);
            let (set_outcome, set_events) = report.into_parts();
            if matches!(set_outcome, WithdrawSetOutcome::Withdrawn { .. }) {
                outcome.withdrawn_resource_sets.push(resources);
            }
            events.extend(set_events);
        }

        if let Some(resources) = self.active_sets.get(tool).cloned() {
            let report = self.release_set(tool, &resources);
            let (set_outcome, set_events) = report.into_parts();
            if let ReleaseSetOutcome::Released { activated } = set_outcome {
                outcome.released_resource_sets.push(resources);
                if let Some(activated_tool) = activated {
                    if let Some(activated_resources) =
                        self.active_sets.get(&activated_tool).cloned()
                    {
                        outcome
                            .activated_resource_sets
                            .push((activated_resources, activated_tool));
                    }
                }
            }
            events.extend(set_events);
        }

        for resource in ExclusiveResource::ALL {
            let withdrawn = self.withdraw(tool, resource);
            let (withdrawn_outcome, withdrawn_events) = withdrawn.into_parts();
            if matches!(withdrawn_outcome, WithdrawOutcome::Withdrawn { .. }) {
                outcome.withdrawn_resources.push(resource);
            }
            events.extend(withdrawn_events);

            let released = self.release(tool, resource);
            let (released_outcome, released_events) = released.into_parts();
            if let ReleaseOutcome::Released { activated } = released_outcome {
                outcome.released_resources.push(resource);
                if let Some(activated_tool) = activated {
                    outcome.activated_tools.push((resource, activated_tool));
                }
            }
            events.extend(released_events);
        }

        ToolScheduleReport::new(outcome, events)
    }

    fn resources_are_available(&self, resources: &ToolResourceSet) -> bool {
        resources
            .as_slice()
            .iter()
            .all(|resource| self.holder(*resource).is_none())
    }

    fn activate_set(&mut self, tool: ToolId, resources: ToolResourceSet) {
        for resource in resources.as_slice() {
            self.resources.entry(*resource).or_default().holder = Some(tool.clone());
        }
        self.active_sets.insert(tool, resources);
    }

    fn promote_set_head(
        &mut self,
        events: &mut Vec<ToolLifecycleEvent>,
    ) -> Option<(ToolId, ToolResourceSet)> {
        let request = self.set_queue.front()?.clone();
        if !self.resources_are_available(&request.resources) {
            return None;
        }
        self.set_queue.pop_front();
        self.activate_set(request.tool.clone(), request.resources.clone());
        events.push(ToolLifecycleEvent::SetActivated {
            tool: request.tool.clone(),
            resources: request.resources.clone(),
        });
        Some((request.tool, request.resources))
    }

    fn promote_single_resource(
        &mut self,
        resource: ExclusiveResource,
        events: &mut Vec<ToolLifecycleEvent>,
    ) -> Option<ToolId> {
        let state = self.resources.get_mut(&resource)?;
        if state.holder.is_some() {
            return None;
        }
        let activated = state.queue.pop_front()?;
        state.holder = Some(activated.clone());
        events.push(ToolLifecycleEvent::Activated {
            tool: activated.clone(),
            resource,
        });
        Some(activated)
    }

    fn promote_waiting_single_resources(&mut self, events: &mut Vec<ToolLifecycleEvent>) {
        for resource in ExclusiveResource::ALL {
            self.promote_single_resource(resource, events);
        }
    }

    fn remove_empty_resource_state(&mut self, resource: ExclusiveResource) {
        let remove_state = self
            .resources
            .get(&resource)
            .is_some_and(|state| state.holder.is_none() && state.queue.is_empty());
        if remove_state {
            self.resources.remove(&resource);
        }
    }
}

impl Default for ToolScheduler {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_QUEUE_PER_RESOURCE)
    }
}
