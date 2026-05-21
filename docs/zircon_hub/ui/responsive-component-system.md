---
related_code:
  - zircon_hub/Cargo.toml
  - zircon_hub/build.rs
  - zircon_hub/ui/components.slint
  - zircon_hub/ui/tokens.slint
  - zircon_hub/ui/material_bridge.slint
  - zircon_hub/ui/theme.slint
  - zircon_hub/ui/layout.slint
  - zircon_hub/ui/surfaces.slint
  - zircon_hub/ui/inputs.slint
  - zircon_hub/ui/shell.slint
  - zircon_hub/ui/navigation.slint
  - zircon_hub/ui/data_display.slint
  - zircon_hub/ui/overlays.slint
  - zircon_hub/ui/shared.slint
  - zircon_hub/ui/app.slint
  - zircon_hub/ui/projects.slint
  - zircon_hub/ui/project_dashboard.slint
  - zircon_hub/ui/project_pages.slint
  - zircon_hub/ui/project_page_components.slint
  - zircon_hub/ui/editor.slint
  - zircon_hub/ui/builds.slint
  - zircon_hub/ui/assets.slint
  - zircon_hub/ui/cloud.slint
  - zircon_hub/ui/learn.slint
  - zircon_hub/ui/plugins.slint
  - zircon_hub/ui/team.slint
  - zircon_hub/ui/settings.slint
  - zircon_hub/src/app/runtime.rs
  - zircon_hub/src/app/runtime/project_workspace.rs
  - zircon_hub/src/app/view_model/quick_actions.rs
  - zircon_hub/tests/ui_contract.rs
  - dev/material-rust-template/README.md
  - dev/material-rust-template/material-1.0/material.slint
implementation_files:
  - zircon_hub/Cargo.toml
  - zircon_hub/build.rs
  - zircon_hub/ui/components.slint
  - zircon_hub/ui/tokens.slint
  - zircon_hub/ui/material_bridge.slint
  - zircon_hub/ui/theme.slint
  - zircon_hub/ui/layout.slint
  - zircon_hub/ui/surfaces.slint
  - zircon_hub/ui/inputs.slint
  - zircon_hub/ui/shell.slint
  - zircon_hub/ui/navigation.slint
  - zircon_hub/ui/data_display.slint
  - zircon_hub/ui/overlays.slint
  - zircon_hub/ui/shared.slint
  - zircon_hub/ui/project_dashboard.slint
  - zircon_hub/ui/project_pages.slint
  - zircon_hub/ui/project_page_components.slint
  - zircon_hub/ui/editor.slint
  - zircon_hub/ui/builds.slint
  - zircon_hub/ui/settings.slint
  - zircon_hub/src/app/runtime.rs
  - zircon_hub/src/app/runtime/project_workspace.rs
  - zircon_hub/src/app/view_model/quick_actions.rs
  - zircon_hub/tests/ui_contract.rs
plan_sources:
  - user: 2026-05-19 implement Zircon Hub responsive componentization plan
  - user: 2026-05-19 directly introduce the Slint Material UI template instead of designing a custom clone
  - https://mui.com/material-ui/all-components/
  - dev/material-ui/packages/mui-system
  - dev/material-rust-template/README.md
  - dev/material-rust-template/material-1.0/material.slint
tests:
  - cargo fmt -p zircon_hub --check
  - cargo check -p zircon_hub --locked --offline --jobs 1 --target-dir target\hub-material-check --color never
  - cargo check -p zircon_hub --locked --offline --jobs 1 --target-dir target\hub-material-check --message-format short --color never
  - cargo check -p zircon_hub --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-hub-info-row-0520 --message-format short --color never
  - cargo test -p zircon_hub --test ui_contract --locked --offline --jobs 1 --target-dir target\hub-material-check --message-format short --color never
  - CARGO_MANIFEST_DIR=E:\Git\ZirconEngine\zircon_hub rustc --edition=2021 --test zircon_hub\tests\ui_contract.rs -o target\hub-material-check\ui_contract_windowbutton_direct.exe; target\hub-material-check\ui_contract_windowbutton_direct.exe --nocapture
  - cargo test -p zircon_hub --locked --offline --jobs 1 --target-dir target\hub-material-check --color never
  - cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1 --target-dir target\hub-material-check --color never
  - CARGO_PROFILE_DEV_DEBUG=0 cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-hub-ui-contract-nodebug-0520 --color never
  - CARGO_PROFILE_DEV_DEBUG=0 cargo check -p zircon_hub --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-hub-ui-contract-nodebug-0520 --message-format short --color never
  - CARGO_PROFILE_DEV_DEBUG=0 cargo test -p zircon_hub --test ui_contract --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-hub-ui-contract-nodebug-0520 --message-format short --color never
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
  - OUT_DIR=target\hub-ui-check-project-browser-detail-button CARGO_MANIFEST_DIR=E:\Git\ZirconEngine\zircon_hub CARGO_PKG_NAME=zircon_hub target\hub-project-scroll-check\debug\build\zircon_hub-7fc227b3e5306fb7\build-script-build.exe
  - target/hub-ui-check-project-browser-detail-button/app.rs
  - zircon_hub/tests/ui_contract.rs
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-window.ps1
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-project-pages.ps1
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
  - target/hub-visual-check/material-card-final-project-pages-v8/hub-projects-dashboard.png
  - target/hub-visual-check/material-card-final-project-pages-v8/hub-projects-new-project.png
  - target/hub-visual-check/material-card-final-project-pages-v8/hub-projects-browser.png
  - target/hub-visual-check/material-card-final-project-pages-v8/hub-projects-detail.png
  - target/hub-visual-check/material-card-final-project-pages-v8-1366b/hub-projects-detail.png
  - target/hub-visual-check/material-card-final-responsive-1100x760.png
  - target/hub-visual-check/material-card-final-responsive-960x640.png
  - target/hub-visual-check/material-buttons-dashboard-fixed-960x640.png
  - target/hub-visual-check/material-textfield-settings-960x640.png
  - target/hub-visual-check/material-textfield-editor-960x640.png
  - target/hub-visual-check/material-textfield-editor-fields-960x640.png
  - target/hub-visual-check/material-textfield-project-pages/hub-projects-new-project.png
  - target/hub-visual-check/material-toolbar-popup-960x640.png
  - target/hub-visual-check/material-segment-settings-960x640.png
  - target/hub-visual-check/material-dropdown-header-fixed-960x640.png
  - target/hub-visual-check/material-dropdown-popup-960x640.png
  - target/hub-visual-check/material-searchbox-1600x1024-v3/hub-projects-dashboard.png
  - target/hub-visual-check/material-searchbox-1600x1024-v3/hub-projects-new-project.png
  - target/hub-visual-check/material-searchbox-1600x1024-v3/hub-projects-browser.png
  - target/hub-visual-check/material-searchbox-1600x1024-v3/hub-projects-detail.png
  - target/hub-visual-check/material-searchbox-1280x900-v5/hub-projects-dashboard.png
  - target/hub-visual-check/material-searchbox-1280x900-v5/hub-projects-browser.png
  - target/hub-visual-check/material-searchbox-1280x900-v5/hub-projects-detail.png
  - target/hub-visual-check/material-searchbox-1024x768-v2/hub-projects-new-project.png
  - target/hub-visual-check/material-searchbox-1024x768-v2/hub-projects-browser.png
  - target/hub-visual-check/material-searchbox-1024x768-v2/hub-projects-detail.png
  - target/hub-visual-check/zircon-theme-material-listtile-1280x900-v3/hub-projects-dashboard.png
  - target/hub-visual-check/zircon-theme-material-listtile-1280x900-v3/hub-projects-new-project.png
  - target/hub-visual-check/zircon-theme-material-listtile-1280x900-v3/hub-projects-browser.png
  - target/hub-visual-check/zircon-theme-material-listtile-1280x900-v3/hub-projects-detail.png
  - target/hub-visual-check/zircon-theme-material-listtile-1024x768-v1/hub-projects-dashboard.png
  - target/hub-visual-check/zircon-theme-material-listtile-1024x768-v1/hub-projects-new-project.png
  - target/hub-visual-check/zircon-theme-material-listtile-1024x768-v1/hub-projects-browser.png
  - target/hub-visual-check/zircon-theme-material-listtile-1024x768-v1/hub-projects-detail.png
  - target/hub-visual-check/material-searchbar-1024x768-v2/hub-projects-dashboard.png
  - target/hub-visual-check/material-searchbar-1024x768-v2/hub-projects-new-project.png
  - target/hub-visual-check/material-searchbar-1024x768-v2/hub-projects-browser.png
  - target/hub-visual-check/material-searchbar-1024x768-v2/hub-projects-detail.png
  - target/hub-visual-check/material-navigationrail-baseline-nosize-main-984x640.png
  - target/hub-visual-check/material-navigationrail-collapsed-nosize-1100x820.png
  - target/hub-visual-check/material-navbutton-listtile-expanded-nosize-1100x820.png
  - target/hub-visual-check/material-statuspill-actionchip-1600x1024.png
  - target/hub-visual-check/material-actionrow-1600x1024-v1/hub-projects-dashboard.png
  - target/hub-visual-check/material-actionrow-1600x1024-v1/hub-projects-browser.png
  - target/hub-visual-check/material-actionrow-1600x1024-v1/hub-projects-detail.png
  - target/hub-visual-check/material-actionrow-1024x768-v1/hub-projects-dashboard.png
  - target/hub-visual-check/material-actionrow-1024x768-v1/hub-projects-browser.png
  - target/hub-visual-check/material-actionrow-1024x768-v1/hub-projects-detail.png
  - target/hub-visual-check/material-header-engine-listtile-984x640-v1.png
  - target/hub-visual-check/material-inforow-editor-984x640-v1.png
  - target/hub-visual-check/material-inforow-builds-984x640-v2.png
  - target/hub-visual-check/material-inforow-settings-1280x900-v3.png
  - target/hub-visual-check/material-windowbutton-984x640-v1.png
  - target/hub-visual-check/material-divider-1600x1024.png
  - target/hub-visual-check/material-datadisplay-scrollview-1100x820.png
  - target/hub-visual-check/material-project-scrollview-1100x820/hub-projects-dashboard.png
  - target/hub-visual-check/material-project-scrollview-1100x820/hub-projects-new-project.png
  - target/hub-visual-check/material-project-scrollview-1100x820/hub-projects-browser.png
  - target/hub-visual-check/material-project-scrollview-1100x820/hub-projects-detail.png
  - target/hub-visual-check/material-badge-materialtext-1100x820-software.png
  - target/hub-visual-check/material-badge-materialtext-pages-1100x820/hub-projects-dashboard.png
  - target/hub-visual-check/material-badge-materialtext-pages-1100x820/hub-projects-new-project.png
  - target/hub-visual-check/material-badge-materialtext-pages-1100x820/hub-projects-browser.png
  - target/hub-visual-check/material-badge-materialtext-pages-1100x820/hub-projects-detail.png
  - target/hub-visual-check/shared-materialtext-typography-1100x820.png
  - target/hub-visual-check/surface-title-materialtext-1100x820.png
  - target/hub-visual-check/data-table-materialtext-1100x820.png
  - target/hub-visual-check/catalog-list-materialtext-1100x820-assets.png
  - target/hub-visual-check/catalog-list-materialtext-1100x820-plugins.png
  - target/hub-visual-check/shell-page-header-materialtext-1100x820-assets.png
  - target/hub-visual-check/shell-chrome-materialtext-1100x820-assets.png
  - target/hub-visual-check/dashboard-card-materialtext-1100x820.png
  - target/hub-visual-check/project-workflow-materialtext-1100x820/hub-projects-{dashboard,new-project,browser,detail}.png
  - target/hub-visual-check/cloud-team-materialtext-1100x820/{cloud,team}-materialtext.png
  - target/hub-visual-check/builds-materialtext-1100x820/builds-materialtext.png
  - target/hub-visual-check/builds-materialtext-1600x1024/builds-materialtext.png
  - target/hub-material-check/ui_contract_project_browser_state_direct.exe --nocapture
  - target/hub-material-check/ui_contract_project_selector_state_direct.exe --nocapture
  - target/hub-material-check/ui_contract_sidebar_collapse_state_direct.exe --nocapture
  - target/hub-material-check/ui_contract_project_browser_select_direct.exe --nocapture
  - target/hub-material-check/ui_contract_global_typography_direct.exe --nocapture
  - target/hub-material-check/ui_contract_character_icons_direct.exe --nocapture
  - target/hub-material-check/ui_contract_toucharea_direct.exe --nocapture
  - target/hub-material-check/ui_contract_percentage_sizing_direct.exe --nocapture
  - target/hub-material-check/ui_contract_workspace_slot_direct.exe --nocapture
  - target/hub-ui-check-project-browser-state/app.rs
  - target/hub-ui-check-project-selector-state/app.rs
  - target/hub-ui-check-sidebar-collapse-state/app.rs
  - target/hub-ui-check-project-browser-select/app.rs
  - target/hub-ui-layout-gen/app.rs
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-project-pages.ps1
doc_type: module-detail
---

# Zircon Hub Responsive Component System

Zircon Hub keeps Slint as the rendering and interaction layer. It does not embed HTML, CSS, WebView, or a browser layout engine. The Hub UI now exposes a Slint-native component layer that borrows the useful parts of Material UI's component taxonomy: tokens first, layout primitives second, then surfaces, inputs, navigation, data display, and overlays.

The Hub now imports the local Slint Material UI template directly. `zircon_hub/build.rs` registers `dev/material-rust-template/material-1.0/material.slint` as the Slint library path `@material`, and `material_bridge.slint` re-exports the template's layout, card, button, input, navigation, list, progress, tab, tooltip, and style primitives for Hub pages that need the real template components. `theme.slint` installs a Zircon teal/dark `MaterialPalette.schemes` override at UI initialization time, so Material-owned controls keep the template behavior without inheriting the template's default blue palette. This is a direct template dependency, not a hand-written Material clone and not an HTML/CSS engine.

Hub surfaces keep the Zircon dark tool UI contract, but they are now backed by real Material template cards where the template provides the matching primitive. `HubPanel` selects `ElevatedCard` for elevated variants and `OutlinedCard` for standard panel variants, then overlays Hub-owned border, tint, hover, and children content so existing page APIs continue to work while the base surface comes from the imported template. Its panel-level hover `TouchArea` is placed behind `@children` and only enabled for interactive variants; default panels must not cover nested list rows, buttons, inputs, or detail affordances.

Shared Hub button APIs also delegate to the imported Material template instead of reimplementing pointer behavior locally. `PillButton` now wraps `FilledButton` for primary actions and `OutlineButton` for secondary actions, while Hub `IconButton` wraps `FilledIconButton` for active icon actions and `OutlineIconButton` for inactive icon actions. `WindowButton` wraps the template `IconButton` with the compact inline icon metric and the template error color path for the close action, so title-bar minimize/maximize/close controls use the same centered icon layout as other Material icon controls instead of a custom `TouchArea`. `PillButton` owns a stable atom-level preferred width and clip boundary so Material text buttons cannot overpaint adjacent responsive header actions at compact window sizes; Hub `IconButton` clips the Material icon button to the requested atom size for dense rows. The Hub-facing properties remain stable (`text`, `icon-image`, `has-icon-image`, `primary`, `active`, `enabled`, and `clicked`), so existing pages keep semantic button usage while hover, disabled, ripple, accessibility, and icon button behavior come from the direct Material template import.

Hub text input fields now follow the same compatibility-wrapper pattern. `HubTextField` lives in `inputs.slint`, exposes Hub-facing `label`, `placeholder`, `supporting-text`, `text`, `enabled`, and edit/accept callbacks, and renders the imported Material template `TextField` internally. Settings uses this wrapper for toolchain paths, build jobs, and default path rows; Editor uses it for active engine name, source checkout, and staged output fields; Projects New Project uses it for project name and location. Hub editable form pages should not import `std-widgets` `LineEdit` directly, while `HubTokens.input-field` owns the Material field atom height and `HubTokens.input-width` gives Material text fields a stable preferred width before row stretch is available. Toolbar search remains exposed as the Hub-facing `SearchBox` API, but it now wraps the imported Material `SearchBar` with the search SVG as the leading icon and a visible placeholder prompt, so Hub no longer needs a custom painted `TextInput` or a `TextField` label workaround for toolbar search.

Hub segmented controls also use the imported template now. The Hub-facing `SegmentButton` wrapper keeps the existing `text`, `active`, and `clicked` API for Settings and Hub tool surfaces, but its internal state is a one-item Material `SegmentedButton` rather than a hand-painted rectangle plus `TouchArea`. The wrapper uses the Material component's 40px minimum control height through `HubTokens.control-md`, so Settings build-profile and language rows do not clip the template's state layer or selected check affordance.

Toolbar dropdowns now follow the same rule. `ToolbarSelect` keeps the Hub-facing label, icon, legacy option id, and callback contract, but its trigger is the imported Material `OutlineButton` and its popup list is the imported Material `PopupMenu` fed by `MenuItem` data. Projects dashboard and browser pages pass both Material `menu-items` and the existing id-bearing `options`, so the visual/menu layer comes from the local template while filter and sort actions still dispatch the stable Hub ids.

The generic Hub `DropDownButton` wrapper is also backed by Material buttons. It now renders an imported `OutlineButton` for inactive dropdown triggers and `TonalButton` for active triggers, using the existing Hub `text`, `icon-image`, `has-icon-image`, `active`, and `clicked` properties. The top Source Engine selector still owns its custom `PopupWindow` engine list because that list is not a simple `MenuItem` projection, but the trigger itself no longer paints local borders/backgrounds or owns a separate pointer `TouchArea`.

Collapsed Hub navigation now delegates to the imported Material template `NavigationRail`. Rust still projects the stable Hub `NavItemData` model for page ids, titles, and active state, and `view_model.rs` derives a parallel `NavigationItem` model from the same data for the Material rail. `NavRail` uses that Material model only in collapsed mode and dispatches clicks back through the original Hub page ids by index, so page routing stays unchanged. Expanded navigation remains the Hub-owned left-aligned `NavButton` row list because the template `NavigationRail` is an 80px icon rail and the template `NavigationDrawer` has a wider drawer contract than the Hub's 200px sidebar, but each expanded `NavButton` row now delegates its icon, title, and row interaction body to the imported Material `ListTile`. The sidebar collapse control delegates hover, press, and click state to Material `StateLayerArea`, leaving the shell's direct `TouchArea` usage for the frameless title-bar drag region only. `HubTokens.nav-width-collapsed-*` therefore uses the Material `size_80` rail width instead of the old 64/72px custom icon strip.

`components.slint` is intentionally only a public entrypoint. Existing pages can continue importing from it, but implementation belongs in the category files below:

- `tokens.slint` owns spacing, breakpoints, page padding, panel gaps, toolbar gaps, control heights, icon sizes, list/table row heights, workspace panel row heights, borders, panel minimums, and shared dark Hub colors.
- `material_bridge.slint` owns no Hub styling. It directly re-exports the local Material template components that Hub is allowed to consume through `components.slint`, while leaving conflicting public names such as Hub `Badge` under Hub ownership; badge labels can still use bridged Material typography primitives such as `MaterialText`.
- `layout.slint` owns CSS-like composition primitives: `Stack`, `Row`, `Column`, `Flow`, `Page`, `PanelGrid`, `ResponsiveSlot`, `Fill`, `Divider`, `PageScrollSurface`, `ResponsivePanelFlow`, and `WorkspacePanelSection`. `Page` uses the Material template `ScrollView`, exposes padded `content-width`, usable `viewport-height`, and padded `content-height`, and delegates `Divider` to Material `HorizontalDivider`/`VerticalDivider`; content width is still constrained by the Hub shell so the page body only sees the viewport left after the navigation rail.
- `theme.slint` owns the Hub-level Material palette override. It is installed by `HubWindow` through a zero-size `ZirconMaterialTheme` component so every imported Material control uses the same Zircon teal/dark scheme.
- `shared.slint` owns shared DTO structs plus compatibility primitives such as `PillButton`, Hub `IconButton`, header `StatusPill`, the expanded-navigation `NavButton`, `FieldLabel`, and `MutedText`. Those wrappers preserve the Hub API and route their internals through Material `FilledButton`, `OutlineButton`, `FilledIconButton`, `OutlineIconButton`, `ActionChip`, `ListTile`, and `MaterialText` where the local template owns the atom behavior, text metrics, and accessibility surface.
- `surfaces.slint` owns panel/card/header/badge/banner surfaces. `HubPanel` is the compatibility wrapper around Material `ElevatedCard` and `OutlinedCard`; Hub-specific variants only add the Zircon border/tint layer above that template surface. `PanelHeader` keeps the Hub right-action API, but the action itself is an imported Material `OutlineButton`, and its title typography now goes through Material `MaterialText`. `StatusBanner` titles use the same `MaterialText` path. `Badge` and `StatusBadge` keep the Hub tone shell and public name while delegating label typography and alignment to Material `MaterialText`.
- `inputs.slint` owns search, select, dropdown, segmented controls, the Material `SearchBar`-backed `SearchBox` wrapper, the Material `TextField`-backed `HubTextField` wrapper, the Material `SegmentedButton`-backed Hub `SegmentButton` wrapper, the Material `OutlineButton`/`PopupMenu`-backed Hub `ToolbarSelect` wrapper, and the Material `OutlineButton`/`TonalButton`-backed Hub `DropDownButton` wrapper.
- `data_display.slint` owns shared data and operation rows. `InfoRow`, `ActionRow`, and `BuildHistoryRow` now wrap the imported Material `ListTile` for icon/avatar, title/detail text, disabled handling, and state-layer interaction while preserving existing Hub row contracts and trailing badge/chevron affordances. `BuildHistoryRow` is shared by Editor and Builds so recent source-build records use one data-display implementation instead of a page-local Editor row. `DataTable`, `CatalogListPanel`, and `PanelListViewport` use the Material `ScrollView` surface instead of `std-widgets.slint`, so shared table/list scrolling follows the same template path as page-level `PageScrollSurface`. `TableColumnHeader`, `ProjectTableRow`, and `CatalogListPanel` empty-state titles now use Material `MaterialText`, keeping proportional column/catalog layout in Hub code while delegating typography metrics to the template. Editor source-engine rows follow the same Material `ListTile` rule while keeping their Hub-specific trailing status and remove controls.
- `shell.slint` owns the app chrome: top header, source-engine picker, nav sidebar composition, page header actions, bottom status bar, window controls, and drag regions. Source-engine popup heading/count text, top-header brand and user text, nav engine-status/update labels, nav current-project label, `HubPageHeader` page titles, non-Projects task labels, and `HubStatusBar` task detail now use Material `MaterialText` typography. `HubTopHeader` receives the selected project and active Source Engine context from `app.slint`, showing the selected project in the brand subtitle when available while the Source Engine selector stays beside it. `HubNavSidebar` receives the same context, so its expanded lower status card shows engine state plus the current selected-project title before the update action. `HubPageHeader` and `HubStatusBar` receive the same selected-project and active Source Engine context as the pages, plus compact state and the tokenized context-badge width from `app.slint`; non-Projects page headers and the bottom status bar display those badges on wider windows and hide them below the medium breakpoint without deriving layout from their own resolved width.
- The source-engine popup inside `shell.slint` keeps the Hub-owned popup/card shell but uses imported Material `ListTile` for each engine option, matching the same list-row behavior as Projects source-engine choices.
- `navigation.slint` owns Hub navigation composition. Its collapsed branch wraps the imported Material `NavigationRail`; its expanded branch keeps the wider Hub row-style `NavButton` list, with each row body backed by Material `ListTile`.
- `data_display.slint` owns reusable rows, tables, `CatalogPage`, list panels, and Material-backed panel scroll viewports. Development-only component sample surfaces stay removed; contracts validate real Hub pages and wrappers instead.
- `overlays.slint` owns popup-style reusable surfaces.
- `project_dashboard.slint` owns the Projects dashboard composition: toolbar flow, project card flow, recent-project table panel, and quick-action panel grid. The dashboard root now uses `PageScrollSurface` for page overflow and content-width/viewport-height sizing, while project card titles and dashboard empty-state titles use Material `MaterialText` typography and keep proportional card layout in Hub code. `project_page_components.slint` owns shared Projects secondary-page building blocks; `PageHeader`, `ProjectSettingSummaryRow`, and `ProjectBrowserRow` now use Material `MaterialText` for visible title, label, and meta typography instead of page-local raw `Text` font bindings. `project_pages.slint` owns New Project, Browser, and Detail page flows, and those page roots also derive content width from `PageScrollSurface`. `projects.slint` remains a subpage router.
- `app.slint` is now only the public `HubWindow` binding surface plus shell/page routing. It delegates chrome to `HubTopHeader`, `HubNavSidebar`, `HubPageHeader`, and `HubStatusBar` instead of directly drawing header/nav/status layout, and forwards the selected project, active Source Engine, compact state, and localized shell copy into the top header, nav sidebar, page header, and status bar.
- `editor.slint`, `builds.slint`, `settings.slint`, `cloud.slint`, and `team.slint` use `WorkspacePanelSection` so workspace panel rows share tokenized heights and compact behavior instead of repeating `flow-height` breakpoint formulas in each page. These workspace pages now derive compact thresholds from `PageScrollSurface.content-width` instead of repeating `root.width - HubTokens.page-padding * 2` in each page, and they let `PageScrollSurface` plus stretch layout fill rows instead of assigning `width: root.content-width`. `editor.slint`, `builds.slint`, and `settings.slint` route all `WorkspacePanelSection` child sizing through `ResponsiveSlot`, keeping Taffy basis/grow/shrink at the shared wrapper boundary instead of on page-local panels. Editor, Builds, and the Settings detail row now give their main slots a larger token basis with grow weight 2 and their side slots a side-panel token basis with grow weight 1, so pages no longer precompute overview/actions/config/detail widths from a remaining-width formula. Settings default-path rows keep a token preferred width and stretch inside `PanelListViewport` instead of deriving row width from the panel's resolved width. `project_dashboard.slint` and `project_pages.slint` follow the same rule for toolbar and secondary-page flex items, and their roots now use `PageScrollSurface.content-width` instead of local page-width subtraction; dashboard toolbar selects, lower Recent/Quick panels, proportional card basis, New/Detail main-side panel bases, and Browser toolbar select bases now avoid width-minus-gap formulas, while dashboard repeat cards use preferred/min widths because Slint 1.16.1 rejects a `ResponsiveSlot` wrapper inside that repeated card path. `builds.slint` consumes selected project detail so build/package rows describe the current project context instead of only the active engine, sizes its summary row token for five Build/Open/Launch/Package/Install action rows, and uses Material `MaterialText` for the current-task status headline. `cloud.slint` sets `compact-stack: false`, drives `compact-rows` for wrapped metric cards, routes metric card basis/min-width/grow through tokenized `ResponsiveSlot` inputs, and uses four-column/two-column/one-column Taffy growth instead of page-local content-width-minus-gap division for multi-column metric rows. Cloud service rows and Team member rows consume `PageScrollSurface.content-height` for their list panel viewport budget rather than recomputing `root.height - page-padding - bottom-status` in page code. `team.slint` keeps summary and empty-state titles on Material `MaterialText`, while the other workspace pages use the default compact stacking behavior.

## Layout Contract

Pages should describe semantic regions instead of pixel-placement details: shell, header, nav rail, toolbar, card flow, panel grid, list/table, detail, and action regions. Page-level code may choose `min-width`, `preferred-width`, row-height parameters, and breakpoint tokens, while workspace rows, Projects toolbar items, and Projects secondary-page columns should express Taffy basis/grow/shrink through `ResponsiveSlot` rather than direct child `flex-*` properties. It should not place business regions with `x`/`y`, nor introduce literal page breakpoints such as `root.width < 980px`.

Workspace-style pages should use `WorkspacePanelSection` for repeated two-panel rows and metric groups that need shared height math. The primitive is still Slint-native `FlexboxLayout`, but it centralizes compact height calculation, exposes `compact-stack` for rows that must wrap instead of stack, exposes `compact-rows` for wrapped metric groups that need more than two rows, and keeps row heights in `HubTokens`. Workspace pages derive their semantic content width from the surrounding `PageScrollSurface`, while `WorkspacePanelSection` itself keeps `preferred-width: 0px` so it does not derive preferred width from its own layout result. This makes `editor`, `builds`, `settings`, `cloud`, and `team` easier to scan and keeps future breakpoint changes in one place. The shell lets Slint/Taffy allocate the page host as the horizontally and vertically stretched sibling after the navigation rail; `app.slint` must not hand-write `parent.width - root.nav-width` content formulas, because those formulas create layout-info dependency loops when the shell, page header, and routed page all query each other.

Percent sizing is intentionally avoided in Hub-owned Slint files. Shared stretch components such as `InfoRow`, `ActionRow`, `PanelGrid`, and `PanelHeader` use `horizontal-stretch`, `min-width`, parent/content-width inputs, and `preferred-width: 0px` instead of `preferred-width: 100%`, so the component does not ask Slint to derive its preferred size from the same layout result it participates in.

Fixed sizes remain allowed at the atom layer where they carry real interaction meaning: icon sizes, button/control heights, Material text-field height, table/list row heights, 1px borders/dividers, popup anchors, and window drag/resize math. Those values should come from `HubTokens` whenever they are part of the common Hub scale.

Visible Hub typography follows the same component contract. Page and component files should use Material `MaterialText` or Hub wrappers such as `FieldLabel` and `MutedText` instead of raw Slint `Text`, `inherits Text`, or direct `font-size`/`font-weight` bindings. Read-only `MaterialTypography.*.font_size` references remain allowed when a layout needs to derive row or section height from the template's text metrics.

Icon-like affordances follow the same rule. Toolbar, row, status, popup, and window controls should use bundled SVG assets or Material icon slots instead of character literals such as `+`, `>`, `v`, `!`, `?`, `[]`, `::`, `==`, or `...` in `text` and `fallback-text` bindings.

Pointer interaction follows the imported Material/template path. Buttons, list rows, cards, toolbars, selectors, and row action zones should use Material button primitives, `ListTile`, `PopupMenu`, or `StateLayerArea`; direct Slint `TouchArea` usage is reserved for the frameless shell drag region in `shell.slint`.

The current responsive breakpoints are:

- compact: widths below `HubTokens.breakpoint-compact` (`1040px`)
- medium: widths below `HubTokens.breakpoint-medium` (`1280px`)
- wide: widths at or above `HubTokens.breakpoint-wide` (`1536px`)

The shell treats widths below the wide breakpoint as compact for the top title bar. At 1280px, the title bar has enough room for the brand, source-engine selector, user controls, and window buttons, but not enough room for the full task/success/warning/error status-pill group without overlap. `app.slint` therefore passes `compact: true` to `HubTopHeader` below `HubTokens.breakpoint-wide`; the status pills remain visible at the 1600px reference width and are hidden at 1280px and 1024px so the chrome stays aligned while page bodies enter their own responsive layouts.

The shell also treats heights below `HubTokens.breakpoint-short` as compact for the left rail. The threshold is intentionally above the 900px and 768px validation sizes so expanded Material `ListTile` navigation rows have enough vertical room; the lower engine/project status card and collapse action are hidden before they can compress the navigation list into the bottom status bar. When visible, the lower card uses token-derived shell row height to reserve room for both active Source Engine status and current selected-project context.

The page header and bottom status bar treat widths below `HubTokens.breakpoint-medium` as compact, but that breakpoint is evaluated in `app.slint` and passed into the shell components. At wide sizes `HubPageHeader` places selected-project plus active Source Engine badges beside non-Projects page title actions, while `HubStatusBar` keeps the current task text flexible through Material `MaterialText` and places the same badges after it using `HubTokens.status-badge-width`; at compact sizes those badges are hidden so the title band and task detail can elide cleanly instead of overlapping shell chrome. These shell components must not calculate compact state or badge width from their own resolved width because those values participate in Slint layout info. `HubTopHeader` sizes the Source Engine selector from `nav-width`, header button size, and token clamps instead of dividing `root.width`, so compact chrome width remains a shell input contract rather than page-local geometry math.

Projects secondary pages use content-relative breakpoints rather than a single shell width. `ProjectNewPage` keeps project name, location, Source Engine choice, create action, and summary in the main panel, with templates in a side rail only when `PageScrollSurface.content-width` can hold both columns; both panels are `ResponsiveSlot` children of the page `FlexboxLayout`, and the page scroll surface uses the shared Material-backed `PageScrollSurface`. Its main/side columns use tokenized panel bases and minimums instead of subtracting gaps from `content-width`. At 1024px, that rail wraps below the settings panel and remains reachable through the page scroll. `ProjectBrowserPage` wraps search, filter, and sort controls into a second toolbar row when the content width cannot hold the wide toolbar, with each toolbar control sized through `ResponsiveSlot`; filter and sort selects use a shared toolbar-select basis/grow path rather than a half-width subtraction, and the browser page uses a separate inner Material `ScrollView` only for list rows. `ProjectDetailPage` uses the same content-driven flow: the main cover/info panel remains first, the Actions/Bound Source Engine panel stacks below it at 1024px, both detail columns use tokenized `ResponsiveSlot` bases, and the detail page owns a `PageScrollSurface` so short windows scroll instead of clipping project-info rows. Workspace pages use `WorkspacePanelSection` for wrapped summary/metric rows; Cloud passes a compact row count plus tokenized metric slot basis/min-width/grow for the four-card metric set, so one-column, two-column, and wide four-column layouts expand by row token and Taffy growth instead of relying on page-local height or half-width formulas.

## Validation

Quick-action fallback is part of the responsive workflow contract. When Build, Package, Install, or Open Editor promotes the latest recent project because no project is selected, the runtime refreshes selected-project scoped Assets, Plugins, and Team data before projecting the next Hub snapshot. Builds page Open Editor, Package, and Install rows route through the same underlying runtime paths where applicable, but their visible rows stay disabled until `ProjectDetailData` has an actual selected project so the Builds workflow remains explicitly project-scoped. Removing or confirming deletion of the selected project clears those scoped views through the same refresh path; the shared remove helper also clears pending-delete state for the removed path. Project-scoped actions also activate the project's bound Source Engine when Hub metadata has one, while `ProjectDetailData` remains an explicit no-selection row until `selected_project_path` is set.

`zircon_hub/tests/ui_contract.rs` locks the important structural rules as an integration test so the file scanner does not force the full Slint-generated Hub library into a Rust unit-test harness: the `components.slint` entrypoint must stay thin, the direct `@material` bridge must stay available, `Page` must use Material `ScrollView`, `Divider` must use Material `HorizontalDivider`/`VerticalDivider`, data-display table/list surfaces and Projects dashboard/secondary scroll surfaces must use Material `ScrollView`, data-display table headers, `ProjectTableRow` cells, `CatalogListPanel` empty-state titles, Projects dashboard card/empty titles, Projects secondary-page shared header/summary/browser labels, Projects page section/status labels, Builds current-task status headline, Cloud metric status labels, and Team summary/empty-state titles must use Material `MaterialText`, the Hub window must install `ZirconMaterialTheme`, `HubPanel` must remain backed by Material card surfaces without placing a panel-level `TouchArea` above `@children`, `PanelHeader` right actions must stay on Material `OutlineButton`, Hub `Badge`/`StatusBadge` labels plus `PanelHeader`, `StatusBanner`, and shared `FieldLabel`/`MutedText` title wrappers must remain backed by Material `MaterialText` without returning to raw `Text` font bindings, and every Hub UI `.slint` file is globally scanned so raw `Text`, `inherits Text`, or direct `font-size`/`font-weight` bindings cannot return outside those focused component checks. The same global scan blocks character-icon `text`/`fallback-text` literals for icon-like symbols so controls keep using SVG assets or Material icon slots, reserves direct `TouchArea` for only the shell window-drag layer, and forbids percent sizing in width/height/preferred/min/max bindings. Hub `PillButton`/`IconButton` must remain backed by Material button primitives without local `TouchArea` pointer layers, header `StatusPill` must remain backed by Material `ActionChip`, Hub form pages must use the Material-backed `HubTextField` wrapper instead of `LineEdit`, Hub `SearchBox` must remain backed by Material `SearchBar` instead of direct `TextInput`, collapsed Hub navigation must remain backed by Material `NavigationRail`, expanded Hub `NavButton` rows, Hub `InfoRow`, `ActionRow`, shared `BuildHistoryRow`, Editor source-engine rows, and source-engine popup option rows must remain backed by Material `ListTile`, dashboard `ProjectCard`, dashboard `ProjectTableRow`, `ProjectBrowserRow`, and the sidebar collapse control must use Material `StateLayerArea` instead of custom card/row/thumbnail/body/collapse hit targets for hover/press feedback, while `ProjectBrowserRow` keeps row selection and the trailing detail transition in separate tokenized Material `StateLayerArea` zones with a centered SVG detail icon and a fallback branch for captured trailing clicks. Hub `SegmentButton` must remain backed by Material `SegmentedButton`, Hub `ToolbarSelect` must remain backed by Material `OutlineButton` and `PopupMenu`, Hub `DropDownButton` must remain backed by Material `OutlineButton`/`TonalButton`, Projects engine/template choice rows must remain backed by Material `ListTile`, `HubPageHeader` and `HubStatusBar` must receive selected-project and active Source Engine context from `HubWindow` and hide those badges on compact widths, Builds must receive selected project detail and active source-build history rows, Assets must prioritize selected-project assets and pass selected-project empty-state copy into the page, `ProjectDetailData` must remain an explicit no-selection state until `selected_project_path` resolves, Plugins must surface selected-project/Source Engine scope and pass selected-project empty-state copy into the page, Team must pass selected-project empty-state copy into its local Git view, Quick Actions must derive their selected/latest project targeting from `HubSnapshot`, the component modules and tokens must exist, workspace pages must route panel rows through `WorkspacePanelSection`, Editor/Settings workspace rows plus Projects dashboard and secondary pages must not use direct `flex-basis`/`flex-grow`/`flex-shrink`, page compact breakpoints must use `HubTokens`, business pages may not reintroduce `x`/`y` layout positioning, the app shell may not return to hand-written remaining-width subtraction, and Hub UI files may not introduce raw pixel literals above `1px` outside the `MaterialStyleMetrics`/`HubTokens` scale. The same contract also asserts that the removed `ButtonStates` development surface does not return and that `ComponentSamples` stays internal-only for Material/Taffy coverage. `zircon_hub/Cargo.toml` sets the library target's default unit-test harness to `test = false`; Hub package tests still run through integration tests, while normal `cargo check`/`cargo build` continue compiling the Slint UI library and binary. Compile and visual validation still remain required because Slint layout behavior is ultimately checked through generated UI and real Hub screenshots.

The shell-context contract also covers `HubTopHeader` and `HubNavSidebar`: `app.slint` must pass `ProjectDetailData`, `SourceEngineData`, and `UiTextData` into those shell components, the top-header brand subtitle must derive from the selected project without adding a page-local badge, and the expanded lower sidebar status card must render current-project context with Material `MaterialText` inside token-derived height instead of adding page-local geometry.

The status-bar contract also forbids deriving compact state or badge width from `HubStatusBar`'s own resolved width, because that creates a Slint layout-info binding loop.

The same selected-project workflow contract now includes Cloud: when a project is selected, the Cloud page receives selected-project package/install/output title and detail copy instead of presenting the local readiness panel as a generic cloud account view.

The latest Material-card validation captured Projects dashboard, new-project, project-browser, and project-detail pages at 1280x900 and 1024x768 after `ZirconMaterialTheme` moved Material controls back to the teal/dark Hub palette and Projects engine/template choices moved to Material `ListTile`. The Projects Taffy validation also captured those four pages at 1600x1024, 1280x900, and 1024x768 after dashboard toolbar and Projects secondary-page flex items moved to `ResponsiveSlot`; all three capture runs wrote empty stdout/stderr logs. Later focused captures and static contracts verify Material-backed SearchBar, NavigationRail, NavButton, StatusPill, Divider, ScrollView, Badge/MaterialText, PanelHeader/StatusBanner title MaterialText, shared FieldLabel/MutedText MaterialText typography, DataTable/ProjectTableRow MaterialText typography, dashboard ProjectCard/empty-state MaterialText typography, Projects secondary-page PageHeader/summary/browser/status MaterialText typography, Builds current-task status MaterialText typography, Cloud/Team metric and empty-state MaterialText typography, `CatalogListPanel` empty-state MaterialText typography, CatalogListPanel list rendering on Assets/Plugins at 1100x820, shell chrome/page-header/status MaterialText typography, the global raw Text/direct font-binding scan, the character-icon literal scan, the direct TouchArea reservation scan, the percent-sizing scan, the Editor/Builds workspace slot grow scan, ActionRow, InfoRow, source-engine popup, Cloud/Team workspace rows, project selector state layers, status-bar context badges without self-width binding loops, and the Project Browser trailing detail hit zone through real Hub pages. The current captures at `target/hub-visual-check/material-state-detail-1280x900` and `target/hub-visual-check/material-responsive-state-detail-1024x720` were run against the current built Hub binary through the dashboard secondary-page path plus a centered trailing detail click, and verify that the browser detail control opens Project Detail while row-body clicks remain reserved for selection. The static contract now checks Material/Taffy coverage through bridge exports, shared wrappers, Projects, Editor, Builds, Settings, Cloud four-card metrics, and data-display rows, while also asserting that the removed `ButtonStates` development surface does not return and that internal `ComponentSamples` stays out of user-facing pages. Earlier captures still cover the 1600x1024 search/filter/sort toolbar row, the 1366x820 detail layout, 1100x760 and 960x640 dashboard layouts, the Material-button-backed dashboard header, the Material text-field wrappers in Settings/Editor/Projects, the Material `PopupMenu` selector surface, the Material `SegmentedButton` wrapper in Settings, and the Material `DropDownButton` trigger plus Hub-owned engine picker.
