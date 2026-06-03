(function () {
  const asset = (path) => `../../../${path}`;
  const icon = (name) => asset(`zircon_hub/assets/icons/${name}`);
  const brand = asset("zircon_hub/assets/brand/zircon-mark.svg"), projectCover = window.ZirconHubCover.projectCover;
  const components = window.ZirconHubMaterialComponents.createMaterialComponents({ icon, brand, projectCover });
  const {
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
    metricGrid,
    sectionTitle,
    tabStrip,
    htmlCell,
    dataTable,
    rowIcon,
    infoRow,
    sourceEngineRow,
    settingSummaryRow,
    projectStatusStrip,
    projectDetailActionsSection,
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
  } = components;
  const navItems = [
    ["projects", "Projects", "nav/projects.svg", "projects-dashboard"],
    ["editor", "Editor", "nav/editor.svg", "hub-editor"],
    ["assets", "Assets", "nav/assets.svg", "hub-assets"],
    ["builds", "Builds", "nav/builds.svg", "hub-builds"],
    ["plugins", "Plugins", "nav/plugins.svg", "hub-plugins"],
    ["cloud", "Cloud", "nav/cloud.svg", "hub-cloud"],
    ["team", "Team", "nav/team.svg", "hub-team"],
    ["learn", "Learn", "nav/learn.svg", "hub-learn"],
    ["settings", "Settings", "nav/settings.svg", "hub-settings"],
  ];

  const projects = [
    {
      title: "Elysium Chronicles",
      path: "C:\\ZirconProjects\\Elysium",
      modified: "Modified 2h ago",
      tableModified: "2h ago",
      version: "1.8.2",
      platform: "Windows",
      cover: "elysium",
    },
    {
      title: "Stellar Outpost",
      path: "C:\\ZirconProjects\\StellarOutpost",
      modified: "Modified yesterday",
      tableModified: "Yesterday",
      version: "1.8.2",
      platform: "Windows",
      cover: "stellar",
    },
    {
      title: "Sands of Time",
      path: "C:\\ZirconProjects\\SandsOfTime",
      modified: "Modified 3d ago",
      tableModified: "3d ago",
      version: "1.8.1",
      platform: "Linux",
      cover: "sands",
    },
    {
      title: "Whispering Woods",
      path: "C:\\ZirconProjects\\WhisperingWoods",
      modified: "Modified 1w ago",
      tableModified: "1w ago",
      version: "1.8.0",
      platform: "Windows",
      cover: "woods",
    },
    {
      title: "Neon Streets",
      path: "C:\\ZirconProjects\\NeonStreets",
      modified: "Modified 2w ago",
      tableModified: "2w ago",
      version: "1.7.9",
      platform: "Windows",
      cover: "neon",
    },
    {
      title: "Prototype Lab",
      path: "C:\\ZirconProjects\\PrototypeLab",
      modified: "Modified 1mo ago",
      tableModified: "1mo ago",
      version: "1.8.2",
      platform: "Windows",
      cover: "prototype",
    },
  ];

  const mainPageData = {
    "hub-editor": {
      nav: "editor",
      title: "Editor",
      subtitle: "Configure source engines, workspace defaults, and launch targets.",
      kind: "tooling",
      sourceTitle: "Editor Source",
      controlTitle: "Editor Controls",
      historyTitle: "Launch Timeline",
      sourceRows: [
        ["ZE", "Zircon Engine 1.8.2", "D:\\Engines\\ZirconEngine\\main", "Ready", "success"],
        ["BR", "Stable Branch", "origin/release-1.8", "Active", "accent"],
        ["LD", "Local Debug Editor", "target\\debug\\zircon_editor.exe", "Local", "accent"],
        ["CL", "Command Line", "--project Elysium --profile editor", "Ready", "success"],
      ],
      controls: [
        ["Open Editor", "Launch selected project in editor", "primary"],
        ["Sync Source", "Refresh registered source engine metadata", ""],
        ["Open Output", "Review editor launch output and logs", ""],
      ],
      history: [
        ["Editor launch", "Elysium opened with Zircon Engine 1.8.2", "Ready", "success"],
        ["Project scan", "6 registered projects validated", "Complete", "success"],
        ["Source refresh", "Remote metadata updated 2h ago", "Info", "accent"],
        ["Build output", "Last editor build produced no diagnostics", "Clean", "success"],
      ],
    },
    "hub-builds": {
      nav: "builds",
      title: "Builds",
      subtitle: "Build, package, and inspect task history.",
      kind: "builds",
    },
  };

  const catalogPages = {
    "hub-assets": {
      nav: "assets",
      title: "Assets",
      subtitle: "Browse imported assets, catalogs, and paths.",
      panel: "Assets Catalog",
      search: "Search assets...",
      icon: "AS",
      tag: "Texture",
      rows: [
        ["CastleVista_Albedo", "Texture asset, 4096px, streamed from project package", "Texture"],
        ["HangarDoor_Normal", "Normal map, BC5, used by Stellar Outpost", "Texture"],
        ["DesertRuins_Mesh", "Static mesh imported from source asset pipeline", "Mesh"],
        ["ForestCabin_Material", "Material instance with texture slot bindings", "Material"],
        ["NeonSign_Emissive", "Texture and material pair with HDR metadata", "Texture"],
        ["Prototype_Player", "Character mesh with skeleton reference", "Mesh"],
        ["UI_Hub_Icons", "Atlas group with verified icon slices", "Atlas"],
        ["Audio_Footstep_Bank", "Sound bank registered for editor preview", "Audio"],
      ],
    },
    "hub-plugins": {
      nav: "plugins",
      title: "Plugins",
      subtitle: "Manage runtime and editor plugin packages.",
      panel: "Plugin Catalog",
      search: "Search plugins...",
      icon: "PL",
      tag: "Runtime",
      rows: [
        ["Texture Importer", "DDS, KTX, and ASTC texture container support", "Importer"],
        ["Sound Runtime", "Dynamic sound events and output device services", "Runtime"],
        ["Material Lab", "Editor-side material preview and parameter tooling", "Editor"],
        ["Navigation Tools", "Nav mesh diagnostics and route visualization", "Editor"],
        ["Hybrid GI", "Lighting probe authoring and runtime validation", "Runtime"],
        ["Animation Events", "Clip events, montage markers, and retarget helpers", "Runtime"],
        ["Networking Shell", "Session diagnostics and package download support", "Runtime"],
        ["Automation Reports", "Test capture summaries and workflow evidence", "Editor"],
      ],
    },
    "hub-learn": {
      nav: "learn",
      title: "Learn",
      subtitle: "Open guides, examples, and reference material.",
      panel: "Learning Library",
      search: "Search guides...",
      icon: "LR",
      tag: "Guide",
      rows: [
        ["Create a Project", "Project templates, source engine selection, and launch flow", "Selected Project", "Project setup"],
        ["Import Assets", "Asset descriptors, package roots, and importer diagnostics", "Source Engine", "Assets and rendering"],
        ["Build for Device", "Build profiles, packaging, and install-to-device workflow", "Selected Project", "Build pipeline"],
        ["Editor Workbench", "Scene, material, prefab, and UI editing surfaces", "Source Engine", "Editor workflow"],
        ["Runtime Plugins", "Plugin manifests, service types, and package loading", "Source Engine", "Runtime systems"],
        ["Troubleshooting", "Recover from failed builds, missing paths, and stale config", "Selected Project", "Diagnostics"],
        ["Release Checklist", "Packaging, validation, runtime evidence, and docs", "Selected Project", "Checklist"],
        ["API Index", "Runtime, editor, UI, and asset module documentation", "Source Engine", "Reference"],
      ],
    },
  };

  const pages = {
    "projects-dashboard": {
      nav: "projects",
      title: "Projects",
      subtitle: "Manage your projects and start building worlds.",
      render: renderProjectsDashboard,
    },
    "hub-projects-new": {
      nav: "projects",
      title: "New Project",
      subtitle: "Create a project from templates and source engines.",
      render: renderNewProject,
    },
    "hub-projects-browser": {
      nav: "projects",
      title: "Project Browser",
      subtitle: "Search and inspect all registered projects.",
      render: () => renderProjectBrowser(null),
    },
    "hub-projects-browser-filter-menu": {
      nav: "projects",
      title: "Project Browser",
      subtitle: "Filter menu open.",
      render: () => renderProjectBrowser("filter"),
    },
    "hub-projects-browser-sort-menu": {
      nav: "projects",
      title: "Project Browser",
      subtitle: "Sort menu open.",
      render: () => renderProjectBrowser("sort"),
    },
    "hub-projects-detail": {
      nav: "projects",
      title: "Project Detail",
      subtitle: "Inspect project metadata and actions.",
      render: () => renderProjectDetail(false),
    },
    "hub-projects-detail-delete-confirm": {
      nav: "projects",
      title: "Project Detail",
      subtitle: "Delete confirmation state.",
      render: () => renderProjectDetail(true),
    },
    "hub-source-engine-popup": {
      nav: "projects",
      title: "Projects",
      subtitle: "Source engine selector popup.",
      render: renderProjectsDashboard,
      overlay: renderSourceEnginePopup,
    },
    "hub-user-menu": {
      nav: "projects",
      title: "Projects",
      subtitle: "User menu open.",
      render: renderProjectsDashboard,
      overlay: renderUserMenu,
    },
    "hub-state-empty": {
      nav: "projects",
      title: "Empty State",
      subtitle: "No matching Hub data.",
      render: () => renderState("empty"),
    },
    "hub-state-loading": {
      nav: "projects",
      title: "Loading State",
      subtitle: "Long running Hub task.",
      render: () => renderState("loading"),
    },
    "hub-state-error": {
      nav: "projects",
      title: "Error State",
      subtitle: "Recoverable Hub failure.",
      render: () => renderState("error"),
    },
    ...Object.fromEntries(Object.entries(mainPageData).map(([id, page]) => [id, { ...page, render: () => renderToolPage(page) }])),
    ...Object.fromEntries(Object.entries(catalogPages).map(([id, page]) => [id, { ...page, render: () => renderCatalogPage(page) }])),
    "hub-cloud": {
      nav: "cloud",
      title: "Cloud",
      subtitle: "Package, deploy, and monitor cloud services.",
      render: () => renderCloudPage(),
    },
    "hub-team": {
      nav: "team",
      title: "Team",
      subtitle: "Review local Git identity and collaborators.",
      render: () => renderOverviewPage("team"),
    },
    "hub-settings": {
      nav: "settings",
      title: "Settings",
      subtitle: "Tune toolchains, paths, language, and defaults.",
      render: renderSettingsPage,
    },
  };

  function renderProjectsDashboard() {
    const page = pages["projects-dashboard"];
    return `
      ${pageHeading(page, `${button("Import Project", "ui/import.svg")}${button("New Project", "ui/plus.svg", "primary")}`)}
      ${toolbar("Search projects...")}
      <section class="project-cards" aria-label="Featured projects">${projects.slice(0, 4).map(projectCard).join("")}</section>
      <section class="lower-grid">${projectTable(projects.slice(0, 5))}${quickActions()}</section>`;
  }

  function renderToolPage(page) {
    if (page.kind === "builds") {
      return renderBuildsPage();
    }

    const controlMarks = {
      "Open Editor": "ED",
      "Sync Source": "SY",
      "Open Output": "LG",
    };

    return `
      ${pageHeading(page, `${button("Refresh Sources", "ui/refresh.svg")}${button("Open Editor", "actions/open-editor.svg", "primary")}`)}
      ${tabStrip(["Overview", "Source Paths", "Timeline", "Actions"], 0)}
      <section class="editor-layout">
        <article class="panel editor-source-panel">
          ${sectionTitle(page.sourceTitle, "Registered source channels, launch paths, and profile state reuse the compact Hub component set.")}
          <div class="editor-source-summary">
            ${smallStat("Active Source", "1.8.2", "origin/main", "accent")}
            ${smallStat("Editor Target", "Debug", "ready to launch", "success")}
            ${smallStat("Warnings", "0", "contract clean", "")}
          </div>
          <div class="path-field-list">
            ${pathFieldRow("Active Engine", "Zircon Engine 1.8.2", "", "Rename", "ui/edit.svg")}
            ${pathFieldRow("Source Checkout", "D:\\Engines\\ZirconEngine\\main", "", "Browse")}
            ${pathFieldRow("Staged Output", "D:\\Builds\\Zircon\\Editor", "", "Browse")}
          </div>
          <div class="row-list source-engine-list compact">
            ${page.sourceRows.map(([mark, title, detail, badge, tone], index) => sourceEngineRow(mark, title, detail, badge, tone, index === 1)).join("")}
          </div>
        </article>
        <aside class="panel editor-side-panel">
          ${sectionTitle(page.controlTitle, "Launch profile")}
          <div class="launch-card">
            <strong>Elysium Chronicles</strong>
            <span>C:\\ZirconProjects\\Elysium</span>
            ${progress("Asset scan", 94, "success")}
            ${progress("Shader cache", 81, "accent")}
            ${progress("Template bridge", 72, "")}
          </div>
          <div class="control-stack">
            ${page.controls.map(([title, detail, variant]) => actionRow(controlMarks[title] || title.slice(0, 2).toUpperCase(), title, detail, variant)).join("")}
          </div>
        </aside>
        <article class="panel editor-timeline-panel">
          ${sectionTitle(page.historyTitle, "Recent launch and source events")}
          <div class="timeline-grid">
            ${page.history.map(([title, detail, badge, tone]) => `
              <div class="timeline-item ${tone}">
                <span></span>
                <strong>${esc(title)}</strong>
                <em>${esc(detail)}</em>
                ${tag(badge, tone)}
              </div>`).join("")}
          </div>
        </article>
      </section>`;
  }

  function renderBuildsPage() {
    const page = mainPageData["hub-builds"];
    const pipeline = [
      ["Validate Source", "D:\\Engines\\ZirconEngine", "Active", "accent"],
      ["Compile Editor", "Select a project before building", "Unavailable", "warning"],
      ["Stage Runtime", "D:\\Builds\\Zircon", "Not built yet", ""],
      ["Package Project", "Select a project before packaging", "Unavailable", "warning"],
    ];
    return `
      ${pageHeading(page, `${button("Open Output", "actions/open-editor.svg")}${button("Build Project", "actions/build-project.svg", "primary")}`)}
      ${tabStrip(["Overview", "Pipeline", "History", "Timeline"], 0)}
      <section class="build-summary-layout">
        <article class="panel build-main build-overview-panel">
          ${sectionTitle("Source Build", "No project selected / ZirconEngine Source / Active")}
          <div class="row-list compact build-source-list">
            ${infoRow("BD", "Source:", "D:\\Engines\\ZirconEngine / Last build: Not built yet", "Active", "warning")}
            ${infoRow("OUT", "Output:", "D:\\Builds\\Zircon / Profile debug", "1 jobs", "")}
          </div>
          <div class="build-chip-row">
            ${tag("debug", "accent")}
            ${tag("1 jobs", "")}
          </div>
        </article>
        <article class="panel build-controls-panel">
          ${sectionTitle("Selected Project Actions")}
          <div class="row-list compact build-actions-list">
            ${actionRow("BD", "Build Editor", "Select a project before building", "disabled")}
            ${actionRow("OUT", "Open Output", "D:\\Builds\\Zircon")}
            ${actionRow("ED", "Open Editor", "Select a project before opening", "disabled")}
            ${actionRow("PK", "Package Project", "Create a distributable package")}
            ${actionRow("DV", "Install to Device", "Select a project before installing", "disabled")}
          </div>
        </article>
      </section>
      <section class="build-detail-layout">
        <article class="panel build-pipeline-panel">
          ${sectionTitle("Build Pipeline", "4")}
          <div class="row-list compact build-pipeline-list">
            ${pipeline.map(([title, detail, badge, tone]) => infoRow("BD", title, detail, badge, tone)).join("")}
          </div>
        </article>
        <article class="panel build-task-panel">
          ${sectionTitle("Current Task", "Ready")}
          <div class="build-current-state">
            <strong>Ready</strong>
            <span>No operation timeline</span>
          </div>
          ${sectionTitle("Build History", "0")}
          ${emptyState("No build history")}
        </article>
        <article class="panel build-timeline-panel">
          ${sectionTitle("Operation Timeline", "0")}
          ${emptyState("No operation history", "Run a build to populate.")}
        </article>
      </section>`;
  }

  function renderCloudPage() {
    const page = pages["hub-cloud"];
    const metrics = [
      ["Local Mode", "Ready", "Offline package workflow"],
      ["Build Output", "Ready", "C:\\Users\\HeJiahui\\ZirconBuilds"],
      ["Device Install", "Ready", "Local device target"],
      ["Packages", "4 local", "Package cache warm"],
    ];
    const services = [
      ["PS", "Profile Sync Slot", "Reserved for optional profile sync after local workflows are stable.", "Reserved", "accent"],
      ["RB", "Remote Build Slot", "Reserved for hosted build workers after local packaging is stable.", "Local", ""],
      ["AU", "Artifact Upload Slot", "Reserved for artifact upload from the local package directory.", "Offline", "warning"],
    ];
    const operations = [
      ["PK", "Package Elysium", "Windows package staged 12m ago", "Ready", "success"],
      ["DV", "Install Preview", "Waiting for connected device", "Idle", ""],
    ];
    return `
      ${pageHeading(page, `${button("Package Project", "actions/package-project.svg")}${button("Deploy Preview", "ui/plus.svg", "primary")}`)}
      ${tabStrip(["Overview", "Packages", "Services", "Timeline"], 0)}
      ${metricGrid(metrics.map(([label, value, detail], index) => [label, value, detail, index === 0 ? "accent" : ""]), "four")}
      <section class="cloud-layout">
        <article class="panel cloud-services-panel">
          ${sectionTitle("Service Slots", "Reserved package and deploy channels share the same compact row rhythm as Projects.")}
          <div class="row-list compact cloud-service-list">
            ${services.map(([mark, title, detail, badge, tone]) => infoRow(mark, title, detail, badge, tone)).join("")}
          </div>
        </article>
        <aside class="panel cloud-actions-panel">
          ${sectionTitle("Packages", "Local package actions and operation feedback")}
          <div class="row-list compact">
            ${actionRow("PK", "Package Selected Project", "Create a distributable package for the selected project")}
            ${actionRow("DV", "Install Selected Project", "Install the latest package to a connected device")}
          </div>
          <div class="build-side-divider compact"></div>
          ${sectionTitle("Cloud Operation Status", "Recent package and install events")}
          <div class="row-list compact cloud-timeline-list">
            ${operations.map(([mark, title, detail, badge, tone]) => infoRow(mark, title, detail, badge, tone)).join("")}
          </div>
        </aside>
      </section>`;
  }

  function renderCatalogPage(page) {
    const extra = `
      ${selectButton("All Projects", "ui/folder.svg", "hub-projects-browser-filter-menu")}
      ${selectButton("Last Modified", "ui/sort.svg", "hub-projects-browser-sort-menu")}
      <span class="toolbar-divider"></span>
      ${modeButton("ui/grid.svg", true)}
      ${modeButton("ui/list.svg", false, "hub-projects-browser")}`;
    const stats =
      page.title === "Assets"
        ? [["Indexed", "1,284", "project assets"], ["Missing", "3", "path warnings"], ["Imported", "42", "today"]]
        : page.title === "Plugins"
          ? [["Enabled", "18", "runtime/editor"], ["Updates", "2", "available"], ["Disabled", "4", "by profile"]]
          : [["Guides", "36", "published"], ["Checks", "12", "workflow lists"], ["Recent", "5", "opened"]];
    const listSurface = `<div class="catalog-column-list">
      ${page.rows
        .map(([title, detail, value, category], index) =>
          catalogColumnRow(
            page.icon,
            title,
            detail,
            page.title === "Learn" ? value : index % 4 === 0 ? "Selected Project" : "Source Engine",
            page.title === "Learn" ? category : value,
            index % 5 === 0 ? "success" : "accent",
            page.title === "Learn" ? "hub-learn" : "",
            page.title === "Learn"
          )
        )
        .join("")}
    </div>`;
    return `
      ${pageHeading(page, `${button("Refresh", "ui/refresh.svg")}${button(`Add ${page.title === "Assets" ? "Asset" : page.title === "Plugins" ? "Plugin" : "Guide"}`, "ui/plus.svg", "primary")}`)}
      ${toolbar(page.search, extra)}
      <section class="catalog-layout">
        <article class="panel catalog-main">
          ${sectionTitle(page.panel, "Dense catalog rows with stable badges, metadata columns, and trailing action affordances.")}
          ${tabStrip(["All", "Project", "Engine", "Workspace"], 0)}
          ${listSurface}
        </article>
        <aside class="panel catalog-side">
          ${sectionTitle(`${page.title} Detail`, "Selected row")}
          <div class="catalog-preview">
            ${rowIcon(page.icon)}
            <strong>${esc(page.rows[0][0])}</strong>
            <span>${esc(page.rows[0][1])}</span>
          </div>
          <div class="mini-stat-grid">${stats.map(([label, value, detail], index) => smallStat(label, value, detail, index === 0 ? "accent" : index === 1 ? "warning" : "success")).join("")}</div>
          <div class="row-list compact">
            ${checkLine("Project link", "Resolved for current project", "ready")}
            ${checkLine("Metadata", "Descriptor and tags are valid", "ready")}
            ${checkLine("Review", "One optional warning remains", "warn")}
          </div>
        </aside>
      </section>`;
  }

  function renderOverviewPage(kind) {
    const isCloud = kind === "cloud";
    const page = pages[isCloud ? "hub-cloud" : "hub-team"];
    const metrics = isCloud
      ? [
          ["Package", "Ready", "Last build 12m ago"],
          ["Services", "4 online", "All endpoints healthy"],
          ["Storage", "82%", "Artifact cache warm"],
          ["Health", "Good", "No deploy blockers"],
        ]
      : [
          ["Identity", "Alex Developer", "Git user configured"],
          ["Repository", "main", "Workspace has local changes"],
          ["Members", "6 active", "Reviewers assigned"],
          ["Access", "Local", "No cloud sign-in required"],
        ];
    const leftRows = isCloud
      ? [
          ["PK", "Development Package", "Windows x64, Elysium, debug symbols included", "Ready", "success"],
          ["SV", "Asset Sync Service", "Uploads staged artifacts when connected", "Online", "success"],
          ["CL", "Cloud Deploy Slot", "Preview environment linked to current project", "Synced", "accent"],
          ["HL", "Health Probe", "Package manifest and deploy paths validated", "Good", "success"],
        ]
      : [
          ["ID", "Alex Developer", "alex@zircon.local, local Git identity", "Active", "success"],
          ["RV", "Maya Chen", "Reviewer for source engine updates", "Review", "accent"],
          ["QA", "Jordan Lee", "Build and install verification owner", "Ready", "success"],
          ["DS", "Design Ops", "Visual reference and documentation reviewer", "Queued", "warning"],
        ];
    const actions = isCloud
      ? [
          ["UP", "Upload Package", "Publish the latest distributable package"],
          ["DP", "Deploy Preview", "Install package into preview service slot"],
          ["LG", "Open Service Logs", "Inspect service, storage, and deploy logs"],
          ["CN", "Configure Cloud", "Review endpoint and package settings"],
        ]
      : [
          ["SC", "Open Source Control", "Review pending changes and local status"],
          ["RS", "Request Review", "Prepare a visual and contract review note"],
          ["HM", "Open Team Home", "Show local team and collaboration settings"],
          ["ST", "Sync Metadata", "Refresh collaborator and repository details"],
        ];
    return `
      ${pageHeading(page, `${button(isCloud ? "Package Project" : "Request Review", isCloud ? "actions/package-project.svg" : "ui/plus.svg")}${button(isCloud ? "Deploy Preview" : "Open Source Control", "ui/plus.svg", "primary")}`)}
      ${metricGrid(metrics.map(([label, value, detail], index) => [label, value, detail, index === 0 ? "accent" : ""]), "four")}
      <section class="content-grid two-wide overview-grid">
        <article class="panel tall">
          ${sectionTitle(isCloud ? "Cloud Overview" : "Team Overview", isCloud ? "Package, service, and deploy status in the same row density as Projects." : "Identity, reviewers, and local collaboration status in compact Hub rows.")}
          ${tabStrip(isCloud ? ["Package", "Services", "Health"] : ["Identity", "Review", "Access"], 0)}
          <div class="row-list compact">${leftRows.map(([mark, title, detail, badge, tone]) => infoRow(mark, title, detail, badge, tone)).join("")}</div>
        </article>
        <article class="panel tall">
          ${sectionTitle("Actions", "Primary page actions reuse the Dashboard quick-action row rhythm.")}
          <div class="row-list compact">${actions.map(([mark, title, detail]) => actionRow(mark, title, detail)).join("")}</div>
          <div class="overview-checks">
            ${toggleRow(isCloud ? "Auto upload artifacts" : "Notify reviewers", isCloud ? "Queue upload after successful package" : "Send local review notes when evidence updates", true)}
            ${checkboxRow(isCloud ? "Include debug symbols" : "Attach visual evidence", isCloud ? "Keep development package inspectable" : "Link exported screenshots and contract output", true)}
          </div>
        </article>
      </section>`;
  }

  function renderSettingsPage() {
    const page = pages["hub-settings"];
    const panels = [
      ["Toolchain", [
        comboBox("Engine Channel", "Zircon Engine 1.8.2", "Active source engine"),
        inputBox("Rust Toolchain", "stable-x86_64-pc-windows-msvc", "Used for local builds"),
      ]],
      ["Build Defaults", [
        comboBox("Profile", "Development", "Default project build profile"),
        comboBox("Target Platform", "Windows", "Primary local package target"),
        toggleRow("Symbols", "Generate debug symbols for local packages", true),
      ]],
      ["Default Paths", [
        inputBox("Project Root", "C:\\ZirconProjects", "New and imported projects"),
        inputBox("Package Output", "D:\\ZirconPackages", "Build and cloud package staging"),
      ]],
      ["Configuration Health", [
        infoRow("SP", "Selected Project", "No project selected / actions paused", "Review", "warning"),
        infoRow("SE", "Source Engine", "Active / Zircon Engine 1.8.2", "Register", "warning"),
        infoRow("PY", "Python", "PATH / python", "PATH", "accent"),
      ]],
    ];
    return `
      ${pageHeading(page, `${button("Reset", "ui/refresh.svg")}${button("Save Settings", "ui/plus.svg", "primary")}`)}
      ${tabStrip(["General", "Build", "Paths", "Health"], 0)}
      <section class="settings-layout">
        <article class="panel settings-health">
          ${sectionTitle("Configuration Overview", "Shared Hub defaults, local source roots, and launch policy.")}
          <div class="mini-stat-grid">
            ${smallStat("Config", "Valid", "hub.toml parsed", "success")}
            ${smallStat("Sources", "3", "registered engines", "accent")}
            ${smallStat("Warnings", "1", "optional path", "warning")}
          </div>
          ${progress("Toolchain readiness", 92, "success")}
          ${progress("Path coverage", 78, "accent")}
        </article>
        ${panels.filter(([title]) => title !== "Build Defaults").map(([title, rows]) => `
          <article class="panel settings-panel">
            ${sectionTitle(title)}
            <div class="row-list compact">
              ${rows.join("")}
            </div>
          </article>`).join("")}
      </section>`;
  }

  function renderNewProject() {
    const page = pages["hub-projects-new"];
    const templates = [
      ["Standard Service", "official", "v1.4.0", "Alpha 2.8+", "Selected", "accent"],
      ["API Service", "official", "v2.1.0", "Alpha 2.6+", "Ready", ""],
      ["Data Pipeline", "official", "v1.7.2", "Alpha 2.5+", "Ready", ""],
      ["Worker", "community", "v0.9.1", "Alpha 2.4+", "Ready", ""],
      ["Monolith Starter", "community", "v1.0.3", "Alpha 2.3+", "Ready", ""],
    ];
    return `
      ${pageHeading(page, `${button("Import Project", "ui/import.svg")}${button("New Project", "ui/plus.svg", "primary")}`)}
      <section class="new-project-layout">
        <article class="panel wizard-panel">
          <div class="wizard-steps">
            ${["Source & Template", "Details", "Settings", "Review"].map((label, index) => `
              <span class="${index === 0 ? "active" : ""}"><b>${index + 1}</b>${esc(label)}</span>`).join("")}
          </div>
          <div class="wizard-section source-section">
            ${sectionTitle("1. Source Engine", "Dropdown-open state is represented through selectable engine rows and a fixed detail card.")}
            <div class="source-template-grid">
              <div class="engine-selector-list">
                <button class="field-box active" type="button" data-route="hub-source-engine-popup">Select source engine <span>v</span></button>
                ${[
                  ["Engine Alpha", "v2.8.1", true],
                  ["Engine Beta", "v1.9.4", false],
                  ["Engine Gamma", "v3.2.0", false],
                  ["Engine Delta", "v0.8.7", false],
                ].map(([name, version, selected]) => `
                  <button class="engine-option ${selected ? "selected" : ""}" type="button" data-route="hub-source-engine-popup">
                    ${rowIcon("EN")}
                    <strong>${esc(name)}</strong>
                    ${tag(version, selected ? "accent" : "")}
                  </button>`).join("")}
              </div>
              <div class="engine-info-card">
                ${rowIcon("ZE")}
                <strong>Zircon Engine 1.8.2</strong>
                <span>Runtime engine, source checkout, editor launch target, and template compatibility are resolved.</span>
                <div class="chip-row">${tag("Docker")}${tag("K8s")}${tag("CLI")}${tag("SDK")}</div>
              </div>
            </div>
          </div>
          <div class="wizard-section template-section">
            ${sectionTitle("2. Template Selection", "The selected template keeps the primary create action enabled.")}
            <div class="template-toolbar">
              ${searchBox("Search templates...", true)}
              ${selectButton("Category", "", "hub-projects-browser-filter-menu", true)}
              ${selectButton("Sort", "", "hub-projects-browser-sort-menu", true)}
            </div>
            <div class="template-table">
              <div class="template-head"><span></span><span>Name</span><span>Version</span><span>Updated</span><span>Compatibility</span><span></span></div>
              ${templates.map(([title, scope, version, compat, state, tone], index) => `
                <button class="template-row ${index === 0 ? "selected" : ""}" type="button" data-route="hub-projects-detail">
                  <span>${index === 0 ? "*" : ""}</span>
                  <strong>${esc(title)} ${tag(scope)}</strong>
                  <span>${esc(version)}</span>
                  <span>${esc(index === 0 ? "2d ago" : `${index + 3}d ago`)}</span>
                  <span>${esc(compat)}</span>
                  ${tag(state, tone)}
                </button>`).join("")}
            </div>
          </div>
          <div class="wizard-footer">
            <div class="quick-options">
              ${checkLine("Initialize Repository", "enabled by default", "ready")}
              ${checkLine("Setup CI Pipeline", "optional", "idle")}
              ${checkLine("Add Sample Data", "optional", "idle")}
              ${checkLine("Enable Quality Gate", "optional", "idle")}
            </div>
            <div class="create-progress">
              ${button("Cancel", "", "")}
              ${progress("All required selections", 66, "accent")}
              ${button("Create Project", "ui/plus.svg", "primary")}
            </div>
          </div>
        </article>
        <aside class="panel blueprint-panel">
          ${sectionTitle("Project Blueprint", "Live summary")}
          <div class="blueprint-card">
            ${rowIcon("3D")}
            <strong>Standard Service</strong>
            <span>Engine Alpha / Zircon 1.8.2</span>
          </div>
          <div class="row-list compact">
            ${settingSummaryRow("Source Engine", "Engine Alpha v2.8.1")}
            ${settingSummaryRow("Template", "Standard Service")}
            ${settingSummaryRow("Compatibility", "Ready", true, "success")}
          </div>
          <div class="blueprint-bars">
            ${progress("Storage", 42)}
            ${progress("Compute", 57, "accent")}
            ${progress("Pipelines", 24)}
          </div>
          <div class="tips-box">${checkLine("Ready to create", "primary action is enabled", "ready")}${checkLine("Details step", "next section captures name and location", "idle")}</div>
        </aside>
      </section>`;
  }

  function renderProjectBrowser(menu) {
    const page = pages[menu === "filter" ? "hub-projects-browser-filter-menu" : menu === "sort" ? "hub-projects-browser-sort-menu" : "hub-projects-browser"];
    return `
      ${pageHeading(page, `${button("Import Project", "ui/import.svg")}${button("New Project", "ui/plus.svg", "primary")}`)}
      ${toolbar("Search all projects...")}
      <section class="browser-layout">
        <article class="panel browser-main">
          ${sectionTitle("All Projects", "List-first browser with stable columns, active filters, and selected-row detail reference.")}
          <div class="browser-filter-strip">
            ${["All", "Recent", "Windows", "Linux", "Missing Paths"].map((label, index) => tag(label, index === 0 ? "accent" : "")).join("")}
          </div>
          <div class="browser-table">
            <div class="browser-head"><span></span><span>Name</span><span>Engine</span><span>Platform</span><span>Modified</span><span>Status</span></div>
            ${projects.map((project, index) => `
              <button class="browser-row row-surface table-row-surface ${index === 0 ? "selected" : ""}" data-material-slot="browser-row table-row row-surface${index === 0 ? " selected-row" : ""}" type="button" data-route="hub-projects-detail">
                ${projectCover(project, "browser-thumb")}
                <strong>${esc(project.title)}<em>${esc(project.path)}</em></strong>
                <span>${esc(project.version)}</span>
                <span>${esc(project.platform)}</span>
                <span>${esc(project.tableModified)}</span>
                <span class="row-trailing-slot" data-component="row-trailing-slot" data-material-slot="row-trailing-slot">
                  ${tag(index === 2 ? "Needs Sync" : "Ready", index === 2 ? "warning" : "success")}
                </span>
              </button>`).join("")}
          </div>
          <div class="browser-footer">
            <span>Showing 1-6 of 24 projects</span>
            <div><button type="button">1</button><button type="button">2</button><button type="button">3</button><button type="button">...</button></div>
          </div>
        </article>
        <aside class="panel browser-side">
          ${sectionTitle("Selected Project", "Elysium Chronicles")}
          <div class="side-cover">${projectCover(projects[0], "side")}</div>
          <div class="mini-stat-grid">
            ${smallStat("Engine", "1.8.2", "up to date", "accent")}
            ${smallStat("Tasks", "4", "ready actions", "success")}
          </div>
          <div class="row-list compact">
            ${infoRow("ST", "Status", "Files indexed and ready", "Ready", "success")}
            ${infoRow("SC", "Source", "Zircon Engine 1.8.2", "Active", "accent")}
            ${infoRow("MD", "Last Modified", "2h ago", "Recent", "")}
          </div>
          <div class="side-actions">${button("Open in Editor", "actions/open-editor.svg", "primary")}${button("Build Project", "actions/build-project.svg")}</div>
        </aside>
      </section>
      ${menu === "filter" ? renderMenu("filter", ["All Projects", "Pinned", "Missing Paths", "Windows", "Linux"]) : ""}
      ${menu === "sort" ? renderMenu("sort", ["Last Modified", "Name", "Engine Version", "Platform", "Path"]) : ""}`;
  }

  function renderProjectDetail(confirmDelete) {
    const page = pages[confirmDelete ? "hub-projects-detail-delete-confirm" : "hub-projects-detail"];
    const selected = projects[0];
    return `
      ${pageHeading(page, `${button("Project Browser", "nav/projects.svg")}${button("Open in Editor", "actions/open-editor.svg", "primary")}`)}
      <section class="detail-layout">
        <article class="panel detail-main">
          <div class="detail-hero">
            ${projectCover(selected, "hero")}
            <div>
              <h3 class="detail-title">${esc(selected.title)}</h3>
              <p class="detail-path">${esc(selected.path)}</p>
              ${projectStatusStrip(selected.version, "Not pinned", selected.modified)}
            </div>
          </div>
          <div class="detail-stats">
            ${smallStat("Assets", "1,284", "indexed", "accent")}
            ${smallStat("Builds", "12", "last 7 days", "success")}
            ${smallStat("Warnings", "1", "non-blocking", "warning")}
            ${smallStat("Size", "12.8 GB", "workspace", "")}
          </div>
          <div class="detail-grid">
            <div class="row-list compact">
              ${infoRow("ST", "Status", "Project files are present and indexed", "Ready", "success")}
              ${infoRow("EN", "Source Engine", "Zircon Engine 1.8.2", "Active", "accent")}
              ${infoRow("VR", "Engine Version", selected.version, selected.version, "accent")}
              ${infoRow("PL", "Platform", selected.platform, selected.platform)}
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
        </article>
        ${projectDetailActionsSection(confirmDelete)}
      </section>
      ${confirmDelete ? renderDeleteConfirm() : ""}`;
  }

  function renderState(kind) {
    const page = pages[`hub-state-${kind}`];
    const copy = {
      empty: ["[]", "No projects match", "Adjust filters or import a project to repopulate this Hub view.", "Import Project", ""],
      loading: ["", "Preparing project workspace", "Loading states keep panel rhythm stable while long-running tasks report progress.", "Open Task Log", "loading"],
      error: ["!", "Project action failed", "Recoverable errors stay inside the shared Hub surface with a concrete next action.", "Retry Action", "error"],
    }[kind];
    return `
      ${pageHeading(page)}
      <section class="state-canvas ${kind}">
        <article class="state-card">
          <div class="state-mark ${copy[4]}">${esc(copy[0])}</div>
          <h3>${esc(copy[1])}</h3>
          <p>${esc(copy[2])}</p>
          ${button(copy[3], kind === "empty" ? "ui/import.svg" : "ui/refresh.svg", kind === "error" ? "danger" : "primary")}
        </article>
        <aside class="panel state-side">
          ${sectionTitle(kind === "error" ? "Recovery Checklist" : kind === "loading" ? "Task Progress" : "Suggested Filters")}
          <div class="row-list compact">
            ${kind === "error"
              ? `${checkLine("Retry available", "operation can be run again", "ready")}${checkLine("Local files intact", "no disk changes were made", "ready")}${checkLine("Diagnostics", "copyable error record", "warn")}`
              : kind === "loading"
                ? `${progress("Scanning projects", 82, "accent")}${progress("Indexing metadata", 64)}${progress("Preparing actions", 41)}`
                : `${checkLine("Clear search", "remove current text filter", "idle")}${checkLine("Show missing paths", "inspect invalid project roots", "idle")}${checkLine("Import project", "add existing folder", "ready")}`}
          </div>
        </aside>
      </section>`;
  }

  function renderNav(activeNav) {
    return navItems
      .map(([id, label, iconPath, pageId]) => `
        <a class="nav-item ${id === activeNav ? "active" : ""}" href="?page=${pageId}">
          <img src="${icon(iconPath)}" alt="">
          <span>${esc(label)}</span>
        </a>`)
      .join("");
  }

  function normalizePageId(value) {
    if (!value) return "projects-dashboard";
    if (pages[value]) return value;
    const byOutput = Object.keys(pages).find((id) => `${id}.png` === value);
    return byOutput || "projects-dashboard";
  }

  function render() {
    const search = new URLSearchParams(window.location.search);
    const pageId = normalizePageId(search.get("page"));
    const page = pages[pageId];
    document.title = `${page.title} - Zircon Hub Web Reference`;
    document.querySelector(".hub-shell").dataset.page = pageId;
    document.getElementById("hub-nav").innerHTML = renderNav(page.nav);
    document.getElementById("workspace").innerHTML = page.render();
    document.getElementById("overlay-root").innerHTML = page.overlay ? page.overlay() : "";
  }

  document.addEventListener("click", (event) => {
    const routeTarget = event.target.closest("[data-route]");
    if (!routeTarget) return;
    const pageId = routeTarget.getAttribute("data-route");
    if (!pages[pageId]) return;
    event.preventDefault();
    event.stopPropagation();
    const nextUrl = new URL(window.location.href);
    nextUrl.searchParams.set("page", pageId);
    window.history.pushState({}, "", nextUrl);
    render();
  });

  window.addEventListener("popstate", render);

  render();
})();
