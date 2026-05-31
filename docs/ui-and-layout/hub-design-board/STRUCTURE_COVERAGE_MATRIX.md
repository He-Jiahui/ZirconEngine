# Hub Design Board Structure Coverage Matrix

This matrix maps each overall-structure review item to the design-board artifact
that should be inspected first. The primary review path is structural; local
functional content is only secondary evidence.

| Review item | Primary artifact | Secondary artifact | Inspect for |
| --- | --- | --- | --- |
| Shell frame | `hub-design-structure-layout.png` | `hub-design-structure-supplement.png` | Topbar, Sidebar, Workspace, and Bottom strip keep fixed ownership. |
| Navigation | `hub-design-structure-layout.png` | `hub-design-structure-supplement.png` | Sidebar routes replace Workspace content only. |
| Workspace | `hub-design-structure-layout.png` | `hub-design-functional-details.png` | Page title, actions, toolbar, cards, tables, and secondary pages stay inside Workspace. |
| Overlay layer | `hub-design-structure-layout.png` | `hub-design-structure-supplement.png` | Source Engine, User Menu, Filter, Sort, and Delete Confirm float without reflow. |
| State layer | `hub-design-structure-supplement.png` | `hub-design-functional-details.png` | Empty, Loading, and Error reuse Workspace state canvas. |
| Responsive structure | `hub-design-structure-supplement.png` | `hub-design-structure-layout.png` | Compact window and collapsed sidebar preserve shell ownership. |
| Bottom strip | `hub-design-structure-layout.png` | `hub-design-structure-supplement.png` | Button-state samples remain outside page content. |
| Functional detail | `hub-design-functional-details.png` | `hub-design-structure-layout.png` | Page-content coverage supports, but does not override, the structure artifacts. |

## Review Rule

Approve structure only after every review item above is checked against its
primary artifact. Use the secondary artifact to clarify the decision, not to
replace the primary artifact.
