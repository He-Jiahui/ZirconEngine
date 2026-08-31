use crate::graphics::scene::render_scene::{
    RenderSceneChangeJournal, RenderScenePrimitive, RenderScenePrimitiveDirtyFlags,
    RenderScenePrimitiveHandle,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuSceneJournalResidentWriteKind {
    Full,
    Dirty,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GpuSceneJournalResidentWrite<'journal> {
    handle: RenderScenePrimitiveHandle,
    dirty: RenderScenePrimitiveDirtyFlags,
    kind: GpuSceneJournalResidentWriteKind,
    primitive: &'journal RenderScenePrimitive,
}

impl<'journal> GpuSceneJournalResidentWrite<'journal> {
    const fn new(
        handle: RenderScenePrimitiveHandle,
        dirty: RenderScenePrimitiveDirtyFlags,
        kind: GpuSceneJournalResidentWriteKind,
        primitive: &'journal RenderScenePrimitive,
    ) -> Self {
        Self {
            handle,
            dirty,
            kind,
            primitive,
        }
    }

    pub(super) const fn full(
        handle: RenderScenePrimitiveHandle,
        primitive: &'journal RenderScenePrimitive,
    ) -> Self {
        Self::new(
            handle,
            RenderScenePrimitiveDirtyFlags::ALL,
            GpuSceneJournalResidentWriteKind::Full,
            primitive,
        )
    }

    pub(crate) const fn handle(&self) -> RenderScenePrimitiveHandle {
        self.handle
    }

    pub(crate) const fn dirty(&self) -> RenderScenePrimitiveDirtyFlags {
        self.dirty
    }

    pub(crate) const fn kind(&self) -> GpuSceneJournalResidentWriteKind {
        self.kind
    }

    pub(crate) const fn primitive(&self) -> &'journal RenderScenePrimitive {
        self.primitive
    }

    pub(crate) const fn requires_instance_transform_write(&self) -> bool {
        matches!(self.kind, GpuSceneJournalResidentWriteKind::Full)
            || self
                .dirty
                .contains(RenderScenePrimitiveDirtyFlags::TRANSFORM)
    }

    pub(crate) const fn requires_local_bounds_write(&self) -> bool {
        matches!(self.kind, GpuSceneJournalResidentWriteKind::Full)
            || self
                .dirty
                .contains(RenderScenePrimitiveDirtyFlags::LOCAL_BOUNDS)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GpuSceneJournalRetirement<'journal> {
    handle: RenderScenePrimitiveHandle,
    primitive: &'journal RenderScenePrimitive,
}

impl<'journal> GpuSceneJournalRetirement<'journal> {
    const fn new(
        handle: RenderScenePrimitiveHandle,
        primitive: &'journal RenderScenePrimitive,
    ) -> Self {
        Self { handle, primitive }
    }

    pub(crate) const fn handle(&self) -> RenderScenePrimitiveHandle {
        self.handle
    }

    pub(crate) const fn stable_instance_key(&self) -> u64 {
        self.primitive.stable_instance_key()
    }

    pub(crate) const fn primitive(&self) -> &'journal RenderScenePrimitive {
        self.primitive
    }
}

#[derive(Debug)]
pub(super) struct GpuSceneJournalWorkSet<'journal> {
    resident_writes: Vec<GpuSceneJournalResidentWrite<'journal>>,
    retirements: Vec<GpuSceneJournalRetirement<'journal>>,
    full_resident_write_count: usize,
    dirty_resident_write_count: usize,
    instance_transform_write_count: usize,
    local_bounds_write_count: usize,
}

impl<'journal> GpuSceneJournalWorkSet<'journal> {
    pub(super) const fn empty() -> Self {
        Self {
            resident_writes: Vec::new(),
            retirements: Vec::new(),
            full_resident_write_count: 0,
            dirty_resident_write_count: 0,
            instance_transform_write_count: 0,
            local_bounds_write_count: 0,
        }
    }

    pub(super) fn compile(journal: &'journal RenderSceneChangeJournal) -> Self {
        let dirty_resident_write_count = journal.updates().len();
        let full_resident_write_count = journal.additions().len();
        let mut resident_writes = Vec::with_capacity(
            dirty_resident_write_count.saturating_add(full_resident_write_count),
        );
        let mut instance_transform_write_count = full_resident_write_count;
        let mut local_bounds_write_count = full_resident_write_count;
        for update in journal.updates() {
            let write = GpuSceneJournalResidentWrite::new(
                update.handle(),
                update.dirty(),
                GpuSceneJournalResidentWriteKind::Dirty,
                update.primitive().as_ref(),
            );
            instance_transform_write_count += write.requires_instance_transform_write() as usize;
            local_bounds_write_count += write.requires_local_bounds_write() as usize;
            resident_writes.push(write);
        }
        resident_writes.extend(journal.additions().iter().map(|addition| {
            GpuSceneJournalResidentWrite::new(
                addition.handle(),
                RenderScenePrimitiveDirtyFlags::ALL,
                GpuSceneJournalResidentWriteKind::Full,
                addition.primitive().as_ref(),
            )
        }));
        resident_writes.sort_unstable_by_key(|write| write.handle().slot());

        let mut retirements = journal
            .removals()
            .iter()
            .map(|removal| {
                GpuSceneJournalRetirement::new(removal.handle(), removal.primitive().as_ref())
            })
            .collect::<Vec<_>>();
        retirements.sort_unstable_by_key(|retirement| retirement.handle().slot());

        Self {
            resident_writes,
            retirements,
            full_resident_write_count,
            dirty_resident_write_count,
            instance_transform_write_count,
            local_bounds_write_count,
        }
    }

    pub(super) fn resident_writes(&self) -> &[GpuSceneJournalResidentWrite<'journal>] {
        &self.resident_writes
    }

    pub(super) fn retirements(&self) -> &[GpuSceneJournalRetirement<'journal>] {
        &self.retirements
    }

    pub(super) const fn full_resident_write_count(&self) -> usize {
        self.full_resident_write_count
    }

    pub(super) const fn dirty_resident_write_count(&self) -> usize {
        self.dirty_resident_write_count
    }

    pub(super) const fn instance_transform_write_count(&self) -> usize {
        self.instance_transform_write_count
    }

    pub(super) const fn local_bounds_write_count(&self) -> usize {
        self.local_bounds_write_count
    }

    pub(super) fn is_empty(&self) -> bool {
        self.resident_writes.is_empty()
            && self.retirements.is_empty()
            && self.full_resident_write_count == 0
            && self.dirty_resident_write_count == 0
            && self.instance_transform_write_count == 0
            && self.local_bounds_write_count == 0
    }
}
