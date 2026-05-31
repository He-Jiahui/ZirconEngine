(function () {
  const actionRoutes = {
    "Refresh Sources": "hub-state-loading",
    "Open Output": "hub-state-loading",
    "Sync Source": "hub-state-loading",
    "Open Editor": "hub-editor",
    "Open in Editor": "hub-editor",
    "Build Project": "hub-builds",
    "Package Project": "hub-cloud",
    "Install to Device": "hub-builds",
    "Upload Package": "hub-state-loading",
    "Deploy Preview": "hub-state-loading",
    "Open Service Logs": "hub-state-loading",
    "Configure Cloud": "hub-settings",
    "Open Source Control": "hub-team",
    "Request Review": "hub-state-loading",
    "Open Team Home": "hub-team",
    "Sync Metadata": "hub-state-loading",
    "Delete Project": "hub-projects-detail-delete-confirm",
    "Open Task Log": "hub-builds",
    "Check for Updates": "hub-state-loading",
    "Add Asset": "hub-state-loading",
    "Add Plugin": "hub-state-loading",
    "Add Guide": "hub-state-loading",
  };
  const defaultFilter = "All Projects";
  const defaultSort = "Last Modified";
  const rowsPerInteractivePage = 2;

  function shell() {
    return document.querySelector(".hub-shell");
  }

  function normalizeLabel(value) {
    return String(value ?? "")
      .replace(/\s+/g, " ")
      .replace(/[>!]+$/g, "")
      .trim();
  }

  function controlLabel(button) {
    const strong = button.querySelector("strong");
    if (strong) {
      return normalizeLabel(strong.textContent);
    }
    const explicitSpan = button.querySelector("span:not(.split-caret)");
    return normalizeLabel(explicitSpan?.textContent || button.textContent || button.getAttribute("aria-label"));
  }

  function navigate(pageId) {
    if (!pageId) {
      return false;
    }
    const nextUrl = new URL(window.location.href);
    nextUrl.searchParams.set("page", pageId);
    window.history.pushState({}, "", nextUrl);
    window.dispatchEvent(new PopStateEvent("popstate"));
    return true;
  }

  function setUrlState(nextState) {
    const nextUrl = new URL(window.location.href);
    Object.entries(nextState).forEach(([key, value]) => {
      if (value === null || value === undefined || value === "") {
        nextUrl.searchParams.delete(key);
      } else {
        nextUrl.searchParams.set(key, value);
      }
    });
    window.history.pushState({}, "", nextUrl);
    window.dispatchEvent(new PopStateEvent("popstate"));
  }

  function currentPage() {
    return shell()?.dataset.page || new URLSearchParams(window.location.search).get("page") || "projects-dashboard";
  }

  function currentFilter() {
    const value = new URLSearchParams(window.location.search).get("filter") || defaultFilter;
    return value === "All" ? defaultFilter : value;
  }

  function currentSort() {
    return new URLSearchParams(window.location.search).get("sort") || defaultSort;
  }

  function replaceSelectLabel(button, label) {
    const icons = [...button.querySelectorAll("img")].map((image) => image.cloneNode(true));
    button.replaceChildren();
    if (icons[0]) {
      button.append(icons[0]);
    }
    button.append(document.createTextNode(label));
    if (icons[1]) {
      button.append(icons[1]);
    }
  }

  function routeForButton(button) {
    if (!button || button.disabled) {
      return "";
    }
    const label = controlLabel(button);
    if (button.closest(".confirm-panel")) {
      if (label === "Cancel") {
        return "hub-projects-detail";
      }
      if (label === "Delete Project") {
        return "hub-state-empty";
      }
    }
    if (button.closest(".browser-footer")) {
      return "hub-projects-browser";
    }
    if (button.matches(".toolbar .mode-button")) {
      const buttons = [...button.closest(".toolbar").querySelectorAll(".mode-button")];
      return buttons.indexOf(button) === 0 ? "projects-dashboard" : "hub-projects-browser";
    }
    if (button.closest(".engine-card")) {
      return "hub-state-loading";
    }
    return actionRoutes[label] || "";
  }

  function isLocalUiButton(button) {
    return Boolean(
      button?.matches(".collapse-button, .window-control, .mode-button.active") ||
        button?.closest(".button-states"),
    );
  }

  function hasButtonHandler(button) {
    return Boolean(
      button &&
        !button.disabled &&
        (button.dataset.route || button.dataset.uiAction || routeForButton(button) || isLocalUiButton(button)),
    );
  }

  function setPressed(button, pressed) {
    button.setAttribute("aria-pressed", pressed ? "true" : "false");
    button.classList.toggle("is-demo-selected", Boolean(pressed));
  }

  function handleLocalUi(button) {
    const hubShell = shell();
    if (!hubShell) {
      return false;
    }

    if (button.matches(".collapse-button")) {
      const collapsed = !hubShell.classList.contains("sidebar-collapsed");
      hubShell.classList.toggle("sidebar-collapsed", collapsed);
      setPressed(button, collapsed);
      return true;
    }

    if (button.matches(".window-control")) {
      const className = button.matches(".close")
        ? "window-closed"
        : button.matches(".square")
          ? "window-maximized"
          : "window-minimized";
      const active = !hubShell.classList.contains(className);
      hubShell.classList.toggle(className, active);
      setPressed(button, active);
      return true;
    }

    if (button.closest(".button-states")) {
      const group = button.closest(".state-group");
      group?.querySelectorAll("button").forEach((candidate) => setPressed(candidate, candidate === button));
      return true;
    }

    if (button.matches(".mode-button.active")) {
      setPressed(button, true);
      return true;
    }

    return false;
  }

  function applySearch(input) {
    const query = normalizeLabel(input.value).toLowerCase();
    const workspace = input.closest("#workspace") || document.getElementById("workspace");
    if (!workspace) {
      return;
    }
    const candidates = workspace.querySelectorAll(
      ".project-card, .project-row, .browser-row, .catalog-row, .template-row, .quick-row, .action-row",
    );
    candidates.forEach((candidate) => {
      const hidden = Boolean(query) && !candidate.innerText.toLowerCase().includes(query);
      candidate.classList.toggle("is-filter-hidden", hidden);
      candidate.setAttribute("aria-hidden", hidden ? "true" : "false");
    });
  }

  function readBrowserRows() {
    return [...document.querySelectorAll(".browser-row")].map((row, index) => {
      if (!row.dataset.originalIndex) {
        row.dataset.originalIndex = String(index);
      }
      const strong = row.querySelector("strong");
      const detail = strong?.querySelector("em");
      const spans = [...row.children].filter((child) => child.tagName === "SPAN" && !child.classList.contains("project-cover") && !child.classList.contains("tag"));
      return {
        row,
        originalIndex: Number(row.dataset.originalIndex),
        name: normalizeLabel(strong?.childNodes?.[0]?.textContent || strong?.textContent || ""),
        path: normalizeLabel(detail?.textContent || ""),
        engine: normalizeLabel(spans[0]?.textContent || ""),
        platform: normalizeLabel(spans[1]?.textContent || ""),
        modified: normalizeLabel(spans[2]?.textContent || ""),
        status: normalizeLabel(row.querySelector(".tag")?.textContent || ""),
        text: normalizeLabel(row.textContent).toLowerCase(),
      };
    });
  }

  function matchesBrowserFilter(item, filter) {
    if (filter === defaultFilter) return true;
    if (filter === "Recent") return item.originalIndex < 3;
    if (filter === "Pinned") return item.originalIndex < 2;
    if (filter === "Missing Paths") return item.status === "Needs Sync";
    return item.text.includes(filter.toLowerCase());
  }

  function sortBrowserRows(items, sortLabel) {
    const key = {
      Name: "name",
      "Engine Version": "engine",
      Platform: "platform",
      Path: "path",
    }[sortLabel];
    if (!key) {
      return [...items].sort((a, b) => a.originalIndex - b.originalIndex);
    }
    return [...items].sort((a, b) => a[key].localeCompare(b[key]) || a.originalIndex - b.originalIndex);
  }

  function applyBrowserState() {
    const page = currentPage();
    const isBrowser = page.startsWith("hub-projects-browser");
    const toolbar = document.querySelector(".toolbar");
    if (toolbar) {
      const modes = [...toolbar.querySelectorAll(".mode-button")];
      modes.forEach((button, index) => {
        const active = isBrowser ? index === 1 : index === 0;
        button.classList.toggle("active", active);
        button.setAttribute("aria-pressed", active ? "true" : "false");
      });
    }
    if (!isBrowser) {
      return;
    }

    const filter = currentFilter();
    const sort = currentSort();
    const selectButtons = [...document.querySelectorAll(".toolbar .select-button")];
    if (selectButtons[0]) replaceSelectLabel(selectButtons[0], filter);
    if (selectButtons[1]) replaceSelectLabel(selectButtons[1], sort);

    document.querySelectorAll(".browser-filter-strip .tag").forEach((chip) => {
      const label = normalizeLabel(chip.textContent) === "All" ? defaultFilter : normalizeLabel(chip.textContent);
      chip.classList.toggle("accent", label === filter);
    });

    document.querySelectorAll(".menu-panel button").forEach((button) => {
      const label = normalizeLabel(button.querySelector("span")?.textContent || button.textContent);
      const active = label === (button.closest(".filter") ? filter : sort);
      button.classList.toggle("active", active);
      const marker = button.querySelector("span:last-child");
      if (marker) marker.textContent = active ? "OK" : "";
    });

    const table = document.querySelector(".browser-table");
    const items = sortBrowserRows(readBrowserRows(), sort);
    items.forEach((item) => table?.append(item.row));
    const filtered = items.filter((item) => matchesBrowserFilter(item, filter));
    const pageParam = Number(new URLSearchParams(window.location.search).get("browserPage") || 0);
    const pageItems = pageParam > 0 ? filtered.slice((pageParam - 1) * rowsPerInteractivePage, pageParam * rowsPerInteractivePage) : filtered;
    items.forEach((item) => {
      const hidden = !pageItems.includes(item);
      item.row.classList.toggle("is-state-hidden", hidden);
      item.row.setAttribute("aria-hidden", hidden ? "true" : "false");
      item.row.classList.toggle("selected", item === pageItems[0]);
    });

    const footerLabel = document.querySelector(".browser-footer > span");
    const hasState = filter !== defaultFilter || sort !== defaultSort || pageParam > 0;
    if (footerLabel && hasState) {
      const start = pageItems.length ? (pageParam > 0 ? (pageParam - 1) * rowsPerInteractivePage + 1 : 1) : 0;
      const end = pageItems.length ? start + pageItems.length - 1 : 0;
      footerLabel.textContent = `Showing ${start}-${end} of ${filtered.length} projects`;
    }
    document.querySelectorAll(".browser-footer button").forEach((button) => {
      const label = normalizeLabel(button.textContent);
      const active = String(pageParam || 1) === (label === "..." ? "3" : label);
      button.classList.toggle("active", active);
      button.setAttribute("aria-pressed", active ? "true" : "false");
    });
  }

  function engineLabelFromOption(button) {
    const name = normalizeLabel(button.querySelector("strong")?.textContent || button.querySelector("span:nth-child(2)")?.textContent || button.textContent);
    const version = normalizeLabel(button.querySelector(".tag")?.textContent || "");
    return version && !name.includes(version) ? `${name} ${version}` : name;
  }

  function applyEngineState() {
    const engine = new URLSearchParams(window.location.search).get("engine");
    if (!engine) {
      return;
    }
    const topbarLabel = document.querySelector(".engine-select span");
    if (topbarLabel) {
      topbarLabel.textContent = engine;
    }
    document.querySelectorAll(".engine-pop-row, .engine-option").forEach((button) => {
      const selected = engineLabelFromOption(button) === engine || engine.startsWith(normalizeLabel(button.querySelector("strong")?.textContent || ""));
      button.classList.toggle("is-menu-selected", selected);
      button.classList.toggle("selected", button.classList.contains("engine-option") && selected);
      button.setAttribute("aria-selected", selected ? "true" : "false");
    });
    const field = document.querySelector(".field-box.active");
    const caret = field?.querySelector("span")?.cloneNode(true);
    if (field && caret) {
      field.replaceChildren(document.createTextNode(`Source: ${engine}`), caret);
    }
  }

  function applyInteractiveState() {
    applyBrowserState();
    applyEngineState();
  }

  document.addEventListener(
    "click",
    (event) => {
      const menuButton = event.target.closest(".menu-panel button");
      if (menuButton) {
        const label = normalizeLabel(menuButton.querySelector("span")?.textContent || menuButton.textContent);
        const key = menuButton.closest(".filter") ? "filter" : "sort";
        event.preventDefault();
        event.stopPropagation();
        setUrlState({
          page: "hub-projects-browser",
          browserPage: null,
          [key]: key === "filter" && label === defaultFilter ? null : label,
        });
        return;
      }

      const footerButton = event.target.closest(".browser-footer button");
      if (footerButton) {
        const label = normalizeLabel(footerButton.textContent);
        event.preventDefault();
        event.stopPropagation();
        setUrlState({ page: "hub-projects-browser", browserPage: label === "..." ? "3" : label });
        return;
      }

      const engineOption = event.target.closest(".engine-option");
      if (engineOption) {
        event.preventDefault();
        event.stopPropagation();
        setUrlState({ page: "hub-projects-new", engine: engineLabelFromOption(engineOption) });
        return;
      }

      const enginePopoverRow = event.target.closest(".engine-pop-row");
      if (enginePopoverRow) {
        event.preventDefault();
        event.stopPropagation();
        setUrlState({ page: "projects-dashboard", engine: engineLabelFromOption(enginePopoverRow) });
      }
    },
    true,
  );

  document.addEventListener("click", (event) => {
    if (event.target.closest("[data-route]")) {
      return;
    }
    const button = event.target.closest("button");
    if (!button || button.disabled) {
      return;
    }
    if (handleLocalUi(button)) {
      event.preventDefault();
      return;
    }
    const pageId = routeForButton(button);
    if (pageId) {
      event.preventDefault();
      event.stopPropagation();
      navigate(pageId);
    }
  });

  document.addEventListener("input", (event) => {
    const input = event.target.closest(".search-box input");
    if (input) {
      applySearch(input);
    }
  });

  window.addEventListener("popstate", applyInteractiveState);

  window.ZirconHubInteractions = {
    ready: true,
    actionRouteLabels: Object.keys(actionRoutes),
    hasButtonHandler,
    applyInteractiveState,
    routeForButtonLabel(label) {
      return actionRoutes[normalizeLabel(label)] || "";
    },
  };
  applyInteractiveState();
})();
