use crate::scene::ecs::InternalSceneSystem;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DerivedStateDirty {
    hierarchy: bool,
    active: bool,
    transforms: bool,
    node_cache: bool,
    render_extract: bool,
    defer_flush: bool,
}

impl Default for DerivedStateDirty {
    fn default() -> Self {
        Self::all()
    }
}

impl DerivedStateDirty {
    pub(super) fn all() -> Self {
        Self {
            hierarchy: true,
            active: true,
            transforms: true,
            node_cache: true,
            render_extract: true,
            defer_flush: false,
        }
    }

    pub(super) fn mark_hierarchy(&mut self) {
        self.hierarchy = true;
        self.active = true;
        self.transforms = true;
        self.node_cache = true;
        self.render_extract = true;
    }

    pub(super) fn mark_active(&mut self) {
        self.active = true;
        self.node_cache = true;
        self.render_extract = true;
    }

    pub(super) fn mark_transform(&mut self) {
        self.transforms = true;
        self.node_cache = true;
        self.render_extract = true;
    }

    pub(super) fn mark_node_cache(&mut self) {
        self.node_cache = true;
        self.render_extract = true;
    }

    pub(super) fn should_run(&self, system: InternalSceneSystem) -> bool {
        match system {
            InternalSceneSystem::HierarchyValidity => self.hierarchy,
            InternalSceneSystem::ActiveHierarchy => self.active,
            InternalSceneSystem::WorldTransform => self.transforms,
            InternalSceneSystem::NodeCache => self.node_cache,
            InternalSceneSystem::RenderExtractPrepare => self.render_extract,
        }
    }

    pub(super) fn clear(&mut self, system: InternalSceneSystem) {
        match system {
            InternalSceneSystem::HierarchyValidity => self.hierarchy = false,
            InternalSceneSystem::ActiveHierarchy => self.active = false,
            InternalSceneSystem::WorldTransform => self.transforms = false,
            InternalSceneSystem::NodeCache => self.node_cache = false,
            InternalSceneSystem::RenderExtractPrepare => self.render_extract = false,
        }
    }

    pub(super) fn set_defer_flush(&mut self, defer_flush: bool) {
        self.defer_flush = defer_flush;
    }

    pub(super) fn hierarchy_or_transform_pending(&self) -> bool {
        self.hierarchy || self.transforms
    }

    pub(super) fn active_pending(&self) -> bool {
        self.hierarchy || self.active
    }

    pub(super) fn has_pending(&self) -> bool {
        self.hierarchy || self.active || self.transforms || self.node_cache || self.render_extract
    }
}
