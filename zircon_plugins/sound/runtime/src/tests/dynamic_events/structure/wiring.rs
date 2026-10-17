// 固定动态事件的目录、执行器与 ABI 模块分层；该守卫检查源码形状，不执行运行时事件。
use super::support::{assert_structural_module, src_root};

#[test]
fn dynamic_event_module_wiring_remains_folder_backed() {
    let src = src_root();

    assert_structural_module(
        &src,
        "service_types/dynamic_events/mod.rs",
        &[
            "#[path = \"catalog.rs\"]",
            "mod catalog;",
            "#[path = \"dispatch.rs\"]",
            "mod dispatch;",
            "#[path = \"handlers.rs\"]",
            "mod handlers;",
            "#[path = \"invocation.rs\"]",
            "mod invocation;",
        ],
    );
    assert_structural_module(
        &src,
        "service_types/dynamic_event_executors/mod.rs",
        &[
            "#[path = \"../execution.rs\"]",
            "mod execution;",
            "#[path = \"../registration.rs\"]",
            "mod registration;",
            "#[path = \"../unregistration.rs\"]",
            "mod unregistration;",
        ],
    );
    assert_structural_module(
        &src,
        "dynamic_events/mod.rs",
        &[
            "#[path = \"catalog.rs\"]",
            "pub(crate) mod catalog;",
            "#[path = \"dispatch.rs\"]",
            "pub(crate) mod dispatch;",
            "#[path = \"handlers.rs\"]",
            "pub(crate) mod handlers;",
            "#[path = \"invocation.rs\"]",
            "pub(crate) mod invocation;",
        ],
    );
    assert_structural_module(
        &src,
        "dynamic_event_abi/mod.rs",
        &[
            "#[path = \"callback.rs\"]",
            "pub(crate) mod callback;",
            "#[path = \"executor.rs\"]",
            "pub(crate) mod executor;",
            "#[path = \"request.rs\"]",
            "pub(crate) mod request;",
            "#[path = \"slice.rs\"]",
            "pub(crate) mod slice;",
            "#[path = \"status.rs\"]",
            "pub(crate) mod status;",
        ],
    );
}
