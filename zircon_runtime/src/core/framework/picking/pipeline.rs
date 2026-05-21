use super::{
    CameraRaySource, PickingBackend, PickingEventState, PickingHoverMap, PickingPipelineReport,
    PickingPointerEvent, PickingScheduleLabel, PickingSettings, PointerHits, PointerInput,
    PointerLocation, RayMap,
};

pub struct PickingPipelineInput<'a> {
    pub settings: PickingSettings,
    pub pointer_locations: &'a [PointerLocation],
    pub pointer_inputs: &'a [PointerInput],
    pub cameras: &'a [CameraRaySource],
    pub backends: &'a [&'a dyn PickingBackend],
}

impl<'a> PickingPipelineInput<'a> {
    pub const fn new(
        pointer_locations: &'a [PointerLocation],
        pointer_inputs: &'a [PointerInput],
        cameras: &'a [CameraRaySource],
        backends: &'a [&'a dyn PickingBackend],
    ) -> Self {
        Self {
            settings: PickingSettings::DEFAULT,
            pointer_locations,
            pointer_inputs,
            cameras,
            backends,
        }
    }

    pub const fn with_settings(mut self, settings: PickingSettings) -> Self {
        self.settings = settings;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PickingPipelineOutput {
    pub ray_map: RayMap,
    pub backend_outputs: Vec<PointerHits>,
    pub hover_map: PickingHoverMap,
    pub events: Vec<PickingPointerEvent>,
    pub report: PickingPipelineReport,
    pub stages: Vec<PickingPipelineStageReport>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PickingPipelineStageReport {
    pub label: PickingScheduleLabel,
    pub enabled: bool,
    pub input_count: usize,
    pub output_count: usize,
}

impl PickingPipelineStageReport {
    pub const fn new(
        label: PickingScheduleLabel,
        enabled: bool,
        input_count: usize,
        output_count: usize,
    ) -> Self {
        Self {
            label,
            enabled,
            input_count,
            output_count,
        }
    }
}

pub fn run_picking_pipeline(
    event_state: &mut PickingEventState,
    input: PickingPipelineInput<'_>,
) -> PickingPipelineOutput {
    if !input.settings.enabled {
        event_state.clear();
        return disabled_output(input);
    }

    let mut stages = vec![PickingPipelineStageReport::new(
        PickingScheduleLabel::Input,
        true,
        input.pointer_locations.len() + input.pointer_inputs.len(),
        input.pointer_locations.len(),
    )];

    let mut ray_map = RayMap::default();
    if input.settings.ray_map_enabled {
        ray_map.rebuild(input.pointer_locations, input.cameras);
    }
    stages.push(PickingPipelineStageReport::new(
        PickingScheduleLabel::RayMap,
        input.settings.ray_map_enabled,
        input.pointer_locations.len() * input.cameras.len(),
        ray_map.len(),
    ));

    let backend_outputs = if input.settings.ray_map_enabled {
        input
            .backends
            .iter()
            .flat_map(|backend| backend.collect_hits(&ray_map))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    stages.push(PickingPipelineStageReport::new(
        PickingScheduleLabel::Backend,
        input.settings.ray_map_enabled,
        input.backends.len(),
        backend_outputs.len(),
    ));

    let hover_map = PickingHoverMap::from_outputs(&backend_outputs);
    let hovered_hit_count = hover_map.iter().map(|(_, hits)| hits.len()).sum();
    stages.push(PickingPipelineStageReport::new(
        PickingScheduleLabel::Hover,
        true,
        backend_outputs.iter().map(|output| output.hits.len()).sum(),
        hovered_hit_count,
    ));

    let events = event_state.dispatch_frame(
        hover_map.clone(),
        input.pointer_locations,
        input.pointer_inputs,
    );
    stages.push(PickingPipelineStageReport::new(
        PickingScheduleLabel::Events,
        true,
        input.pointer_inputs.len() + hovered_hit_count,
        events.len(),
    ));

    let report = PickingPipelineReport::from_ray_map_and_outputs(&ray_map, &backend_outputs);

    PickingPipelineOutput {
        ray_map,
        backend_outputs,
        hover_map,
        events,
        report,
        stages,
    }
}

fn disabled_output(input: PickingPipelineInput<'_>) -> PickingPipelineOutput {
    let stages = vec![
        PickingPipelineStageReport::new(
            PickingScheduleLabel::Input,
            false,
            input.pointer_locations.len() + input.pointer_inputs.len(),
            0,
        ),
        PickingPipelineStageReport::new(PickingScheduleLabel::RayMap, false, 0, 0),
        PickingPipelineStageReport::new(PickingScheduleLabel::Backend, false, 0, 0),
        PickingPipelineStageReport::new(PickingScheduleLabel::Hover, false, 0, 0),
        PickingPipelineStageReport::new(PickingScheduleLabel::Events, false, 0, 0),
    ];

    PickingPipelineOutput {
        ray_map: RayMap::default(),
        backend_outputs: Vec::new(),
        hover_map: PickingHoverMap::default(),
        events: Vec::new(),
        report: PickingPipelineReport::default(),
        stages,
    }
}
