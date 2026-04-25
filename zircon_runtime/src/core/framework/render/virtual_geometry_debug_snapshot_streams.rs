use super::virtual_geometry_debug_snapshot::{
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
    RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord,
};

impl RenderVirtualGeometryDebugSnapshot {
    pub fn node_and_cluster_cull_instance_seed_words(&self) -> Vec<u32> {
        packed_word_stream(
            &self.node_and_cluster_cull_instance_seeds,
            RenderVirtualGeometryNodeAndClusterCullInstanceSeed::packed_words,
        )
    }

    pub fn node_and_cluster_cull_instance_work_item_words(&self) -> Vec<u32> {
        packed_word_stream(
            &self.node_and_cluster_cull_instance_work_items,
            RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::packed_words,
        )
    }

    pub fn node_and_cluster_cull_cluster_work_item_words(&self) -> Vec<u32> {
        packed_word_stream(
            &self.node_and_cluster_cull_cluster_work_items,
            RenderVirtualGeometryNodeAndClusterCullClusterWorkItem::packed_words,
        )
    }

    pub fn node_and_cluster_cull_child_work_item_words(&self) -> Vec<u32> {
        packed_word_stream(
            &self.node_and_cluster_cull_child_work_items,
            RenderVirtualGeometryNodeAndClusterCullChildWorkItem::packed_words,
        )
    }

    pub fn node_and_cluster_cull_traversal_record_words(&self) -> Vec<u32> {
        packed_word_stream(
            &self.node_and_cluster_cull_traversal_records,
            RenderVirtualGeometryNodeAndClusterCullTraversalRecord::packed_words,
        )
    }

    pub fn node_and_cluster_cull_hierarchy_child_id_words(&self) -> Vec<u32> {
        self.node_and_cluster_cull_hierarchy_child_ids.clone()
    }

    pub fn node_and_cluster_cull_page_request_id_words(&self) -> Vec<u32> {
        self.node_and_cluster_cull_page_request_ids.clone()
    }

    pub fn node_and_cluster_cull_global_state_words(&self) -> Option<Vec<u32>> {
        self.node_and_cluster_cull_global_state
            .as_ref()
            .map(|global_state| global_state.packed_words().to_vec())
    }

    pub fn node_and_cluster_cull_dispatch_setup_words(&self) -> Option<Vec<u32>> {
        self.node_and_cluster_cull_dispatch_setup
            .as_ref()
            .map(|dispatch_setup| dispatch_setup.packed_words().to_vec())
    }

    pub fn node_and_cluster_cull_launch_worklist_words(&self) -> Option<Vec<u32>> {
        self.node_and_cluster_cull_launch_worklist
            .as_ref()
            .map(|worklist| worklist.packed_words())
    }

    pub fn decode_node_and_cluster_cull_instance_seed_words(
        words: &[u32],
    ) -> Option<Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>> {
        decode_packed_word_stream::<
            RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
            { RenderVirtualGeometryNodeAndClusterCullInstanceSeed::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometryNodeAndClusterCullInstanceSeed::from_packed_words,
        )
    }

    pub fn decode_node_and_cluster_cull_instance_work_item_words(
        words: &[u32],
    ) -> Option<Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>> {
        decode_packed_word_stream::<
            RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
            { RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::from_packed_words,
        )
    }

    pub fn decode_node_and_cluster_cull_cluster_work_item_words(
        words: &[u32],
    ) -> Option<Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>> {
        decode_packed_word_stream::<
            RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
            { RenderVirtualGeometryNodeAndClusterCullClusterWorkItem::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometryNodeAndClusterCullClusterWorkItem::from_packed_words,
        )
    }

    pub fn decode_node_and_cluster_cull_child_work_item_words(
        words: &[u32],
    ) -> Option<Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem>> {
        decode_packed_word_stream::<
            RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
            { RenderVirtualGeometryNodeAndClusterCullChildWorkItem::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometryNodeAndClusterCullChildWorkItem::from_packed_words,
        )
    }

    pub fn decode_node_and_cluster_cull_traversal_record_words(
        words: &[u32],
    ) -> Option<Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>> {
        decode_packed_word_stream::<
            RenderVirtualGeometryNodeAndClusterCullTraversalRecord,
            { RenderVirtualGeometryNodeAndClusterCullTraversalRecord::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometryNodeAndClusterCullTraversalRecord::from_packed_words,
        )
    }

    pub fn decode_node_and_cluster_cull_hierarchy_child_id_words(words: &[u32]) -> Vec<u32> {
        words.to_vec()
    }

    pub fn decode_node_and_cluster_cull_page_request_id_words(words: &[u32]) -> Vec<u32> {
        words.to_vec()
    }

    pub fn decode_node_and_cluster_cull_global_state_words(
        words: &[u32],
    ) -> Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot> {
        decode_exact_packed_words::<
            RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
            { RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot::from_packed_words,
        )
    }

    pub fn decode_node_and_cluster_cull_dispatch_setup_words(
        words: &[u32],
    ) -> Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot> {
        decode_exact_packed_words::<
            RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
            { RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot::GPU_WORD_COUNT },
        >(
            words,
            RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot::from_packed_words,
        )
    }

    pub fn decode_node_and_cluster_cull_launch_worklist_words(
        words: &[u32],
    ) -> Option<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot> {
        let worklist =
            RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot::from_packed_words(
                words,
            )?;
        (worklist.packed_words().len() == words.len()).then_some(worklist)
    }
}

fn packed_word_stream<T, const N: usize>(entries: &[T], pack: impl Fn(&T) -> [u32; N]) -> Vec<u32> {
    entries.iter().flat_map(pack).collect()
}

fn decode_packed_word_stream<T, const N: usize>(
    words: &[u32],
    decode: impl Fn(&[u32]) -> Option<T>,
) -> Option<Vec<T>> {
    if words.len() % N != 0 {
        return None;
    }

    words.chunks_exact(N).map(decode).collect()
}

fn decode_exact_packed_words<T, const N: usize>(
    words: &[u32],
    decode: impl Fn(&[u32]) -> Option<T>,
) -> Option<T> {
    if words.len() != N {
        return None;
    }

    decode(words)
}
