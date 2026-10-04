#[test]
fn clustered_lighting_avoids_cpu_clear_and_inactive_light_uploads() {
    let source = include_str!("../execute_clustered_lighting.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("clustered-lighting production source");
    let cpu_clear = ["vec![0_u8;", " cluster_buffer_bytes]"].concat();
    let legacy_size_argument = ["cluster_buffer_", "bytes: usize"].concat();
    let gpu_clear = ["encoder.clear_", "buffer(cluster_buffer, 0, None)"].concat();
    let active_prefix = ["&gpu_lights[..", "directional_light_count]"].concat();

    assert!(!source.contains(&cpu_clear));
    assert!(!source.contains(&legacy_size_argument));
    // BUG: [CR-SCENE-POST-0004] 此断言匹配旧的整缓冲调用；当前生产路径使用带 offset/size 的绑定范围，源码匹配必为 false。
    assert!(source.contains(&gpu_clear));
    assert!(source.contains(&active_prefix));
    assert!(!production.contains("queue.write_buffer"));
    assert_eq!(production.matches("let payload: Arc<[u8]>").count(), 1);
    assert!(production.contains("WgpuBufferUploadBatch"));
    assert!(production.contains("self.light_buffer.clone()"));
    assert!(production.contains("self.cluster_params_buffer.clone()"));
}
