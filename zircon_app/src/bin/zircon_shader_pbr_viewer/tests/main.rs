const MAIN_SOURCE: &str = include_str!("../main.rs");

#[test]
fn early_fatal_paths_write_a_terminal_source_chain() {
    let production = MAIN_SOURCE
        .split("#[cfg(test)]")
        .next()
        .expect("main source should retain production code before tests");

    assert!(production.contains("fn viewer_terminal_source_chain(config: &ViewerConfig)"));
    // BUG: [CR-APP-VIEWER-0001] 当前三个早期失败分支都记录输入来源，但本断言仍要求两次，导致此源码守卫失败；证据：上方身份校验、调试库预加载和事件循环创建分支。
    assert_eq!(
        production
            .matches(".with_source_chain(viewer_terminal_source_chain(&config))")
            .count(),
        2,
        "RenderDoc preload and EventLoop creation failures must retain input provenance"
    );
}

#[test]
fn profiling_build_uses_environment_capture_lifecycle() {
    let production = MAIN_SOURCE
        .split("#[cfg(test)]")
        .next()
        .expect("main source should retain production code before tests");

    assert!(production.contains("start_capture_from_env(\"shader-pbr-viewer\")"));
    assert!(production.contains("stop_and_export_capture_from_env()"));
    assert!(
        production
            .find("start_capture_from_env")
            .expect("capture start")
            < production
                .find("run_viewer(config)")
                .expect("viewer execution")
    );
    assert!(
        production
            .find("run_viewer(config)")
            .expect("viewer execution")
            < production
                .find("stop_and_export_capture_from_env")
                .expect("capture export")
    );
}
