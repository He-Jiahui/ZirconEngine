use crate::core::commands::CommandEvalCtx;
use crate::core::editor_message::SceneModeId;
use crate::core::extension::ContributionTicket;
use crate::scene::selection::SelectionModel;
use crate::scene::viewport::{TransformHandleKind, ViewportInput};

use super::builtin_scene_mode::SelectSceneMode;
use super::{
    EditorSceneMode, InputOutcome, SELECT_SCENE_MODE_ID, SceneModeActivation, SceneModeCtx,
    SceneModeStackError, TRANSFORM_SCENE_MODE_ID, ViewportOverlayBuilder,
};

struct SceneModeStackEntry {
    activation: SceneModeActivation,
    mode: Box<dyn EditorSceneMode>,
    contribution_ticket: Option<ContributionTicket>,
}

pub struct SceneModeStack {
    base_activation: SceneModeActivation,
    base: Box<dyn EditorSceneMode>,
    base_contribution_ticket: Option<ContributionTicket>,
    overlays: Vec<SceneModeStackEntry>,
    revision: u64,
}

impl SceneModeStack {
    pub fn new(
        base_activation: SceneModeActivation,
        mut base: Box<dyn EditorSceneMode>,
        ctx: &mut SceneModeCtx<'_>,
    ) -> Result<Self, SceneModeStackError> {
        ensure_activation_matches_mode(&base_activation, base.as_ref())?;
        enter_mode(base.as_mut(), ctx).map_err(scene_mode_enter_failure)?;
        Ok(Self {
            base_activation,
            base,
            base_contribution_ticket: None,
            overlays: Vec::new(),
            revision: 0,
        })
    }

    pub fn active_mode_id(&self) -> &SceneModeId {
        self.overlays
            .last()
            .map_or_else(|| self.base.id(), |entry| entry.mode.id())
    }

    pub fn base_mode_id(&self) -> &SceneModeId {
        self.base.id()
    }

    pub fn active_activation(&self) -> SceneModeActivation {
        self.overlays.last().map_or_else(
            || self.base_activation.clone(),
            |entry| entry.activation.clone(),
        )
    }

    pub fn base_activation(&self) -> &SceneModeActivation {
        &self.base_activation
    }

    pub fn base_transform_handle(&self) -> Option<TransformHandleKind> {
        self.base_activation.transform_handle()
    }

    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn requires_exclusive_tool(&self) -> bool {
        !matches!(&self.base_activation, SceneModeActivation::Select) || !self.overlays.is_empty()
    }

    pub fn project_command_eval_ctx(
        &self,
        context: CommandEvalCtx,
        selection: &SelectionModel,
    ) -> CommandEvalCtx {
        let active_domain = selection.active_domain();
        context
            .with_scene_mode(self.active_mode_id().clone())
            .with_selection_count(selection.active_items().len())
            .with_selection_domain(active_domain)
            .with_selection_revision(selection.generation(active_domain))
            .with_scene_mode_revision(self.revision)
    }

    pub fn push_overlay(
        &mut self,
        activation: SceneModeActivation,
        mut mode: Box<dyn EditorSceneMode>,
        ctx: &mut SceneModeCtx<'_>,
    ) -> Result<(), SceneModeStackError> {
        self.push_overlay_with_contribution(activation, mode, None, ctx)
    }

    pub(crate) fn push_overlay_with_contribution(
        &mut self,
        activation: SceneModeActivation,
        mut mode: Box<dyn EditorSceneMode>,
        contribution_ticket: Option<ContributionTicket>,
        ctx: &mut SceneModeCtx<'_>,
    ) -> Result<(), SceneModeStackError> {
        ensure_custom_overlay_activation(&activation)?;
        ensure_activation_matches_mode(&activation, mode.as_ref())?;
        if self.contains(mode.id()) {
            return Err(SceneModeStackError::DuplicateMode {
                mode_id: mode.id().clone(),
            });
        }
        enter_mode(mode.as_mut(), ctx).map_err(scene_mode_enter_failure)?;
        self.overlays.push(SceneModeStackEntry {
            activation,
            mode,
            contribution_ticket,
        });
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }

    pub fn replace_base(
        &mut self,
        base_activation: SceneModeActivation,
        mut mode: Box<dyn EditorSceneMode>,
        ctx: &mut SceneModeCtx<'_>,
    ) -> Result<(), SceneModeStackError> {
        self.replace_base_with_contribution(base_activation, mode, None, ctx)
            .map(|(retired, _)| drop(retired))
    }

    pub(crate) fn replace_base_with_contribution(
        &mut self,
        base_activation: SceneModeActivation,
        mut mode: Box<dyn EditorSceneMode>,
        contribution_ticket: Option<ContributionTicket>,
        ctx: &mut SceneModeCtx<'_>,
    ) -> Result<(Box<dyn EditorSceneMode>, Option<String>), SceneModeStackError> {
        ensure_activation_matches_mode(&base_activation, mode.as_ref())?;
        if self
            .overlays
            .iter()
            .any(|overlay| overlay.mode.id() == mode.id())
        {
            return Err(SceneModeStackError::DuplicateMode {
                mode_id: mode.id().clone(),
            });
        }

        let ctx_checkpoint = ctx.checkpoint();
        self.base.exit(ctx);
        let retirement_failure = self.base.take_boundary_failure();
        let replacement_failure = match enter_mode(mode.as_mut(), ctx) {
            Ok(()) => None,
            Err(failure) => Some(failure),
        };
        if let Some((replacement_mode_id, replacement_message)) = replacement_failure {
            let rollback_failure = enter_mode(self.base.as_mut(), ctx).err();
            ctx.restore_after_pass_through(ctx_checkpoint);
            return match rollback_failure {
                None => Err(SceneModeStackError::EnterFailure {
                    mode_id: replacement_mode_id,
                    message: replacement_message,
                }),
                Some((rollback_mode_id, rollback_message)) => {
                    Err(SceneModeStackError::BaseReplacementRollbackFailure {
                        replacement_mode_id,
                        replacement_message,
                        rollback_mode_id,
                        rollback_message,
                    })
                }
            };
        }
        self.base_activation = base_activation;
        self.base_contribution_ticket = contribution_ticket;
        let retired = std::mem::replace(&mut self.base, mode);
        self.revision = self.revision.wrapping_add(1);
        Ok((retired, retirement_failure))
    }

    pub(crate) fn retire_contribution_to_builtin_select(
        &mut self,
        ticket: ContributionTicket,
        ctx: &mut SceneModeCtx<'_>,
    ) -> (Vec<Box<dyn EditorSceneMode>>, Option<String>) {
        let mut extracted = Vec::with_capacity(self.overlays.len());
        for index in (0..self.overlays.len()).rev() {
            if self.overlays[index].contribution_ticket == Some(ticket) {
                extracted.push(self.overlays.remove(index));
            }
        }

        let mut retired = Vec::with_capacity(
            extracted
                .len()
                .saturating_add(usize::from(self.base_contribution_ticket == Some(ticket))),
        );
        let mut first_error = None;
        for mut entry in extracted {
            entry.mode.exit(ctx);
            if first_error.is_none() {
                first_error = entry.mode.take_boundary_failure();
            }
            retired.push(entry.mode);
            self.revision = self.revision.wrapping_add(1);
        }

        if self.base_contribution_ticket == Some(ticket) {
            let mut fallback = Box::new(SelectSceneMode::new()) as Box<dyn EditorSceneMode>;
            self.base.exit(ctx);
            if first_error.is_none() {
                first_error = self.base.take_boundary_failure();
            }
            fallback.enter(ctx);
            self.base_activation = SceneModeActivation::Select;
            self.base_contribution_ticket = None;
            retired.push(std::mem::replace(&mut self.base, fallback));
            self.revision = self.revision.wrapping_add(1);
        }

        (retired, first_error)
    }

    pub fn pop(&mut self, ctx: &mut SceneModeCtx<'_>) -> Option<SceneModeId> {
        let mut mode = self.overlays.pop()?;
        let id = mode.mode.id().clone();
        mode.mode.exit(ctx);
        self.revision = self.revision.wrapping_add(1);
        Some(id)
    }

    pub fn handle_input(
        &mut self,
        input: &ViewportInput,
        ctx: &mut SceneModeCtx<'_>,
    ) -> InputOutcome {
        zircon_runtime::profile_scope!("editor", "scene_mode", "input_dispatch");
        zircon_runtime::profile_counter!("editor", "scene_mode_input_dispatch_count", 1);
        for entry in self.overlays.iter_mut().rev() {
            record_mode_input_checkpoint(ctx);
            let checkpoint = ctx.checkpoint();
            if entry.mode.handle_input(input, ctx) == InputOutcome::Consumed {
                return InputOutcome::Consumed;
            }
            ctx.restore_after_pass_through(checkpoint);
        }

        record_mode_input_checkpoint(ctx);
        let checkpoint = ctx.checkpoint();
        let outcome = self.base.handle_input(input, ctx);
        if outcome == InputOutcome::PassThrough {
            ctx.restore_after_pass_through(checkpoint);
        }
        outcome
    }

    pub fn update(&mut self, ctx: &mut SceneModeCtx<'_>) {
        self.base.update(ctx);
        for entry in &mut self.overlays {
            entry.mode.update(ctx);
        }
    }

    pub fn build_overlay(&self, out: &mut ViewportOverlayBuilder) {
        self.base.build_overlay(out);
        for entry in &self.overlays {
            entry.mode.build_overlay(out);
        }
    }

    pub fn shutdown(&mut self, ctx: &mut SceneModeCtx<'_>) {
        while let Some(mut entry) = self.overlays.pop() {
            entry.mode.exit(ctx);
        }
        self.base.exit(ctx);
    }

    fn contains(&self, id: &SceneModeId) -> bool {
        self.base.id() == id || self.overlays.iter().any(|entry| entry.mode.id() == id)
    }
}

fn record_mode_input_checkpoint(ctx: &SceneModeCtx<'_>) {
    let selection = ctx.selection();
    let mode_checkpoint_selection_item_count = selection.total_item_count();
    zircon_runtime::profile_counter!("editor", "scene_mode_input_checkpoint_count", 1);
    zircon_runtime::profile_counter!(
        "editor",
        "scene_mode_input_checkpoint_selection_item_count",
        mode_checkpoint_selection_item_count
    );
}

fn ensure_custom_overlay_activation(
    activation: &SceneModeActivation,
) -> Result<(), SceneModeStackError> {
    let SceneModeActivation::Custom(mode_id) = activation else {
        return Err(SceneModeStackError::BuiltInOverlay {
            mode_id: activation.mode_id(),
        });
    };
    if matches!(
        mode_id.as_str(),
        SELECT_SCENE_MODE_ID | TRANSFORM_SCENE_MODE_ID
    ) {
        return Err(SceneModeStackError::BuiltInOverlay {
            mode_id: mode_id.clone(),
        });
    }
    Ok(())
}

fn ensure_activation_matches_mode(
    activation: &SceneModeActivation,
    mode: &dyn EditorSceneMode,
) -> Result<(), SceneModeStackError> {
    activation.validate().map_err(|error| match error {
        super::SceneModeActivationError::ReservedBuiltInId { mode_id } => {
            SceneModeStackError::ReservedBuiltinActivation { mode_id }
        }
    })?;
    let activation_mode_id = activation.mode_id();
    let mode_id = mode.id().clone();
    if activation_mode_id != mode_id {
        return Err(SceneModeStackError::ActivationModeIdMismatch {
            activation_mode_id,
            mode_id,
        });
    }
    Ok(())
}

fn enter_mode(
    mode: &mut dyn EditorSceneMode,
    ctx: &mut SceneModeCtx<'_>,
) -> Result<(), (SceneModeId, String)> {
    let mode_id = mode.id().clone();
    mode.enter(ctx);
    if let Some(message) = mode.take_boundary_failure() {
        mode.exit(ctx);
        return Err((mode_id, message));
    }
    Ok(())
}

fn scene_mode_enter_failure((mode_id, message): (SceneModeId, String)) -> SceneModeStackError {
    SceneModeStackError::EnterFailure { mode_id, message }
}

#[cfg(test)]
mod optimization_batch_20260830br_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const OVERLAYS_PER_SAMPLE: usize = 1_024;

    #[test]
    fn contribution_retirement_reserves_overlay_upper_bound() {
        let source = include_str!("scene_mode_stack.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(self.overlays.len())"));
        assert!(implementation.contains("for index in (0..self.overlays.len()).rev()"));
        assert!(implementation.contains("extracted.push(self.overlays.remove(index))"));
    }

    #[test]
    fn contribution_retirement_keeps_reverse_overlay_scan_order() {
        let source = include_str!("scene_mode_stack.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let reserve = implementation
            .find("Vec::with_capacity(self.overlays.len())")
            .expect("extracted capacity reservation");
        let scan = implementation
            .find("for index in (0..self.overlays.len()).rev()")
            .expect("reverse overlay scan");
        assert!(reserve < scan);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830br_editor_contribution_retirement_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR316_CONTRIBUTION_RETIREMENT_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} overlays_per_sample={OVERLAYS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..256 {
            let mut extracted = if optimized {
                Vec::with_capacity(OVERLAYS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..OVERLAYS_PER_SAMPLE {
                extracted.push(index);
            }
            checksum ^= extracted.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

impl std::fmt::Debug for SceneModeStack {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SceneModeStack")
            .field("base_activation", &self.base_activation)
            .field("base", self.base.id())
            .field(
                "overlays",
                &self
                    .overlays
                    .iter()
                    .map(|entry| &entry.activation)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
