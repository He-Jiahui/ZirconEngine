export const DESIGN_BOARD_SOURCE = "docs/ui-and-layout/hub-design-board/index.html";
export const DESIGN_BOARD_MANIFEST = "docs/ui-and-layout/hub-design-board/manifest.json";
export const DESIGN_BOARD_MANIFEST_SCHEMA = "docs/ui-and-layout/hub-design-board/manifest.schema.json";
export const DESIGN_BOARD_EXPORT_METADATA = "docs/ui-and-layout/hub-design-board/export-metadata.json";
export const HUB_REFERENCE_TARGET = "docs/ui-and-layout/hub.png";
export const STRUCTURE_REVIEW_CHECKLIST = "docs/ui-and-layout/hub-design-board/STRUCTURE_REVIEW.md";
export const STRUCTURE_COVERAGE_MATRIX = "docs/ui-and-layout/hub-design-board/STRUCTURE_COVERAGE_MATRIX.md";
export const STRUCTURE_GEOMETRY_EVIDENCE = "docs/ui-and-layout/hub-design-board/STRUCTURE_GEOMETRY_EVIDENCE.md";
export const STRUCTURE_GEOMETRY_BASELINE = "docs/ui-and-layout/hub-design-board/structure-geometry-baseline.json";
export const STRUCTURE_RESPONSIVE_BASELINE = "docs/ui-and-layout/hub-design-board/structure-responsive-baseline.json";
export const STRUCTURE_REVIEW_ROUTE_BASELINE = "docs/ui-and-layout/hub-design-board/structure-review-route-baseline.json";
export const STRUCTURE_OVERLAY_BASELINE = "docs/ui-and-layout/hub-design-board/structure-overlay-baseline.json";
export const STRUCTURE_REFERENCE_ROUTE_BASELINE = "docs/ui-and-layout/hub-design-board/structure-reference-route-baseline.json";
export const STRUCTURE_REVIEW_GUIDE = "docs/ui-and-layout/hub-design-board/STRUCTURE_REVIEW_GUIDE.md";
export const DESIGN_BOARD_REVIEW_INDEX = "docs/ui-and-layout/hub-design-board/REVIEW_INDEX.md";
export const STRUCTURE_TO_REFERENCE_MAP = "docs/ui-and-layout/hub-design-board/STRUCTURE_TO_REFERENCE_MAP.md";
export const REFERENCE_ALIGNMENT_MATRIX = "docs/ui-and-layout/hub-design-board/REFERENCE_ALIGNMENT_MATRIX.md";
export const STRUCTURE_SIGNOFF_CHECKLIST = "docs/ui-and-layout/hub-design-board/STRUCTURE_SIGNOFF_CHECKLIST.md";
export const STRUCTURE_DECISION_LOG = "docs/ui-and-layout/hub-design-board/STRUCTURE_DECISION_LOG.md";
export const STRUCTURE_REVIEW_STATUS = "docs/ui-and-layout/hub-design-board/STRUCTURE_REVIEW_STATUS.md";
export const STRUCTURE_ACCEPTANCE_RECORD = "docs/ui-and-layout/hub-design-board/STRUCTURE_ACCEPTANCE_RECORD.md";
export const STRUCTURE_REVIEW_PACKET = "docs/ui-and-layout/hub-design-board/structure-review-packet.json";
export const STRUCTURE_REVIEW_PACKET_SCHEMA = "docs/ui-and-layout/hub-design-board/structure-review-packet.schema.json";

export const DESIGN_BOARD_EXPORT_HASH_INPUTS = [
  DESIGN_BOARD_SOURCE,
  "docs/ui-and-layout/hub-design-board/styles.css",
  "docs/ui-and-layout/hub-design-board/board-registry.mjs",
];

export const STRUCTURE_REVIEW_REQUIRED_TEXT = [
  "Overall Structure Checklist",
  "Shell frame",
  "Navigation",
  "Workspace",
  "Overlay layer",
  "State layer",
  "Responsive structure",
  "Bottom strip",
  "Functional Detail Checklist",
  "Acceptance Order",
  "Automated Geometry Guard",
];

export const STRUCTURE_COVERAGE_REQUIRED_ITEMS = [
  "Shell frame",
  "Navigation",
  "Workspace",
  "Overlay layer",
  "State layer",
  "Responsive structure",
  "Bottom strip",
  "Functional detail",
  "Review Rule",
];

export const STRUCTURE_COVERAGE_EXPECTED_ROWS = [
  {
    reviewItem: "Shell frame",
    primaryArtifact: "hub-design-structure-layout.png",
    secondaryArtifact: "hub-design-structure-supplement.png",
  },
  {
    reviewItem: "Navigation",
    primaryArtifact: "hub-design-structure-layout.png",
    secondaryArtifact: "hub-design-structure-supplement.png",
  },
  {
    reviewItem: "Workspace",
    primaryArtifact: "hub-design-structure-layout.png",
    secondaryArtifact: "hub-design-functional-details.png",
  },
  {
    reviewItem: "Overlay layer",
    primaryArtifact: "hub-design-structure-layout.png",
    secondaryArtifact: "hub-design-structure-supplement.png",
  },
  {
    reviewItem: "State layer",
    primaryArtifact: "hub-design-structure-supplement.png",
    secondaryArtifact: "hub-design-functional-details.png",
  },
  {
    reviewItem: "Responsive structure",
    primaryArtifact: "hub-design-structure-supplement.png",
    secondaryArtifact: "hub-design-structure-layout.png",
  },
  {
    reviewItem: "Bottom strip",
    primaryArtifact: "hub-design-structure-layout.png",
    secondaryArtifact: "hub-design-structure-supplement.png",
  },
  {
    reviewItem: "Functional detail",
    primaryArtifact: "hub-design-functional-details.png",
    secondaryArtifact: "hub-design-structure-layout.png",
  },
];

export const STRUCTURE_GEOMETRY_REQUIRED_TEXT = [
  "Primary Shell Geometry",
  "Hub frame",
  "Topbar",
  "Sidebar",
  "Workspace",
  "Bottom strip",
  "Source Engine overlay",
  "Account overlay",
  "structure-geometry-baseline.json",
  "Structure Assertions",
  "Functional-content details remain secondary evidence",
];

export const STRUCTURE_GEOMETRY_BASELINE_REQUIRED_FIELDS = [
  "name",
  "source",
  "artifact",
  "canvas",
  "measurement_space",
  "dimension_strip",
  "regions",
  "relationships",
];

export const STRUCTURE_GEOMETRY_BASELINE_EXPECTED_REGIONS = [
  { id: "hub-frame", label: "Hub frame", left: 0, top: 0, width: 1100, height: 612, right: 1100, bottom: 612 },
  { id: "topbar", label: "Topbar", left: 1, top: 1, width: 1098, height: 58, right: 1099, bottom: 59 },
  { id: "sidebar", label: "Sidebar", left: 1, top: 59, width: 178, height: 552, right: 179, bottom: 611 },
  { id: "workspace", label: "Workspace", left: 179, top: 59, width: 920, height: 459, right: 1099, bottom: 518 },
  { id: "bottom-strip", label: "Bottom strip", left: 179, top: 518, width: 920, height: 93, right: 1099, bottom: 611 },
  { id: "source-engine-overlay", label: "Source Engine overlay", left: 253, top: 63, width: 245, height: 118, right: 498, bottom: 181 },
  { id: "account-overlay", label: "Account overlay", left: 841, top: 63, width: 190, height: 136, right: 1031, bottom: 199 },
];

export const STRUCTURE_RESPONSIVE_BASELINE_REQUIRED_FIELDS = [
  "name",
  "source",
  "artifact",
  "review_focus",
  "breakpoints",
  "invariants",
];

export const STRUCTURE_RESPONSIVE_BASELINE_EXPECTED_BREAKPOINTS = [
  {
    id: "desktop-1568x1003",
    label: "Desktop 1568x1003",
    sidebar_mode: "full-sidebar",
    workspace_rule: "Complete Topbar, Sidebar, Workspace, and Bottom strip are visible.",
  },
  {
    id: "compact-window",
    label: "Compact Window",
    sidebar_mode: "compact-sidebar",
    workspace_rule: "Workspace content compresses before shell ownership changes.",
  },
  {
    id: "collapsed-sidebar",
    label: "Collapsed Sidebar",
    sidebar_mode: "icon-sidebar",
    workspace_rule: "Sidebar width collapses while navigation ownership remains in the Sidebar lane.",
  },
];

export const STRUCTURE_REVIEW_ROUTE_BASELINE_REQUIRED_FIELDS = [
  "name",
  "source",
  "review_focus",
  "review_order",
  "blocking_rules",
];

export const STRUCTURE_REVIEW_ROUTE_BASELINE_EXPECTED_ORDER = [
  {
    step: 1,
    artifact_id: "structure",
    artifact: "docs/ui-and-layout/hub-design-structure-layout.png",
    focus: "overall-interaction-structure-layout",
    decision_gate: "overall-shell-accepted",
  },
  {
    step: 2,
    artifact_id: "flow",
    artifact: "docs/ui-and-layout/hub-design-structure-supplement.png",
    focus: "route-and-responsive-structure",
    decision_gate: "route-responsive-ownership-accepted",
  },
  {
    step: 3,
    artifact_id: "details",
    artifact: "docs/ui-and-layout/hub-design-functional-details.png",
    focus: "functional-content-local-detail",
    decision_gate: "local-content-reviewed-after-structure",
  },
];

export const STRUCTURE_REVIEW_ROUTE_BASELINE_BLOCKING_RULES = [
  "Do not approve functional-content-local-detail before overall-interaction-structure-layout is accepted.",
  "Do not let hub-design-functional-details.png change shell, Topbar, Sidebar, Workspace, overlay, or Bottom strip ownership.",
  "If structure and details disagree, revise structure artifacts before judging local detail.",
  "Manual sign-off remains pending until docs/ui-and-layout/hub-design-structure-layout.png is accepted against docs/ui-and-layout/hub.png.",
];

export const STRUCTURE_OVERLAY_BASELINE_REQUIRED_FIELDS = [
  "name",
  "source",
  "review_focus",
  "overlay_layers",
  "invariants",
];

export const STRUCTURE_OVERLAY_BASELINE_EXPECTED_LAYERS = [
  {
    id: "header-source-popup",
    owner: "topbar-global-commands",
    primary_artifact: "docs/ui-and-layout/hub-design-structure-layout.png",
    anchor_region: "Topbar Source Engine control",
    layout_effect: "floating-no-reflow",
  },
  {
    id: "header-account-menu",
    owner: "topbar-global-commands",
    primary_artifact: "docs/ui-and-layout/hub-design-structure-layout.png",
    anchor_region: "Topbar account control",
    layout_effect: "floating-no-reflow",
  },
  {
    id: "workspace-filter-sort-menus",
    owner: "workspace-page-owner",
    primary_artifact: "docs/ui-and-layout/hub-design-structure-supplement.png",
    anchor_region: "Workspace toolbar",
    layout_effect: "floating-no-reflow",
  },
  {
    id: "project-card-overflow-menu",
    owner: "workspace-page-owner",
    primary_artifact: "docs/ui-and-layout/hub-design-structure-supplement.png",
    anchor_region: "Workspace project card",
    layout_effect: "floating-no-reflow",
  },
  {
    id: "delete-confirm-overlay",
    owner: "overlay-layer-owner",
    primary_artifact: "docs/ui-and-layout/hub-design-structure-supplement.png",
    anchor_region: "Workspace modal overlay layer",
    layout_effect: "floating-no-reflow",
  },
];

export const STRUCTURE_OVERLAY_BASELINE_INVARIANTS = [
  "Overlays float above the accepted shell and never resize Topbar, Sidebar, Workspace, or Bottom strip.",
  "Header overlays remain owned by Topbar global commands even when they visually overlap Workspace.",
  "Workspace menus remain owned by Workspace page content and do not become global shell chrome.",
  "Confirm overlays use the overlay layer and do not change route ownership or page geometry.",
];

export const STRUCTURE_REFERENCE_ROUTE_BASELINE_REQUIRED_FIELDS = [
  "name",
  "source",
  "reference_target",
  "review_focus",
  "routes",
  "invariants",
];

export const STRUCTURE_REFERENCE_ROUTE_BASELINE_EXPECTED_ROUTES = [
  {
    id: "shell-frame",
    review_item: "Shell frame",
    design_artifact: "docs/ui-and-layout/hub-design-structure-layout.png",
    final_reference: "hub-web-reference-1568x1003.png",
    verify_rule: "Topbar, Sidebar, Workspace, and Bottom strip proportions match the accepted shell.",
  },
  {
    id: "navigation",
    review_item: "Navigation",
    design_artifact: "docs/ui-and-layout/hub-design-structure-layout.png",
    final_reference: "hub-editor.png",
    verify_rule: "Sidebar keeps shell geometry while switching global pages.",
  },
  {
    id: "projects-dashboard",
    review_item: "Projects dashboard",
    design_artifact: "docs/ui-and-layout/hub-design-structure-layout.png",
    final_reference: "hub-web-reference-1568x1003.png",
    verify_rule: "Search, cards, Recent Projects, and Quick Actions stay inside Workspace.",
  },
  {
    id: "projects-secondary-pages",
    review_item: "Projects secondary pages",
    design_artifact: "docs/ui-and-layout/hub-design-functional-details.png",
    final_reference: "hub-projects-new.png",
    verify_rule: "Secondary pages replace Workspace content only.",
  },
  {
    id: "project-browser",
    review_item: "Project browser",
    design_artifact: "docs/ui-and-layout/hub-design-functional-details.png",
    final_reference: "hub-projects-browser.png",
    verify_rule: "Browser list and toolbar remain Workspace-local.",
  },
  {
    id: "project-detail",
    review_item: "Project detail",
    design_artifact: "docs/ui-and-layout/hub-design-functional-details.png",
    final_reference: "hub-projects-detail.png",
    verify_rule: "Detail action areas stay inside Workspace and overlays remain separate.",
  },
  {
    id: "header-source-overlay",
    review_item: "Header Source overlay",
    design_artifact: "docs/ui-and-layout/hub-design-structure-layout.png",
    final_reference: "hub-source-engine-popup.png",
    verify_rule: "Source Engine popup floats without reflowing shell regions.",
  },
  {
    id: "user-menu-overlay",
    review_item: "User menu overlay",
    design_artifact: "docs/ui-and-layout/hub-design-structure-layout.png",
    final_reference: "hub-user-menu.png",
    verify_rule: "User menu floats from Topbar and does not resize Workspace.",
  },
  {
    id: "browser-filter-menu",
    review_item: "Browser filter menu",
    design_artifact: "docs/ui-and-layout/hub-design-structure-supplement.png",
    final_reference: "hub-projects-browser-filter-menu.png",
    verify_rule: "Filter menu overlays the Workspace toolbar area.",
  },
  {
    id: "browser-sort-menu",
    review_item: "Browser sort menu",
    design_artifact: "docs/ui-and-layout/hub-design-structure-supplement.png",
    final_reference: "hub-projects-browser-sort-menu.png",
    verify_rule: "Sort menu overlays the Workspace toolbar area.",
  },
  {
    id: "delete-confirm",
    review_item: "Delete confirm",
    design_artifact: "docs/ui-and-layout/hub-design-structure-supplement.png",
    final_reference: "hub-projects-detail-delete-confirm.png",
    verify_rule: "Confirm layer remains an overlay over the detail page.",
  },
  {
    id: "state-empty",
    review_item: "Empty state",
    design_artifact: "docs/ui-and-layout/hub-design-structure-supplement.png",
    final_reference: "hub-state-empty.png",
    verify_rule: "Empty state reuses Workspace canvas without shell changes.",
  },
  {
    id: "state-loading",
    review_item: "Loading state",
    design_artifact: "docs/ui-and-layout/hub-design-structure-supplement.png",
    final_reference: "hub-state-loading.png",
    verify_rule: "Loading state reuses Workspace canvas without shell changes.",
  },
  {
    id: "state-error",
    review_item: "Error state",
    design_artifact: "docs/ui-and-layout/hub-design-structure-supplement.png",
    final_reference: "hub-state-error.png",
    verify_rule: "Error state reuses Workspace canvas without shell changes.",
  },
];

export const STRUCTURE_REFERENCE_ROUTE_BASELINE_INVARIANTS = [
  "Every route starts from a design-board structure decision before checking final visual detail.",
  "docs/ui-and-layout/hub.png remains the target-ui-reference for shell structure and palette comparison.",
  "Final web-reference PNGs confirm visual detail and cannot override the accepted design-board structure.",
];

export const STRUCTURE_REVIEW_GUIDE_REQUIRED_TEXT = [
  "Review Inputs",
  "Manual Checklist",
  "Rejection Signals",
  "Review Order",
  "Shell proportions",
  "Navigation ownership",
  "Workspace replacement",
  "Overlay ownership",
  "Overlay baseline",
  "Route grouping",
  "Responsive ownership",
  "Review route baseline",
  "Reference route baseline",
  "Functional density",
];

export const DESIGN_BOARD_REVIEW_INDEX_REQUIRED_TEXT = [
  "Screenshot Artifacts",
  "Review Documents",
  "Validation Commands",
  "Acceptance Rule",
  "overall interaction structure layout first",
  "hub-design-structure-layout.png",
  "STRUCTURE_REVIEW_GUIDE.md",
  "STRUCTURE_GEOMETRY_EVIDENCE.md",
  "structure-geometry-baseline.json",
  "structure-responsive-baseline.json",
  "structure-review-route-baseline.json",
  "structure-overlay-baseline.json",
  "structure-reference-route-baseline.json",
  "structure-review-packet.json",
  "structure-review-packet.schema.json",
  "manifest.schema.json",
  "REFERENCE_ALIGNMENT_MATRIX.md",
  "STRUCTURE_SIGNOFF_CHECKLIST.md",
  "STRUCTURE_DECISION_LOG.md",
  "export-metadata.json",
  "validate-design-board.mjs",
];

export const STRUCTURE_REVIEW_STATUS_REQUIRED_TEXT = [
  "Current Structure Review Status",
  "Review Priority",
  "Primary Review Evidence",
  "Automated Gate Coverage",
  "Manual Structure Checks",
  "Acceptance State",
  "overall interaction structure layout first",
  "hub-design-structure-layout.png",
  "STRUCTURE_GEOMETRY_EVIDENCE.md",
  "structure-geometry-baseline.json",
  "structure-responsive-baseline.json",
  "structure-review-route-baseline.json",
  "structure-overlay-baseline.json",
  "structure-reference-route-baseline.json",
  "structure-review-packet.json",
  "structure-review-packet.schema.json",
  "manifest.schema.json",
  "REFERENCE_ALIGNMENT_MATRIX.md",
  "STRUCTURE_SIGNOFF_CHECKLIST.md",
  "STRUCTURE_DECISION_LOG.md",
  "export-metadata.json",
  "validate-design-board.mjs",
];

export const STRUCTURE_ACCEPTANCE_RECORD_REQUIRED_TEXT = [
  "Structure Acceptance Record",
  "Review Result",
  "Manual sign-off pending",
  "Automated Evidence",
  "Manual Review Items",
  "Acceptance Boundary",
  "overall interaction structure layout first",
  "hub-design-structure-layout.png",
  "STRUCTURE_REVIEW_STATUS.md",
  "structure-geometry-baseline.json",
  "structure-responsive-baseline.json",
  "structure-review-route-baseline.json",
  "structure-overlay-baseline.json",
  "structure-reference-route-baseline.json",
  "structure-review-packet.json",
  "structure-review-packet.schema.json",
  "manifest.schema.json",
  "REFERENCE_ALIGNMENT_MATRIX.md",
  "STRUCTURE_SIGNOFF_CHECKLIST.md",
  "STRUCTURE_DECISION_LOG.md",
  "export-metadata.json",
  "validate-design-board.mjs",
];

export const STRUCTURE_REVIEW_PACKET_REQUIRED_COMMANDS = [
  "node docs/ui-and-layout/hub-design-board/export-design-board.mjs",
  "node docs/ui-and-layout/hub-design-board/validate-design-board.mjs",
  "node docs/ui-and-layout/hub-web-reference/validate-visuals.mjs",
  "node docs/ui-and-layout/hub-web-reference/validate-interactions.mjs",
];

export const STRUCTURE_REVIEW_PACKET_REVIEW_SEQUENCE = [
  {
    step: 1,
    artifact_id: "structure",
    focus: "overall-interaction-structure-layout",
    decision_scope: "Accept or revise shell proportions, navigation ownership, Workspace ownership, overlays, and bottom strip first.",
  },
  {
    step: 2,
    artifact_id: "flow",
    focus: "route-and-responsive-structure",
    decision_scope: "Use route grouping, overlay ownership, responsive ownership, and collapsed-sidebar behavior as secondary structure evidence.",
  },
  {
    step: 3,
    artifact_id: "details",
    focus: "functional-content-local-detail",
    decision_scope: "Confirm local Projects content density only after the primary structure artifacts are acceptable.",
  },
];

export const REFERENCE_ALIGNMENT_CHECKS = [
  {
    id: "shell-frame",
    target_region: "Topbar, Sidebar, Workspace, and Bottom strip in docs/ui-and-layout/hub.png",
    primary_artifact: "hub-design-structure-layout.png",
    review_focus: "overall-interaction-structure-layout",
    acceptance_rule: "Shell regions keep the same ownership and proportions before local functional details are judged.",
  },
  {
    id: "topbar-status-actions",
    target_region: "Engine selector, run statuses, utility icons, account, and window controls in docs/ui-and-layout/hub.png",
    primary_artifact: "hub-design-structure-layout.png",
    review_focus: "topbar-command-structure",
    acceptance_rule: "Global commands stay in the Topbar and do not consume Workspace content area.",
  },
  {
    id: "sidebar-navigation",
    target_region: "Projects through Settings navigation and Engine Status card in docs/ui-and-layout/hub.png",
    primary_artifact: "hub-design-structure-layout.png",
    review_focus: "navigation-ownership",
    acceptance_rule: "Navigation changes only replace Workspace content and preserve the Sidebar owner boundary.",
  },
  {
    id: "workspace-projects",
    target_region: "Projects title, search/filter controls, project cards, recent list, and quick actions in docs/ui-and-layout/hub.png",
    primary_artifact: "hub-design-functional-details.png",
    review_focus: "workspace-content-density",
    acceptance_rule: "Functional density supports the accepted shell structure and never overrides the structure-first decision.",
  },
  {
    id: "overlay-layer",
    target_region: "Project menus, sort/filter menus, account menu, and confirm overlays in docs/ui-and-layout/hub.png",
    primary_artifact: "hub-design-structure-supplement.png",
    review_focus: "overlay-ownership",
    acceptance_rule: "Overlays float above the shell without resizing Topbar, Sidebar, Workspace, or Bottom strip.",
  },
  {
    id: "bottom-state-strip",
    target_region: "Button States strip in docs/ui-and-layout/hub.png",
    primary_artifact: "hub-design-structure-layout.png",
    review_focus: "bottom-strip-ownership",
    acceptance_rule: "The bottom strip remains a separate state-review band and does not merge into the Workspace.",
  },
];

export const STRUCTURE_REVIEW_PACKET_REQUIRED_FIELDS = [
  "name",
  "schema",
  "source",
  "canvas",
  "review_priority",
  "manual_signoff",
  "reference_target",
  "reference_alignment_checks",
  "reference_alignment_matrix",
  "structure_geometry_baseline",
  "structure_responsive_baseline",
  "structure_review_route_baseline",
  "structure_overlay_baseline",
  "structure_reference_route_baseline",
  "structure_signoff_checklist",
  "structure_decision_log",
  "artifacts",
  "review_sequence",
  "manual_review_items",
  "support_documents",
  "validation_commands",
  "acceptance_boundary",
];

export const STRUCTURE_TO_REFERENCE_REQUIRED_TEXT = [
  "Reference Baseline",
  "Structure Map",
  "Review Rule",
  "Shell frame",
  "Navigation",
  "Projects dashboard",
  "Header overlays",
  "Browser sort menu",
  "State pages",
  "hub-web-reference-1568x1003.png",
  "hub-projects-detail-delete-confirm.png",
  "structure-reference-route-baseline.json",
  "Do not let final functional detail override",
];

export const REFERENCE_ALIGNMENT_MATRIX_REQUIRED_TEXT = [
  "Reference Alignment Matrix",
  "docs/ui-and-layout/hub.png",
  "Structure-First Rule",
  "shell-frame",
  "topbar-status-actions",
  "sidebar-navigation",
  "workspace-projects",
  "overlay-layer",
  "bottom-state-strip",
  "hub-design-structure-layout.png",
  "hub-design-structure-supplement.png",
  "hub-design-functional-details.png",
];

export const STRUCTURE_SIGNOFF_CHECKLIST_REQUIRED_TEXT = [
  "Structure Sign-Off Checklist",
  "Manual status",
  "pending",
  "docs/ui-and-layout/hub.png",
  "shell-frame",
  "topbar-status-actions",
  "sidebar-navigation",
  "workspace-projects",
  "overlay-layer",
  "bottom-state-strip",
  "hub-design-structure-layout.png",
  "hub-design-structure-supplement.png",
  "hub-design-functional-details.png",
];

export const STRUCTURE_DECISION_LOG_REQUIRED_TEXT = [
  "Structure Decision Log",
  "Decision order",
  "overall interaction structure layout first",
  "Decision state",
  "automated-evidence-ready",
  "manual-signoff-pending",
  "docs/ui-and-layout/hub.png",
  "shell-frame",
  "topbar-status-actions",
  "sidebar-navigation",
  "workspace-projects",
  "overlay-layer",
  "bottom-state-strip",
  "hub-design-structure-layout.png",
  "hub-design-structure-supplement.png",
  "hub-design-functional-details.png",
];

export const DESIGN_BOARD_LIST = [
  {
    id: "structure",
    output: "hub-design-structure-layout.png",
    category: "primary-structure",
    requiredText: ["总体结构布局", "Canvas 1568x1003", "Workspace 1345x792", "Shell 骨架稳定", "Bottom State Strip 137px"],
  },
  {
    id: "flow",
    output: "hub-design-structure-supplement.png",
    category: "primary-structure",
    requiredText: ["总体结构补充", "总体交互流", "Desktop 1568x1003", "Compact Window", "Collapsed Sidebar", "结构不变量"],
  },
  {
    id: "details",
    output: "hub-design-functional-details.png",
    category: "secondary-functional-detail",
    requiredText: ["功能内容局部", "Projects Dashboard", "局部内容图的用途"],
  },
];
