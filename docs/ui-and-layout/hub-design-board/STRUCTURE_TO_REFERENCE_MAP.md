# Hub Structure To Reference Map

This map connects the design-board structure review items to the final
web-reference PNG set. Use it when a structural concern needs to be checked
against the full visual reference pages.

## Reference Baseline

- Source baseline: `docs/ui-and-layout/hub.png`
- Web reference source: `docs/ui-and-layout/hub-web-reference/index.html`
- Design-board source: `docs/ui-and-layout/hub-design-board/index.html`
- Machine-readable route baseline: `structure-reference-route-baseline.json`
- Canvas: `1568x1003`

## Structure Map

| Structure item | Design-board artifact | Final reference artifact | Verify in reference |
| --- | --- | --- | --- |
| Shell frame | `hub-design-structure-layout.png` | `hub-web-reference-1568x1003.png` | Topbar, Sidebar, Workspace, and Bottom strip proportions. |
| Navigation | `hub-design-structure-layout.png` | `hub-editor.png` | Sidebar keeps shell geometry while switching global pages. |
| Projects dashboard | `hub-design-structure-layout.png` | `hub-web-reference-1568x1003.png` | Search, cards, Recent Projects, and Quick Actions stay inside Workspace. |
| Projects secondary pages | `hub-design-functional-details.png` | `hub-projects-new.png` | Secondary pages replace Workspace content only. |
| Project browser | `hub-design-functional-details.png` | `hub-projects-browser.png` | Browser list and toolbar remain Workspace-local. |
| Project detail | `hub-design-functional-details.png` | `hub-projects-detail.png` | Detail action areas stay inside Workspace and overlays remain separate. |
| Header overlays | `hub-design-structure-layout.png` | `hub-source-engine-popup.png` | Source Engine popup floats without reflowing shell regions. |
| User menu overlay | `hub-design-structure-layout.png` | `hub-user-menu.png` | User menu floats from Topbar and does not resize Workspace. |
| Browser filter menu | `hub-design-structure-supplement.png` | `hub-projects-browser-filter-menu.png` | Filter menu overlays the Workspace toolbar area. |
| Browser sort menu | `hub-design-structure-supplement.png` | `hub-projects-browser-sort-menu.png` | Sort menu overlays the Workspace toolbar area. |
| Delete confirm | `hub-design-structure-supplement.png` | `hub-projects-detail-delete-confirm.png` | Confirm layer remains an overlay over the detail page. |
| State pages | `hub-design-structure-supplement.png` | `hub-state-empty.png` | Empty state reuses Workspace canvas without shell changes. |
| Loading state | `hub-design-structure-supplement.png` | `hub-state-loading.png` | Loading state reuses Workspace canvas without shell changes. |
| Error state | `hub-design-structure-supplement.png` | `hub-state-error.png` | Error state reuses Workspace canvas without shell changes. |

## Review Rule

Start with the design-board artifact. Use the final reference artifact only to
confirm visual density, content copy, icon treatment, and final color/spacing.
Do not let final functional detail override the design-board structure decision.
