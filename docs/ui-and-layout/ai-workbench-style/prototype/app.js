import { renderStageBody } from "./stage-renderers.js";
import { pages } from "./page-data.js";
import { escapeAttr, escapeHtml } from "./view-utils.js";

const compactQuery = window.matchMedia("(max-width: 1100px)");

const layoutPresets = {
  authoring: {
    label: "Authoring",
    left: "regular",
    right: "regular",
    bottom: "regular",
    note: "Default docked editor layout: both side drawers visible, bottom output medium.",
  },
  review: {
    label: "Review",
    left: "compact",
    right: "wide",
    bottom: "regular",
    note: "Inspection-heavy layout: right drawer gets more room for properties and validation.",
  },
  focus: {
    label: "Focus",
    left: "compact",
    right: "compact",
    bottom: "compact",
    note: "Document-first layout: drawers stay present but narrow for viewport or graph work.",
  },
  debug: {
    label: "Debug",
    left: "regular",
    right: "regular",
    bottom: "tall",
    note: "Output-heavy layout: console, timeline, trace, and validation panes gain height.",
  },
  custom: {
    label: "Custom",
    left: "regular",
    right: "regular",
    bottom: "regular",
    note: "Manual layout sizing selected from the Taffy inspector controls.",
  },
};

const state = {
  pageId: normalizePage(location.hash.replace("#", "")) ?? "scene",
  tool: "Select",
  leftOpen: !compactQuery.matches,
  rightOpen: !compactQuery.matches,
  outputOpen: true,
  layoutPreset: "authoring",
  leftSize: "regular",
  rightSize: "regular",
  bottomSize: "regular",
  selectedRegion: "center-stage",
  dockPreview: "none",
  layoutTrayOpen: false,
  activePanel: {},
  activeBottom: {},
  overlay: null,
};

const app = document.querySelector("#app");

window.addEventListener("hashchange", () => {
  const pageId = normalizePage(location.hash.replace("#", ""));
  if (pageId) {
    state.pageId = pageId;
    state.tool = page().tools[0];
    render();
  }
});

compactQuery.addEventListener("change", (event) => {
  state.leftOpen = !event.matches;
  state.rightOpen = !event.matches;
  render();
});

render();

function render() {
  app.dataset.ready = "true";
  app.className = [
    "app-root",
    state.leftOpen ? "is-left-open" : "is-left-closed",
    state.rightOpen ? "is-right-open" : "is-right-closed",
    state.outputOpen ? "is-output-open" : "is-output-closed",
    `left-${state.leftSize}`,
    `right-${state.rightSize}`,
    `bottom-${state.bottomSize}`,
    `dock-preview-${state.dockPreview}`,
    state.layoutTrayOpen ? "show-layout-tray" : "hide-layout-tray",
  ].join(" ");

  app.innerHTML = `
    <section class="workbench-shell" aria-label="Zircon editor workbench prototype">
      ${renderTopBar()}
      ${renderPageTabs()}
      <main class="workbench-grid">
        ${renderRail()}
        ${renderDrawerColumn("left", page().left)}
        ${renderCenter()}
        ${renderDrawerColumn("right", page().right)}
        ${renderBottom()}
        ${renderDockOverlay()}
      </main>
      ${renderStatusBar()}
      ${renderInspectorTray()}
      ${renderOverlay()}
    </section>
  `;

  bindEvents();
}

function renderTopBar() {
  return `
    <header class="top-bar">
      <button class="brand-button" data-action="command" aria-label="Open command palette">
        <span class="brand-mark">Zr</span>
        <span class="brand-title">Nebula Station</span>
      </button>
      <div class="command-group">
        ${toolbarButton("Save", "save")}
        ${toolbarButton("Undo", "undo")}
        ${toolbarButton("Redo", "redo")}
        ${toolbarButton("Play", "play", true)}
        ${toolbarButton("Build", "build")}
      </div>
      <div class="layout-preset-group" aria-label="Layout presets">
        ${Object.entries(layoutPresets)
          .map(
            ([id, preset]) =>
              `<button class="${state.layoutPreset === id ? "active" : ""}" data-layout-preset="${id}">${escapeHtml(preset.label)}</button>`
          )
          .join("")}
      </div>
      <div class="context-strip" aria-label="Current document path">
        ${page()
          .crumbs.map((crumb) => `<span>${escapeHtml(crumb)}</span>`)
          .join("<b>/</b>")}
      </div>
      <div class="view-toggles" aria-label="Layout controls">
        <button class="${state.leftOpen ? "active" : ""}" data-action="toggle-left">Left</button>
        <button class="${state.outputOpen ? "active" : ""}" data-action="toggle-output">Output</button>
        <button class="${state.rightOpen ? "active" : ""}" data-action="toggle-right">Right</button>
        <button data-action="menu">Menu</button>
      </div>
    </header>
  `;
}

function toolbarButton(label, action, primary = false) {
  return `<button class="tool-command ${primary ? "primary" : ""}" data-command="${action}">${label}</button>`;
}

function renderPageTabs() {
  return `
    <nav class="page-tabs" aria-label="Workbench pages">
      ${pages
        .map(
          (item) => `
            <a class="page-tab ${item.id === page().id ? "active" : ""}" href="#${item.id}" data-page="${item.id}">
              <span>${escapeHtml(item.title)}</span>
              <small>${escapeHtml(item.shortTitle)}</small>
            </a>
          `
        )
        .join("")}
      <button class="page-tab add-tab" data-action="command">+</button>
    </nav>
  `;
}

function renderRail() {
  return `
    <aside class="tool-rail" aria-label="Workbench tools">
      ${page()
        .tools.map(
          (tool) => `
            <button
              class="rail-tool ${state.tool === tool ? "active" : ""}"
              data-tool="${escapeAttr(tool)}"
              title="${escapeAttr(tool)}"
            >
              <span>${tool.slice(0, 2).toUpperCase()}</span>
            </button>
          `
        )
        .join("")}
      <span class="rail-spacer"></span>
      <button class="rail-tool" data-action="command" title="Command palette"><span>CM</span></button>
      <button class="rail-tool" data-action="menu" title="Settings"><span>ST</span></button>
    </aside>
  `;
}

function renderDrawerColumn(side, panels) {
  const openClass = side === "left" ? state.leftOpen : state.rightOpen;
  const region = side === "left" ? "left-drawer" : "right-drawer";
  return `
    <aside class="drawer-column ${side}-column ${openClass ? "" : "closed"} ${state.selectedRegion === region ? "selected-region" : ""}" data-side="${side}" data-region="${region}">
      <button class="region-hotspot" data-region="${region}">${side === "left" ? "Left Drawer" : "Right Drawer"}</button>
      ${panels.map((panel, index) => renderPanel(side, panel, index)).join("")}
    </aside>
  `;
}

function renderPanel(side, panel, index) {
  const key = `${page().id}:${side}:${index}`;
  const active = state.activePanel[key] ?? panel.id;
  const sibling = side === "left" ? page().left : page().right;
  const displayed = sibling.find((tab) => tab.id === active) ?? panel;
  return `
    <section class="dock-panel ${active === displayed.id ? "is-active" : ""}">
      <header class="panel-header">
        <div>
          <strong>${escapeHtml(displayed.title)}</strong>
          <span>${escapeHtml(displayed.eyebrow)}</span>
        </div>
        <div class="panel-actions">
          ${sibling
            .map(
              (tab) =>
                `<button class="${active === tab.id ? "active" : ""}" data-panel-key="${key}" data-panel="${tab.id}">${escapeHtml(
                  tab.title.split(" ")[0]
                )}</button>`
            )
            .join("")}
        </div>
      </header>
      <div class="panel-body">${renderPanelContent(displayed)}</div>
    </section>
  `;
}

function renderPanelContent(panel) {
  if (panel.tree) {
    return `
      <label class="search-field">
        <span>Search</span>
        <input value="" placeholder="Filter ${escapeAttr(panel.title.toLowerCase())}" />
      </label>
      <ul class="tree-list">
        ${panel.tree
          .map(
            (item, index) => `
              <li class="${item === "TestCube" || item === "SM_Block_02" ? "selected" : ""}">
                <span class="twisty">${index % 3 === 0 ? "v" : ">"}</span>
                <span class="node-icon"></span>
                <span>${escapeHtml(item)}</span>
              </li>
            `
          )
          .join("")}
      </ul>
    `;
  }

  if (panel.table) {
    return `
      <div class="compact-table">
        ${panel.table
          .map(
            (row, index) => `
              <div class="table-row ${index === 2 || row[1] === "Selected" ? "selected" : ""}">
                <span>${escapeHtml(row[0])}</span>
                <em>${escapeHtml(row[1])}</em>
              </div>
            `
          )
          .join("")}
      </div>
    `;
  }

  if (panel.fields) {
    return `
      <div class="field-stack">
        ${panel.fields
          .map(
            ([label, value], index) => `
              <label class="property-row">
                <span>${escapeHtml(label)}</span>
                <input value="${escapeAttr(value)}" ${index === 0 ? "class=\"focus-ring\"" : ""} />
              </label>
            `
          )
          .join("")}
      </div>
      <button class="full-button">Add Component</button>
    `;
  }

  return `
    <div class="option-stack">
      ${panel.rows
        .map(
          ([label, value], index) => `
            <button class="option-row ${index === 0 ? "selected" : ""}">
              <span>${escapeHtml(label)}</span>
              <em>${escapeHtml(value)}</em>
            </button>
          `
        )
        .join("")}
    </div>
  `;
}

function renderCenter() {
  return `
    <section class="center-stage ${state.selectedRegion === "center-stage" ? "selected-region" : ""}" data-center="${page().center}" data-region="center-stage">
      <header class="stage-header">
        <div>
          <strong>${escapeHtml(page().title)}</strong>
          <span>Normalized from ${escapeHtml(page().source)}</span>
        </div>
        <div class="stage-tools">
          ${["Perspective", "Lit", "Show", "Snap", "Stats"].map((item) => `<button>${item}</button>`).join("")}
          <button class="${state.selectedRegion === "center-stage" ? "active" : ""}" data-region="center-stage">Layout</button>
        </div>
      </header>
      <div class="stage-body">${renderStageBody(page().center)}</div>
    </section>
  `;
}

function renderBottom() {
  const tabs = page().bottom;
  const key = `${page().id}:bottom`;
  const active = state.activeBottom[key] ?? tabs[0][0];
  const activeText = tabs.find(([title]) => title === active)?.[1] ?? tabs[0][1];
  return `
    <section class="bottom-output ${state.outputOpen ? "" : "closed"} ${state.selectedRegion === "bottom-output" ? "selected-region" : ""}" data-region="bottom-output">
      <header class="bottom-tabs">
        ${tabs
          .map(
            ([title]) => `
              <button class="${title === active ? "active" : ""}" data-bottom-key="${key}" data-bottom="${escapeAttr(title)}">
                ${escapeHtml(title)}
              </button>
            `
          )
          .join("")}
        <span></span>
        <button class="${state.selectedRegion === "bottom-output" ? "active" : ""}" data-region="bottom-output">Layout</button>
        <button data-action="toggle-output">${state.outputOpen ? "Collapse" : "Open"}</button>
      </header>
      <div class="output-body">
        <code>${escapeHtml(activeText)}</code>
        <code>[12:18:25] ${escapeHtml(page().status)}</code>
        <code>[12:18:26] Active tool: ${escapeHtml(state.tool)}</code>
      </div>
    </section>
  `;
}

function renderDockOverlay() {
  if (!state.layoutTrayOpen && state.dockPreview === "none") return "";
  const targets = [
    ["left", "Left"],
    ["center", "Center"],
    ["right", "Right"],
    ["bottom", "Bottom"],
  ];
  return `
    <div class="dock-overlay" aria-label="Dock preview targets">
      ${targets
        .map(
          ([target, label]) => `
            <button class="dock-target ${state.dockPreview === target ? "active" : ""}" data-dock-preview="${target}">
              ${label}
            </button>
          `
        )
        .join("")}
      <button class="dock-target clear ${state.dockPreview === "none" ? "active" : ""}" data-dock-preview="none">Clear</button>
    </div>
  `;
}

function renderInspectorTray() {
  const preset = layoutPresets[state.layoutPreset];
  const region = layoutRegionDetails(state.selectedRegion);
  return `
    <aside class="layout-tray" aria-label="Taffy migration layout inspector">
      <header>
        <div>
          <strong>Taffy Layout Inspector</strong>
          <span>${escapeHtml(preset.note)}</span>
        </div>
        <button data-action="toggle-layout-tray">${state.layoutTrayOpen ? "Hide" : "Inspect"}</button>
      </header>
      <div class="layout-controls">
        ${renderSegment("Left drawer", "left-size", state.leftSize, ["compact", "regular", "wide"])}
        ${renderSegment("Right drawer", "right-size", state.rightSize, ["compact", "regular", "wide"])}
        ${renderSegment("Bottom", "bottom-size", state.bottomSize, ["compact", "regular", "tall"])}
      </div>
      <div class="layout-node-card">
        <strong>${escapeHtml(region.title)}</strong>
        <dl>
          <div><dt>Display</dt><dd>${escapeHtml(region.display)}</dd></div>
          <div><dt>Flex/Grid</dt><dd>${escapeHtml(region.layout)}</dd></div>
          <div><dt>Sizing</dt><dd>${escapeHtml(region.sizing)}</dd></div>
          <div><dt>Rust role</dt><dd>${escapeHtml(region.role)}</dd></div>
          <div><dt>Dock preview</dt><dd>${escapeHtml(state.dockPreview)}</dd></div>
        </dl>
      </div>
    </aside>
  `;
}

function renderSegment(label, action, active, options) {
  return `
    <fieldset class="segmented-control">
      <legend>${escapeHtml(label)}</legend>
      ${options
        .map(
          (option) =>
            `<button class="${active === option ? "active" : ""}" data-size-action="${action}" data-size-value="${option}">${escapeHtml(option)}</button>`
        )
        .join("")}
    </fieldset>
  `;
}

function renderStatusBar() {
  return `
    <footer class="status-bar">
      <span class="status-dot"></span>
      <span>Ready</span>
      <span>${escapeHtml(page().status)}</span>
      <button class="status-action ${state.layoutTrayOpen ? "active" : ""}" data-action="toggle-layout-tray">Inspect</button>
      <span class="status-spacer"></span>
      <span>${escapeHtml(layoutPresets[state.layoutPreset].label)} layout</span>
      <span>${escapeHtml(state.selectedRegion)}</span>
      <span>Responsive prototype</span>
      <span>No screenshot UI</span>
    </footer>
  `;
}

function renderOverlay() {
  if (!state.overlay) return "";
  const command = state.overlay === "command";
  return `
    <div class="overlay-backdrop" data-action="close-overlay">
      <section class="floating-panel" role="dialog" aria-modal="true" aria-label="${command ? "Command palette" : "Workbench menu"}">
        <header>
          <strong>${command ? "Command Palette" : "Workbench Menu"}</strong>
          <button data-action="close-overlay">Close</button>
        </header>
        ${
          command
            ? `<label class="command-input"><span>Run command</span><input autofocus placeholder="Type an editor action" /></label>`
            : ""
        }
        <div class="menu-list">
          ${["Open Scene Editor", "Switch to Material Editor", "Validate Current Asset", "Toggle Left Drawer", "Toggle Right Drawer", "Export Layout Notes"]
            .map((item) => `<button data-menu-item>${item}</button>`)
            .join("")}
        </div>
      </section>
    </div>
  `;
}

function bindEvents() {
  app.querySelectorAll("[data-action]").forEach((node) => {
    node.addEventListener("click", (event) => {
      const action = event.currentTarget.dataset.action;
      if (action === "toggle-left") state.leftOpen = !state.leftOpen;
      if (action === "toggle-right") state.rightOpen = !state.rightOpen;
      if (action === "toggle-output") state.outputOpen = !state.outputOpen;
      if (action === "command") state.overlay = "command";
      if (action === "menu") state.overlay = "menu";
      if (action === "toggle-layout-tray") {
        state.layoutTrayOpen = !state.layoutTrayOpen;
        state.selectedRegion = state.layoutTrayOpen ? "layout-tray" : "center-stage";
      }
      if (action === "close-overlay") {
        const backdropClick = event.currentTarget.classList.contains("overlay-backdrop");
        if (backdropClick && event.target !== event.currentTarget) return;
        state.overlay = null;
      }
      render();
    });
  });

  app.querySelectorAll("[data-tool]").forEach((node) => {
    node.addEventListener("click", (event) => {
      state.tool = event.currentTarget.dataset.tool;
      render();
    });
  });

  app.querySelectorAll("[data-panel-key]").forEach((node) => {
    node.addEventListener("click", (event) => {
      state.activePanel[event.currentTarget.dataset.panelKey] = event.currentTarget.dataset.panel;
      render();
    });
  });

  app.querySelectorAll("[data-bottom-key]").forEach((node) => {
    node.addEventListener("click", (event) => {
      state.activeBottom[event.currentTarget.dataset.bottomKey] = event.currentTarget.dataset.bottom;
      render();
    });
  });

  app.querySelectorAll("[data-layout-preset]").forEach((node) => {
    node.addEventListener("click", (event) => {
      const presetId = event.currentTarget.dataset.layoutPreset;
      const preset = layoutPresets[presetId];
      if (!preset) return;
      state.layoutPreset = presetId;
      state.leftSize = preset.left;
      state.rightSize = preset.right;
      state.bottomSize = preset.bottom;
      render();
    });
  });

  app.querySelectorAll("[data-size-action]").forEach((node) => {
    node.addEventListener("click", (event) => {
      const action = event.currentTarget.dataset.sizeAction;
      const value = event.currentTarget.dataset.sizeValue;
      if (action === "left-size") state.leftSize = value;
      if (action === "right-size") state.rightSize = value;
      if (action === "bottom-size") state.bottomSize = value;
      state.layoutPreset = "custom";
      render();
    });
  });

  app.querySelectorAll("[data-region]").forEach((node) => {
    node.addEventListener("click", (event) => {
      event.stopPropagation();
      state.selectedRegion = event.currentTarget.dataset.region;
      render();
    });
  });

  app.querySelectorAll("[data-dock-preview]").forEach((node) => {
    node.addEventListener("click", (event) => {
      state.dockPreview = event.currentTarget.dataset.dockPreview;
      render();
    });
  });

  app.querySelectorAll(".overlay-backdrop").forEach((node) => {
    node.addEventListener("click", (event) => {
      if (event.target === node) {
        state.overlay = null;
        render();
      }
    });
  });
}

function normalizePage(value) {
  return pages.find((item) => item.id === value)?.id;
}

function page() {
  return pages.find((item) => item.id === state.pageId) ?? pages[0];
}

function layoutRegionDetails(region) {
  const regions = {
    "left-drawer": {
      title: "Left Drawer Column",
      display: "grid",
      layout: "rows: 0.55fr / 0.45fr",
      sizing: `width token: ${state.leftSize}`,
      role: "DockSlot::LeftColumn",
    },
    "right-drawer": {
      title: "Right Drawer Column",
      display: "grid",
      layout: "rows: 0.5fr / 0.5fr",
      sizing: `width token: ${state.rightSize}`,
      role: "DockSlot::RightColumn",
    },
    "center-stage": {
      title: "Center Stage",
      display: "grid",
      layout: "rows: header / document",
      sizing: "minmax(0, 1fr)",
      role: "DocumentSurface::Active",
    },
    "bottom-output": {
      title: "Bottom Output",
      display: "grid",
      layout: "rows: tabs / output body",
      sizing: `height token: ${state.bottomSize}`,
      role: "DockSlot::BottomOutput",
    },
    "layout-tray": {
      title: "Layout Inspector Tray",
      display: "grid",
      layout: "rows: header / controls / node card",
      sizing: "content-sized bottom tray",
      role: "DebugOverlay::LayoutInspector",
    },
  };
  return regions[region] ?? regions["center-stage"];
}
