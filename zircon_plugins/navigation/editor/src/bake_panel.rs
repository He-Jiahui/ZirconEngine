use std::fmt;

use zircon_editor::core::editor_operation::{EditorOperationInvocation, EditorOperationPath};
use zircon_runtime::core::framework::navigation::{NavMeshBakeReport, NavMeshBakeRequest};
use zircon_runtime::core::framework::navigation::{
    NAVIGATION_BAKE_SURFACE_OPERATION, NAVIGATION_CLEAR_SURFACE_OPERATION,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationBakeSurfaceRow {
    pub surface_entity: u64,
    pub label: String,
}

impl NavigationBakeSurfaceRow {
    pub fn new(surface_entity: u64, label: impl Into<String>) -> Self {
        Self {
            surface_entity,
            label: label.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationBakeSelectionError {
    NoSurfaceSelected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavigationBakePanelEvent {
    ReplaceSurfaceRows(Vec<NavigationBakeSurfaceRow>),
    SelectSurface(u64),
    ClearSelection,
    ForceFullRebuildChanged(bool),
    BakeSelectedClicked,
    ClearSelectedClicked,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NavigationBakePanelEventOutcome {
    StateChanged,
    Operation(EditorOperationInvocation),
    Ignored(NavigationBakeSelectionError),
}

impl fmt::Display for NavigationBakeSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSurfaceSelected => formatter.write_str("no navigation surface is selected"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationBakePhase {
    #[default]
    Idle,
    Queued,
    Baking,
    Clearing,
    Complete,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavigationBakeAction {
    BakeScene {
        request: NavMeshBakeRequest,
    },
    BakeSelectedSurface {
        entity: u64,
        request: NavMeshBakeRequest,
    },
    ClearSelectedSurface {
        entity: u64,
    },
}

impl NavigationBakeAction {
    pub fn bake_scene(force_full_rebuild: bool) -> Self {
        Self::BakeScene {
            request: NavMeshBakeRequest {
                force_full_rebuild,
                ..NavMeshBakeRequest::default()
            },
        }
    }

    pub fn bake_selected_surface(entity: u64, force_full_rebuild: bool) -> Self {
        Self::BakeSelectedSurface {
            entity,
            request: NavMeshBakeRequest {
                surface_entity: Some(entity),
                force_full_rebuild,
                ..NavMeshBakeRequest::default()
            },
        }
    }

    pub fn runtime_request(&self) -> Option<&NavMeshBakeRequest> {
        match self {
            Self::BakeScene { request } | Self::BakeSelectedSurface { request, .. } => {
                Some(request)
            }
            Self::ClearSelectedSurface { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationBakeRequest {
    pub id: u64,
    pub action: NavigationBakeAction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationBakeProgress {
    pub request_id: u64,
    pub phase: NavigationBakePhase,
    pub completed_steps: usize,
    pub total_steps: usize,
    pub message: String,
}

impl NavigationBakeProgress {
    pub fn new(
        request_id: u64,
        phase: NavigationBakePhase,
        completed_steps: usize,
        total_steps: usize,
        message: impl Into<String>,
    ) -> Self {
        Self {
            request_id,
            phase,
            completed_steps: completed_steps.min(total_steps),
            total_steps,
            message: message.into(),
        }
    }

    pub fn fraction(&self) -> f32 {
        if self.total_steps == 0 {
            return if self.phase == NavigationBakePhase::Complete {
                1.0
            } else {
                0.0
            };
        }
        self.completed_steps as f32 / self.total_steps as f32
    }
}

impl Default for NavigationBakeProgress {
    fn default() -> Self {
        Self::new(0, NavigationBakePhase::Idle, 0, 0, "Ready")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationBakePanelBusy {
    pub active_request_id: u64,
}

pub trait NavigationBakeBackend {
    type Error: fmt::Display;

    fn submit(&mut self, request: NavigationBakeRequest) -> Result<(), Self::Error>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum NavigationBakeSubmitError<E> {
    Busy(NavigationBakePanelBusy),
    Backend(E),
}

#[derive(Debug, PartialEq, Eq)]
pub enum NavigationBakeSelectedSubmitError<E> {
    Selection(NavigationBakeSelectionError),
    Submit(NavigationBakeSubmitError<E>),
}

impl<E: fmt::Display> fmt::Display for NavigationBakeSelectedSubmitError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Selection(error) => error.fmt(formatter),
            Self::Submit(error) => error.fmt(formatter),
        }
    }
}

impl<E: fmt::Display> fmt::Display for NavigationBakeSubmitError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Busy(error) => write!(
                formatter,
                "navigation bake request {} is still active",
                error.active_request_id
            ),
            Self::Backend(error) => error.fmt(formatter),
        }
    }
}

pub struct NavigationBakePanelController<B> {
    panel: NavigationBakePanel,
    backend: B,
}

impl<B: NavigationBakeBackend> NavigationBakePanelController<B> {
    pub fn new(backend: B) -> Self {
        Self {
            panel: NavigationBakePanel::default(),
            backend,
        }
    }

    pub fn submit(
        &mut self,
        action: NavigationBakeAction,
    ) -> Result<NavigationBakeRequest, NavigationBakeSubmitError<B::Error>> {
        let request = self
            .panel
            .submit(action)
            .map_err(NavigationBakeSubmitError::Busy)?;
        if let Err(error) = self.backend.submit(request.clone()) {
            self.panel.complete(request.id, Err(error.to_string()));
            return Err(NavigationBakeSubmitError::Backend(error));
        }
        Ok(request)
    }

    pub fn replace_surface_rows<I>(&mut self, rows: I)
    where
        I: IntoIterator<Item = NavigationBakeSurfaceRow>,
    {
        self.panel.replace_surface_rows(rows);
    }

    pub fn select_surface(&mut self, surface_entity: u64) -> bool {
        self.panel.select_surface(surface_entity)
    }

    pub fn set_force_full_rebuild(&mut self, force_full_rebuild: bool) {
        self.panel.set_force_full_rebuild(force_full_rebuild);
    }

    pub fn bake_selected(
        &mut self,
    ) -> Result<NavigationBakeRequest, NavigationBakeSelectedSubmitError<B::Error>> {
        let action = self
            .panel
            .bake_selected_action()
            .map_err(NavigationBakeSelectedSubmitError::Selection)?;
        self.submit(action)
            .map_err(NavigationBakeSelectedSubmitError::Submit)
    }

    pub fn clear_selected(
        &mut self,
    ) -> Result<NavigationBakeRequest, NavigationBakeSelectedSubmitError<B::Error>> {
        let action = self
            .panel
            .clear_selected_action()
            .map_err(NavigationBakeSelectedSubmitError::Selection)?;
        self.submit(action)
            .map_err(NavigationBakeSelectedSubmitError::Submit)
    }

    pub fn observe_progress(&mut self, progress: NavigationBakeProgress) -> bool {
        self.panel.observe_progress(progress)
    }

    pub fn complete(&mut self, request_id: u64, result: Result<NavMeshBakeReport, String>) -> bool {
        self.panel.complete(request_id, result)
    }

    pub fn panel(&self) -> &NavigationBakePanel {
        &self.panel
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }
}

#[derive(Clone, Debug, Default)]
pub struct NavigationBakePanel {
    surface_rows: Vec<NavigationBakeSurfaceRow>,
    selected_surface_entity: Option<u64>,
    force_full_rebuild: bool,
    next_request_id: u64,
    active_request: Option<NavigationBakeRequest>,
    progress: NavigationBakeProgress,
    last_report: Option<NavMeshBakeReport>,
    last_error: Option<String>,
}

impl NavigationBakePanel {
    pub fn handle_retained_event(
        &mut self,
        event: NavigationBakePanelEvent,
    ) -> NavigationBakePanelEventOutcome {
        match event {
            NavigationBakePanelEvent::ReplaceSurfaceRows(rows) => {
                self.replace_surface_rows(rows);
                NavigationBakePanelEventOutcome::StateChanged
            }
            NavigationBakePanelEvent::SelectSurface(surface_entity) => {
                if self.select_surface(surface_entity) {
                    NavigationBakePanelEventOutcome::StateChanged
                } else {
                    NavigationBakePanelEventOutcome::Ignored(
                        NavigationBakeSelectionError::NoSurfaceSelected,
                    )
                }
            }
            NavigationBakePanelEvent::ClearSelection => {
                self.clear_surface_selection();
                NavigationBakePanelEventOutcome::StateChanged
            }
            NavigationBakePanelEvent::ForceFullRebuildChanged(force_full_rebuild) => {
                self.set_force_full_rebuild(force_full_rebuild);
                NavigationBakePanelEventOutcome::StateChanged
            }
            NavigationBakePanelEvent::BakeSelectedClicked => self
                .selected_operation_invocation(NAVIGATION_BAKE_SURFACE_OPERATION)
                .map(NavigationBakePanelEventOutcome::Operation)
                .unwrap_or_else(NavigationBakePanelEventOutcome::Ignored),
            NavigationBakePanelEvent::ClearSelectedClicked => self
                .selected_operation_invocation(NAVIGATION_CLEAR_SURFACE_OPERATION)
                .map(NavigationBakePanelEventOutcome::Operation)
                .unwrap_or_else(NavigationBakePanelEventOutcome::Ignored),
        }
    }

    pub fn selected_actions_enabled(&self) -> bool {
        self.selected_surface_entity.is_some()
    }

    pub fn replace_surface_rows<I>(&mut self, rows: I)
    where
        I: IntoIterator<Item = NavigationBakeSurfaceRow>,
    {
        self.surface_rows = rows.into_iter().collect();
        if self.selected_surface_entity.is_some_and(|selected| {
            !self
                .surface_rows
                .iter()
                .any(|row| row.surface_entity == selected)
        }) {
            self.selected_surface_entity = None;
        }
    }

    pub fn surface_rows(&self) -> &[NavigationBakeSurfaceRow] {
        &self.surface_rows
    }

    pub fn select_surface(&mut self, surface_entity: u64) -> bool {
        if !self
            .surface_rows
            .iter()
            .any(|row| row.surface_entity == surface_entity)
        {
            return false;
        }
        self.selected_surface_entity = Some(surface_entity);
        true
    }

    pub fn clear_surface_selection(&mut self) {
        self.selected_surface_entity = None;
    }

    pub fn selected_surface_entity(&self) -> Option<u64> {
        self.selected_surface_entity
    }

    pub fn set_force_full_rebuild(&mut self, force_full_rebuild: bool) {
        self.force_full_rebuild = force_full_rebuild;
    }

    pub fn force_full_rebuild(&self) -> bool {
        self.force_full_rebuild
    }

    pub fn bake_selected_action(
        &self,
    ) -> Result<NavigationBakeAction, NavigationBakeSelectionError> {
        let entity = self
            .selected_surface_entity
            .ok_or(NavigationBakeSelectionError::NoSurfaceSelected)?;
        Ok(NavigationBakeAction::bake_selected_surface(
            entity,
            self.force_full_rebuild,
        ))
    }

    pub fn clear_selected_action(
        &self,
    ) -> Result<NavigationBakeAction, NavigationBakeSelectionError> {
        let entity = self
            .selected_surface_entity
            .ok_or(NavigationBakeSelectionError::NoSurfaceSelected)?;
        Ok(NavigationBakeAction::ClearSelectedSurface { entity })
    }

    fn selected_operation_invocation(
        &self,
        operation: &str,
    ) -> Result<EditorOperationInvocation, NavigationBakeSelectionError> {
        let surface_entity = self
            .selected_surface_entity
            .ok_or(NavigationBakeSelectionError::NoSurfaceSelected)?;
        let operation = EditorOperationPath::parse(operation)
            .expect("navigation retained operation constants must remain valid");
        let arguments = if operation.as_str() == NAVIGATION_BAKE_SURFACE_OPERATION {
            serde_json::json!([surface_entity, self.force_full_rebuild])
        } else {
            serde_json::json!([surface_entity])
        };
        Ok(EditorOperationInvocation::new(operation).with_arguments(arguments))
    }

    pub fn submit(
        &mut self,
        action: NavigationBakeAction,
    ) -> Result<NavigationBakeRequest, NavigationBakePanelBusy> {
        if let Some(active) = &self.active_request {
            return Err(NavigationBakePanelBusy {
                active_request_id: active.id,
            });
        }
        self.next_request_id = self.next_request_id.saturating_add(1).max(1);
        let request = NavigationBakeRequest {
            id: self.next_request_id,
            action,
        };
        let phase = if matches!(
            request.action,
            NavigationBakeAction::ClearSelectedSurface { .. }
        ) {
            NavigationBakePhase::Clearing
        } else {
            NavigationBakePhase::Queued
        };
        self.progress = NavigationBakeProgress::new(request.id, phase, 0, 0, "Queued");
        self.active_request = Some(request.clone());
        self.last_report = None;
        self.last_error = None;
        Ok(request)
    }

    pub fn observe_progress(&mut self, progress: NavigationBakeProgress) -> bool {
        let Some(active) = &self.active_request else {
            return false;
        };
        if progress.request_id != active.id
            || !progress_matches_action(&active.action, progress.phase)
            || !progress_is_monotonic(&self.progress, &progress)
        {
            return false;
        }
        self.progress = progress;
        true
    }

    pub fn complete(&mut self, request_id: u64, result: Result<NavMeshBakeReport, String>) -> bool {
        if self.active_request.as_ref().map(|request| request.id) != Some(request_id) {
            return false;
        }
        self.active_request = None;
        match result {
            Ok(report) => {
                let total = self.progress.total_steps.max(report.tiles).max(1);
                self.progress = NavigationBakeProgress::new(
                    request_id,
                    NavigationBakePhase::Complete,
                    total,
                    total,
                    "Bake complete",
                );
                self.last_report = Some(report);
                self.last_error = None;
            }
            Err(error) => {
                self.progress.phase = NavigationBakePhase::Failed;
                self.progress.message.clone_from(&error);
                self.last_report = None;
                self.last_error = Some(error);
            }
        }
        true
    }

    pub fn phase(&self) -> NavigationBakePhase {
        self.progress.phase
    }

    pub fn progress(&self) -> &NavigationBakeProgress {
        &self.progress
    }

    pub fn active_request(&self) -> Option<&NavigationBakeRequest> {
        self.active_request.as_ref()
    }

    pub fn last_report(&self) -> Option<&NavMeshBakeReport> {
        self.last_report.as_ref()
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }
}

fn progress_matches_action(action: &NavigationBakeAction, phase: NavigationBakePhase) -> bool {
    match action {
        NavigationBakeAction::BakeScene { .. }
        | NavigationBakeAction::BakeSelectedSurface { .. } => {
            matches!(
                phase,
                NavigationBakePhase::Queued | NavigationBakePhase::Baking
            )
        }
        NavigationBakeAction::ClearSelectedSurface { .. } => phase == NavigationBakePhase::Clearing,
    }
}

fn progress_is_monotonic(current: &NavigationBakeProgress, next: &NavigationBakeProgress) -> bool {
    if phase_rank(next.phase) < phase_rank(current.phase)
        || matches!(
            next.phase,
            NavigationBakePhase::Idle | NavigationBakePhase::Complete | NavigationBakePhase::Failed
        )
    {
        return false;
    }
    if current.total_steps == 0 {
        return true;
    }
    if next.total_steps == 0 {
        return false;
    }
    next.completed_steps.saturating_mul(current.total_steps)
        >= current.completed_steps.saturating_mul(next.total_steps)
}

fn phase_rank(phase: NavigationBakePhase) -> u8 {
    match phase {
        NavigationBakePhase::Idle => 0,
        NavigationBakePhase::Queued => 1,
        NavigationBakePhase::Baking | NavigationBakePhase::Clearing => 2,
        NavigationBakePhase::Complete | NavigationBakePhase::Failed => 3,
    }
}
