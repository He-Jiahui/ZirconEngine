pub(crate) struct CleanupFixture {
    event_id: &'static str,
    event_display_name: &'static str,
    payload_schema: &'static str,
    plugin_id: &'static str,
    handler_id: &'static str,
    handler_display_name: &'static str,
}

impl CleanupFixture {
    pub(crate) fn event_unregister() -> Self {
        Self {
            event_id: "sound.dynamic.cleanup",
            event_display_name: "Cleanup",
            payload_schema: "sound.dynamic.cleanup.v1",
            plugin_id: "cleanup_plugin",
            handler_id: "cleanup_handler",
            handler_display_name: "Cleanup Handler",
        }
    }

    pub(crate) fn graph_reconfigure() -> Self {
        Self {
            event_id: "sound.dynamic.graph_cleanup",
            event_display_name: "Graph Cleanup",
            payload_schema: "sound.dynamic.graph_cleanup.v1",
            plugin_id: "graph_plugin",
            handler_id: "graph_handler",
            handler_display_name: "Graph Handler",
        }
    }

    pub(crate) fn event_id(&self) -> &'static str {
        self.event_id
    }

    pub(crate) fn event_display_name(&self) -> &'static str {
        self.event_display_name
    }

    pub(crate) fn payload_schema(&self) -> &'static str {
        self.payload_schema
    }

    pub(crate) fn plugin_id(&self) -> &'static str {
        self.plugin_id
    }

    pub(crate) fn handler_id(&self) -> &'static str {
        self.handler_id
    }

    pub(crate) fn handler_display_name(&self) -> &'static str {
        self.handler_display_name
    }
}
