(function () {
  const componentCatalog = Object.freeze({
    atoms: [
      "button",
      "icon-button",
      "search-input",
      "select-button",
      "select-trigger",
      "tag",
      "progress",
      "check-line",
      "input-box",
      "combo-box",
      "checkbox",
      "toggle",
      "action-command-button",
      "small-stat",
      "row-icon",
      "row-leading-icon-slot",
      "row-selection-slot",
      "row-main-slot",
      "row-meta-slot",
      "row-trailing-slot",
    ],
    molecules: [
      "page-heading",
      "toolbar",
      "section-title",
      "tab-strip",
      "metric-card",
      "info-row",
      "source-engine-row",
      "tree-row",
      "setting-summary-row",
      "project-status-strip",
      "project-detail-actions-section",
      "media-content-panel",
      "project-detail-main-panel",
      "action-row",
      "path-field-row",
      "form-panel",
      "content-panel",
      "catalog-column-row",
      "project-card",
    ],
    collections: [
      "metric-grid",
      "data-table",
      "project-table",
      "quick-actions",
      "tabbed-list-panel",
      "menu",
      "select-menu",
      "menu-list",
      "menu-item",
      "row-surface",
      "table-row-surface",
      "menu-row",
      "selected-row",
      "project-action-stack",
      "row-list",
      "browser-table",
      "tree-view",
    ],
    overlays: [
      "anchored-popover",
      "delete-confirm",
      "popover-paper",
      "select-menu-surface",
      "source-engine-popup",
      "user-menu-popup",
    ],
    windowSurfaces: [
      "window-view",
      "app-bar",
      "topbar",
      "drawer-shell",
      "sidebar-drawer",
      "workspace-surface",
      "workspace",
      "page-content-slot",
      "panel",
      "popover",
      "modal",
    ],
  });

  function createMaterialComponents({ icon, brand, projectCover }) {
    function esc(value) {
      return String(value)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
    }

    function routeAttr(route) {
      return route ? ` data-route="${esc(route)}"` : "";
    }

    function iconImage(iconName) {
      return iconName ? `<img src="${icon(iconName)}" alt="">` : "";
    }

    function pageHeading(page, actions = "") {
      return `
      <div class="page-heading" data-component="page-heading">
        <div>
          <h2 class="page-title">${esc(page.title)}</h2>
          <p>${esc(page.subtitle)}</p>
        </div>
        ${actions ? `<div class="heading-actions">${actions}</div>` : ""}
      </div>`;
    }

    function button(label, iconName, variant = "") {
      const split = label === "New Project";
      const routes = {
        "Import Project": "hub-projects-browser",
        "New Project": "hub-projects-new",
        "Project Browser": "hub-projects-browser",
        "Open in Editor": "hub-editor",
        "Build Project": "hub-builds",
        "Package Project": "hub-cloud",
        "Install to Device": "hub-builds",
        "Save Settings": "hub-settings",
        "Create Project": "hub-projects-detail",
        Cancel: "projects-dashboard",
        Refresh: "projects-dashboard",
        Reset: "hub-settings",
        "Retry Action": "projects-dashboard",
        "Remove from Hub": "hub-state-empty",
        "Delete Project": "hub-projects-detail-delete-confirm",
      };
      const route = routeAttr(routes[label]);
      return `<button class="button ${variant}${split ? " split-button" : ""}" data-component="button" type="button"${route}>${iconImage(iconName)}<span>${esc(label)}</span>${split ? `<span class="split-caret"><img src="${icon("ui/chevron-down.svg")}" alt=""></span>` : ""}</button>`;
    }

    function actionCommandButton(label, iconName, variant = "") {
      const base = button(label, iconName, variant);
      return base
        .replace('class="button ', 'class="button action-command-button ')
        .replace('data-component="button"', 'data-component="action-command-button" data-material-slot="button action-command-button"');
    }

    function searchBox(search, compact = false) {
      return `
        <label class="search-box${compact ? " compact" : ""}" data-component="search-input">
          <img src="${icon("ui/search.svg")}" alt="">
          <input type="text" placeholder="${esc(search)}" aria-label="${esc(search)}">
        </label>`;
    }

    function selectButton(label, leadingIconName, route, compact = false) {
      return `<button class="select-button select-trigger${compact ? " compact" : ""}" data-component="select-button" data-material-slot="select-trigger menu-anchor" type="button"${routeAttr(route)}>${iconImage(leadingIconName)}${esc(label)} <img src="${icon("ui/chevron-down.svg")}" alt=""></button>`;
    }

    function modeButton(iconName, active = false, route = "") {
      return `<button class="mode-button ${active ? "active" : ""}" data-component="icon-button" type="button"${routeAttr(route)}><img src="${icon(iconName)}" alt=""></button>`;
    }

    function toolbar(search, extra = "") {
      return `
      <div class="toolbar" data-component="toolbar">
        ${searchBox(search)}
        <div class="toolbar-spacer"></div>
        ${extra || `
          ${selectButton("All Projects", "ui/folder.svg", "hub-projects-browser-filter-menu")}
          ${selectButton("Last Modified", "ui/sort.svg", "hub-projects-browser-sort-menu")}
          <span class="toolbar-divider"></span>
          ${modeButton("ui/grid.svg", true)}
          ${modeButton("ui/list.svg", false, "hub-projects-browser")}
        `}
      </div>`;
    }

    function tag(label, tone = "") {
      return `<span class="tag ${tone}" data-component="tag">${esc(label)}</span>`;
    }

    function progress(label, value, tone = "") {
      return `
      <div class="progress-row ${tone}" data-component="progress">
        <span>${esc(label)}</span>
        <div><i style="width: ${Math.max(0, Math.min(100, Number(value)))}%;"></i></div>
        <strong>${esc(value)}%</strong>
      </div>`;
    }

    function checkLine(label, detail, state = "ready") {
      return `
      <div class="check-line ${state}" data-component="check-line">
        <span class="row-leading-icon-slot" data-component="row-leading-icon-slot" data-material-slot="row-leading-icon-slot"></span>
        <strong>${esc(label)}</strong>
        <em>${esc(detail)}</em>
      </div>`;
    }

    function smallStat(label, value, detail, tone = "") {
      return `
      <article class="small-stat ${tone}" data-component="small-stat">
        <p>${esc(label)}</p>
        <strong>${esc(value)}</strong>
        <em>${esc(detail)}</em>
      </article>`;
    }

    function metricCard(label, value, detail, tone = "") {
      return `
      <article class="metric-card ${tone}" data-component="metric-card">
        <p>${esc(label)}</p>
        <strong>${esc(value)}</strong>
        <em>${esc(detail)}</em>
      </article>`;
    }

    function metricGrid(items, columns = "four") {
      return `
      <section class="content-grid ${columns}" data-component="metric-grid">
        ${items.map(([label, value, detail, tone = ""]) => metricCard(label, value, detail, tone)).join("")}
      </section>`;
    }

    function sectionTitle(title, detail = "") {
      return `
      <div class="section-title" data-component="section-title">
        <h3>${esc(title)}</h3>
        ${detail ? `<p>${esc(detail)}</p>` : ""}
      </div>`;
    }

    function tabStrip(items, activeIndex = 0) {
      return `
      <div class="tab-strip" data-component="tab-strip">
        ${items.map((item, index) => `<button class="${index === activeIndex ? "active" : ""}" type="button" data-ui-action="tab-select">${esc(item)}</button>`).join("")}
      </div>`;
    }

    function tabbedListPanel(title, detail, tabs, rowsHtml, className = "") {
      return `
      <article class="panel tall tabbed-list-panel ${className}" data-component="tabbed-list-panel" data-material-slot="panel tabbed-list-panel tab-strip row-list">
        ${sectionTitle(title, detail)}
        ${tabStrip(tabs, 0)}
        <div class="row-list compact">${rowsHtml}</div>
      </article>`;
    }

    function formPanel(title, rowsHtml, className = "", detail = "") {
      return `
      <article class="panel settings-panel form-panel ${className}" data-component="form-panel" data-material-slot="panel form-panel form-control-stack">
        ${sectionTitle(title, detail)}
        <div class="row-list compact form-stack">${rowsHtml}</div>
      </article>`;
    }

    function contentPanel(title, bodyHtml, className = "", detail = "") {
      return `
      <article class="panel content-panel ${className}" data-component="content-panel" data-material-slot="panel content-panel card-header card-content">
        ${sectionTitle(title, detail)}
        <div class="content-panel-body">${bodyHtml}</div>
      </article>`;
    }

    function mediaContentPanel(title, detail, mediaHtml, bodyHtml, className = "", componentName = "media-content-panel") {
      const componentAttr = componentName === "project-detail-main-panel"
        ? 'data-component="project-detail-main-panel"'
        : `data-component="${esc(componentName)}"`;
      return `
      <article class="panel content-panel media-content-panel ${className}" ${componentAttr} data-material-slot="panel content-panel media-content-panel ${esc(componentName)} card-media card-header card-content">
        <div class="media-panel-media">${mediaHtml}</div>
        ${sectionTitle(title, detail)}
        <div class="content-panel-body media-content-panel-body">${bodyHtml}</div>
      </article>`;
    }

    function htmlCell(html) {
      return { html };
    }

    function renderCell(cell) {
      if (cell && typeof cell === "object" && "html" in cell) {
        return cell.html;
      }
      return esc(cell);
    }

    function dataTable(columns, rows, className = "") {
      return `
      <div class="data-table ${className}" data-component="data-table">
        <div class="data-table-head">${columns.map((column) => `<span>${esc(column)}</span>`).join("")}</div>
        ${rows.map((row) => `<div class="data-table-row row-surface table-row-surface" data-material-slot="table-row row-surface">${row.map((cell) => `<span>${renderCell(cell)}</span>`).join("")}</div>`).join("")}
      </div>`;
    }

    function rowIcon(label) {
      return `<span class="row-icon row-leading-icon-slot" data-component="row-leading-icon-slot" data-material-slot="row-icon row-leading-icon-slot">${esc(label)}</span>`;
    }

    function infoRow(iconLabel, title, detail, badge, tone = "accent") {
      return `
      <div class="info-row" data-component="info-row">
        ${rowIcon(iconLabel)}
        <span class="row-main row-main-slot" data-component="row-main-slot" data-material-slot="row-main-slot"><strong>${esc(title)}</strong><span>${esc(detail)}</span></span>
        <span class="row-trailing-slot" data-component="row-trailing-slot" data-material-slot="row-trailing-slot">
          ${tag(badge, tone)}
          <span class="row-arrow">></span>
        </span>
      </div>`;
    }

    function sourceEngineRow(iconLabel, title, detail, badge, tone = "accent", selected = false, options = {}) {
      const classes = ["source-engine-row", "row-surface"];
      if (options.className) {
        classes.push(options.className);
      }
      if (selected) {
        classes.push("selected");
      }
      const route = options.route ?? "hub-source-engine-popup";
      const showAction = options.showAction ?? true;
      return `
      <button class="${classes.join(" ")}" data-component="source-engine-row" data-material-slot="source-engine-row row-surface${selected ? " selected-row" : ""}" type="button" data-route="${esc(route)}">
        ${rowIcon(iconLabel)}
        <span class="row-main row-main-slot" data-component="row-main-slot" data-material-slot="row-main-slot"><strong>${esc(title)}</strong><span>${esc(detail)}</span></span>
        <span class="row-trailing-slot" data-component="row-trailing-slot" data-material-slot="row-trailing-slot">
          ${tag(badge, tone)}
          ${showAction ? `<span class="row-action" data-component="icon-button"><img src="${icon("ui/close.svg")}" alt=""></span>` : ""}
        </span>
      </button>`;
    }

    function treeRow(iconLabel, title, detail, badge = "", tone = "neutral", depth = 0, expanded = false, hasChildren = false, selected = false) {
      const disclosure = hasChildren
        ? `<span class="tree-disclosure row-action" data-component="icon-button" data-material-slot="tree-disclosure">${expanded ? "v" : ">"}</span>`
        : `<span class="tree-disclosure empty" data-material-slot="tree-disclosure"></span>`;
      return `
      <button class="tree-row row-surface${selected ? " selected" : ""}" data-component="tree-row" data-material-slot="tree-row row-surface${selected ? " selected-row" : ""}" type="button" style="--tree-depth: ${Math.max(0, Number(depth) || 0)}">
        <span class="tree-indent" aria-hidden="true"></span>
        ${disclosure}
        ${rowIcon(iconLabel)}
        <span class="row-main row-main-slot" data-component="row-main-slot" data-material-slot="row-main-slot"><strong>${esc(title)}</strong><span>${esc(detail)}</span></span>
        ${badge ? `<span class="row-trailing-slot" data-component="row-trailing-slot" data-material-slot="row-trailing-slot">${tag(badge, tone)}</span>` : ""}
      </button>`;
    }

    function treeView(title, rowsHtml, className = "") {
      return `
      <article class="panel tree-view ${className}" data-component="tree-view" data-material-slot="panel tree-view row-list">
        ${sectionTitle(title)}
        <div class="row-list compact">${rowsHtml}</div>
      </article>`;
    }

    function settingSummaryRow(label, value, badgeValue = false, tone = "neutral") {
      const valueSlot = badgeValue
        ? `<span class="row-main row-main-slot setting-summary-spacer" data-component="row-main-slot" data-material-slot="row-main-slot"></span>
           <span class="row-trailing-slot" data-component="row-trailing-slot" data-material-slot="row-trailing-slot">${tag(value, tone)}</span>`
        : `<span class="row-main row-main-slot setting-summary-value" data-component="row-main-slot" data-material-slot="row-main-slot">${esc(value)}</span>`;
      return `
      <div class="setting-row setting-summary-row row-surface" data-component="setting-summary-row" data-material-slot="setting-summary-row row-surface">
        <span class="row-meta-slot setting-summary-label" data-component="row-meta-slot" data-material-slot="row-meta-slot">${esc(label)}</span>
        ${valueSlot}
      </div>`;
    }

    function projectStatusStrip(version, pinLabel, modifiedLabel, pinTone = "neutral") {
      return `
      <div class="project-status-strip row-surface" data-component="project-status-strip" data-material-slot="project-status-strip row-surface">
        <span class="row-trailing-slot" data-component="row-trailing-slot" data-material-slot="row-trailing-slot">${tag(version, "accent")}</span>
        <span class="row-trailing-slot" data-component="row-trailing-slot" data-material-slot="row-trailing-slot">${tag(pinLabel, pinTone)}</span>
        <span class="row-meta-slot project-status-meta" data-component="row-meta-slot" data-material-slot="row-meta-slot">${esc(modifiedLabel)}</span>
      </div>`;
    }

    function actionRow(iconLabel, title, detail, tone = "") {
      const disabled = tone.split(/\s+/).includes("disabled");
      const element = disabled ? "div" : "button";
      const attrs = disabled ? "" : ' type="button"';
      return `
      <${element} class="action-row ${tone}" data-component="action-row"${attrs}>
        ${rowIcon(iconLabel)}
        <span class="row-main row-main-slot" data-component="row-main-slot" data-material-slot="row-main-slot"><strong>${esc(title)}</strong><span>${esc(detail)}</span></span>
        <span class="row-trailing-slot" data-component="row-trailing-slot" data-material-slot="row-trailing-slot">
          <span class="row-arrow">></span>
        </span>
      </${element}>`;
    }

    function projectDetailActionsSection(confirmDelete = false) {
      const standardActions = [
        actionCommandButton("Open in Editor", "actions/open-editor.svg", "primary"),
        toggleRow("Not pinned", "Pin Project", false),
        actionCommandButton("Remove from Hub", "ui/close.svg"),
        `<p class="project-action-note">Removes the Hub entry only; files stay on disk.</p>`,
        actionCommandButton("Delete Project", "ui/alert.svg", "danger"),
      ].join("");
      const deleteActions = [
        `<div class="status-banner error" data-component="status-banner"><strong>Confirm Delete</strong><span>This removes the project from Hub records; files stay on disk.</span></div>`,
        actionCommandButton("Delete Project", "ui/alert.svg", "danger"),
        actionCommandButton("Cancel", "ui/close.svg"),
      ].join("");
      return `
      <aside class="panel detail-side project-detail-actions-section" data-component="project-detail-actions-section">
        ${sectionTitle("Project Actions", "Context commands")}
        <div class="project-action-stack" data-component="project-action-stack" data-material-slot="stack action-stack">
          ${confirmDelete ? deleteActions : standardActions}
        </div>
        ${!confirmDelete ? `
          <div class="project-action-engines">
            ${sectionTitle("Change Source Engine", "Bound Source Engine: Zircon Engine 1.8.2")}
            <div class="row-list compact source-engine-list">
              ${sourceEngineRow("ZE", "Zircon Engine 1.8.2", "D:\\Engines\\ZirconEngine\\main", "Active", "accent", true)}
              ${sourceEngineRow("ZE", "Zircon Engine 1.8.1", "D:\\Engines\\ZirconEngine\\stable", "Registered", "")}
              ${sourceEngineRow("ZE", "Zircon Engine 1.8.0", "D:\\Engines\\ZirconEngine\\legacy", "Registered", "")}
            </div>
          </div>` : ""}
      </aside>`;
    }

    function projectDetailMainPanel(project) {
      const panel = mediaContentPanel(
        project.title,
        project.path,
        projectCover(project, "hero"),
        `
          ${projectStatusStrip(project.version, "Not pinned", project.modified)}
          <div class="detail-stats">
            ${smallStat("Assets", "1,284", "indexed", "accent")}
            ${smallStat("Builds", "12", "last 7 days", "success")}
            ${smallStat("Warnings", "1", "non-blocking", "warning")}
            ${smallStat("Size", "12.8 GB", "workspace", "")}
          </div>
          <div class="detail-grid">
            <div class="row-list compact">
              ${settingSummaryRow("Status", "Ready", true, "success")}
              ${settingSummaryRow("Project Root", project.path)}
              ${settingSummaryRow("Source Engine", "Zircon Engine 1.8.2")}
              ${settingSummaryRow("Engine Version", project.version)}
              ${settingSummaryRow("Last Modified", project.modified)}
            </div>
            <div class="activity-panel">
              ${sectionTitle("Activity", "Latest project events")}
              ${[
                ["Asset catalog refreshed", "2h ago", "success"],
                ["Build package queued", "5h ago", "accent"],
                ["Shader warning recorded", "Yesterday", "warning"],
              ].map(([title, time, tone]) => `
                <div class="timeline-item ${tone}"><span></span><strong>${esc(title)}</strong><em>${esc(time)}</em>${tag(tone || "Info", tone)}</div>`).join("")}
            </div>
          </div>
        `,
        "detail-main project-detail-main-panel",
        "project-detail-main-panel",
      );
      return panel;
    }

    function emptyState(title, detail = "") {
      return `
      <div class="empty-state-compact" data-component="empty-state">
        <strong>${esc(title)}</strong>
        ${detail ? `<span>${esc(detail)}</span>` : ""}
      </div>`;
    }

    function catalogColumnRow(iconLabel, title, detail, metadata, badge, tone = "accent", route = "", showArrow = true) {
      const interactive = Boolean(route || showArrow);
      const element = interactive ? "button" : "div";
      const attrs = interactive ? ` type="button"${routeAttr(route)}` : "";
      return `
      <${element} class="catalog-column-row ${showArrow ? "" : "no-arrow"}" data-component="catalog-column-row"${attrs}>
        ${rowIcon(iconLabel)}
        <span class="row-main row-main-slot" data-component="row-main-slot" data-material-slot="row-main-slot"><strong>${esc(title)}</strong><span>${esc(detail)}</span></span>
        <span class="catalog-row-meta row-meta-slot" data-component="row-meta-slot" data-material-slot="row-meta-slot">${esc(metadata)}</span>
        <span class="row-trailing-slot" data-component="row-trailing-slot" data-material-slot="row-trailing-slot">
          ${tag(badge, tone)}
          ${showArrow ? `<span class="row-arrow">></span>` : ""}
        </span>
      </${element}>`;
    }

    function pathFieldRow(label, value, supporting = "", actionLabel = "", actionIcon = "ui/folder.svg") {
      const action = actionLabel
        ? `<button class="button path-field-action" data-component="button" type="button" data-ui-action="path-field-action">${iconImage(actionIcon)}<span>${esc(actionLabel)}</span></button>`
        : "";
      return `
      <div class="path-field-row${action ? "" : " no-action"}" data-component="path-field-row">
        ${inputBox(label, value, supporting)}
        ${action}
      </div>`;
    }

    function inputBox(label, value, supporting = "") {
      return `
      <label class="field-control input-box" data-component="input-box">
        <span>${esc(label)}</span>
        <input type="text" value="${esc(value)}" aria-label="${esc(label)}">
        ${supporting ? `<em>${esc(supporting)}</em>` : ""}
      </label>`;
    }

    function comboBox(label, value, supporting = "") {
      return `
      <button class="field-control combo-box select-trigger" data-component="combo-box" data-material-slot="select-trigger combo-box-trigger menu-anchor" type="button" data-ui-action="combo-preview">
        <span>${esc(label)}</span>
        <strong>${esc(value)}</strong>
        <img src="${icon("ui/chevron-down.svg")}" alt="">
        ${supporting ? `<em>${esc(supporting)}</em>` : ""}
      </button>`;
    }

    function checkboxRow(label, detail, checked = true) {
      return `
      <button class="choice-row checkbox-row ${checked ? "checked" : ""}" data-component="checkbox" type="button" data-ui-action="checkbox-toggle">
        <span class="row-selection-slot" data-component="row-selection-slot" data-material-slot="row-selection-slot"><i></i></span>
        <span class="row-main row-main-slot" data-component="row-main-slot" data-material-slot="row-main-slot"><strong>${esc(label)}</strong><span>${esc(detail)}</span></span>
      </button>`;
    }

    function toggleRow(label, detail, checked = true) {
      return `
      <button class="choice-row toggle-row ${checked ? "checked" : ""}" data-component="toggle" type="button" data-ui-action="toggle-switch">
        <span class="row-main"><strong>${esc(label)}</strong><span>${esc(detail)}</span></span>
        <i></i>
      </button>`;
    }

    function projectCard(project) {
      return `
      <article class="project-card" data-component="project-card" data-route="hub-projects-detail">
        <div class="cover">
          ${projectCover(project, "card")}
          <button type="button" data-component="icon-button" data-route="hub-projects-detail-delete-confirm"><img src="${icon("ui/more-vertical.svg")}" alt=""></button>
          <span class="cover-brand"><img src="${brand}" alt=""></span>
        </div>
        <h3>${esc(project.title)}</h3>
        <p>${esc(project.path)}</p>
        <p>${esc(project.modified)}</p>
        <div class="tag-row">${tag(project.version, "accent")}${tag(project.platform)}</div>
      </article>`;
    }

    function projectTable(projectList) {
      return `
      <article class="panel recent-panel" data-component="project-table">
        <h3>Recent Projects</h3>
        <div class="table-head"><span>Name</span><span>Engine Version</span><span>Last Modified</span><span>Location</span></div>
        ${projectList
          .map(
            (project) => `
          <div class="project-row">
            <span>${projectCover(project, "thumb")}${esc(project.title)}</span>
            <span>${esc(project.version)}</span>
            <span>${esc(project.tableModified)}</span>
            <span>${esc(project.path)}</span>
            <span class="row-trailing-slot" data-component="row-trailing-slot" data-material-slot="row-trailing-slot">
              <button type="button" data-component="icon-button" data-route="hub-projects-detail-delete-confirm"><img src="${icon("ui/more-vertical.svg")}" alt=""></button>
            </span>
          </div>`
          )
          .join("")}
        <button class="view-all" type="button" data-component="button" data-route="hub-projects-browser"><img src="${icon("nav/projects.svg")}" alt="">View All Projects <img src="${icon("ui/chevron-right.svg")}" alt=""></button>
      </article>`;
    }

    function quickActions() {
      const rows = [
        ["actions/build-project.svg", "Build Project", "Build your project for development or release"],
        ["actions/install-device.svg", "Install to Device", "Deploy your project to a connected device"],
        ["actions/package-project.svg", "Package Project", "Create a distributable package"],
        ["actions/open-editor.svg", "Open in Editor", "Launch the editor with a project"],
      ];
      return `
      <article class="panel quick-panel" data-component="quick-actions">
        <h3>Quick Actions</h3>
        <div class="quick-list">
          ${rows
            .map(
              ([rowIconName, title, detail]) => `
            <button class="quick-row" type="button" data-component="action-row" data-route="${title === "Build Project" || title === "Install to Device" ? "hub-builds" : title === "Package Project" ? "hub-cloud" : "hub-editor"}">
              <img src="${icon(rowIconName)}" alt="">
              <span><strong>${esc(title)}</strong><em>${esc(detail)}</em></span>
              <img src="${icon("ui/chevron-right.svg")}" alt="">
            </button>`
            )
            .join("")}
        </div>
      </article>`;
    }

    function renderMenu(kind, items) {
      return `
      <div class="menu-panel select-menu-surface ${kind}" data-component="menu" data-material-slot="anchored-popover select-menu popover-paper" role="menu">
        <div class="menu-list" data-component="menu-list" data-material-slot="menu-list">
          ${items.map((item, index) => `<button class="menu-item row-surface menu-row dense-menu-row ${index === 0 ? "active selected" : ""}" data-component="menu-item" data-material-slot="menu-item row-surface menu-row dense${index === 0 ? " selected-item" : ""}" type="button" data-route="hub-projects-browser"><span>${esc(item)}</span><span>${index === 0 ? "OK" : ""}</span></button>`).join("")}
        </div>
      </div>`;
    }

    function renderDeleteConfirm() {
      return `
      <aside class="confirm-panel" data-component="delete-confirm">
        <h3>Delete project from Hub?</h3>
        <p>This removes the Hub record. Project files stay on disk until deleted manually.</p>
        <div class="confirm-actions">
          <button class="button" type="button" data-component="button" data-route="hub-projects-detail"><span>Cancel</span></button>
          <button class="button danger" type="button" data-component="button" data-route="hub-state-empty"><span>Delete Project</span></button>
        </div>
      </aside>`;
    }

    function renderSourceEnginePopup() {
      const rows = [
        ["Zircon Engine 1.8.2", "Ready, local source checkout", "Active"],
        ["Zircon Engine 1.8.1", "Installed fallback source", "Ready"],
        ["Custom Source Build", "D:\\Engines\\Experimental", "Local"],
      ];
      return `
      <aside class="source-popover" data-component="source-engine-popup" data-material-slot="anchored-popover popover-paper menu-list">
        <p class="popover-title">Source Engines</p>
        ${rows.map(([title, detail, badge]) => `
          <button class="popover-row row-surface menu-row dense-menu-row engine-pop-row ${badge === "Active" ? "selected" : ""}" data-component="menu-item" data-material-slot="menu-item row-surface menu-row dense${badge === "Active" ? " selected-item" : ""}" type="button" data-route="projects-dashboard">
            ${rowIcon("ZE")}
            <span>${esc(title)}<br><small>${esc(detail)}</small></span>
            ${tag(badge, badge === "Active" ? "accent" : badge === "Ready" ? "success" : "")}
          </button>`).join("")}
      </aside>`;
    }

    function renderUserMenu() {
      return `
      <aside class="user-popover" data-component="user-menu-popup" data-material-slot="anchored-popover popover-paper menu-list">
        <p class="popover-title">Alex Developer</p>
        <div class="account-card"><span>AD</span><strong>alex@zircon.local</strong><em>Local workspace profile</em></div>
        ${["Profile", "Preferences", "Documentation", "Sign out"].map((item, index) => `
          <button class="popover-row row-surface menu-row dense-menu-row ${index === 3 ? "disabled" : ""}" data-component="menu-item" data-material-slot="menu-item row-surface menu-row dense${index === 3 ? " disabled-item" : ""}" type="button" data-route="${index === 1 ? "hub-settings" : index === 2 ? "hub-learn" : "projects-dashboard"}"><span>${esc(item)}</span><span>${index === 3 ? "!" : ">"}</span></button>`).join("")}
      </aside>`;
    }

    return Object.freeze({
      esc,
      pageHeading,
      button,
      searchBox,
      selectButton,
      modeButton,
      toolbar,
      tag,
      progress,
      checkLine,
      smallStat,
      metricCard,
      metricGrid,
      sectionTitle,
      tabStrip,
      tabbedListPanel,
      formPanel,
      contentPanel,
      mediaContentPanel,
      htmlCell,
      dataTable,
      rowIcon,
      infoRow,
      sourceEngineRow,
      treeRow,
      treeView,
      settingSummaryRow,
      projectStatusStrip,
      projectDetailMainPanel,
      projectDetailActionsSection,
      actionCommandButton,
      actionRow,
      emptyState,
      catalogColumnRow,
      pathFieldRow,
      inputBox,
      comboBox,
      checkboxRow,
      toggleRow,
      projectCard,
      projectTable,
      quickActions,
      renderMenu,
      renderDeleteConfirm,
      renderSourceEnginePopup,
      renderUserMenu,
    });
  }

  window.ZirconHubMaterialComponents = Object.freeze({
    componentCatalog,
    createMaterialComponents,
  });
})();
