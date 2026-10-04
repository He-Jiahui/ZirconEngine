#[test]
fn retained_viewport_submit_paths_use_async_capture_without_sync_rgba_readback() {
    let extract_submit = include_str!("../submit.rs");
    let runtime_submit = include_str!("../submit_runtime_frame.rs");
    let present_submit = include_str!("../present_frame_extract.rs");

    for source in [extract_submit, runtime_submit, present_submit] {
        assert!(source.contains("render_frame_with_pipeline_async_capture_task_pool"));
        assert!(!source.contains("render_frame_with_pipeline_task_pool("));
        assert!(!source.contains("finish_viewport_frame"));
    }
}

#[test]
fn environment_only_profile_uses_direct_rendering_for_every_submission_shape() {
    let extract_submit = include_str!("../submit.rs");
    let runtime_submit = include_str!("../submit_runtime_frame.rs");
    let present_submit = include_str!("../present_frame_extract.rs");

    for source in [extract_submit, runtime_submit, present_submit] {
        assert!(source.contains("supports_compiled_scene_graph()"));
        assert!(source.contains("render_frame_direct_submission"));
    }
    assert!(present_submit.contains("present_frame_direct"));
}

#[test]
fn viewport_generation_is_qualified_before_capture_or_direct_product_publication() {
    let extract_submit = include_str!("../submit.rs");
    let runtime_submit = include_str!("../submit_runtime_frame.rs");
    let present_submit = include_str!("../present_frame_extract.rs");

    for source in [extract_submit, runtime_submit, present_submit] {
        let post_render = source
            .split("let frame_generation")
            .nth(1)
            .expect("each submission path must capture its rendered generation");
        let generation_check = post_render
            .find("validate_viewport_generation(&state, viewport, &context)")
            .expect("each submission path must qualify the viewport after rendering");
        let capture_finish = post_render
            .find("finish_active_capture_and_relock(")
            .expect("each submission path must finish the active capture");

        assert!(
            generation_check < capture_finish,
            "a stale viewport must fail before capture completion can publish a frame"
        );
    }

    let direct_publication = include_str!("../publish_viewport_product.rs");
    assert!(direct_publication.contains("viewport_products.publish("));
    assert!(direct_publication.contains("frame.submission_receipt()"));
    assert!(!direct_publication.contains("replace_submission_receipt"));
    assert!(!direct_publication.contains("retain_viewport_product_submission_receipt"));

    for source in [extract_submit, runtime_submit] {
        let post_render = source
            .split("let frame_generation")
            .nth(1)
            .expect("each direct-product submission path must capture its rendered generation");
        let generation_check = post_render
            .find("validate_viewport_generation(&state, viewport, &context)")
            .expect("each direct-product submission path must qualify the viewport");
        let direct_publication = post_render
            .find("publish_viewport_product(&mut state, viewport, &mut")
            .expect("each direct-product submission path must publish the retained UI image");
        let capture_finish = post_render
            .find("finish_active_capture_and_relock(")
            .expect("each direct-product submission path must finish capture");

        assert!(
            generation_check < direct_publication,
            "a stale viewport must fail before retained UI can observe a direct GPU product"
        );
        assert!(
            direct_publication < capture_finish,
            "direct-product failure must settle capture before capture success is published"
        );
    }
}

#[test]
fn submission_error_paths_publish_the_latest_scene_completion_report() {
    for source in [
        include_str!("../submit.rs"),
        include_str!("../submit_runtime_frame.rs"),
        include_str!("../present_frame_extract.rs"),
    ] {
        assert!(source.contains("publish_scene_submission_completion_stats(&mut state);"));
    }
}
