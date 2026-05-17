---
related_code:
  - zircon_hub/ui/app.slint
  - zircon_hub/ui/shared.slint
  - zircon_hub/ui/projects.slint
  - zircon_hub/src/app/view_model.rs
  - zircon_hub/src/app/view_model/media.rs
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
  - zircon_hub/assets/covers/project-elysium.svg
  - zircon_hub/assets/covers/project-stellar-outpost.svg
  - zircon_hub/assets/covers/project-sands-of-time.svg
  - zircon_hub/assets/covers/project-whispering-woods.svg
  - zircon_hub/assets/covers/project-neon-streets.svg
  - zircon_editor/assets/preview/editor-scifi-room.svg
  - zircon_editor/assets/icons/zircon_editor_shell/toolbar/menu.svg
  - zircon_editor/assets/icons/zircon_editor_shell/activity/play.svg
  - zircon_editor/assets/icons/zircon_editor_shell/scene/root.svg
  - zircon_editor/assets/icons/zircon_editor_shell/inspector/transform.svg
  - zircon_editor/assets/icons/zircon_editor_shell/controls/add.svg
  - zircon_editor/assets/icons/zircon_editor_shell/status/info.svg
  - zircon_editor/assets/icons/zircon_editor_shell/viewport/axis-gizmo.svg
implementation_files:
  - zircon_hub/ui/app.slint
  - zircon_hub/ui/shared.slint
  - zircon_hub/ui/projects.slint
  - zircon_hub/src/app/view_model.rs
  - zircon_hub/src/app/view_model/media.rs
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
  - zircon_hub/assets/covers/project-elysium.svg
  - zircon_hub/assets/covers/project-stellar-outpost.svg
  - zircon_hub/assets/covers/project-sands-of-time.svg
  - zircon_hub/assets/covers/project-whispering-woods.svg
  - zircon_hub/assets/covers/project-neon-streets.svg
  - zircon_editor/assets/preview/editor-scifi-room.svg
  - zircon_editor/assets/icons/zircon_editor_shell/toolbar/menu.svg
  - zircon_editor/assets/icons/zircon_editor_shell/activity/play.svg
  - zircon_editor/assets/icons/zircon_editor_shell/scene/root.svg
  - zircon_editor/assets/icons/zircon_editor_shell/inspector/transform.svg
  - zircon_editor/assets/icons/zircon_editor_shell/controls/add.svg
  - zircon_editor/assets/icons/zircon_editor_shell/status/info.svg
  - zircon_editor/assets/icons/zircon_editor_shell/viewport/axis-gizmo.svg
plan_sources:
  - user: 2026-05-16 generate all image resources shown by the Hub and Editor reference screenshots as SVG assets
  - Zircon Hub UnityHub 风格布局优化计划.md
tests:
  - cargo fmt -p zircon_hub --check
  - cargo check -p zircon_hub --locked
doc_type: milestone-detail
---

# Hub And Editor SVG Resource Design

## Goal

Generate the image assets implied by the Hub and Editor reference screenshots as repository-owned SVG files, then wire the Hub shell to use those assets without changing the existing Hub data or launch behavior.

## Scope

This work covers Hub plus Editor-visible resource gaps. Hub receives the full resource pack because it is an active Slint surface that currently draws most icons and the brand mark as text or rectangles. Editor already owns a set of Ionicons SVG assets, so this milestone only adds one `editor-scifi-room.svg` preview resource as an available screenshot-style visual; it does not duplicate existing Editor icon packs or change retained-host runtime behavior.

All new generated resources are SVG. No PNG, JPG, WebP, external image-generation dependency, or binary art asset is introduced.

## Resource Layout

Hub resources live under `zircon_hub/assets`:

- `brand/zircon-mark.svg` is the faceted teal Zircon mark used by the title bar and project-card badge.
- `icons/nav/*.svg` contains the left-rail navigation symbols for Projects, Editor, Assets, Builds, Plugins, Cloud, Team, Learn, and Settings.
- `icons/actions/*.svg` contains Quick Actions icons for build, install, package, and open-editor.
- `icons/status/*.svg` contains running, success, warning, and error symbols used by the title-bar status pills.
- `covers/project-*.svg` contains five stylized project cover thumbnails matching the sample dashboard categories: fantasy valley, sci-fi outpost, desert ruins, forest cabin, and neon city.

Editor resources live under `zircon_editor/assets`:

- `editor-scifi-room.svg` is a static sci-fi room preview that can be used by documentation, future UI previews, or asset-browser examples. It is intentionally not wired into the Editor runtime in this milestone because the current Editor UI path is not Slint-based image composition.
- `icons/zircon_editor_shell/**` is a screenshot-derived Editor shell SVG pack grouped by surface: `toolbar`, `activity`, `scene`, `inspector`, `controls`, `status`, and `viewport`. It complements the existing Ionicons subset rather than replacing it, so future retained-host or template-driven surfaces can opt into the Zircon-specific icon names without changing existing Ionicons paths.

## Hub Data Flow

Slint component properties keep using `image` values instead of string paths so rendering remains simple in `app.slint` and `projects.slint`. Rust loads bundled SVGs through `slint::Image::load_from_path` and projects them into the existing data models.

`zircon_hub/src/app/view_model/media.rs` owns asset path selection and image loading. This keeps `view_model.rs` focused on snapshot projection and avoids adding media-path constants and fallback loading to an already large file.

The existing project-cover discovery remains authoritative for real user projects. For a recent project card, the Hub first tries `.zircon/cover.*`, `.zircon/thumbnail.*`, root-level cover files, and project asset thumbnails through `project_cover_path`. If none can be loaded, it falls back to one of the bundled cover SVGs by card index. This preserves the current real-project behavior while ensuring the dashboard has image resources even for missing local cover files.

## Slint Integration

The shared Slint data structs gain image fields only where they remove text icon placeholders:

- `NavItemData.icon-image` supplements the existing `icon` text field for left-rail SVG rendering.
- `QuickActionData.icon-image` supplements the existing `icon` text field for Quick Actions rendering.
- `HeaderStatusData.icon-image` supplements the existing `icon` text field for status pills.

The text fields remain for labels, accessibility fallback, and compile-stable data shape, but visible Hub icon surfaces render the SVG when provided. `BrandMark` renders `brand/zircon-mark.svg` directly from Slint using `@image-url`, because the brand mark is static UI chrome rather than Rust state.

## Testing And Acceptance

The milestone is accepted when:

- All listed SVG resources exist in the expected directories.
- Hub Slint compiles with the new image fields and static brand image.
- `cargo fmt -p zircon_hub --check` passes.
- `cargo check -p zircon_hub --locked` passes or any environmental blocker is reported with exact output.

No Editor build is required for the preview SVG because it is an unreferenced asset addition. If future work wires it into Editor UI behavior, that change must add Editor-side validation.
