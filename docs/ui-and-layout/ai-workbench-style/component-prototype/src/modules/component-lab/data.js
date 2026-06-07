export const componentNav = [
  ["Input Atoms", "buttons, fields, checkbox, radio, toggle", "component-lab-right:inputs"],
  ["Data Display", "list, tree, table, row actions", "component-lab-right:data-display"],
  ["Feedback", "alerts, tooltip, toast", "component-lab-right:feedback"],
  ["Overlays", "popup, dropdown, drawer", "component-lab-right:overlays"],
  ["Surfaces", "drawer, window, panel group", "component-lab-right:surfaces"],
  ["Layout Rules", "stack, cluster, grid, flex alignment", "component-lab-right:layout"]
];

export const componentCoverage = [
  ["Button", "components/inputs/buttons.js", "Primary / secondary / icon / danger", "Atom"],
  ["Text Field", "components/inputs/fields.js", "focus, edit, disabled", "Atom"],
  ["Selection", "components/inputs/selection-controls.js", "checkbox, radio, toggle", "Atom"],
  ["Tabs", "components/inputs/tabs.js", "tabs and segmented controls", "Atom"],
  ["Dropdown", "components/inputs/dropdowns.js", "trigger plus popup menu", "Atom"],
  ["Slider", "components/inputs/sliders.js", "value, range, stepped", "Atom"],
  ["List", "components/data/list-view.js", "selected and disabled rows", "Collection"],
  ["Tree", "components/data/tree-view.js", "depth and selection", "Collection"],
  ["Table", "components/data/table-view.js", "row command identity", "Collection"],
  ["Popup", "components/overlays/menu.js", "transient command rows", "Overlay"],
  ["Feedback", "components/feedback/*", "alert, tooltip, toast", "Feedback"],
  ["Surface", "components/surfaces/*", "drawer, panel, window, viewport", "Surface"]
];

export const layoutCoverage = [
  ["Stack", "vertical spacing", "top / center / bottom"],
  ["Cluster", "inline tools", "left / center / right"],
  ["Grid", "panel matrices", "fixed plus stretch tracks"],
  ["Panel Group", "tabbed regions", "active panel route"],
  ["Drawer Surface", "side and bottom regions", "bounded overflow"],
  ["Responsive Shell", "desktop to narrow", "no horizontal overflow"]
];

export const labAssets = [
  "button atom",
  "input field",
  "selection controls",
  "data rows",
  "popup layer",
  "drawer surface"
];

export const labToolbarPanels = new Map([
  ["audit-inputs", "component-lab-right:inputs"],
  ["audit-collections", "component-lab-main:collections"],
  ["audit-surfaces", "component-lab-main:surfaces"],
  ["responsive", "module-bottom-component-lab:responsive"],
  ["native-handoff", "component-lab-right:native-handoff"]
]);
