use super::*;

#[test]
fn borrowed_gpu_metadata_restores_context_when_gpu_is_missing() {
    let mut context = RenderPassExecutionContext::new(
        "post.test-pass",
        RenderPassExecutorId::from("post.test-executor"),
    );

    let result = with_borrowed_gpu_metadata(&mut context, |_, _, _| Ok(()));

    assert_eq!(
        result,
        Err(
            "render pass executor `post.test-executor` for pass `post.test-pass` requires renderer GPU context"
                .to_string()
        )
    );
    assert_eq!(context.pass_name, "post.test-pass");
    assert_eq!(context.executor_id.as_str(), "post.test-executor");
}
