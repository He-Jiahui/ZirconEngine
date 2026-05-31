# Hub Web Reference Spot Checks

These checks record the representative manual visual inspection set and result
evidence for the AI-directed, HTML/CSS-finalized Hub reference PNGs. They do not
replace the full 19-page export validation; they identify the pages that should
be opened first when judging visual quality against `docs/ui-and-layout/hub.png`.

Latest review: 2026-05-30. Result vocabulary is intentionally small: `Pass`
means the representative artifact was inspected for no clipped text, no overlap,
matching density, stable panel hierarchy, and consistent button/badge styling
for the listed concern.

| Artifact | Page id | Inspect for | Result | Evidence |
| --- | --- | --- | --- | --- |
| `hub.png` | `projects-dashboard` | Fixed Projects Dashboard pixel reference, topbar/sidebar/workspace/bottom-strip proportions, no page replacement. | Pass | Source reference remains unchanged and `validate-visuals.mjs` compares the dashboard web capture against it. |
| `hub-editor.png` | `hub-editor` | Main navigation page density, shared panel hierarchy, action buttons, and no clipped controls. | Pass | Opened through `index.html?page=hub-editor`; representative main-page density checked against the shared Hub chrome. |
| `hub-assets.png` | `hub-assets` | Catalog/list page density, metadata alignment, shared row styling, and no text overflow. | Pass | Opened through `index.html?page=hub-assets`; catalog rows, trailing metadata, and toolbar controls remain inside the fixed canvas. |
| `hub-projects-browser.png` | `hub-projects-browser` | All Projects table header/row alignment, selected-row full-width highlight, and stable browser/detail structure. | Pass | Regenerated from `index.html?page=hub-projects-browser`; header cells, every row's cells, and selected-row edges align across the main browser panel, including the `1915x508` design-review crop. |
| `hub-projects-detail-delete-confirm.png` | `hub-projects-detail-delete-confirm` | Project Detail modal/confirmation overlay position, destructive action state, and no workspace reflow. | Pass | Opened through `index.html?page=hub-projects-detail-delete-confirm`; modal sits above the detail page without shifting the workspace. |
| `hub-source-engine-popup.png` | `hub-source-engine-popup` | Header Source Engine popup alignment, row height, and clearance above the first project-card row. | Pass | Opened through `index.html?page=hub-source-engine-popup`; popup aligns below the header selector and clears the card row. |
| `hub-state-empty.png` | `hub-state-empty` | Shared empty-state placement, panel hierarchy, and non-overlapping content. | Pass | Opened through `index.html?page=hub-state-empty`; empty-state copy and import action are centered inside the panel hierarchy. |
| `hub-state-error.png` | `hub-state-error` | Recoverable error-state styling, action affordance, and consistent status color use. | Pass | Opened through `index.html?page=hub-state-error`; error tone, retry action, and supporting text remain readable and non-overlapping. |
