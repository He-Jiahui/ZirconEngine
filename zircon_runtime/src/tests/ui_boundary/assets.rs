//! 区分正式界面入口资源与测试夹具源码；路径、资源种类和默认字体清单检查只约束资源布局与引用关系。
#[test]
fn production_ui_entry_assets_live_under_crate_assets_not_src() {
    let runtime_fixture_source =
        include_str!("../../ui/tests/runtime_ui_support/runtime_ui_fixture.rs");
    let runtime_assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("ui")
        .join("runtime")
        .join("fixtures");

    assert!(
        !runtime_fixture_source.contains("include_str!(\"fixtures/"),
        "runtime UI fixtures should load from crate assets instead of src fixtures"
    );

    for relative in [
        "hud_overlay.zui",
        "pause_menu.zui",
        "settings_dialog.zui",
        "inventory_list.zui",
        "quest_log_dialog.zui",
    ] {
        assert!(
            runtime_assets.join(relative).exists(),
            "expected runtime fixture entry asset {relative} under {:?}",
            runtime_assets
        );
    }

    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime crate should live under the repo root");
    for crate_src in [
        repo_root.join("zircon_editor").join("src"),
        repo_root.join("zircon_runtime").join("src"),
    ] {
        let lingering = collect_production_ui_document_files(&crate_src);
        assert!(
            lingering.is_empty(),
            "production ui document assets must not live under src; found {:?}",
            lingering
        );
    }
}

#[test]
fn runtime_ui_asset_root_contains_only_zui_entries() {
    let runtime_ui_assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("ui");
    let ui_toml_files = collect_ui_toml_files(&runtime_ui_assets);
    let zui_files = collect_zui_files(&runtime_ui_assets);

    assert!(
        ui_toml_files.is_empty(),
        "runtime production UI asset roots must not keep deprecated .ui.toml files; found {:?}",
        ui_toml_files
    );
    assert!(
        zui_files.iter().any(|path| path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().ends_with(".zui"))),
        "runtime UI asset root should contain at least one .zui ui document"
    );
}

#[test]
fn default_runtime_font_manifest_stays_inside_runtime_assets() {
    let assets_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let manifest_path = assets_root.join("fonts").join("default.font.toml");
    let manifest_source = std::fs::read_to_string(&manifest_path)
        .expect("default runtime font manifest should exist under crate assets");
    let manifest: toml::Value =
        toml::from_str(&manifest_source).expect("default runtime font manifest should parse");
    let source = manifest
        .get("source")
        .and_then(toml::Value::as_str)
        .expect("default runtime font manifest should declare a source");
    let resolved_source = std::fs::canonicalize(
        manifest_path
            .parent()
            .expect("default font manifest should live in a folder")
            .join(source),
    )
    .expect("default runtime font source should resolve on disk");
    let canonical_assets_root =
        std::fs::canonicalize(&assets_root).expect("runtime assets root should resolve");

    assert!(
        resolved_source.starts_with(&canonical_assets_root),
        "default runtime font source should stay inside runtime assets; got {:?} outside {:?}",
        resolved_source,
        canonical_assets_root
    );
}

/// 收集旧格式候选，供调用方检查资源目录；读取错误沿用下层扫描的跳过约定。
fn collect_ui_toml_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    collect_files_with_suffixes(root, &[".ui.toml"])
}

/// 收集正式文档候选并返回拥有的路径集合；是否至少命中一项由调用方判断。
fn collect_zui_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    collect_files_with_suffixes(root, &[".zui"])
}

/// 按文件名后缀递归收集路径，供资源位置断言消费。
///
/// # 参数
/// - `root`：借用的扫描起点，不更改目录内容。
/// - `suffixes`：借用的后缀集合，任一匹配即可收录。
///
/// # 返回
/// 返回拥有的路径列表，保留目录枚举顺序，不保证排序或去重。
///
/// # 约束与生命周期
/// 目录读取失败和错误条目均被跳过，因此空集合也可能表示未能读取。
/// 扫描不维护访问记录，递归终止依赖输入树；返回后不保留目录迭代器或对输入的借用。
fn collect_files_with_suffixes(
    root: &std::path::Path,
    suffixes: &[&str],
) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_files_with_suffixes(&path, suffixes));
            } else if path.file_name().is_some_and(|name| {
                let name = name.to_string_lossy();
                suffixes.iter().any(|suffix| name.ends_with(suffix))
            }) {
                files.push(path);
            }
        }
    }
    files
}

/// 查找生产源码树中的文档资源，并排除约定的测试夹具子树。
///
/// # 参数
/// `root` 是借用的源码目录起点，由调用方按仓库布局构造。
///
/// # 返回与约束
/// 返回命中的拥有路径，不读取文档内容，也不进行排序或去重。
/// 路径先经夹具判定再决定递归；读取失败和错误条目被跳过，空结果不证明扫描完整。
/// 返回后不保留目录迭代器；夹具判定参见 [`is_test_fixture_path`]。
fn collect_production_ui_document_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if is_test_fixture_path(&path) {
                continue;
            }
            if path.is_dir() {
                files.extend(collect_production_ui_document_files(&path));
            } else if path.file_name().is_some_and(|name| {
                let name = name.to_string_lossy();
                name.ends_with(".ui.toml") || name.ends_with(".zui")
            }) {
                files.push(path);
            }
        }
    }
    files
}

/// 判断路径分量中是否出现相邻的测试目录名和夹具目录名。
///
/// # 参数与返回
/// `path` 为借用路径；命中约定的相邻分量时返回真，不要求磁盘对象存在。
///
/// # 约束与生命周期
/// 按迭代所得的词法分量区分大小写，不解析磁盘规范路径；仅匹配迭代结果中相邻的测试与夹具目录。
/// 扫描状态只记住前一分量，返回后不保留输入引用。
fn is_test_fixture_path(path: &std::path::Path) -> bool {
    let mut saw_tests = false;
    for component in path.components() {
        let name = component.as_os_str().to_string_lossy();
        if saw_tests && name == "fixtures" {
            return true;
        }
        saw_tests = name == "tests";
    }
    false
}
