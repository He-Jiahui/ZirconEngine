use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::UiComponentEventKind,
    event_ui::UiNodeId,
    template::{UiCompiledBindingGeneration, UiCompiledBindingHandle, UiCompiledBindingProgram},
};

use super::UiSurface;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UiCompiledBindingEventEntry {
    pub(super) source_binding_index: usize,
    pub(super) handle: UiCompiledBindingHandle,
    pub(super) component_event: Option<UiComponentEventKind>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct UiCompiledBindingEventIndex {
    generation: UiCompiledBindingGeneration,
    entries: BTreeMap<(UiNodeId, UiEventKind), Vec<UiCompiledBindingEventEntry>>,
}

impl UiCompiledBindingEventIndex {
    pub(super) fn from_program(program: &UiCompiledBindingProgram) -> Self {
        if program.generation().is_invalid() {
            return Self::default();
        }
        let mut entries = BTreeMap::<_, Vec<_>>::new();
        for binding in program.iter_bindings() {
            let node_id = UiNodeId::new(u64::from(binding.node_id.get()) + 1);
            entries.entry((node_id, binding.event)).or_default().push(
                UiCompiledBindingEventEntry {
                    source_binding_index: binding.source_binding_index as usize,
                    handle: binding.handle,
                    component_event: binding.component_event,
                },
            );
        }
        Self {
            generation: program.generation(),
            entries,
        }
    }

    fn entries_for(
        &self,
        program: &UiCompiledBindingProgram,
        node_id: UiNodeId,
        event_kind: UiEventKind,
    ) -> Option<&[UiCompiledBindingEventEntry]> {
        (self.generation == program.generation() && !self.generation.is_invalid()).then(|| {
            self.entries
                .get(&(node_id, event_kind))
                .map(Vec::as_slice)
                .unwrap_or_default()
        })
    }
}

impl UiSurface {
    pub(in crate::ui::surface) fn compiled_binding_event_sources(
        &self,
        node_id: UiNodeId,
        event_kind: UiEventKind,
    ) -> Option<&[UiCompiledBindingEventEntry]> {
        self.compiled_binding_event_index
            .entries_for(&self.compiled_bindings, node_id, event_kind)
    }

    #[cfg(test)]
    pub(crate) fn compiled_binding_event_source_count_for_test(
        &self,
        node_id: UiNodeId,
        event_kind: UiEventKind,
    ) -> Option<usize> {
        self.compiled_binding_event_sources(node_id, event_kind)
            .map(<[_]>::len)
    }

    #[cfg(test)]
    pub(crate) fn compiled_binding_event_sources_for_benchmark(
        &self,
        node_id: UiNodeId,
        event_kind: UiEventKind,
    ) -> impl Iterator<Item = (usize, UiCompiledBindingHandle, Option<UiComponentEventKind>)> + '_
    {
        self.compiled_binding_event_sources(node_id, event_kind)
            .into_iter()
            .flatten()
            .map(|entry| {
                (
                    entry.source_binding_index,
                    entry.handle,
                    entry.component_event,
                )
            })
    }
}
