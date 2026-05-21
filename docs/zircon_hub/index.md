---
related_code:
  - Cargo.toml
  - zircon_hub/Cargo.toml
  - zircon_hub/build.rs
  - dev/material-rust-template/README.md
  - dev/material-rust-template/material-1.0/material.slint
  - zircon_hub/src/lib.rs
  - zircon_hub/tests/ui_contract.rs
  - zircon_hub/src/main.rs
  - zircon_hub/src/app/mod.rs
  - zircon_hub/src/app/runtime.rs
  - zircon_hub/src/app/runtime/project_workspace.rs
  - zircon_hub/src/app/runtime/asset_catalog.rs
  - zircon_hub/src/app/runtime/folder_picker.rs
  - zircon_hub/src/app/runtime/learn_catalog.rs
  - zircon_hub/src/app/runtime/plugin_catalog.rs
  - zircon_hub/src/app/runtime/window_controls.rs
  - zircon_hub/src/app/runtime/team_overview.rs
  - zircon_hub/src/app/binding.rs
  - zircon_hub/src/app/view_model.rs
  - zircon_hub/src/app/view_model/assets.rs
  - zircon_hub/src/app/view_model/cloud.rs
  - zircon_hub/src/app/view_model/learn.rs
  - zircon_hub/src/app/view_model/media.rs
  - zircon_hub/src/app/view_model/plugins.rs
  - zircon_hub/src/app/view_model/projects.rs
  - zircon_hub/src/app/view_model/quick_actions.rs
  - zircon_hub/src/app/view_model/team.rs
  - zircon_hub/src/app/localization.rs
  - zircon_hub/src/app/quick_action.rs
  - zircon_hub/src/state/mod.rs
  - zircon_hub/src/state/navigation.rs
  - zircon_hub/src/state/project_view.rs
  - zircon_hub/src/projects/mod.rs
  - zircon_hub/src/projects/create_project_request.rs
  - zircon_hub/src/projects/cover.rs
  - zircon_hub/src/projects/metadata.rs
  - zircon_hub/src/projects/package.rs
  - zircon_hub/src/projects/device_install.rs
  - zircon_hub/src/projects/recycle_bin.rs
  - zircon_hub/src/assets/mod.rs
  - zircon_hub/src/assets/catalog.rs
  - zircon_hub/src/learn/mod.rs
  - zircon_hub/src/learn/catalog.rs
  - zircon_hub/src/plugins/mod.rs
  - zircon_hub/src/plugins/catalog.rs
  - zircon_hub/src/team/mod.rs
  - zircon_hub/src/team/local_git.rs
  - zircon_hub/src/engines/mod.rs
  - zircon_hub/src/engines/source_engine_install.rs
  - zircon_hub/src/engines/registry.rs
  - zircon_hub/src/build/mod.rs
  - zircon_hub/src/settings/mod.rs
  - zircon_hub/src/process/mod.rs
  - zircon_hub/src/process/folder_picker.rs
  - zircon_hub/ui/app.slint
  - zircon_hub/ui/shared.slint
  - zircon_hub/ui/components.slint
  - zircon_hub/ui/tokens.slint
  - zircon_hub/ui/material_bridge.slint
  - zircon_hub/ui/layout.slint
  - zircon_hub/ui/surfaces.slint
  - zircon_hub/ui/inputs.slint
  - zircon_hub/ui/shell.slint
  - zircon_hub/ui/navigation.slint
  - zircon_hub/ui/data_display.slint
  - zircon_hub/ui/overlays.slint
  - zircon_hub/ui/projects.slint
  - zircon_hub/ui/project_dashboard.slint
  - zircon_hub/ui/project_page_components.slint
  - zircon_hub/ui/project_pages.slint
  - zircon_hub/ui/editor.slint
  - zircon_hub/ui/builds.slint
  - zircon_hub/ui/assets.slint
  - zircon_hub/ui/cloud.slint
  - zircon_hub/ui/learn.slint
  - zircon_hub/ui/plugins.slint
  - zircon_hub/ui/team.slint
  - zircon_hub/ui/placeholder.slint
  - zircon_hub/ui/settings.slint
  - zircon_hub/assets/brand/zircon-mark.svg
  - zircon_hub/assets/icons/nav/projects.svg
  - zircon_hub/assets/icons/nav/editor.svg
  - zircon_hub/assets/icons/nav/assets.svg
  - zircon_hub/assets/icons/nav/builds.svg
  - zircon_hub/assets/icons/nav/plugins.svg
  - zircon_hub/assets/icons/nav/cloud.svg
  - zircon_hub/assets/icons/nav/team.svg
  - zircon_hub/assets/icons/nav/learn.svg
  - zircon_hub/assets/icons/nav/settings.svg
  - zircon_hub/assets/icons/actions/build-project.svg
  - zircon_hub/assets/icons/actions/install-device.svg
  - zircon_hub/assets/icons/actions/package-project.svg
  - zircon_hub/assets/icons/actions/open-editor.svg
  - zircon_hub/assets/icons/status/running.svg
  - zircon_hub/assets/icons/status/success.svg
  - zircon_hub/assets/icons/status/warning.svg
  - zircon_hub/assets/icons/status/error.svg
  - zircon_hub/assets/icons/ui/search.svg
  - zircon_hub/assets/icons/ui/chevron-down.svg
  - zircon_hub/assets/icons/ui/chevron-left.svg
  - zircon_hub/assets/icons/ui/chevron-right.svg
  - zircon_hub/assets/icons/ui/plus.svg
  - zircon_hub/assets/icons/ui/import.svg
  - zircon_hub/assets/icons/ui/grid.svg
  - zircon_hub/assets/icons/ui/list.svg
  - zircon_hub/assets/icons/ui/folder.svg
  - zircon_hub/assets/icons/ui/sort.svg
  - zircon_hub/assets/icons/ui/bell.svg
  - zircon_hub/assets/icons/ui/help.svg
  - zircon_hub/assets/icons/ui/settings.svg
  - zircon_hub/assets/icons/ui/minimize.svg
  - zircon_hub/assets/icons/ui/maximize.svg
  - zircon_hub/assets/icons/ui/close.svg
  - zircon_hub/assets/icons/ui/more-vertical.svg
  - zircon_hub/assets/icons/ui/refresh.svg
  - zircon_hub/assets/icons/ui/collapse.svg
  - zircon_hub/assets/icons/ui/alert.svg
  - zircon_hub/assets/icons/ui/edit.svg
  - zircon_hub/assets/covers/project-elysium.svg
  - zircon_hub/assets/covers/project-stellar-outpost.svg
  - zircon_hub/assets/covers/project-sands-of-time.svg
  - zircon_hub/assets/covers/project-whispering-woods.svg
  - zircon_hub/assets/covers/project-neon-streets.svg
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/gui_startup_request.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - tools/zircon_build.py
implementation_files:
  - zircon_hub/Cargo.toml
  - zircon_hub/build.rs
  - zircon_hub/src/app/runtime.rs
  - zircon_hub/src/app/runtime/project_workspace.rs
  - zircon_hub/src/app/runtime/asset_catalog.rs
  - zircon_hub/src/app/runtime/folder_picker.rs
  - zircon_hub/src/app/runtime/learn_catalog.rs
  - zircon_hub/src/app/runtime/plugin_catalog.rs
  - zircon_hub/src/app/runtime/window_controls.rs
  - zircon_hub/src/app/runtime/team_overview.rs
  - zircon_hub/src/app/binding.rs
  - zircon_hub/src/app/view_model.rs
  - zircon_hub/src/app/view_model/assets.rs
  - zircon_hub/src/app/view_model/cloud.rs
  - zircon_hub/src/app/view_model/learn.rs
  - zircon_hub/src/app/view_model/media.rs
  - zircon_hub/src/app/view_model/plugins.rs
  - zircon_hub/src/app/view_model/quick_actions.rs
  - zircon_hub/src/app/view_model/team.rs
  - zircon_hub/src/app/localization.rs
  - zircon_hub/src/app/quick_action.rs
  - zircon_hub/tests/ui_contract.rs
  - zircon_hub/src/state/navigation.rs
  - zircon_hub/src/state/project_view.rs
  - zircon_hub/ui/app.slint
  - zircon_hub/ui/shared.slint
  - zircon_hub/ui/components.slint
  - zircon_hub/ui/tokens.slint
  - zircon_hub/ui/material_bridge.slint
  - zircon_hub/ui/layout.slint
  - zircon_hub/ui/surfaces.slint
  - zircon_hub/ui/inputs.slint
  - zircon_hub/ui/shell.slint
  - zircon_hub/ui/navigation.slint
  - zircon_hub/ui/data_display.slint
  - zircon_hub/ui/overlays.slint
  - zircon_hub/ui/projects.slint
  - zircon_hub/ui/project_dashboard.slint
  - zircon_hub/ui/project_page_components.slint
  - zircon_hub/ui/project_pages.slint
  - zircon_hub/ui/editor.slint
  - zircon_hub/ui/builds.slint
  - zircon_hub/ui/assets.slint
  - zircon_hub/ui/cloud.slint
  - zircon_hub/ui/learn.slint
  - zircon_hub/ui/plugins.slint
  - zircon_hub/ui/team.slint
  - zircon_hub/ui/placeholder.slint
  - zircon_hub/ui/settings.slint
  - zircon_hub/assets/brand/zircon-mark.svg
  - zircon_hub/assets/icons/nav/projects.svg
  - zircon_hub/assets/icons/nav/editor.svg
  - zircon_hub/assets/icons/nav/assets.svg
  - zircon_hub/assets/icons/nav/builds.svg
  - zircon_hub/assets/icons/nav/plugins.svg
  - zircon_hub/assets/icons/nav/cloud.svg
  - zircon_hub/assets/icons/nav/team.svg
  - zircon_hub/assets/icons/nav/learn.svg
  - zircon_hub/assets/icons/nav/settings.svg
  - zircon_hub/assets/icons/actions/build-project.svg
  - zircon_hub/assets/icons/actions/install-device.svg
  - zircon_hub/assets/icons/actions/package-project.svg
  - zircon_hub/assets/icons/actions/open-editor.svg
  - zircon_hub/assets/icons/status/running.svg
  - zircon_hub/assets/icons/status/success.svg
  - zircon_hub/assets/icons/status/warning.svg
  - zircon_hub/assets/icons/status/error.svg
  - zircon_hub/assets/icons/ui/search.svg
  - zircon_hub/assets/icons/ui/chevron-down.svg
  - zircon_hub/assets/icons/ui/chevron-left.svg
  - zircon_hub/assets/icons/ui/chevron-right.svg
  - zircon_hub/assets/icons/ui/plus.svg
  - zircon_hub/assets/icons/ui/import.svg
  - zircon_hub/assets/icons/ui/grid.svg
  - zircon_hub/assets/icons/ui/list.svg
  - zircon_hub/assets/icons/ui/folder.svg
  - zircon_hub/assets/icons/ui/sort.svg
  - zircon_hub/assets/icons/ui/bell.svg
  - zircon_hub/assets/icons/ui/help.svg
  - zircon_hub/assets/icons/ui/settings.svg
  - zircon_hub/assets/icons/ui/minimize.svg
  - zircon_hub/assets/icons/ui/maximize.svg
  - zircon_hub/assets/icons/ui/close.svg
  - zircon_hub/assets/icons/ui/more-vertical.svg
  - zircon_hub/assets/icons/ui/refresh.svg
  - zircon_hub/assets/icons/ui/collapse.svg
  - zircon_hub/assets/icons/ui/alert.svg
  - zircon_hub/assets/icons/ui/edit.svg
  - zircon_hub/assets/covers/project-elysium.svg
  - zircon_hub/assets/covers/project-stellar-outpost.svg
  - zircon_hub/assets/covers/project-sands-of-time.svg
  - zircon_hub/assets/covers/project-whispering-woods.svg
  - zircon_hub/assets/covers/project-neon-streets.svg
  - zircon_hub/src/projects/editor_recent_sync.rs
  - zircon_hub/src/projects/cover.rs
  - zircon_hub/src/projects/create_project_request.rs
  - zircon_hub/src/projects/metadata.rs
  - zircon_hub/src/projects/package.rs
  - zircon_hub/src/projects/device_install.rs
  - zircon_hub/src/projects/recycle_bin.rs
  - zircon_hub/src/assets/catalog.rs
  - zircon_hub/src/learn/catalog.rs
  - zircon_hub/src/plugins/catalog.rs
  - zircon_hub/src/team/local_git.rs
  - zircon_hub/src/engines/source_engine_install.rs
  - zircon_hub/src/engines/registry.rs
  - zircon_hub/src/process/editor_launch.rs
  - zircon_hub/src/process/folder_picker.rs
  - zircon_hub/src/build/command.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/gui_startup_request.rs
  - tools/zircon_build.py
plan_sources:
  - .codex/plans/zircon_hub 独立启动器设计.md
  - .codex/plans/Zircon Hub 典型组件样例设计.md
  - .codex/plans/Zircon Hub Flex 组件化重构设计方案.md
  - user: 2026-05-16 generate Hub and Editor SVG resources from reference screenshots
  - user: 2026-05-17 require Slint 1.16.1 Taffy/Flex layout and Material UI component taxonomy
  - user: 2026-05-17 continue Hub componentized design and match docs/ui-and-layout/hub.png
  - user: 2026-05-18 implement approved non-Projects tab component layout plan
  - user: 2026-05-18 continue componentized Settings tab layout
  - user: 2026-05-18 implement Projects secondary pages, project metadata, and proportional Slint/Taffy layout
  - user: 2026-05-19 continue Hub responsive componentization and workspace panel tokenization
  - user: 2026-05-19 directly introduce the local Slint Material UI template instead of designing a custom clone
  - https://mui.com/material-ui/all-components/
  - dev/material-rust-template/README.md
  - dev/material-rust-template/material-1.0/material.slint
  - docs/superpowers/specs/2026-05-16-hub-editor-svg-resources-design.md
  - docs/superpowers/plans/2026-05-16-hub-editor-svg-resources.md
tests:
  - cargo fmt -p zircon_hub --check
  - cargo check -p zircon_hub --offline --jobs 1
  - cargo test -p zircon_hub --locked
  - cargo check -p zircon_hub --locked --jobs 1
  - cargo check -p zircon_hub --locked --offline --jobs 1
  - cargo test -p zircon_hub --locked --offline --jobs 1
  - cargo build -p zircon_hub --locked --offline --jobs 1
  - cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1
  - CARGO_TARGET_DIR=target/hub-tabs-component-final cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1
  - CARGO_TARGET_DIR=target/hub-tabs-component-final cargo check -p zircon_hub --locked --offline --jobs 1
  - CARGO_TARGET_DIR=target/hub-window-resize-check cargo check -p zircon_hub --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=target/hub-window-resize-check cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=target/hub-window-resize-check cargo test -p zircon_hub --locked --offline --jobs 1 --message-format short --color never
  - cargo check -p zircon_hub --locked --offline --jobs 1 --target-dir target\hub-material-check --color never
  - cargo check -p zircon_hub --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-hub-info-row-0520 --message-format short --color never
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test -p zircon_hub --test ui_contract --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-hub-ui-contract-nodebug-0520 --message-format short --color never
  - CARGO_PROFILE_DEV_DEBUG=0 cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-hub-ui-contract-nodebug-0520 --color never
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check -p zircon_hub --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-hub-ui-contract-nodebug-0520 --message-format short --color never
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test -p zircon_hub --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-hub-ui-contract-nodebug-0520 --message-format short --color never
  - rustfmt --edition 2021 --check zircon_hub\tests\ui_contract.rs
  - CARGO_MANIFEST_DIR=E:\Git\ZirconEngine\zircon_hub rustc --edition=2021 --test zircon_hub\tests\ui_contract.rs -o target\hub-material-check\ui_contract_project_browser_detail_direct.exe; target\hub-material-check\ui_contract_project_browser_detail_direct.exe --nocapture
  - CARGO_MANIFEST_DIR=E:\Git\ZirconEngine\zircon_hub rustc --edition=2021 --test zircon_hub\tests\ui_contract.rs -o target\hub-material-check\ui_contract_component_samples_direct.exe; target\hub-material-check\ui_contract_component_samples_direct.exe --nocapture
  - OUT_DIR=target\hub-ui-check-open-detail-callback CARGO_MANIFEST_DIR=E:\Git\ZirconEngine\zircon_hub CARGO_PKG_NAME=zircon_hub target\hub-project-scroll-check\debug\build\zircon_hub-7fc227b3e5306fb7\build-script-build.exe
  - CARGO_PROFILE_DEV_DEBUG=0 cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-hub-project-browser-detail-0520 --color never
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test -p zircon_hub --test ui_contract --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-hub-project-browser-detail-0520 --color never
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check -p zircon_hub --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-hub-project-browser-detail-0520 --color never
  - CARGO_PROFILE_DEV_DEBUG=0 cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-hub-project-browser-detail-0520 --color never
  - target/hub-visual-check/material-open-detail-iconbutton-default-1280x900/hub-projects-dashboard.png
  - target/hub-visual-check/material-open-detail-iconbutton-default-1280x900/hub-projects-new-project.png
  - target/hub-visual-check/material-open-detail-iconbutton-default-1280x900/hub-projects-browser.png
  - target/hub-visual-check/material-open-detail-iconbutton-default-1280x900/hub-projects-detail.png
  - target/hub-visual-check/material-detail-hit-target-1280x900/hub-projects-dashboard.png
  - target/hub-visual-check/material-detail-hit-target-1280x900/hub-projects-new-project.png
  - target/hub-visual-check/material-detail-hit-target-1280x900/hub-projects-browser.png
  - target/hub-visual-check/material-detail-hit-target-1280x900/hub-projects-detail.png
  - target/hub-visual-check/material-browser-detail-current-1280x900/hub-projects-dashboard.png
  - target/hub-visual-check/material-browser-detail-current-1280x900/hub-projects-new-project.png
  - target/hub-visual-check/material-browser-detail-current-1280x900/hub-projects-browser.png
  - target/hub-visual-check/material-browser-detail-current-1280x900/hub-projects-detail.png
  - target/hub-visual-check/material-browser-viewall-detail-1280x900/hub-projects-dashboard.png
  - target/hub-visual-check/material-browser-viewall-detail-1280x900/hub-projects-new-project.png
  - target/hub-visual-check/material-browser-viewall-detail-1280x900/hub-projects-browser.png
  - target/hub-visual-check/material-browser-viewall-detail-1280x900/hub-projects-detail.png
  - target/hub-visual-check/material-state-detail-1280x900/hub-projects-dashboard.png
  - target/hub-visual-check/material-state-detail-1280x900/hub-projects-new-project.png
  - target/hub-visual-check/material-state-detail-1280x900/hub-projects-browser.png
  - target/hub-visual-check/material-state-detail-1280x900/hub-projects-detail.png
  - target/hub-visual-check/material-responsive-state-detail-1024x720/hub-projects-dashboard.png
  - target/hub-visual-check/material-responsive-state-detail-1024x720/hub-projects-new-project.png
  - target/hub-visual-check/material-responsive-state-detail-1024x720/hub-projects-browser.png
  - target/hub-visual-check/material-responsive-state-detail-1024x720/hub-projects-detail.png
  - OUT_DIR=target\hub-ui-check-project-browser-detail-button CARGO_MANIFEST_DIR=E:\Git\ZirconEngine\zircon_hub CARGO_PKG_NAME=zircon_hub target\hub-project-scroll-check\debug\build\zircon_hub-7fc227b3e5306fb7\build-script-build.exe
  - target/hub-ui-check-project-browser-detail-button/app.rs
  - target/hub-visual-check/projects-taffy-1600x1024-0520/hub-projects-dashboard.png
  - target/hub-visual-check/projects-taffy-1600x1024-0520/hub-projects-new-project.png
  - target/hub-visual-check/projects-taffy-1600x1024-0520/hub-projects-browser.png
  - target/hub-visual-check/projects-taffy-1600x1024-0520/hub-projects-detail.png
  - target/hub-visual-check/projects-taffy-1280x900-0520-v2/hub-projects-dashboard.png
  - target/hub-visual-check/projects-taffy-1280x900-0520-v2/hub-projects-new-project.png
  - target/hub-visual-check/projects-taffy-1280x900-0520-v2/hub-projects-browser.png
  - target/hub-visual-check/projects-taffy-1280x900-0520-v2/hub-projects-detail.png
  - target/hub-visual-check/projects-taffy-1024x768-0520-v2/hub-projects-dashboard.png
  - target/hub-visual-check/projects-taffy-1024x768-0520-v2/hub-projects-new-project.png
  - target/hub-visual-check/projects-taffy-1024x768-0520-v2/hub-projects-browser.png
  - target/hub-visual-check/projects-taffy-1024x768-0520-v2/hub-projects-detail.png
  - rustc --edition 2021 --test zircon_hub\tests\ui_contract.rs -o target\hub-verify-logs\ui_contract_direct_0520.exe
  - cargo test -p zircon_hub --test ui_contract --locked --offline --jobs 1 --target-dir target\hub-material-check --message-format short --color never
  - cargo test -p zircon_hub --locked --offline --jobs 1 --target-dir target\hub-material-check --color never
  - cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1 --target-dir target\hub-material-check --color never
  - Select-String -Path 'zircon_hub\ui\*.slint' -Pattern '^\s*(x|y):'
  - Select-String -Path 'zircon_hub\ui\*.slint' -Pattern 'text:\s*"(\+|>|<|\[\]|::|==|v|!|\?)"'
  - .\.codex\skills\zircon-project-skills\capture-hub-window-screenshot\scripts\capture-hub-window.ps1 -BinaryPath target\hub-tabs-component-final\debug\zircon_hub.exe -ConfigMode Isolated
  - Windows Hub screenshots: target/hub-visual-check/latest-panel-list-clean/hub-editor-wide.png, target/hub-visual-check/latest-panel-list-clean/hub-cloud-wide.png, target/hub-visual-check/latest-panel-list-clean/hub-team-wide.png
  - Windows Hub screenshots: target/hub-visual-check/window-resize-verified-pages/hub-{editor,assets,builds,plugins,cloud,team,learn}-{wide,narrow}.png
  - Windows Hub screenshots: target/hub-visual-check/window-resize-final-verified/hub-builds-{wide,narrow}.png
  - Windows Hub screenshots: target/hub-visual-check/window-resize-final-verified/hub-cloud-{wide,narrow}.png
  - Windows Hub screenshots: target/hub-visual-check/final-tabs-componentized-v2/hub-{editor,assets,builds,plugins,cloud,team,learn}-{wide,narrow}.png
  - Windows Hub screenshots: target/hub-visual-check/settings-componentized-v3/hub-settings-{wide,narrow}.png
  - Windows Hub screenshots: target/hub-visual-check/all-tabs-with-settings-v1/hub-{projects,editor,assets,builds,plugins,cloud,team,learn,settings}-{wide,narrow}.png
  - Windows Hub screenshots: target/hub-visual-check/hub-projects-dashboard.png, target/hub-visual-check/hub-projects-new-project.png, target/hub-visual-check/hub-projects-browser.png, target/hub-visual-check/hub-projects-detail.png
  - Windows Hub screenshots: target/hub-visual-check/material-direct-final-1366/hub-{projects,editor,builds,settings}-1366.png
  - Windows Hub screenshots: target/hub-visual-check/material-direct-final-1100-after-compact/hub-projects-1100x760.png
  - Windows Hub screenshots: target/hub-visual-check/material-direct-final-960-after-compact/hub-{projects,editor,builds,settings}-960x640.png
  - Windows Hub screenshots: target/hub-visual-check/material-card-final-v2-1366/hub-projects-1366x820.png
  - Windows Hub screenshots: target/hub-visual-check/material-card-final-v2-960/hub-projects-960x640.png
  - Windows Hub Projects pages: target/hub-visual-check/material-card-final-project-pages-v8/hub-projects-{dashboard,new-project,browser,detail}.png
  - Windows Hub Projects detail: target/hub-visual-check/material-card-final-project-pages-v8-1366b/hub-projects-detail.png
  - Windows Hub responsive Dashboard: target/hub-visual-check/material-card-final-responsive-1100x760.png, target/hub-visual-check/material-card-final-responsive-960x640.png
  - Windows Hub Material button dashboard: target/hub-visual-check/material-buttons-dashboard-fixed-960x640.png
  - Windows Hub Material text field Settings: target/hub-visual-check/material-textfield-settings-960x640.png
  - Windows Hub Material text field Editor: target/hub-visual-check/material-textfield-editor-960x640.png
  - Windows Hub Material text field Editor form: target/hub-visual-check/material-textfield-editor-fields-960x640.png
  - Windows Hub Material text field New Project: target/hub-visual-check/material-textfield-project-pages/hub-projects-new-project.png
  - Windows Hub Material toolbar popup: target/hub-visual-check/material-toolbar-popup-960x640.png
  - Windows Hub Material segmented Settings: target/hub-visual-check/material-segment-settings-960x640.png
  - Windows Hub Material source-engine dropdown header: target/hub-visual-check/material-dropdown-header-fixed-960x640.png
  - Windows Hub Material source-engine dropdown popup: target/hub-visual-check/material-dropdown-popup-960x640.png
  - Windows Hub Material Projects 1600 searchbox reference: target/hub-visual-check/material-searchbox-1600x1024-v3/hub-projects-{dashboard,new-project,browser,detail}.png
  - Windows Hub Material Projects 1280 searchbox responsive: target/hub-visual-check/material-searchbox-1280x900-v5/hub-projects-{dashboard,browser,detail}.png
  - Windows Hub Material Projects 1024 searchbox stacked secondary pages: target/hub-visual-check/material-searchbox-1024x768-v2/hub-projects-{new-project,browser,detail}.png
  - Windows Hub Material Zircon theme 1280: target/hub-visual-check/zircon-theme-material-listtile-1280x900-v3/hub-projects-{dashboard,new-project,browser,detail}.png
  - Windows Hub Material Zircon theme 1024: target/hub-visual-check/zircon-theme-material-listtile-1024x768-v1/hub-projects-{dashboard,new-project,browser,detail}.png
  - Windows Hub Material SearchBar 1024: target/hub-visual-check/material-searchbar-1024x768-v2/hub-projects-{dashboard,new-project,browser,detail}.png
  - Windows Hub Material NavigationRail: target/hub-visual-check/material-navigationrail-baseline-nosize-main-984x640.png, target/hub-visual-check/material-navigationrail-collapsed-nosize-1100x820.png
  - Windows Hub Material expanded NavButton ListTile: target/hub-visual-check/material-navbutton-listtile-expanded-nosize-1100x820.png
  - Windows Hub Material ActionRow 1600: target/hub-visual-check/material-actionrow-1600x1024-v1/hub-projects-{dashboard,browser,detail}.png
  - Windows Hub Material ActionRow 1024: target/hub-visual-check/material-actionrow-1024x768-v1/hub-projects-{dashboard,browser,detail}.png
  - Windows Hub Material header engine ListTile popup: target/hub-visual-check/material-header-engine-listtile-984x640-v1.png
  - Windows Hub Material InfoRow Editor: target/hub-visual-check/material-inforow-editor-984x640-v1.png
  - Windows Hub Material InfoRow Builds: target/hub-visual-check/material-inforow-builds-984x640-v2.png
  - Windows Hub Material Settings tall capture: target/hub-visual-check/material-inforow-settings-1280x900-v3.png
  - Windows Hub Material WindowButton title bar: target/hub-visual-check/material-windowbutton-984x640-v1.png
  - Windows Hub Workspace Cloud: target/hub-visual-check/cloud-workspace-section-1024x768-v5.png
  - Windows Hub Workspace Team: target/hub-visual-check/team-workspace-section-1024x768-v2.png
  - Windows Hub Material StatusPill ActionChip header: target/hub-visual-check/material-statuspill-actionchip-1600x1024.png
  - Windows Hub Material Divider dashboard: target/hub-visual-check/material-divider-1600x1024.png
  - Windows Hub Material data-display ScrollView dashboard: target/hub-visual-check/material-datadisplay-scrollview-1100x820.png
  - Windows Hub Material Projects ScrollView 1100: target/hub-visual-check/material-project-scrollview-1100x820/hub-projects-{dashboard,new-project,browser,detail}.png
  - Windows Hub Material Badge MaterialText software dashboard: target/hub-visual-check/material-badge-materialtext-1100x820-software.png
  - Windows Hub Material Badge MaterialText software Projects pages: target/hub-visual-check/material-badge-materialtext-pages-1100x820/hub-projects-{dashboard,new-project,browser,detail}.png
  - Windows Hub shared MaterialText typography software dashboard: target/hub-visual-check/shared-materialtext-typography-1100x820.png
  - Windows Hub surface title MaterialText software dashboard: target/hub-visual-check/surface-title-materialtext-1100x820.png
  - Windows Hub DataTable MaterialText software dashboard: target/hub-visual-check/data-table-materialtext-1100x820.png
  - Windows Hub CatalogListPanel MaterialText software Assets: target/hub-visual-check/catalog-list-materialtext-1100x820-assets.png
  - Windows Hub CatalogListPanel MaterialText software Plugins: target/hub-visual-check/catalog-list-materialtext-1100x820-plugins.png
  - Windows Hub shell page-header/status MaterialText software Assets: target/hub-visual-check/shell-page-header-materialtext-1100x820-assets.png
  - Windows Hub shell chrome MaterialText software Assets: target/hub-visual-check/shell-chrome-materialtext-1100x820-assets.png
  - Windows Hub dashboard card/empty MaterialText software Projects: target/hub-visual-check/dashboard-card-materialtext-1100x820.png
  - Windows Hub Projects workflow MaterialText 1100: target/hub-visual-check/project-workflow-materialtext-1100x820/hub-projects-{dashboard,new-project,browser,detail}.png
  - Windows Hub Cloud/Team MaterialText 1100: target/hub-visual-check/cloud-team-materialtext-1100x820/{cloud,team}-materialtext.png
  - Windows Hub Builds MaterialText: target/hub-visual-check/builds-materialtext-1100x820/builds-materialtext.png and target/hub-visual-check/builds-materialtext-1600x1024/builds-materialtext.png
  - Hub UI direct static contracts: target/hub-material-check/ui_contract_windowbutton_direct.exe --nocapture
  - Hub UI direct static contracts: target/hub-material-check/ui_contract_project_browser_state_direct.exe --nocapture
  - Hub UI direct static contracts: target/hub-material-check/ui_contract_project_selector_state_direct.exe --nocapture
  - Hub UI direct static contracts: target/hub-material-check/ui_contract_sidebar_collapse_state_direct.exe --nocapture
  - Hub UI direct static contracts: target/hub-material-check/ui_contract_project_browser_select_direct.exe --nocapture
  - Hub UI direct static contracts: target/hub-material-check/ui_contract_global_typography_direct.exe --nocapture
  - Hub UI direct static contracts: target/hub-material-check/ui_contract_character_icons_direct.exe --nocapture
  - Hub UI direct static contracts: target/hub-material-check/ui_contract_toucharea_direct.exe --nocapture
  - Hub UI direct static contracts: target/hub-material-check/ui_contract_percentage_sizing_direct.exe --nocapture
  - Hub UI direct static contracts: target/hub-material-check/ui_contract_workspace_slot_direct.exe --nocapture
  - Hub Slint generation: target/hub-ui-check-project-browser-state/app.rs
  - Hub Slint generation: target/hub-ui-check-project-selector-state/app.rs
  - Hub Slint generation: target/hub-ui-check-sidebar-collapse-state/app.rs
  - Hub Slint generation: target/hub-ui-check-project-browser-select/app.rs
  - Hub Slint generation: target/hub-ui-layout-gen/app.rs
  - PowerShell syntax: .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-project-pages.ps1
  - Slint UI generation: CARGO_MANIFEST_DIR=zircon_hub OUT_DIR=target/hub-ui-check target/debug/build/zircon_hub-*/build-script-build.exe
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-project-pages.ps1
  - python tools/zircon_build.py --targets editor,runtime --out <tmp> --mode debug --cargo cargo-nextest --dry-run
  - cargo test -p zircon_app --features target-editor-host --no-default-features --locked editor_gui_startup_parser
  - cargo check -p zircon_app --features target-editor-host --no-default-features --locked
  - python tools/zircon_build.py --targets hub,editor,runtime --out <tmp> --mode debug --dry-run
  - python -m py_compile tools/zircon_build.py
doc_type: category-index
---

# Zircon Hub

`zircon_hub` is the standalone desktop launcher for ZirconEngine. It is a top-level workspace package, but it is not an engine runtime module and does not register with `zircon_runtime` lifecycle services.

The Hub owns a real Slint desktop shell with a UnityHub-style layout: a frameless top-level window, a self-drawn title bar, a full navigation rail, a Projects dashboard, an Editor/source-engine page, an Assets catalog page, a Plugins catalog page, a local Cloud readiness page, a local Team page, a Learn documentation page, a Builds page for the source build pipeline, and Settings. Slint is intentionally confined to this package; editor UI remains the Rust-owned retained host and does not regain Slint business UI paths.

## Ownership

- `zircon_hub/src/app` initializes Slint, binds callbacks, and projects Rust state into the window.
- `zircon_hub/src/app/runtime/project_workspace.rs` owns Projects workspace subpage actions, including template selection, project-engine binding, pin/remove/delete flows, and project metadata persistence. These callbacks stay under the app runtime implementation so richer Projects shell interactions do not widen the public Hub API.
- `zircon_hub/src/app/runtime/asset_catalog.rs` refreshes the runtime asset catalog from the selected project, recent project roots, and source-checkout roots whenever projects or source engines change.
- `zircon_hub/src/app/runtime/folder_picker.rs` owns runtime folder-browse dispatch for project roots, project locations, source checkouts, staged output, and local device install targets.
- `zircon_hub/src/app/runtime/learn_catalog.rs` refreshes the local Learn catalog from source-checkout and repository documentation roots and opens selected resource files through the system file opener.
- `zircon_hub/src/app/runtime/plugin_catalog.rs` refreshes selected-project plugin manifests and engine plugin manifests as separate Project/Engine scopes for the Plugins page.
- `zircon_hub/src/app/runtime/team_overview.rs` refreshes the local Team overview from the selected project first, then the active source checkout, current working directory, and compile-time workspace root.
- `zircon_hub/src/app/runtime/window_controls.rs` owns frameless-window behavior: startup geometry restore, normal-window geometry capture, minimize/maximize/close callbacks, title-bar dragging, and double-click maximize persistence.
- `zircon_hub/src/app/view_model.rs` converts runtime snapshots into Slint navigation, source-engine, settings, and header/status data models. Its `view_model/projects.rs`, `view_model/assets.rs`, `view_model/cloud.rs`, `view_model/learn.rs`, `view_model/plugins.rs`, `view_model/quick_actions.rs`, and `view_model/team.rs` children own page-specific row projection for Projects, assets, local cloud readiness, Learn resources, Quick Actions, plugins, and local Git team data. `view_model/projects.rs` keeps `ProjectDetailData` tied to `selected_project_path`: without a selected project it returns an explicit unavailable no-selection row. `view_model/quick_actions.rs` keeps the selected-or-latest-recent fallback for build/package/install/open-editor commands; when that fallback promotes the latest recent project or activates a project's bound Source Engine, `project_workspace.rs` refreshes the selected-project scoped Assets, Plugins, and Team data before the next snapshot is projected.
- `zircon_hub/src/app/view_model/media.rs` owns the Hub static SVG image lookup layer. It resolves bundled navigation, quick-action, status, and fallback project-cover assets under `zircon_hub/assets` and preserves the existing real-project cover lookup before falling back to bundled cover art.
- `zircon_hub/src/app/localization.rs` owns the first Hub UI string table for English and Chinese. The view-model layer uses it for navigation, page titles, quick actions, source-engine status, configuration health, header pills, relative time labels, and the Slint `UiTextData` bundle.
- `zircon_hub/src/app/quick_action.rs` owns the stable quick-action identifiers used by the Slint shell and Rust dispatcher.
- `zircon_hub/src/state` stores the selected page, project filter mode, project sort mode, project view mode, search query, and task status snapshot used by the app layer. `HubPage` now covers Projects, Editor, Assets, Builds, Plugins, Cloud, Team, Learn, and Settings.
- `zircon_hub/src/projects` owns recent-project records, Hub-owned project metadata keys, project creation templates, project root validation, project cover discovery, Windows recycle-bin delete command construction, local project package staging, local device-folder installation, and Editor recent sync.
- `zircon_hub/src/assets` owns the local asset catalog scanner. It scans the selected project's `Assets`/`assets` folders first, then other recent project asset folders plus engine asset roots under the active source checkout, current directory, and compile-time workspace root.
- `zircon_hub/src/learn` owns the local Learn documentation scanner. It scans `docs/**/*.md`, extracts headings and summaries, skips transient directories, and returns capped catalog rows for the Hub page.
- `zircon_hub/src/plugins` owns local plugin catalog discovery from selected-project `plugin.toml`/`Plugins/**/plugin.toml`/`plugins/**/plugin.toml` manifests plus engine `zircon_plugins/**/plugin.toml` manifests under the active source checkout, with current-directory and compiled-workspace fallbacks for development runs.
- `zircon_hub/src/team` owns local Git team discovery. It reads repository identity and recent contributors from local `git` commands only; it does not connect accounts, cloud services, or repository hosting providers.
- `zircon_hub/src/engines` owns source checkout records, staged output paths, source-engine validation, recent source-build records, and registry helpers for upsert, active selection, and removal. Hub configuration can now store multiple source-engine records plus an active engine id.
- `zircon_hub/src/build` creates and runs `tools/zircon_build.py --targets editor,runtime` commands for source installs.
- `zircon_hub/src/settings` owns Hub TOML config paths and toolchain/build defaults.
- `zircon_hub/src/process` owns editor launch/open-folder child process commands.
- `zircon_hub/src/process/folder_picker.rs` owns Hub folder selection integration. The current implementation uses the Windows system folder browser through PowerShell/WinForms and returns a clear unavailable status on non-Windows platforms until a cross-platform picker is added.
- `zircon_hub/build.rs` registers the repository-local Slint Material UI template at `dev/material-rust-template/material-1.0/material.slint` as the `@material` Slint library path. Hub keeps Slint as the renderer and does not embed HTML/CSS, but the reusable UI entrypoint now imports the real template rather than recreating a separate clone.
- `zircon_hub/ui/theme.slint` installs the Zircon Material palette at Slint initialization time. It keeps Material component behavior and metrics from the local template while overriding the template's default blue primary scheme with the Hub reference's teal/dark visual language.
- `zircon_hub/ui/components.slint` is now only the reusable Hub UI entrypoint. Implementation is split into `tokens.slint`, `material_bridge.slint`, `layout.slint`, `surfaces.slint`, `inputs.slint`, `shell.slint`, `navigation.slint`, `data_display.slint`, and `overlays.slint`. The bridge directly re-exports selected `@material` layout, card, button, input, navigation, list, progress, tab, tooltip, and style primitives so pages can use one public module while component ownership follows the Material-style categories documented in `docs/zircon_hub/ui/responsive-component-system.md`.
- `zircon_hub/ui/surfaces.slint` keeps the public `HubPanel`/`HubCard` API but now backs those Hub surfaces with the imported Material template `ElevatedCard` and `OutlinedCard` primitives. Zircon-specific panel variants still provide the dark border, tint, hover, and status colors above that template base. The panel hover layer sits behind `@children` and is enabled only for interactive variants so default Material-backed panels do not swallow nested row, button, or input clicks. `PanelHeader` keeps its Hub-facing right action slot, but the action is now an imported Material `OutlineButton` instead of a hand-painted rectangle plus `TouchArea`; its title text and `StatusBanner` titles now use imported Material `MaterialText` typography instead of raw `Text` font bindings. Hub keeps its public `Badge`/`StatusBadge` tone shell instead of importing the conflicting template `Badge` name, while badge label typography is delegated to the imported Material `MaterialText` component.
- `zircon_hub/ui/shared.slint` keeps Hub `PillButton`, Hub `IconButton`, `WindowButton`, header `StatusPill`, expanded-navigation `NavButton`, `FieldLabel`, and `MutedText` as stable public APIs, but those wrappers now use imported Material template primitives internally: `FilledButton`, `OutlineButton`, `FilledIconButton`, `OutlineIconButton`, template `IconButton`, `ActionChip`, `ListTile`, and `MaterialText`. `PillButton` owns the atom-level preferred width and clipping boundary needed by Material text buttons inside compact responsive headers, Hub `IconButton` clips the Material icon button to the requested atom size in dense rows, `WindowButton` uses the template inline icon metric for centered title-bar controls, `StatusPill` keeps the Hub status/tone shell while Material owns the chip icon/text body, `NavButton` keeps Hub page ids while Material owns the expanded row avatar/text body and pointer state, and Hub text labels delegate font metrics to `MaterialTypography` through `MaterialText`. Hub pages still express primary, secondary, active, inactive, text, icon, label, and muted copy through the existing semantic controls while Material owns the button/chip/list/text pointer behavior, disabled state, ripple/state layer, typography metrics, and accessibility surface.
- `zircon_hub/ui/inputs.slint` now exposes `SearchBox` as a Hub-stable wrapper around the imported Material template `SearchBar`, while `HubTextField` remains the Hub form wrapper around Material `TextField`. Toolbar search uses the Material search atom with a leading search icon, visible placeholder prompt, and `HubTokens.input-field` height instead of hand-painting a local input or using `TextField` label text as a toolbar placeholder. Settings uses `HubTextField` for toolchain executables, build jobs, and default paths; Editor uses it for active engine name, source checkout, and staged output; Projects New Project uses it for project name and location. Editable Hub form pages no longer directly depend on `std-widgets` `LineEdit`, and toolbar search no longer directly embeds `TextInput`. `HubTokens.input-field` owns the Material field height and `HubTokens.input-width` owns the preferred input width at the atom layer. The same file now routes Hub `SegmentButton` through the imported Material `SegmentedButton`, routes Hub `ToolbarSelect` through Material `OutlineButton` plus `PopupMenu`, and routes Hub `DropDownButton` through Material `OutlineButton`/`TonalButton`, while keeping the existing Hub-facing value/callback contracts.
- `zircon_hub/ui/shell.slint` owns the Hub chrome components: top header, source-engine picker, nav sidebar, page title/action band, bottom status bar, window buttons, and drag regions. The source-engine popup rows now use imported Material `ListTile` rows so engine title/status/avatar alignment and row interaction come from the template rather than a custom full-row `TouchArea`; the popup heading/count, brand title/subtitle, local-user initials/name, nav engine-status title, current-project label, and update label now use Material `MaterialText` instead of raw `Text` font bindings. `HubTopHeader` receives `ProjectDetailData` and shows the selected project as the brand subtitle when available while keeping the Source Engine selector beside it. `HubNavSidebar` receives `ProjectDetailData` beside `SourceEngineData`, so the expanded lower status card shows both active engine state and the current selected-project title before the update action. `HubPageHeader` receives `ProjectDetailData`, `SourceEngineData`, compact state, and tokenized context-badge width from `app.slint`; non-Projects pages show selected-project and active Source Engine badges beside the page title actions, while Projects keeps Import/New as the primary title-band actions. The page title and non-Projects task label also use Material `MaterialText` styles. The bottom status bar receives the same context, so wide layouts show the current task plus compact selected-project and active Source Engine badges without deriving layout from the status bar's resolved width; compact widths keep the task detail first and hide the badges before they can overlap. Its task detail also uses Material `MaterialText`, keeping shell typography on the imported template. The sidebar collapse control uses imported Material `StateLayerArea`, keeping direct `TouchArea` usage reserved for frameless-window dragging. `zircon_hub/ui/app.slint` keeps the Rust-facing `HubWindow` property/callback contract and routes shell/page content instead of drawing those regions inline.
- `zircon_hub/ui/navigation.slint` now uses the imported Material template `NavigationRail` for the collapsed 80px navigation rail. The existing `NavItemData` projection still owns page ids and active state, while `view_model.rs` derives Material `NavigationItem` rows from the same model for template rendering. Expanded navigation stays on the Hub row-style `NavButton` list because the template rail is icon-first and the template drawer is wider than the Hub sidebar contract, but each row body is now backed by Material `ListTile`.
- `zircon_hub/ui/project_dashboard.slint` owns the Projects dashboard toolbar, project card flow, Recent Projects panel, Quick Actions panel, and Material `ScrollView` dashboard/quick-action scroll surfaces. `zircon_hub/ui/project_page_components.slint` owns the shared Projects secondary-page header, summary rows, engine/template rows, and project browser row; its visible title/meta/summary labels now route through Material `MaterialText` instead of page-local `Text` font bindings. `zircon_hub/ui/projects.slint` stays as the Projects subpage router and forwards data/callbacks to dashboard, browser, new-project, and detail components.

## Shell Layout And Data Projection

`ui/app.slint` is the shell root. It uses `no-frame: true` and `resize-border-width` to remove the native frame while retaining resize behavior, then delegates chrome rendering to `shell.slint`. The title bar is drawn by `HubTopHeader` and exposes callbacks for minimize, maximize/restore, close, and drag. Rust handles those callbacks through Slint's window API. Dragging is implemented by storing the physical window origin at pointer-down and applying logical pointer deltas scaled by the current window scale factor.

Hub UI contract checks live in `zircon_hub/tests/ui_contract.rs` as integration tests. The contract now scans every Hub-owned `.slint` file for raw `Text`, `inherits Text`, direct `font-size`/`font-weight` bindings, character-icon `text`/`fallback-text` literals, direct `TouchArea` usage outside the shell drag layer, and percent sizing in `width`/`height`/`min-*`/`max-*`/`preferred-*` bindings, so visible typography must continue through Material `MaterialText` or shared Hub wrappers while icon-like affordances use SVG assets or Material icon slots, pointer behavior stays on Material controls, `ListTile`, `PopupMenu`, or `StateLayerArea`, and stretch behavior remains explicit through parent constraints plus flex slots instead of `100%` preferred sizes. Read-only `MaterialTypography.*.font_size` metric references remain available for layout height calculations. The Hub library target disables Cargo's default unit-test harness with `test = false` because the current checks are static file/layout contracts and do not need a second `--test` compilation of the full Slint-generated library; regular check/build commands still compile the generated Hub UI and binary.

The default Hub window uses a 1600x1024 preferred size to match the reference Projects composition: four project cards in the first row, Recent Projects and Quick Actions below them, and the shell status bar available at the bottom of the first viewport. Its minimum window dimensions are derived from `HubTokens.window-min-width` and `HubTokens.window-min-height`, so native resize and saved window state can drive page `root.width`/`root.height` and trigger compact Hub layouts instead of clipping a fixed 1600px surface.

Hub chrome uses repository-owned SVG assets instead of placeholder text glyphs for the brand mark, navigation rail, quick-action entries, status pills, toolbar controls, dropdown chevrons, window buttons, and bundled project covers. Static chrome such as the brand mark and Hub-owned UI glyphs are loaded directly from Slint with `@image-url`, while state-projected icons are loaded in Rust by `view_model/media.rs` and passed to Slint as `image` fields with text fallbacks retained only for unavailable asset cases.

Shared icon-bearing controls route their glyphs through fixed-height icon slots before text is laid out. `PillButton`, Hub `IconButton`, `WindowButton`, `StatusPill`, `ToolbarSelect`, `DropDownButton`, `SearchBox`, collapsed `NavRail`, expanded `NavButton`, `InfoRow`, and `ActionRow` now receive those slots from imported Material button/chip/menu/search/navigation/list primitives. `StatusPill` uses Material `ActionChip`, `Badge`/`StatusBadge`, `PanelHeader`, `StatusBanner`, and shared `FieldLabel`/`MutedText` use Material `MaterialText` for typography, while expanded `NavButton`, `InfoRow`, and `ActionRow` use Material `ListTile` for row body, state layer, avatar icon, text, and disabled handling, with Hub retaining only the surrounding status/selected-row shell or trailing badge/chevron affordance slots needed by header status, navigation, catalog, and operation rows. Shared stretch components such as `InfoRow`, `ActionRow`, `PanelGrid`, and `PanelHeader` now use `horizontal-stretch`, `min-width`, parent/content-width inputs, and `preferred-width: 0px` instead of `preferred-width: 100%`, so their preferred size is not derived from the resolved parent width.

The Projects title band and its primary actions now live in `HubPageHeader`, so the title/subtitle group and Import/New buttons remain vertically centered inside the shell title area without page-level coordinates. Non-Projects title bands use the same header component to show current selected-project and active Source Engine context badges on wide layouts, keeping page scope visible without moving that context into each page body. Header height, title-band height, left-rail width, rail padding, rail status-panel height, page action height, and bottom status-bar height are derived from the current window size with tokenized minimum touch-target clamps. `HubNavSidebar` composes `NavRail`, the engine/project status card, and the collapse action; the expanded lower card shows active engine version/status plus the selected-project title, while the collapsed rail is Material `NavigationRail` at the template's 80px width. Expanded navigation remains the left-aligned Hub row list that matches the reference reading direction through Material `ListTile`-backed `NavButton` rows. The sidebar collapse control now uses Material `StateLayerArea` for hover/press/click feedback, leaving shell `TouchArea` usage reserved for the frameless title-bar drag region. Toolbar selectors use the template button/menu path where the Material template owns that atom, and the Source Engine dropdown trigger now uses the same Material button path before opening Material `ListTile` engine rows in the Hub-owned popup. Dashboard tables reserve enough width for `Engine Version` and `Last Modified`, while Quick Actions keep a dedicated right-side arrow slot so the arrow buttons form a stable trailing column.

Window placement is now part of Hub configuration. Rust restores the previous physical position, size, and maximized state after loading the Hub config. It captures normal-window geometry on drag end, minimize, maximize/restore, and close, with a minimum size guard so stale or tiny saved values cannot hide the Hub surface. A second title-bar press within the double-click window toggles maximize through the same persisted path. Platform edge snapping remains delegated to the operating system; no custom snap zones are implemented in the Hub yet.

The top title bar status pills are data-driven through `HeaderStatusData`. The first pill reflects the current task state (`Running` or `Ready`). The success, warning, and error pills aggregate the same Rust-side configuration health data used by Settings, plus action failures where applicable, so the shell no longer shows static sample status labels. On compact-width windows the title bar hides the status-pill group and keeps the engine selector plus window controls visible, preventing the shared shell from overlapping before page bodies enter their own responsive layouts. The Source Engine selector width is now derived from nav-width, header-button, and token clamps rather than a direct `root.width / 4` formula, keeping shell chrome on the same semantic sizing contract as the page body. On compact-height windows, including the 820px visual-validation height, the left rail hides its lower engine/project status card and collapse control so the expanded Material `ListTile` navigation rows and bottom task/status bar remain visible.

The title-bar engine capsule displays the active source engine and opens a Slint `PopupWindow` source-engine selector when clicked. The popup is anchored relative to the dropdown button instead of being positioned from the window root. It is backed by the same `SourceEngineRowData` model as the Editor page and dispatches the same Rust active-engine selection callback, so header switching and Editor-page switching share one persisted active-engine path.

The left rail is model-driven through `NavItemData`. All navigation entries are switchable. Projects, Editor, Assets, Plugins, Cloud, Builds, Team, Learn, and Settings connect to active Hub behavior. The rail can collapse to an icon-only Material `NavigationRail` using local Slint window state; Rust derives the Material `NavigationItem` rows from the same navigation model and the rail maps template index callbacks back to Hub page ids. In expanded mode, the Hub keeps its 200px sidebar contract and page-id callbacks, while each `NavButton` row renders through Material `ListTile` for avatar/title alignment and row interaction. This collapse preference is transient and is not written into the Rust runtime snapshot or persisted Hub configuration.

`binding.rs` remains a thin projection surface. It applies a `HubSnapshot`, pushes Slint `ModelRc` values built by `view_model.rs`, and reads editable settings fields back from the UI. Formatting of project cards, recent rows, engine summaries, and quick actions lives outside the binding entry file. Search edits are sent back through a dedicated callback so the Rust snapshot can immediately rebuild the card and table models without waiting for another action.

`selected_project_path` is the shared workflow context for Hub pages that need project scope. Selecting or remembering a project refreshes asset, plugin, and team projections immediately; removing or confirming deletion of the selected project clears the same scoped projections before the next snapshot. The shared remove helper also clears pending-delete state for the same path, so a Recycle Bin confirmation cannot linger after the project has been removed from Hub metadata. Project Detail remains the authoritative place for pin/remove/delete and Source Engine binding, while Builds, Assets, Plugins, Cloud, and Team receive enough scoped data to explain whether they are showing selected-project, engine, or local-repository state.

The refactored tab bodies avoid the old page-local fixed content-height formulas under the Hub title area. Editor, Assets, Plugins, Cloud, Team, Learn, Builds, and Settings keep their own scroll surfaces where the content can exceed the visible Hub viewport, so lower controls remain reachable instead of being clipped by the bottom status bar. These pages now prefer `PageScrollSurface` for page-level overflow, `CatalogListPanel` for catalog rows and empty states, Material `ScrollView`-backed `PanelListViewport` for panel-owned row lists, and `WorkspacePanelSection` for tokenized workspace rows and wrapped metric rows. `PageScrollSurface` exposes padded content width, viewport height, and padded content height, so Editor, Builds, Settings, Cloud, Team, Projects dashboard, Projects secondary pages, and shared catalog pages no longer repeat `root.width - page-padding * 2` in their page roots or assign `width: root.content-width` to page rows, while Cloud, Team, Assets, Plugins, and Learn avoid recomputing window-height minus page padding and bottom status for their list panels. Stretch layout owns row width fill behavior. Projects remains the layout baseline page and still owns proportional dashboard/subpage height tokens for its card flow, browser, and secondary project workflows; dashboard, quick-action, browser, new-project, and detail scrolling now use the same Material `ScrollView` path through `PageScrollSurface` where it is page-level, Projects toolbar and secondary-page flex items route sizing through `ResponsiveSlot`, and dashboard repeat cards use preferred/min widths because Slint rejects a repeated `ResponsiveSlot` wrapper in that card path.

Hub layout is owned by Slint page composition and reusable components, not by a Rust-side coordinate calculator. `zircon_hub` depends on Slint 1.16.1 or newer and the build script compiles the UI through `i-slint-compiler` with experimental layout support enabled so the Projects dashboard can use Slint's native `FlexboxLayout` and `flex-wrap`. Slint owns drawing, input, model repeaters, flex wrapping, Material-template `ScrollView` clipping, and Material-template divider atoms; Rust only projects data models and handles business callbacks.

The Hub component system now starts from the local Slint Material UI template and uses a Material UI-style taxonomy for organization, while the visual language remains the custom Zircon Hub reference screenshot rather than Material Design styling. MUI's public component catalog groups primitives into Inputs, Data display, Feedback, Surfaces, Navigation, Layout, and Utils; Hub maps those categories into Slint components and re-exports selected `@material` primitives so page files compose named primitives instead of rebuilding local layouts:

- Inputs: `PillButton`, `IconButton`, `WindowButton`, Material-backed `SearchBox`, `ToolbarSelect`, `DropDownButton`, segmented controls, and text-field wrappers.
- Data display: Material `MaterialText`-backed `Badge`/`StatusBadge`/`PanelHeader`/`StatusBanner`/`FieldLabel`/`MutedText`/`TableColumnHeader`/`ProjectTableRow`/`CatalogListPanel` empty titles, Projects dashboard card/empty titles, Projects secondary-page headers/summary labels/browser meta labels, Builds current-task status text, Cloud metric status labels, Team summary/empty-state titles, the global raw Text/direct font-binding, character-icon, and direct TouchArea contracts, Material `ListTile`-backed `InfoRow`/`ActionRow`, Material `ScrollView`-backed `DataTable`, `CatalogListPanel`, `PanelListViewport`, project cards, and typography wrappers.
- Feedback: Material `ActionChip`-backed header status pills, configuration-health rows, disabled-state rows, and future alert/snackbar/dialog surfaces.
- Surfaces: `HubPanel`, `PanelHeader`, project cards, Recent Projects, Quick Actions, and settings panels. `HubPanel` wraps Material `ElevatedCard`/`OutlinedCard` instead of being a fully custom panel surface.
- Navigation: `HeaderGroup`, title-bar engine selector, Material `NavigationRail` for collapsed Hub navigation, Material `ListTile`-backed expanded `NavButton` rows, nav rail collapse control, tabs/view-mode controls, and popup/menu affordances.
- Layout: `PageScrollSurface`, `PanelListViewport`, `WorkspacePanelSection`, `ResponsivePanelFlow`, page scroll shells, toolbar rows, Flexbox card grids, catalog list shells, and stack-style horizontal/vertical groups.
- Utils: SVG icon resources, popup anchoring, hover/pressed state wrappers, and local fallback behavior for missing image assets.

Slint 1.16.1's Taffy-backed `FlexboxLayout` is the required basis for wrapping dashboard regions such as project cards and lower two-column work surfaces. Business pages should prefer `VerticalLayout`, `HorizontalLayout`, `FlexboxLayout`, and Material-template `ScrollView`/divider composition. Pixel values are allowed for design tokens such as button height, icon size, spacing, border radius, popup anchors, and minimum touch targets; they should not be used as page-level coordinate math. The Hub shell derives its header group, title band, rail, and status bar dimensions from the window, constrains the page host to the width remaining after the navigation rail, and Projects dashboard/secondary pages derive card basis, card row height, lower-panel height, subpage header height, browser toolbar wrap height, browser row height, thumbnail size, source-engine row height, template row height, action height, compact-table row height, compact-table column tokens, and scroll viewport height from `PageScrollSurface`, parent dimensions, or row tokens so the layout behaves like a proportional web surface. Projects dashboard toolbars and New/Browser/Detail columns keep Taffy basis/grow/shrink behind `ResponsiveSlot`; New/Detail main-side columns now use tokenized main/side bases, while repeated dashboard cards stay on preferred/min width sizing to avoid Slint 1.16.1 binding-analysis failures inside the repeater. Workspace metric pages and Projects page roots now consume `PageScrollSurface.content-width` for compact thresholds instead of page-local remaining-width formulas. Editor, Builds, and Settings detail rows now use ResponsiveSlot basis plus grow weights instead of precomputing overview/side or primary/detail widths from remaining-width formulas. Shared table rows consume row/column tokens from their parent page instead of calculating their own width or height from their own layout result, which avoids Slint binding loops while keeping dashboard density responsive.

## Projects Dashboard

The Projects page now uses real recent-project data in two projections:

- Project cards show the first four filtered recent projects with a real project cover when the project directory contains `.zircon/cover.*`, `.zircon/thumbnail.*`, root-level cover/thumbnail/project images, or matching `Assets`/`assets` images. If no supported image exists or Slint cannot load it, the card falls back to one of the bundled SVG cover thumbnails under `zircon_hub/assets/covers`; the Slint-drawn accent background remains the last-resort visual if a bundled asset cannot be loaded.
- The Recent Projects table shows up to eight filtered projects with cover thumbnail, name, engine version placeholder, last-opened label, path, and a row action affordance.

The Projects shell title area owns the primary Import Project and New Project actions, matching the reference screenshot's title-and-actions composition. The page toolbar below it carries search, project filter, sort selector, and grid/list view buttons on an aligned Material-height row, wrapping into two rows when the content width cannot hold all controls. Toolbar select sizing now uses a tokenized `ResponsiveSlot` basis with grow enabled in wrapped mode instead of subtracting fixed icon-button space from the page width. Search uses the shared `SearchBox` component, which now wraps the imported Material `SearchBar` with a bundled search SVG and visible placeholder prompt instead of directly embedding `TextInput`. Search, project filtering, and sort update the current Projects projection immediately. New Project now opens a Projects-internal `new-project` subpage instead of an inline dashboard form. The list view button and Recent Projects `View All Projects` action open the Projects-internal `project-browser` subpage instead of sharing the dashboard Show More behavior.

The Projects dashboard uses Slint `FlexboxLayout` for the upper project-card flow. Each card is a vertical cover/title/path/time/tag surface whose flex basis, minimum width, cover height, and row height are derived from the available page width rather than a fixed four-card coordinate grid. Project card titles and the dashboard empty-state title now use Material `MaterialText` styles instead of raw `Text` font bindings. Wide windows still land on the reference-style four-card row, while narrower windows naturally wrap down to fewer columns. Card basis now uses a proportional content-width token rather than a page-local width-minus-gap formula. The card area starts as one visible row and can expand to additional rows through the Show More control. The dashboard page body uses `PageScrollSurface` for page-level scrolling and derives content width plus usable viewport height from that shared primitive instead of maintaining a separate dashboard scroll shell. The parent Projects page now passes the card-height token into `ProjectFlow`, so the scroll budget and the real card layout use the same proportional value instead of maintaining two independent height formulas. Lower Recent Projects and Quick Actions panels sit in a wrapping flex surface and use tokenized main/side `ResponsiveSlot` basis/grow/min-width values instead of a page-local remaining-width formula; the recent list is a real `DataTable` with proportional columns, parent-provided row height, Material `ScrollView`, and Material `MaterialText` table header/cell typography. Quick Actions use the shared Material `ListTile`-backed `ActionRow` inside a Material `ScrollView` quick-action viewport, with a parent-provided row-height token clamped to the Material list row minimum, so the row state layer and avatar/text alignment come from the template while the right-side arrow slot remains fixed; other pages keep the component's default Material list-row height.

The Recent Projects panel keeps the `View All Projects` action in the header's right action slot. The action now renders through `PanelHeader`'s Material `OutlineButton` slot and navigates to the wider `ProjectBrowserPage`; it no longer expands dashboard cards and no longer shares behavior with Show More. The dashboard list projection is named Pinned Projects when pinned metadata exists, and otherwise falls back to Recent Projects, with the title localized through the Rust view-model language path. Project cards surface pinned and missing status badges next to the project title when those Hub metadata states apply. The compact dashboard row count is projected separately from the raw recent-project count, so pinned-only dashboards keep empty-state and scroll math aligned with the actual rows. Dashboard project cards and compact table rows keep their selected/background shell in Hub code but delegate whole-card/whole-row hover, press, and click feedback to Material `StateLayerArea`. The browser subpage uses the same Pinned/Recent title source and a loose selector row with cover/icon, path, version badge, bound source engine, last-opened label, missing/pinned badges, and a dedicated detail action. The browser row body uses the same Material state-layer path for row selection, so project selectors no longer need separate thumbnail/body/full-row `TouchArea` layers. Browser row meta stays close to the project title/path instead of being stretched into sparse table columns, while the trailing detail button remains a separate right-aligned control and is the only browser-row transition into Project Detail.

The `ProjectNewPage` places core project settings in the main panel: project name, target location, Source Engine selection, a live create summary, validation status through the create button state, and the create command. The name/location inputs use `HubTextField`, so the new-project form shares the same imported Material `TextField` atom and `HubTokens.input-field` height as Settings and Editor. Source Engine and template choices use imported Material `ListTile` rows, with Hub badges and selected-state borders layered around the template row behavior. The summary resolves the final project root from the editable name/location fields and displays the selected Source Engine, selected template, and create readiness, so the central panel remains the primary project-setup surface instead of leaving an empty form area; its label/value typography now goes through Material `MaterialText`. Templates are a right-side selection rail rather than the primary visual area. The first implemented template is `Renderable Empty`, which maps to the existing Editor launch contract `--create-project --template renderable-empty`; placeholder 2D, 3D, and sample templates are visible but disabled until real creation paths exist. Page titles, panel labels, source-engine empty states, template availability labels, summary labels, and create actions are fed from `UiTextData`, so English and Chinese language modes use the same Projects secondary-page structure.

Projects secondary pages share a `PageHeader` whose height is passed from each page's row token, so the back button, title, subtitle, and Material `MaterialText` title style scale together. New Project, Browser, and Detail page roots now use `PageScrollSurface.content-width` for content-relative breakpoints instead of a local `root.width - page-pad * 2` subtraction. The New Project and Detail pages choose their two-column-to-stacked breakpoint from available content width relative to their row/action height instead of a fixed window width, and their panel columns are `ResponsiveSlot` children of the page `FlexboxLayout`. The Project Browser toolbar derives its wrap threshold and two-row height from the toolbar control token, keeping search, filter, and sort controls aligned on wide windows while allowing a wrapped toolbar on narrow windows; those toolbar items also route basis/grow/shrink through `ResponsiveSlot`. Filter and sort selectors pass Material `MenuItem` rows into `ToolbarSelect`, while the legacy option-id array remains the dispatch map for `all`, `existing`, `missing`, `last-modified`, and `name`. Project Browser row title, engine label, and modified label also use Material `MaterialText`, keeping row typography on the imported template while Hub still owns row selection, cover, badges, and the separate detail affordance.

The `ProjectDetailPage` shows the selected project's name, path, cover/status, last-opened label, bound Source Engine, and Hub metadata controls. Its main panel now includes a Project Information section with status, project root, bound engine, engine version, and last-modified rows before the editable Source Engine binding list, keeping the detail view useful even when the action panel is not being used. Open validates and launches the selected project; Source Engine rows update the Hub-owned project metadata; Pin toggles the pinned projection; Remove from Hub deletes only Hub recent/pin/engine metadata and leaves disk files untouched; Delete Project requires an explicit second confirmation and uses the Windows Recycle Bin command path. Non-Windows delete remains unavailable in this first pass. Detail action labels, pinned/missing/not-pinned badges, delete confirmation labels, info labels, and recycle-bin confirmation copy are also projected through `UiTextData`, keeping the Projects detail workflow localized without duplicating text inside Slint page code. Dashboard compact rows keep their whole-row selection layer behind the right-side detail button, so the Recent Projects detail affordance opens the same secondary detail page instead of being swallowed by row selection.

The earlier development-only Button States strip and internal `ComponentSamples` surface have been removed from the Hub page set. Static contracts now validate Material/Taffy coverage through real bridge exports, Hub wrappers, Projects, Editor, Builds, Settings, and shared data-display surfaces, globally block raw Slint `Text`, direct font bindings, character-icon literals, direct business-layer `TouchArea` usage, and percent sizing in Hub UI files, and guard against accidentally reintroducing those sample-only components.

Project filtering and sorting are part of the Rust snapshot rather than local Slint-only state. The filter selector cycles through all projects, existing paths, and missing paths. The sort selector cycles between newest-first and name sorting. Dashboard cards, dashboard compact rows, project-browser rows, and project-detail data are rebuilt from the same filtered recent-project source plus Hub-owned metadata. Grid mode remains the dashboard card flow; list mode navigates to the loose project browser rather than replacing only the upper dashboard region.

Clicking a project card or recent row now selects that project as the dashboard target and projects a selected state back into the card, compact-list row, and Recent Projects table row through Material `StateLayerArea`. Project-browser row bodies use Material `StateLayerArea` for selection, while the token-derived trailing detail slot owns a separate Material `StateLayerArea` with a centered SVG icon so the Project Detail transition follows the visible trailing affordance instead of being swallowed by row selection. Explicit row/card action buttons and the manual open flow still launch `zircon_editor --project <path>` through the same validation and recent-sync path. Empty states are rendered inside the dashboard instead of falling back to plain text summaries. The current capture run verified dashboard, new-project, project-browser, and project-detail at 1600x1024, 1280x900, and 1024x768 through `target/hub-visual-check/projects-selected-context-1600x1024-0521-v7`, `target/hub-visual-check/projects-selected-context-1280x900-0521-v3`, and `target/hub-visual-check/projects-selected-context-1024x768-0521-v3`; 1280x900 keeps the title bar compact before status pills overlap, while 1024x768 hides the sidebar status card and lets Project Detail scroll so project-info rows are not clipped. The browser detail captures were run against the current built Hub binary through the dashboard secondary-page path plus a centered trailing detail click, and verify the real Hub window behavior: the Project Browser row body still selects only the row, and the trailing detail control opens Project Detail with the right-side Actions panel visible when space allows.

Quick Actions are identified by stable ids and are projected from the current `HubSnapshot`, not from a static action list. `build-project`, `package-project`, and `install-device` use the same selected-project/newest-recent target rule; their rows name the selected or fallback project, and they are disabled when there is no project target. Before those actions run, the runtime activates the project's bound Source Engine when Hub metadata has one, so build output and package paths follow the project context instead of whatever engine happened to be selected earlier. `open-editor` opens the selected recent project when one is selected, falls back to the newest recent project when the dashboard has no explicit selection, and launches the staged editor without a project only when no local project target exists. Status details include the project name plus output path. The package action copies the project into `packages/<project>-<timestamp>/project`, skips heavy transient folders such as `.git` and `target`, and writes `zircon-package.toml` metadata beside the staged project. `install-device` now uses the same package step and copies that package into the configured local device install folder. This is a real local deployment path, but it is still a folder-backed device target rather than a USB, ADB, console SDK, or remote runtime deployer.

## Builds Page

The Builds page is the Hub-level source build dashboard. It uses the same `SourceEngineData` projection as the Editor page plus `ProjectDetailData` from the selected-project workflow context. Build controls run the configured editor/runtime source build, open the staged output folder, launch the selected project in the staged editor, package the selected project, or install the selected project package to the configured local device folder. Open Editor, Package, and Install rows stay disabled until an actual selected project is available, so Builds remains explicitly project-scoped. Package and Install controls route through the existing Hub quick-action dispatcher so they share the same runtime path as Dashboard Quick Actions. The compact build pipeline shows source validation, editor compilation, runtime staging, and a package-project row scoped to the selected project path instead of a placeholder export slot. Builds now also receives the active source engine's `SourceBuildHistoryRowData` model, so recent build records are visible from the build dashboard instead of only from the Editor/source-engine page.

The Builds page now composes the shared Hub design-system components instead of local card and row implementations. Source/output summaries use the shared Material `ListTile`-backed `InfoRow`, build controls use `ActionRow`, and Editor plus Builds share the same `BuildHistoryRow` for recent source-build records. Pipeline status uses right-aligned badges, the current-task status headline now uses Material `MaterialText`, and all visible action marks come from bundled SVG assets rather than text glyphs. Its overview/control and pipeline/status-history regions sit inside `WorkspacePanelSection` groups under a `PageScrollSurface`, so narrow windows stack the panels vertically while preserving the same internal action and row components. Each Builds panel is wrapped in `ResponsiveSlot` with tokenized basis/grow/min-width values; overview slots use the larger overview minimum with grow weight 2 and side slots use the side-panel minimum with grow weight 1, so Taffy distributes the main and side panels instead of the page computing `overview-width` and `side-panel-width`. The Builds summary row height is sized for the five visible Build/Open/Launch/Package/Install action rows instead of relying on the Editor summary-row height. The overview panel leaves build launching to the Build Controls panel so the source-build title and status badge do not squeeze each other.

## Assets Page

The Assets page is backed by a lightweight local catalog. On Hub startup, after project open/create, when a project is selected, and whenever the active source checkout changes, Rust refreshes `HubSnapshot.assets` from the selected project, other recent project roots, and engine roots. The selected project scans `Assets` and `assets` first, is labeled as the selected project scope, and is sorted ahead of other recent-project and Source Engine asset groups. Other recent projects keep a recent-project scope, while engine roots scan `zircon_editor/assets` and `zircon_runtime/assets` from the active source checkout, current directory, and compile-time workspace root.

The scanner classifies common extensions into image, model, audio, shader, data, scene, UI, or file kinds, skips transient `.git` and `target` directories, sorts entries by selected-project/project/engine priority plus source/kind/name/path, and caps the list to keep the Hub responsive. Slint receives compact `AssetData` rows with name, kind, localized source/scope label, size, path, and accent index. When no assets are found, the empty-state detail explains whether the scan included a selected project or only recent project/source-engine roots. This page is a catalog browser only; import pipelines, thumbnails, dependency graphs, and asset editing remain owned by the Editor/runtime asset systems.

The Assets page now uses `PageScrollSurface` plus `CatalogListPanel` around shared `InfoRow` rows. Asset rows use the bundled Assets SVG as the leading mark and put the asset kind in the standardized right-side badge slot, so the page follows the same Data display and Surfaces component categories as Projects, Team, and Cloud without carrying its own hand-built title, empty-state, or content-height formula. The shared `CatalogPage` panel height comes from `PageScrollSurface.content-height`, so Assets shares the same viewport budget as Plugins and Learn. The shared catalog empty-state title now uses Material `MaterialText`, with `MutedText` retaining the detail copy.

## Plugins Page

The Plugins page is backed by real local source data. On Hub startup, when a project is selected, and whenever the active source checkout changes, Rust scans selected-project plugin manifests under `plugin.toml`, `Plugins/**/plugin.toml`, or `plugins/**/plugin.toml` with Project scope, then scans the first available engine `zircon_plugins/**/plugin.toml` tree with Engine scope. It parses each manifest, sorts Project entries ahead of Engine entries, and projects display name, localized scope label, category, maturity, default packaging modes, module count, description, and package path into `PluginData` rows. Project scope is shown as the selected project; engine scope is shown as Source Engine. If the configured source checkout is missing or does not contain a plugin tree, discovery falls back to the current working directory and the compile-time workspace root so development runs from this repository still show the local catalog.

This page is a catalog browser, not yet a plugin build or enable/disable manager. Plugin build execution remains in `tools/zircon_build.py --targets plugins`, and runtime load-manifest generation remains owned by that build script.

The Plugins page now uses `PageScrollSurface` plus `CatalogListPanel` instead of a locally hand-built catalog shell. Each plugin row uses the bundled Plugins SVG leading mark, projects category into the shared badge slot, and leaves selected-project/Source Engine scope plus plugin packaging/module/maturity details in the row meta line. Its title and empty-state detail switch when a project is selected so the page explains whether discovery checked selected-project manifests plus engine plugins or only the Source Engine/repository fallback roots. Plugin list height is inherited from the shared `CatalogPage` content-height path rather than a page-local window-height subtraction.

## Cloud Page

The Cloud page is a local readiness surface, not an account or network integration. Rust projects `CloudSummaryData` from existing Hub settings: account status is always local/offline, and build output plus device install rows use the configured directories. Without a selected project, package status still counts local package directories under `<build-output>/packages`. With a selected project, Cloud only counts package directories whose `zircon-package.toml` manifest has a `source_project` matching the selected project path; unrelated project packages no longer satisfy the selected-project package state. When no matching package exists, the status explicitly says the selected project has no local package yet instead of implying a cloud/account problem.

The page title and subtitle also switch to selected-project local package/install/output wording so the status panel reads as project-scoped instead of a global cloud account view.

The service list is intentionally a set of reserved slots. It shows account sync, remote build, and package upload as non-connected local/offline capabilities so the navigation entry has a real operational surface without implying that hosted services, authentication, licenses, uploads, or remote build workers already exist. Future cloud work can replace these rows with real service health once those backends exist.

The Cloud page uses `PageScrollSurface` for page overflow, `WorkspacePanelSection` for the four readiness metrics, and `PanelListViewport` for the reserved service rows. Cloud opts out of compact stacking on that section so the metric cards keep wrapping as a compact grid, uses `ResponsiveSlot` only as the flex item wrapper for each metric card, and uses the section's `compact-rows` input to reserve one four-column row, two metric rows, or four single-column rows. The row height now comes from `HubTokens.workspace-row-cloud-metrics`; metric card sizing uses token basis/min-width/grow values and lets Taffy distribute the remaining row width instead of dividing `content-width` by hand; the service list uses `PageScrollSurface.content-height` as its viewport budget instead of recomputing window height, page padding, and bottom status in the page; metric status typography routes through Material `MaterialText` instead of page-local raw `Text` font bindings.

## Learn Page

The Learn page is backed by local repository documentation instead of static placeholder cards. On Hub startup and whenever the active source checkout changes, Rust scans `docs/**/*.md` from the configured source checkout, current working directory, and compile-time workspace root. Discovery deduplicates documentation roots, skips transient `.git` and `target` folders, sorts by category/title/path, and caps the result set to keep the Hub responsive.

Each Learn row extracts the first markdown H1 as the title, the first readable paragraph as the summary, the first folder below `docs` as the category, and the source path as the open target. The page is a local documentation browser only; it does not yet manage online tutorials, sample downloads, or remote learning feeds.

The Learn page now uses `PageScrollSurface` plus `CatalogListPanel` as a clickable documentation list. The category is pinned in the badge slot, the right arrow uses the shared SVG action affordance, and clicking the row dispatches the existing local-resource open callback. Its list panel shares the same `CatalogPage` content-height budget used by Assets and Plugins.

## Team Page

The Team page is backed by local Git data instead of an account service. On Hub startup, when a project is selected, and whenever source-engine settings change, Rust looks for a Git workspace from the selected project path first, then the active source checkout, current working directory, and compile-time workspace root. The first valid repository root becomes the Team overview source, and the projected status labels it as either the selected project repository or the Source Engine repository. The page title and empty-contributor detail also switch when a project is selected, so an empty Team page explains whether Hub checked the selected project first or only the active Source Engine/local fallback repositories.

Discovery reads `git config --get user.name`, `git config --get user.email`, and recent author pairs from `git log --all --format=%an%x1f%ae -n 200`. Authors are counted by name/email, sorted by commit count and stable identity text, capped for the Hub surface, and projected as `TeamMemberData` rows. Missing Git, missing config values, or a non-repository workspace produce a local empty state rather than failing Hub startup.

This page is intentionally local-only. It does not manage repository permissions, invite members, connect hosting providers, sync cloud teams, or display remote review state. Those behaviors belong to a future collaboration service or Cloud page instead of the current source-local Hub shell. Team summary primary text and empty-member titles use Material `MaterialText`, while muted labels/details continue through the shared `MutedText` wrapper.

The Team page uses `PageScrollSurface` and `WorkspacePanelSection` for its local identity and repository summary cards, then keeps member rows in `PanelListViewport`. The summary row height comes from `HubTokens.workspace-row-team-summary`, so compact windows stack the local identity and repository cards through the shared Taffy primitive instead of repeating a page-local breakpoint formula. Member-list height uses the same `PageScrollSurface.content-height` viewport budget as Cloud, so the page no longer repeats window-height minus padding and bottom-status math.

## Config And Recent Sync

Hub config is TOML under the user config directory, for example `%LOCALAPPDATA%\ZirconHub\config.toml` on Windows. It stores Hub settings, recent projects, Hub-owned project metadata, source-engine records, the active source-engine id, and the last known Hub window placement. Project metadata is keyed by normalized project path and currently stores `pinned`, `engine_id`, and `last_selected_template`. Editor recent-project sync reads and writes the existing JSON config shape at `editor.startup.session`, preserving unrelated keys in `%LOCALAPPDATA%\ZirconEngine\config.json` or the path selected by `ZIRCON_CONFIG_PATH`.

Recent entries merge by normalized path, keep the entry with the newest `last_opened_unix_ms`, sort newest first, and truncate to eight entries. Hub startup writes the merged recent list back to Hub TOML and the Editor JSON session while preserving the Editor `last_project_path`; Hub only overrides `last_project_path` after an explicit open or create request successfully spawns `zircon_editor` for that project. Hub-owned pin, bound-engine, and template metadata are never written into the Editor recent JSON.

The Settings page edits the stored toolchain paths, default project/source/output directories, default local device install directory, build profile, build job count, and language preference. Toolchain and path values remain text fields because they can point at arbitrary local commands or directories. Build profile and language use Hub `SegmentButton` controls backed by the imported Material `SegmentedButton`, with the same stored values: `debug` or `release` for build profile and `English` or `Chinese` for language. Hub uses the configured Python executable as the build-script runner and forwards the configured Cargo executable through `tools/zircon_build.py --cargo <cargo-path>`, so source installs can use a non-default Cargo shim without editing the environment.

Settings also projects a Configuration Health panel from Rust view-model data. Bare command names such as `python`, `cargo`, or `rustup` are marked as environment-resolved commands, path-like executable values are checked for existence, and default project/source/output/device directories are classified as ready or create-on-use paths, with source checkout still reported as missing when the checkout is absent. This is only local configuration readability for now; it does not run toolchain subprocess probes during normal Hub rendering.

The Settings page now uses the same component-first layout contract as the other Hub pages. `PageScrollSurface` owns page overflow and the standard content margin, `WorkspacePanelSection` groups Toolchain with Build Defaults and Default Paths with Configuration Health, and `PanelListViewport` owns the health-row scrolling. The old fixed two-column Settings shell and panel-local hand-built `ScrollView` content sizing were removed. Text fields use `HubTextField`, which wraps the local Slint Material template `TextField`; segmented controls use the template `SegmentedButton` through the Hub wrapper; Browse callbacks and Save Settings keep the same Rust-facing behavior. The 960x640 Settings captures verify the Material-backed text fields and segmented controls render without overlapping the Settings panels or bottom status bar.

Settings workspace section children now use `ResponsiveSlot` for Taffy basis/grow/shrink, matching the Editor page and Projects secondary pages' no direct page-local `flex-*` contract. The Default Paths and Configuration Health detail row now uses tokenized main/side basis values with grow weights instead of page-local `detail-primary-width`/`detail-side-width` formulas; the path list rows keep only a token preferred width and stretch through the panel `PanelListViewport` instead of deriving row width from `DefaultPathsPanel.root.width`.

Default project, source checkout, staged output, and local device install directories now expose Browse buttons in Settings. The Editor/source-engine page also exposes Browse buttons for source checkout and staged output. Selecting a folder updates the in-memory Hub settings and the visible source-engine projection; settings are still persisted through the normal Save Settings action.

The language setting now drives a real English/Chinese UI text bundle instead of only being stored in config. Rust projects localized navigation labels, page titles/subtitles, project filter/sort labels, quick actions, source-engine status, build-history status, relative time labels, Settings health rows, and header status pills. Slint receives the remaining static surface text through `UiTextData`, which currently covers the shell, local user label, Projects, Editor, Assets, Plugins, Cloud, Team, Learn, Builds, and Settings. This is still a compact two-language string table rather than a full resource-file i18n system; future work can move the same keys into external resource catalogs if the language set grows.

The Editor page replaces the old Installs page. It shows the active source engine record, source checkout path, staged output directory, last build label, build profile, build jobs, and actions for saving source settings, building, opening output, and launching the editor. Hub startup registers a source-engine record from saved source/output settings when those settings exist, so returning users see their source install immediately even before pressing Save again.

The Editor page now follows the same component-first layout contract as Projects and Builds. The active-engine overview uses `HubPanel`, `PanelHeader`, Material `ListTile`-backed `InfoRow`, and `Badge`; action rows use the shared `ActionRow`; source-engine rows use imported Material `ListTile` with Hub trailing badge/remove affordances; build-history lists use `PanelListViewport` so long lists scroll inside their panels instead of pushing the status bar or adjacent form region. Its active-engine name, source checkout, and staged output inputs use `HubTextField` rather than `LineEdit`, keeping Editor on the same imported Material `TextField` path as Settings and Projects New Project. Its overview/actions and settings/list regions are grouped with `WorkspacePanelSection` under `PageScrollSurface`, and every section child routes Taffy basis/grow/shrink through `ResponsiveSlot` instead of direct page-local `flex-*` properties. The overview/action and settings/list rows now use overview/side minimum tokens as `ResponsiveSlot` basis values with grow weights instead of page-local `overview-width`, `actions-width`, or `config-width` formulas, while narrow windows still stack the same panels without compressing the source path fields and build-history surfaces.

The Editor page also renders the registered source-engine list from `SourceEngineRowData`. Saving a source checkout now appends or updates a stable source-engine record derived from the source directory instead of replacing the entire engine list. Existing display names and build history are preserved when the same source checkout is saved again, so saving or building does not discard user-managed engine metadata. Selecting an engine marks it active, updates the visible source/output settings, persists the active engine id, and makes subsequent build/launch paths use that engine's staged output.

The active engine can be renamed from the Editor page, and any registered engine can be removed from the list; removing the active engine automatically falls back to the next available engine. Each source engine also stores a compact build history through `SourceBuildRecord`. Successful builds update `last_build_unix_ms`; successful and failed build attempts are both inserted into the per-engine history and truncated to the newest eight records. The Editor page shows the newest active-engine build-history rows with status, profile/jobs, output path, relative finish time, and detail. Version tagging beyond the package version label and expanded build-log inspection remain reserved for later engine-management work.

## Editor Launch Contract

Hub launches `zircon_editor` as an independent child process. Existing projects use `--project <path>`. New projects use `--create-project --project-name <name> --location <dir> --template renderable-empty`.

Before opening or creating a project, Hub checks whether a preferred editor executable is available. A sibling staged `zircon_editor(.exe)` beside the running Hub takes priority; otherwise Hub uses the configured staged output path. If neither exists, Hub runs the source-install build command first so project launch can use the freshly staged editor/runtime payload.

`zircon_app` parses these GUI startup arguments before the headless operation parser. When one is present, `zircon_editor` receives an `EditorGuiStartupRequest` and opens or creates that project directly through `EditorManager`; it does not call the normal last-project restore path. Empty editor args still use the existing fallback behavior.

## Staged Builds

`tools/zircon_build.py` now accepts the `hub` target. A staged payload can include `zircon_hub.exe`, `zircon_editor.exe`, and the runtime library under one `ZirconEngine` directory. The Hub's own build action still calls the tool with `--targets editor,runtime` because Hub source installs need a staged editor/runtime payload to launch.
