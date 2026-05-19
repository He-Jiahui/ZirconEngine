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
  - zircon_hub/ui/editor.slint
  - zircon_hub/ui/builds.slint
  - zircon_hub/ui/settings.slint
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
  - cargo test -p zircon_hub --locked --offline --jobs 1 --target-dir target\hub-material-check --color never
  - cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1 --target-dir target\hub-material-check --color never
  - zircon_hub/tests/ui_contract.rs
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-window.ps1
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-project-pages.ps1
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
  - target/hub-visual-check/material-actionrow-1600x1024-v1/hub-projects-dashboard.png
  - target/hub-visual-check/material-actionrow-1600x1024-v1/hub-projects-browser.png
  - target/hub-visual-check/material-actionrow-1600x1024-v1/hub-projects-detail.png
  - target/hub-visual-check/material-actionrow-1024x768-v1/hub-projects-dashboard.png
  - target/hub-visual-check/material-actionrow-1024x768-v1/hub-projects-browser.png
  - target/hub-visual-check/material-actionrow-1024x768-v1/hub-projects-detail.png
  - target/hub-visual-check/material-header-engine-listtile-984x640-v1.png
doc_type: module-detail
---

# Zircon Hub Responsive Component System

Zircon Hub keeps Slint as the rendering and interaction layer. It does not embed HTML, CSS, WebView, or a browser layout engine. The Hub UI now exposes a Slint-native component layer that borrows the useful parts of Material UI's component taxonomy: tokens first, layout primitives second, then surfaces, inputs, navigation, data display, and overlays.

The Hub now imports the local Slint Material UI template directly. `zircon_hub/build.rs` registers `dev/material-rust-template/material-1.0/material.slint` as the Slint library path `@material`, and `material_bridge.slint` re-exports the template's layout, card, button, input, navigation, list, progress, tab, tooltip, and style primitives for Hub pages that need the real template components. `theme.slint` installs a Zircon teal/dark `MaterialPalette.schemes` override at UI initialization time, so Material-owned controls keep the template behavior without inheriting the template's default blue palette. This is a direct template dependency, not a hand-written Material clone and not an HTML/CSS engine.

Hub surfaces keep the Zircon dark tool UI contract, but they are now backed by real Material template cards where the template provides the matching primitive. `HubPanel` selects `ElevatedCard` for elevated variants and `OutlinedCard` for standard panel variants, then overlays Hub-owned border, tint, hover, and children content so existing page APIs continue to work while the base surface comes from the imported template. Its panel-level hover `TouchArea` is placed behind `@children` and only enabled for interactive variants; default panels must not cover nested list rows, buttons, inputs, or detail affordances.

Shared Hub button APIs also delegate to the imported Material template instead of reimplementing pointer behavior locally. `PillButton` now wraps `FilledButton` for primary actions and `OutlineButton` for secondary actions, while Hub `IconButton` wraps `FilledIconButton` for active icon actions and `OutlineIconButton` for inactive icon actions. `PillButton` owns a stable atom-level preferred width and clip boundary so Material text buttons cannot overpaint adjacent responsive header actions at compact window sizes; Hub `IconButton` clips the Material icon button to the requested atom size for dense rows. The Hub-facing properties remain stable (`text`, `icon-image`, `has-icon-image`, `primary`, `active`, `enabled`, and `clicked`), so existing pages keep semantic button usage while hover, disabled, ripple, accessibility, and icon button behavior come from the direct Material template import.

Hub text input fields now follow the same compatibility-wrapper pattern. `HubTextField` lives in `inputs.slint`, exposes Hub-facing `label`, `placeholder`, `supporting-text`, `text`, `enabled`, and edit/accept callbacks, and renders the imported Material template `TextField` internally. Settings uses this wrapper for toolchain paths, build jobs, and default path rows; Editor uses it for active engine name, source checkout, and staged output fields; Projects New Project uses it for project name and location. Hub editable form pages should not import `std-widgets` `LineEdit` directly, while `HubTokens.input-field` owns the Material field atom height and `HubTokens.input-width` gives Material text fields a stable preferred width before row stretch is available. Toolbar search remains exposed as the Hub-facing `SearchBox` API, but it now wraps the imported Material `SearchBar` with the search SVG as the leading icon and a visible placeholder prompt, so Hub no longer needs a custom painted `TextInput` or a `TextField` label workaround for toolbar search.

Hub segmented controls also use the imported template now. The Hub-facing `SegmentButton` wrapper keeps the existing `text`, `active`, and `clicked` API for Settings and component samples, but its internal state is a one-item Material `SegmentedButton` rather than a hand-painted rectangle plus `TouchArea`. The wrapper uses the Material component's 40px minimum control height through `HubTokens.control-md`, so Settings build-profile and language rows do not clip the template's state layer or selected check affordance.

Toolbar dropdowns now follow the same rule. `ToolbarSelect` keeps the Hub-facing label, icon, legacy option id, and callback contract, but its trigger is the imported Material `OutlineButton` and its popup list is the imported Material `PopupMenu` fed by `MenuItem` data. Projects dashboard and browser pages pass both Material `menu-items` and the existing id-bearing `options`, so the visual/menu layer comes from the local template while filter and sort actions still dispatch the stable Hub ids.

The generic Hub `DropDownButton` wrapper is also backed by Material buttons. It now renders an imported `OutlineButton` for inactive dropdown triggers and `TonalButton` for active triggers, using the existing Hub `text`, `icon-image`, `has-icon-image`, `active`, and `clicked` properties. The top Source Engine selector still owns its custom `PopupWindow` engine list because that list is not a simple `MenuItem` projection, but the trigger itself no longer paints local borders/backgrounds or owns a separate pointer `TouchArea`.

`components.slint` is intentionally only a public entrypoint. Existing pages can continue importing from it, but implementation belongs in the category files below:

- `tokens.slint` owns spacing, breakpoints, page padding, panel gaps, toolbar gaps, control heights, icon sizes, list/table row heights, workspace panel row heights, borders, panel minimums, and shared dark Hub colors.
- `material_bridge.slint` owns no Hub styling. It directly re-exports the local Material template components that Hub is allowed to consume through `components.slint`, while leaving conflicting names such as Hub `Badge` under Hub ownership.
- `layout.slint` owns CSS-like composition primitives: `Stack`, `Row`, `Column`, `Flow`, `Page`, `PanelGrid`, `ResponsiveSlot`, `Fill`, `Divider`, `PageScrollSurface`, `ResponsivePanelFlow`, and `WorkspacePanelSection`. `Page` uses the Material template `ScrollView`, while its content width is still constrained by the Hub shell so the page body only sees the viewport left after the navigation rail.
- `theme.slint` owns the Hub-level Material palette override. It is installed by `HubWindow` through a zero-size `ZirconMaterialTheme` component so every imported Material control uses the same Zircon teal/dark scheme.
- `shared.slint` owns shared DTO structs plus compatibility button primitives such as `PillButton` and Hub `IconButton`. Those button wrappers preserve the Hub API and route their internals through Material `FilledButton`, `OutlineButton`, `FilledIconButton`, and `OutlineIconButton`.
- `surfaces.slint` owns panel/card/header/badge/banner surfaces. `HubPanel` is the compatibility wrapper around Material `ElevatedCard` and `OutlinedCard`; Hub-specific variants only add the Zircon border/tint layer above that template surface. `PanelHeader` keeps the Hub right-action API, but the action itself is an imported Material `OutlineButton`.
- `inputs.slint` owns search, select, dropdown, segmented controls, the Material `SearchBar`-backed `SearchBox` wrapper, the Material `TextField`-backed `HubTextField` wrapper, the Material `SegmentedButton`-backed Hub `SegmentButton` wrapper, the Material `OutlineButton`/`PopupMenu`-backed Hub `ToolbarSelect` wrapper, and the Material `OutlineButton`/`TonalButton`-backed Hub `DropDownButton` wrapper.
- `data_display.slint` owns shared data and operation rows. `ActionRow` now wraps the imported Material `ListTile` for icon/avatar, title/detail text, disabled handling, and state-layer interaction while preserving the existing Hub quick-action callback contract and trailing chevron affordance.
- `shell.slint` owns the app chrome: top header, source-engine picker, nav sidebar composition, page header actions, bottom status bar, window controls, and drag regions.
- The source-engine popup inside `shell.slint` keeps the Hub-owned popup/card shell but uses imported Material `ListTile` for each engine option, matching the same list-row behavior as Projects source-engine choices.
- `navigation.slint` owns Hub navigation composition.
- `data_display.slint` owns reusable rows, tables, `CatalogPage`, list panels, panel scroll viewports, and the internal component samples surface.
- `overlays.slint` owns popup-style reusable surfaces.
- `project_dashboard.slint` owns the Projects dashboard composition: toolbar flow, project card flow, recent-project table panel, and quick-action panel grid. `projects.slint` remains a subpage router.
- `app.slint` is now only the public `HubWindow` binding surface plus shell/page routing. It delegates chrome to `HubTopHeader`, `HubNavSidebar`, `HubPageHeader`, and `HubStatusBar` instead of directly drawing header/nav/status layout.
- `editor.slint`, `builds.slint`, and `settings.slint` use `WorkspacePanelSection` so workspace panel rows share tokenized heights and compact stacking behavior instead of repeating `flow-height` breakpoint formulas in each page.

## Layout Contract

Pages should describe semantic regions instead of pixel-placement details: shell, header, nav rail, toolbar, card flow, panel grid, list/table, detail, and action regions. Page-level code may choose `min-width`, `preferred-width`, `flex-basis`, `flex-grow`, `flex-shrink`, row-height parameters, and breakpoint tokens. It should not place business regions with `x`/`y`, nor introduce literal page breakpoints such as `root.width < 980px`.

Workspace-style pages should use `WorkspacePanelSection` for repeated two-panel rows. The primitive is still Slint-native `FlexboxLayout`, but it centralizes the compact height calculation and keeps the row height in `HubTokens`, which makes `editor`, `builds`, and `settings` easier to scan and keeps future breakpoint changes in one place. The shell lets Slint/Taffy allocate the page host as the horizontally and vertically stretched sibling after the navigation rail; `app.slint` must not hand-write `parent.width - root.nav-width` content formulas, because those formulas create layout-info dependency loops when the shell, page header, and routed page all query each other.

Fixed sizes remain allowed at the atom layer where they carry real interaction meaning: icon sizes, button/control heights, Material text-field height, table/list row heights, 1px borders/dividers, popup anchors, and window drag/resize math. Those values should come from `HubTokens` whenever they are part of the common Hub scale.

The current responsive breakpoints are:

- compact: widths below `HubTokens.breakpoint-compact` (`1040px`)
- medium: widths below `HubTokens.breakpoint-medium` (`1280px`)
- wide: widths at or above `HubTokens.breakpoint-wide` (`1536px`)

The shell treats widths below the wide breakpoint as compact for the top title bar. At 1280px, the title bar has enough room for the brand, source-engine selector, user controls, and window buttons, but not enough room for the full task/success/warning/error status-pill group without overlap. `app.slint` therefore passes `compact: true` to `HubTopHeader` below `HubTokens.breakpoint-wide`; the status pills remain visible at the 1600px reference width and are hidden at 1280px and 1024px so the chrome stays aligned while page bodies enter their own responsive layouts.

Projects secondary pages use content-relative breakpoints rather than a single shell width. `ProjectNewPage` keeps project name, location, Source Engine choice, create action, and summary in the main panel, with templates in a side rail only when the content width can hold both columns. At 1024px, that rail wraps below the settings panel and remains reachable through the page scroll. `ProjectBrowserPage` wraps search, filter, and sort controls into a second toolbar row when the content width cannot hold the wide toolbar. `ProjectDetailPage` uses the same content-driven flow: the main cover/info panel remains first, and the Actions/Bound Source Engine panel stacks below it at 1024px instead of compressing the detail content into unreadable columns.

## Validation

`zircon_hub/tests/ui_contract.rs` locks the important structural rules as an integration test so the file scanner does not force the full Slint-generated Hub library into a Rust unit-test harness: the `components.slint` entrypoint must stay thin, the direct `@material` bridge must stay available, the Hub window must install `ZirconMaterialTheme`, `HubPanel` must remain backed by Material card surfaces without placing a panel-level `TouchArea` above `@children`, `PanelHeader` right actions must stay on Material `OutlineButton`, Hub `PillButton`/`IconButton` must remain backed by Material button primitives without local `TouchArea` pointer layers, Hub form pages must use the Material-backed `HubTextField` wrapper instead of `LineEdit`, Hub `SearchBox` must remain backed by Material `SearchBar` instead of direct `TextInput`, Hub `ActionRow` and source-engine popup option rows must remain backed by Material `ListTile`, Hub `SegmentButton` must remain backed by Material `SegmentedButton`, Hub `ToolbarSelect` must remain backed by Material `OutlineButton` and `PopupMenu`, Hub `DropDownButton` must remain backed by Material `OutlineButton`/`TonalButton`, Projects engine/template choice rows must remain backed by Material `ListTile`, the component modules and tokens must exist, workspace pages must route panel rows through `WorkspacePanelSection`, page compact breakpoints must use `HubTokens`, business pages may not reintroduce `x`/`y` layout positioning, the app shell may not return to hand-written remaining-width subtraction, and Hub UI files may not introduce raw pixel literals above `1px` outside the `MaterialStyleMetrics`/`HubTokens` scale. `zircon_hub/Cargo.toml` sets the library target's default unit-test harness to `test = false`; Hub package tests still run through integration tests, while normal `cargo check`/`cargo build` continue compiling the Slint UI library and binary. Compile and visual validation still remain required because Slint layout behavior is ultimately checked through generated UI and real Hub screenshots.

The latest Material-card validation captured Projects dashboard, new-project, project-browser, and project-detail pages at 1280x900 and 1024x768 after `ZirconMaterialTheme` moved Material controls back to the teal/dark Hub palette and Projects engine/template choices moved to Material `ListTile`. The 1280x900 capture verifies Dashboard, New Project, Project Browser, and Project Detail keep the reference teal hierarchy; New Project keeps project settings as the wider primary column while the right template rail remains readable. The 1024x768 capture verifies New Project stacks the template rail under the project settings panel, Project Browser keeps a loose selector list after toolbar wrapping, and Project Detail stacks the action panel below the main project information panel. The Material SearchBar capture verifies toolbar search now uses the template search atom with a visible placeholder and no secondary-page popup left open by the capture flow. The Material ActionRow captures verify the dashboard Quick Actions panel now renders list-tile operation rows with stable trailing chevrons at the reference 1600x1024 size and that 1024x768 secondary pages still stack without overlap after the row-height clamp changed to the Material list-row minimum. The header engine popup capture verifies the source-engine selector now opens Material `ListTile` option rows inside the Hub popup shell. Earlier captures still cover the 1600x1024 search/filter/sort toolbar row, the 1366x820 detail layout, 1100x760 and 960x640 dashboard layouts, the Material-button-backed dashboard header, the Material text-field wrappers in Settings/Editor/Projects, the Material `PopupMenu` selector surface, the Material `SegmentedButton` wrapper in Settings, and the Material `DropDownButton` trigger plus Hub-owned engine picker.
