# Reference Alignment Matrix

This matrix is the human-readable companion to the `reference_alignment_checks`
fields in `manifest.json` and `structure-review-packet.json`. It defines what
must be compared against `docs/ui-and-layout/hub.png` before local functional
details can be accepted.

## Structure-First Rule

Check the target reference in this order: shell ownership first, global command
structure second, navigation and workspace ownership next, overlays after the
base shell, and bottom state strip before local content polish. The functional
details board can support the decision, but it must not override the primary
structure board.

## Alignment Checks

| Check | Target region | Primary artifact | Focus | Acceptance rule |
| --- | --- | --- | --- | --- |
| `shell-frame` | Topbar, Sidebar, Workspace, and Bottom strip in `docs/ui-and-layout/hub.png` | `hub-design-structure-layout.png` | overall-interaction-structure-layout | Shell regions keep the same ownership and proportions before local functional details are judged. |
| `topbar-status-actions` | Engine selector, run statuses, utility icons, account, and window controls in `docs/ui-and-layout/hub.png` | `hub-design-structure-layout.png` | topbar-command-structure | Global commands stay in the Topbar and do not consume Workspace content area. |
| `sidebar-navigation` | Projects through Settings navigation and Engine Status card in `docs/ui-and-layout/hub.png` | `hub-design-structure-layout.png` | navigation-ownership | Navigation changes only replace Workspace content and preserve the Sidebar owner boundary. |
| `workspace-projects` | Projects title, search/filter controls, project cards, recent list, and quick actions in `docs/ui-and-layout/hub.png` | `hub-design-functional-details.png` | workspace-content-density | Functional density supports the accepted shell structure and never overrides the structure-first decision. |
| `overlay-layer` | Project menus, sort/filter menus, account menu, and confirm overlays in `docs/ui-and-layout/hub.png` | `hub-design-structure-supplement.png` | overlay-ownership | Overlays float above the shell without resizing Topbar, Sidebar, Workspace, or Bottom strip. |
| `bottom-state-strip` | Button States strip in `docs/ui-and-layout/hub.png` | `hub-design-structure-layout.png` | bottom-strip-ownership | The bottom strip remains a separate state-review band and does not merge into the Workspace. |
