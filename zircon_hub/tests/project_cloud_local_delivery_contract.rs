//! Static contracts for Cloud local-delivery and reserved-service boundaries.
//! 固定 Cloud 页本地交付流程与远程预留能力的边界，使预留服务数量不混入可执行的打包或安装输出。

use std::{fs, path::PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_dir() -> PathBuf {
    crate_dir()
        .parent()
        .expect("zircon_hub crate should live under the repository root")
        .to_path_buf()
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

/// 读取相对 Hub 包根的受审源码作为结构证据；调用方依赖仓库检出完整，读取失败应暴露契约来源缺失。
fn read_crate_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(path))
            .unwrap_or_else(|error| panic!("failed to read Hub crate file {path}: {error}")),
    )
}

/// 读取仓库级交接文档或工具证据；约定 Hub 包位于仓库根下一层，不能依赖测试启动时的工作目录。
fn read_repo_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(repo_dir().join(path))
            .unwrap_or_else(|error| panic!("failed to read repository file {path}: {error}")),
    )
}

fn assert_contains_all(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            source.contains(snippet),
            "{source_name} should contain Cloud local-delivery snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete Cloud local-delivery snippet {snippet:?}"
        );
    }
}

/// 把远程预留项的禁用状态和本地输出入口分别约束，避免服务统计让用户误以为已接通远程能力。
#[test]
fn cloud_reserved_services_are_disabled_dto_entries_not_local_package_or_install_slots() {
    let cloud = read_crate_file("web/src/pages/CloudPage.tsx");
    let coming_soon = read_crate_file("src/tauri_app/view_model/coming_soon.rs");

    assert_contains_all(
        "coming_soon.rs",
        &coming_soon,
        &[
            "\"remote-sync\"",
            "\"account-service\"",
            "\"cloud-repository\"",
            "\"local-delivery\"",
            "Remote marketplace access is outside the local-only v1 scope.",
            "Cloud synchronization is reserved; packages stay local in v1.",
            "No remote account or identity service is required for v1.",
            "Remote package repositories are disabled until the cloud service layer exists.",
            "disabled: true",
        ],
    );
    assert_contains_all(
        "CloudPage.tsx",
        &cloud,
        &[
            "const reservedServices = useMemo(",
            "state.comingSoon.filter((entry) => entry.category === \"local-delivery\")",
            "MetricCard label={text.serviceSlots} value={`${reservedServices.length}`}",
            "detail: formatCountText(common.reservedCountTemplate, reservedServices.length)",
            "children: reservedServices.map((entry) => ({",
            "items={reservedServices.map((entry) => ({",
            "meta: entry.meta,",
            "disabled: entry.disabled,",
            "{ value: \"packages\", label: common.packages }",
            "{ value: \"installs\", label: common.installs }",
            "{ value: \"services\", label: common.services }",
        ],
    );
    assert_not_contains_any(
        "CloudPage.tsx",
        &cloud,
        &[
            "const serviceSlots = useMemo(",
            "title: text.packageOutputs,\n        detail: text.packageOutput,\n        status: common.local,\n        disabled: false,",
            "title: text.deviceInstalls,\n        detail: text.deviceInstall,\n        status: common.ready,\n        disabled: false,",
            "value={`${serviceSlots.length}`}",
            "children: serviceSlots.map((slot)",
            "items={serviceSlots.map((slot)",
        ],
    );
}

/// 要求两份 UI 交接文档共同说明本地工作流与预留服务的区分，保持维护者对统计口径的理解一致。
#[test]
fn cloud_local_delivery_docs_record_reserved_service_split() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "Cloud Reserved Services renders only disabled `local-delivery` `comingSoon` entries",
            "package and install roots stay in the Packages/Installs local workflow panels",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "Cloud Reserved Services counts and lists only disabled `local-delivery` `comingSoon` entries",
            "local package/install rows stay out of the reserved-service count",
        ],
    );
}
