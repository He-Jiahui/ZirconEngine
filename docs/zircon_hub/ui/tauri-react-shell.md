---
related_code:
  - zircon_hub/Cargo.toml
  - zircon_hub/build.rs
  - zircon_hub/tauri.conf.json
  - zircon_hub/package.json
  - zircon_hub/vite.config.ts
  - zircon_hub/src/main.rs
  - zircon_hub/src/lib.rs
  - zircon_hub/src/tauri_app.rs
  - zircon_hub/web/src/App.tsx
  - zircon_hub/web/src/main.tsx
  - zircon_hub/web/src/theme/tokens.ts
  - zircon_hub/web/src/theme/muiTheme.ts
  - zircon_hub/web/src/components/inputs/HubButton.tsx
  - zircon_hub/web/src/components/inputs/HubIconButton.tsx
  - zircon_hub/web/src/components/inputs/HubSearchField.tsx
  - zircon_hub/web/src/components/inputs/HubSelect.tsx
  - zircon_hub/web/src/components/inputs/HubToggle.tsx
  - zircon_hub/web/src/components/data/ProjectCard.tsx
  - zircon_hub/web/src/components/data/ProjectTable.tsx
  - zircon_hub/web/src/components/data/QuickActions.tsx
  - zircon_hub/web/src/components/data/StatusBadge.tsx
  - zircon_hub/web/src/components/overlays/HubMenu.tsx
  - zircon_hub/web/src/components/shell/NavigationDrawer.tsx
  - zircon_hub/web/src/components/shell/TopBar.tsx
  - zircon_hub/web/src/components/shell/HubWindow.tsx
  - zircon_hub/web/src/pages/ProjectsDashboard.tsx
  - zircon_hub/tests/tauri_react_shell_contract.rs
  - zircon_hub/tests/ui_foundation_contract.rs
  - zircon_hub/tests/ui_material_usage_contract.rs
implementation_files:
  - zircon_hub/Cargo.toml
  - zircon_hub/build.rs
  - zircon_hub/tauri.conf.json
  - zircon_hub/package.json
  - zircon_hub/vite.config.ts
  - zircon_hub/src/main.rs
  - zircon_hub/src/lib.rs
  - zircon_hub/src/tauri_app.rs
  - zircon_hub/web/src
plan_sources:
  - user: 2026-06-05 switch Zircon Hub to Tauri + React frontend, use real Material UI, and build bottom-up component layers
  - docs/ui-and-layout/hub.png
  - docs/ui-and-layout/hub-web-reference
  - docs/ui-and-layout/hub-ai-drafts
  - dev/material-ui
tests:
  - cargo test --manifest-path zircon_hub/Cargo.toml --test tauri_react_shell_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_foundation_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_material_usage_contract
  - npm run typecheck
  - npm run build
doc_type: ui-architecture
---

# Zircon Hub Tauri React Shell

This slice starts the hard cut from the Slint launcher to a Tauri v2 desktop shell with a React + Material UI frontend. The Rust binary now enters through `zircon_hub::tauri_app::run()`, `build.rs` delegates to `tauri_build::build()`, and `tauri.conf.json` points Tauri at the Vite frontend on port 1420 with a `1568x1003` undecorated Hub window.

The first command boundary exposes `hub_state` and `hub_action`. It currently returns the Projects dashboard reference state used by the React shell; the next migration step should replace that reference projection with the existing Hub project/runtime state model, converted into serde DTOs that match `web/src/types/hub.ts`.

## Component Order

The React side is intentionally bottom-up:

- `web/src/theme` owns density, color, radius, shadow, typography, and shared MUI component overrides.
- `web/src/components/inputs` owns low-level buttons, icon buttons, search fields, selects, and toggles.
- `web/src/components/data` owns reusable cards, cover media, tables, panels, quick-action lists, status badges, and button-state samples.
- `web/src/components/overlays` owns popup/menu surfaces.
- `web/src/components/shell` owns drawer, topbar, and window composition.
- `web/src/pages` only assembles shared components into the Projects dashboard layout.

The visual asset policy uses `zircon_hub/assets/brand` and `zircon_hub/assets/covers/reference` at runtime. It must not render `docs/ui-and-layout/hub.png`, `hub-web-reference-1568x1003.png`, or AI draft PNGs as application UI.

## Open Migration Work

The Slint source tree still exists as historical static material and many page-level contracts still inspect it. It is no longer compiled by the Hub binary after this slice, but the remaining page contracts need a follow-up migration into React/MUI static contracts before the old Slint UI directory can be deleted cleanly.
