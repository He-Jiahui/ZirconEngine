# Structure Sign-Off Checklist

Manual status: pending.

This checklist is for human approval of the overall Hub structure against
`docs/ui-and-layout/hub.png`. It intentionally stays pending until the primary
structure screenshot has been reviewed.

## Sign-Off Items

| Check | Primary artifact | Target region | Required decision | Manual status |
| --- | --- | --- | --- | --- |
| `shell-frame` | `hub-design-structure-layout.png` | Topbar, Sidebar, Workspace, and Bottom strip in `docs/ui-and-layout/hub.png` | Shell regions keep the same ownership and proportions before local functional details are judged. | pending |
| `topbar-status-actions` | `hub-design-structure-layout.png` | Engine selector, run statuses, utility icons, account, and window controls in `docs/ui-and-layout/hub.png` | Global commands stay in the Topbar and do not consume Workspace content area. | pending |
| `sidebar-navigation` | `hub-design-structure-layout.png` | Projects through Settings navigation and Engine Status card in `docs/ui-and-layout/hub.png` | Navigation changes only replace Workspace content and preserve the Sidebar owner boundary. | pending |
| `workspace-projects` | `hub-design-functional-details.png` | Projects title, search/filter controls, project cards, recent list, and quick actions in `docs/ui-and-layout/hub.png` | Functional density supports the accepted shell structure and never overrides the structure-first decision. | pending |
| `overlay-layer` | `hub-design-structure-supplement.png` | Project menus, sort/filter menus, account menu, and confirm overlays in `docs/ui-and-layout/hub.png` | Overlays float above the shell without resizing Topbar, Sidebar, Workspace, or Bottom strip. | pending |
| `bottom-state-strip` | `hub-design-structure-layout.png` | Button States strip in `docs/ui-and-layout/hub.png` | The bottom strip remains a separate state-review band and does not merge into the Workspace. | pending |
