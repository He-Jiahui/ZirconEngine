#[test]
fn terminal_tail_defers_surface_blit_to_the_compiled_graph() {
    let source = include_str!("../terminal_frame_packet.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("terminal packet test boundary");
    let product = source
        .find("viewport_product_copy.encode_copy(")
        .expect("retained product copy");
    let diagnostic = source.find("scope.prepare(").expect("copy diagnostic tail");
    let finish = source
        .find("context.command_encoders.finish()")
        .expect("terminal packet finish");
    let history_initialization = source
        .find("command_buffers.insert(0, history_initialization)")
        .expect("history initialization must lead the scene packet");

    assert!(product < diagnostic);
    assert!(diagnostic < finish);
    assert!(finish < history_initialization);
    assert!(source.contains("history_initialization_command_buffer"));
    assert!(!source.contains("submit_graphics_command_buffers("));
    assert!(!source.contains("record_frame_target_blit("));
    assert!(!source.contains("encode_output_target_writeback("));
    assert!(!source.contains("skip_output_target_writeback_after_direct_import("));
    assert!(!source.contains("suppress_output_target_writeback("));
    assert!(!source.contains("queue.submit("));
}
