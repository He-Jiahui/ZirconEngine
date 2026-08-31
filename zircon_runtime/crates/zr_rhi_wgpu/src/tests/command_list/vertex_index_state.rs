use super::*;

#[test]
fn command_list_raster_draw_submit_validates_vertex_and_index_buffer_state() {
    let device = DeterministicRhiContractDevice::new_headless();
    let vertex_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "raster-vs",
            ShaderStage::Vertex,
            "vs_main",
            "@vertex fn vs_main() {}",
        ))
        .unwrap();
    let fragment_shader = device
        .create_shader_module(&ShaderModuleDesc::new(
            "raster-fs",
            ShaderStage::Fragment,
            "fs_main",
            "@fragment fn fs_main() {}",
        ))
        .unwrap();
    let pipeline = create_raster_pipeline_with_vertex_input(
        &device,
        "raster-forward",
        vertex_shader,
        fragment_shader,
        create_raster_vertex_input_layout(),
    );
    let vertex_buffer = device
        .create_buffer(&BufferDesc::new("raster-vertices", 36, BufferUsage::VERTEX))
        .unwrap();
    let instance_buffer = device
        .create_buffer(&BufferDesc::new(
            "raster-instances",
            32,
            BufferUsage::VERTEX,
        ))
        .unwrap();
    let index_buffer = device
        .create_buffer(&BufferDesc::new("raster-indices", 12, BufferUsage::INDEX))
        .unwrap();
    let not_vertex = device
        .create_buffer(&BufferDesc::new("not-vertex", 36, BufferUsage::COPY_DST))
        .unwrap();
    let not_index = device
        .create_buffer(&BufferDesc::new("not-index", 12, BufferUsage::VERTEX))
        .unwrap();
    let color =
        create_render_attachment(&device, "raster-state-color", TextureFormat::Rgba8UnormSrgb);
    let depth = create_render_attachment(&device, "raster-state-depth", TextureFormat::Depth24Plus);

    let mut missing_instance = device
        .create_command_list(RenderQueueClass::Graphics, "missing-instance-buffer")
        .unwrap();
    begin_default_render_pass(&mut *missing_instance, color, depth);
    missing_instance.set_pipeline(pipeline);
    missing_instance.set_vertex_buffer(0, vertex_buffer, 0, 36);
    missing_instance.draw(0, 3, 0, 1);
    assert_eq!(
        device.submit(missing_instance).unwrap_err(),
        RhiError::InvalidRasterDraw {
            reason: "draw requires vertex buffer slot 1 to be bound".to_string(),
        }
    );

    let mut invalid_vertex_usage = device
        .create_command_list(RenderQueueClass::Graphics, "invalid-vertex-usage")
        .unwrap();
    invalid_vertex_usage.set_pipeline(pipeline);
    invalid_vertex_usage.set_vertex_buffer(0, not_vertex, 0, 36);
    assert_eq!(
        device.submit(invalid_vertex_usage).unwrap_err(),
        RhiError::InvalidBufferUsage {
            buffer: not_vertex.diagnostic_id(),
            required: BufferUsage::VERTEX,
            actual: BufferUsage::COPY_DST,
        }
    );

    let mut vertex_binding_out_of_range = device
        .create_command_list(RenderQueueClass::Graphics, "vertex-binding-out-of-range")
        .unwrap();
    vertex_binding_out_of_range.set_pipeline(pipeline);
    vertex_binding_out_of_range.set_vertex_buffer(0, vertex_buffer, 0, 40);
    assert_eq!(
        device.submit(vertex_binding_out_of_range).unwrap_err(),
        RhiError::BufferBindingOutOfRange {
            buffer: vertex_buffer.diagnostic_id(),
            offset: 0,
            size: 40,
        }
    );

    let mut vertex_draw_out_of_range = device
        .create_command_list(RenderQueueClass::Graphics, "vertex-draw-out-of-range")
        .unwrap();
    begin_default_render_pass(&mut *vertex_draw_out_of_range, color, depth);
    vertex_draw_out_of_range.set_pipeline(pipeline);
    vertex_draw_out_of_range.set_vertex_buffer(0, vertex_buffer, 0, 24);
    vertex_draw_out_of_range.set_vertex_buffer(1, instance_buffer, 0, 32);
    vertex_draw_out_of_range.draw(0, 3, 0, 1);
    assert_eq!(
        device.submit(vertex_draw_out_of_range).unwrap_err(),
        RhiError::InvalidRasterDraw {
            reason: "vertex draw range exceeds vertex buffer slot 0".to_string(),
        }
    );

    let mut missing_index = device
        .create_command_list(RenderQueueClass::Graphics, "missing-index-buffer")
        .unwrap();
    begin_default_render_pass(&mut *missing_index, color, depth);
    missing_index.set_pipeline(pipeline);
    missing_index.set_vertex_buffer(0, vertex_buffer, 0, 36);
    missing_index.set_vertex_buffer(1, instance_buffer, 0, 32);
    missing_index.draw_indexed(0, 6, 0, 0, 1);
    assert_eq!(
        device.submit(missing_index).unwrap_err(),
        RhiError::InvalidRasterDraw {
            reason: "draw_indexed requires a bound index buffer".to_string(),
        }
    );

    let mut invalid_index_usage = device
        .create_command_list(RenderQueueClass::Graphics, "invalid-index-usage")
        .unwrap();
    invalid_index_usage.set_pipeline(pipeline);
    invalid_index_usage.set_index_buffer(not_index, 0, 12, IndexFormat::Uint16);
    assert_eq!(
        device.submit(invalid_index_usage).unwrap_err(),
        RhiError::InvalidBufferUsage {
            buffer: not_index.diagnostic_id(),
            required: BufferUsage::INDEX,
            actual: BufferUsage::VERTEX,
        }
    );

    let mut index_binding_misaligned = device
        .create_command_list(RenderQueueClass::Graphics, "index-binding-misaligned")
        .unwrap();
    index_binding_misaligned.set_pipeline(pipeline);
    index_binding_misaligned.set_index_buffer(index_buffer, 0, 3, IndexFormat::Uint16);
    assert_eq!(
        device.submit(index_binding_misaligned).unwrap_err(),
        RhiError::InvalidRasterDraw {
            reason: "index buffer binding size must be aligned to Uint16".to_string(),
        }
    );

    let mut index_draw_out_of_range = device
        .create_command_list(RenderQueueClass::Graphics, "index-draw-out-of-range")
        .unwrap();
    begin_default_render_pass(&mut *index_draw_out_of_range, color, depth);
    index_draw_out_of_range.set_pipeline(pipeline);
    index_draw_out_of_range.set_vertex_buffer(0, vertex_buffer, 0, 36);
    index_draw_out_of_range.set_vertex_buffer(1, instance_buffer, 0, 32);
    index_draw_out_of_range.set_index_buffer(index_buffer, 0, 4, IndexFormat::Uint16);
    index_draw_out_of_range.draw_indexed(0, 3, 0, 0, 1);
    assert_eq!(
        device.submit(index_draw_out_of_range).unwrap_err(),
        RhiError::InvalidRasterDraw {
            reason: "indexed draw range exceeds the bound index buffer".to_string(),
        }
    );
}

#[test]
fn command_list_buffer_copy_submit_validates_usage_flags() {
    let device = DeterministicRhiContractDevice::new_headless();
    let invalid_source = device
        .create_buffer(&BufferDesc::new(
            "not-copy-source",
            16,
            BufferUsage::UNIFORM,
        ))
        .unwrap();
    let valid_destination = device
        .create_buffer(&BufferDesc::new(
            "copy-destination",
            16,
            BufferUsage::COPY_DST,
        ))
        .unwrap();

    let mut source_command_list = device
        .create_command_list(RenderQueueClass::Copy, "invalid-source-copy")
        .unwrap();
    source_command_list.copy_buffer_to_buffer(invalid_source, valid_destination, 0, 0, 4);

    assert_eq!(
        device.submit(source_command_list).unwrap_err(),
        RhiError::InvalidBufferUsage {
            buffer: invalid_source.diagnostic_id(),
            required: BufferUsage::COPY_SRC,
            actual: BufferUsage::UNIFORM,
        }
    );

    let valid_source = device
        .create_buffer(&BufferDesc::new("copy-source", 16, BufferUsage::COPY_SRC))
        .unwrap();
    let invalid_destination = device
        .create_buffer(&BufferDesc::new(
            "not-copy-destination",
            16,
            BufferUsage::STORAGE,
        ))
        .unwrap();
    let mut destination_command_list = device
        .create_command_list(RenderQueueClass::Copy, "invalid-destination-copy")
        .unwrap();
    destination_command_list.copy_buffer_to_buffer(valid_source, invalid_destination, 0, 0, 4);

    assert_eq!(
        device.submit(destination_command_list).unwrap_err(),
        RhiError::InvalidBufferUsage {
            buffer: invalid_destination.diagnostic_id(),
            required: BufferUsage::COPY_DST,
            actual: BufferUsage::STORAGE,
        }
    );
}
