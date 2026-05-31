import { spawn } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { DASHBOARD_PAGE_ID, EXPORTS_LIST } from "./page-registry.mjs";

const edgeCandidates = [
  "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe",
  "C:/Program Files/Microsoft/Edge/Application/msedge.exe",
];
const edge = edgeCandidates.find((candidate) => existsSync(candidate));
if (!edge) {
  throw new Error("Microsoft Edge executable not found.");
}

const port = Number.parseInt(
  process.env.ZIRCON_HUB_WEB_REFERENCE_CDP_PORT ?? String(9933 + Math.floor(Math.random() * 500)),
  10,
);
const profile = mkdtempSync(join(tmpdir(), "zircon-hub-cdp-"));
const here = dirname(fileURLToPath(import.meta.url));
const referenceUrl = pathToFileURL(resolve(here, "index.html")).href;
const repoRoot = resolve(here, "../../..");
const exportsIndex = resolve(here, "EXPORTS.md");
const browser = spawn(
  edge,
  [
    "--headless=new",
    "--disable-gpu",
    "--hide-scrollbars",
    "--allow-file-access-from-files",
    "--edge-skip-compat-layer-relaunch",
    `--remote-debugging-port=${port}`,
    `--user-data-dir=${profile}`,
    "about:blank",
  ],
  { stdio: "ignore" },
);

let nextId = 1;

try {
  const list = await waitForJson(`http://127.0.0.1:${port}/json/list`);
  const target = list.find((item) => item.type === "page") ?? list[0];
  const cdp = await connect(target.webSocketDebuggerUrl);
  await cdp.send("Page.enable");
  await cdp.send("Runtime.enable");

  const knownPageIds = new Set([DASHBOARD_PAGE_ID, ...EXPORTS_LIST.map(([pageId]) => pageId)]);
  const baseUrl = `${referenceUrl}?page=${DASHBOARD_PAGE_ID}`;
  // representative Hub web-reference click routes
  const checks = [
    {
      name: "source engine selector opens popup",
      selector: "button.engine-select",
      expectedPage: "hub-source-engine-popup",
      expectedText: "Source Engines",
    },
    {
      name: "user menu opens account menu",
      selector: "button.user-menu",
      expectedPage: "hub-user-menu",
      expectedText: "Alex Developer",
    },
    {
      name: "new project button opens project creation",
      selector: ".heading-actions .button.primary",
      expectedPage: "hub-projects-new",
      expectedText: "New Project",
    },
    {
      name: "project card opens project detail",
      selector: ".project-card",
      expectedPage: "hub-projects-detail",
      expectedText: "Project Detail",
    },
    {
      name: "browser filter select opens filter menu",
      selector: ".toolbar .select-button:first-of-type",
      expectedPage: "hub-projects-browser-filter-menu",
      expectedText: "Filter menu open.",
    },
    {
      name: "browser sort select opens sort menu",
      selector: ".toolbar .select-button:nth-of-type(2)",
      expectedPage: "hub-projects-browser-sort-menu",
      expectedText: "Sort menu open.",
    },
    {
      name: "quick action build opens builds page",
      selector: ".quick-row:first-child",
      expectedPage: "hub-builds",
      expectedText: "Builds",
    },
    {
      name: "notification icon opens loading state",
      selector: ".top-actions .icon-only:first-child",
      expectedPage: "hub-state-loading",
      expectedText: "Preparing project workspace",
    },
    {
      name: "help icon opens learn page",
      selector: ".top-actions .icon-only:nth-child(2)",
      expectedPage: "hub-learn",
      expectedText: "Learn",
    },
    {
      name: "settings icon opens settings page",
      selector: ".top-actions .icon-only:nth-child(3)",
      expectedPage: "hub-settings",
      expectedText: "Settings",
    },
    ...[
      ["sidebar projects nav opens dashboard", `a.nav-item[href="?page=${DASHBOARD_PAGE_ID}"]`, DASHBOARD_PAGE_ID, "Projects"],
      ["sidebar editor nav opens editor", `a.nav-item[href="?page=hub-editor"]`, "hub-editor", "Editor"],
      ["sidebar assets nav opens assets", `a.nav-item[href="?page=hub-assets"]`, "hub-assets", "Assets"],
      ["sidebar builds nav opens builds", `a.nav-item[href="?page=hub-builds"]`, "hub-builds", "Builds"],
      ["sidebar plugins nav opens plugins", `a.nav-item[href="?page=hub-plugins"]`, "hub-plugins", "Plugins"],
      ["sidebar cloud nav opens cloud", `a.nav-item[href="?page=hub-cloud"]`, "hub-cloud", "Cloud"],
      ["sidebar team nav opens team", `a.nav-item[href="?page=hub-team"]`, "hub-team", "Team"],
      ["sidebar learn nav opens learn", `a.nav-item[href="?page=hub-learn"]`, "hub-learn", "Learn"],
      ["sidebar settings nav opens settings", `a.nav-item[href="?page=hub-settings"]`, "hub-settings", "Settings"],
    ].map(([name, selector, expectedPage, expectedText]) => ({ name, selector, expectedPage, expectedText })),
    {
      name: "engine status update button opens loading state",
      selector: ".engine-card button",
      expectedPage: "hub-state-loading",
      expectedText: "Preparing project workspace",
    },
    {
      name: "import project opens project browser",
      selector: ".heading-actions .button:first-child",
      expectedPage: "hub-projects-browser",
      expectedText: "Project Browser",
    },
    {
      name: "project card menu opens delete confirmation",
      selector: ".project-card .cover button",
      expectedPage: "hub-projects-detail-delete-confirm",
      expectedText: "Delete project from Hub?",
    },
    {
      name: "view all opens project browser",
      selector: ".view-all",
      expectedPage: "hub-projects-browser",
      expectedText: "Project Browser",
    },
    {
      name: "quick action install opens builds page",
      selector: ".quick-row:nth-child(2)",
      expectedPage: "hub-builds",
      expectedText: "Builds",
    },
    {
      name: "quick action package opens cloud page",
      selector: ".quick-row:nth-child(3)",
      expectedPage: "hub-cloud",
      expectedText: "Cloud",
    },
    {
      name: "quick action editor opens editor page",
      selector: ".quick-row:nth-child(4)",
      expectedPage: "hub-editor",
      expectedText: "Editor",
    },
    {
      name: "new project source selector opens source popup",
      startPage: "hub-projects-new",
      selector: ".field-box",
      expectedPage: "hub-source-engine-popup",
      expectedText: "Source Engines",
    },
    {
      name: "new project category opens filter menu",
      startPage: "hub-projects-new",
      selector: ".template-toolbar .select-button:first-of-type",
      expectedPage: "hub-projects-browser-filter-menu",
      expectedText: "Filter menu open.",
    },
    {
      name: "new project sort opens sort menu",
      startPage: "hub-projects-new",
      selector: ".template-toolbar .select-button:nth-of-type(2)",
      expectedPage: "hub-projects-browser-sort-menu",
      expectedText: "Sort menu open.",
    },
    {
      name: "new project template row opens project detail",
      startPage: "hub-projects-new",
      selector: ".template-row:first-of-type",
      expectedPage: "hub-projects-detail",
      expectedText: "Project Detail",
    },
    {
      name: "new project create opens project detail",
      startPage: "hub-projects-new",
      selector: ".create-progress .button.primary",
      expectedPage: "hub-projects-detail",
      expectedText: "Project Detail",
    },
    {
      name: "new project cancel returns dashboard",
      startPage: "hub-projects-new",
      selector: ".create-progress .button:first-child",
      expectedPage: DASHBOARD_PAGE_ID,
      expectedText: "Projects",
    },
    {
      name: "browser selected row opens project detail",
      startPage: "hub-projects-browser",
      selector: ".browser-row:first-of-type",
      expectedPage: "hub-projects-detail",
      expectedText: "Project Detail",
    },
    {
      name: "browser side editor action opens editor page",
      startPage: "hub-projects-browser",
      selector: ".side-actions .button.primary",
      expectedPage: "hub-editor",
      expectedText: "Editor",
    },
    {
      name: "browser side build action opens builds page",
      startPage: "hub-projects-browser",
      selector: ".side-actions .button:not(.primary)",
      expectedPage: "hub-builds",
      expectedText: "Builds",
    },
    {
      name: "browser pagination stays in browser with page state",
      startPage: "hub-projects-browser",
      selector: ".browser-footer button:nth-child(2)",
      expectedPage: "hub-projects-browser",
      expectedText: "Project Browser",
    },
    {
      name: "filter menu row returns browser",
      startPage: "hub-projects-browser-filter-menu",
      selector: ".menu-panel.filter button:nth-child(2)",
      expectedPage: "hub-projects-browser",
      expectedText: "Project Browser",
    },
    {
      name: "sort menu row returns browser",
      startPage: "hub-projects-browser-sort-menu",
      selector: ".menu-panel.sort button:nth-child(3)",
      expectedPage: "hub-projects-browser",
      expectedText: "Project Browser",
    },
    {
      name: "detail project browser action returns browser",
      startPage: "hub-projects-detail",
      selector: ".heading-actions .button:first-child",
      expectedPage: "hub-projects-browser",
      expectedText: "Project Browser",
    },
    {
      name: "detail open editor action opens editor page",
      startPage: "hub-projects-detail",
      selector: ".detail-side .action-row:nth-of-type(1)",
      expectedPage: "hub-editor",
      expectedText: "Editor",
    },
    {
      name: "detail build action opens builds page",
      startPage: "hub-projects-detail",
      selector: ".detail-side .action-row:nth-of-type(2)",
      expectedPage: "hub-builds",
      expectedText: "Builds",
    },
    {
      name: "detail package action opens cloud page",
      startPage: "hub-projects-detail",
      selector: ".detail-side .action-row:nth-of-type(3)",
      expectedPage: "hub-cloud",
      expectedText: "Cloud",
    },
    {
      name: "detail delete action opens confirmation",
      startPage: "hub-projects-detail",
      selector: ".detail-side .action-row:nth-of-type(5)",
      expectedPage: "hub-projects-detail-delete-confirm",
      expectedText: "Delete project from Hub?",
    },
    {
      name: "delete confirm cancel returns detail",
      startPage: "hub-projects-detail-delete-confirm",
      selector: ".confirm-panel .button:first-child",
      expectedPage: "hub-projects-detail",
      expectedText: "Project Detail",
    },
    {
      name: "delete confirm action opens empty state",
      startPage: "hub-projects-detail-delete-confirm",
      selector: ".confirm-panel .button.danger",
      expectedPage: "hub-state-empty",
      expectedText: "No projects match",
    },
    {
      name: "editor refresh sources opens loading state",
      startPage: "hub-editor",
      selector: ".heading-actions .button:first-child",
      expectedPage: "hub-state-loading",
      expectedText: "Preparing project workspace",
    },
    {
      name: "editor sync source opens loading state",
      startPage: "hub-editor",
      selector: ".control-stack .command-card:nth-child(2)",
      expectedPage: "hub-state-loading",
      expectedText: "Preparing project workspace",
    },
    {
      name: "build package action opens cloud page",
      startPage: "hub-builds",
      selector: ".build-side .action-row:first-of-type",
      expectedPage: "hub-cloud",
      expectedText: "Cloud",
    },
    {
      name: "assets add action opens loading state",
      startPage: "hub-assets",
      selector: ".heading-actions .button.primary",
      expectedPage: "hub-state-loading",
      expectedText: "Preparing project workspace",
    },
    {
      name: "cloud deploy action opens loading state",
      startPage: "hub-cloud",
      selector: ".two-wide .panel:nth-child(2) .action-row:nth-of-type(2)",
      expectedPage: "hub-state-loading",
      expectedText: "Preparing project workspace",
    },
    {
      name: "team request review opens loading state",
      startPage: "hub-team",
      selector: ".heading-actions .button:first-child",
      expectedPage: "hub-state-loading",
      expectedText: "Preparing project workspace",
    },
    {
      name: "loading task log opens builds page",
      startPage: "hub-state-loading",
      selector: ".state-card .button",
      expectedPage: "hub-builds",
      expectedText: "Builds",
    },
    {
      name: "error retry returns dashboard",
      startPage: "hub-state-error",
      selector: ".state-card .button",
      expectedPage: DASHBOARD_PAGE_ID,
      expectedText: "Projects",
    },
  ];
  const localChecks = [
    {
      name: "sidebar collapse toggles shell state",
      selector: ".collapse-button",
      expectedShellClass: "sidebar-collapsed",
    },
    {
      name: "window maximize toggles shell state",
      selector: ".window-control.square",
      expectedShellClass: "window-maximized",
    },
    {
      name: "button-state sample records selected demo state",
      selector: ".primary-states button:nth-child(2)",
      expectedElementClass: "is-demo-selected",
    },
    {
      name: "active grid mode records selected demo state",
      selector: ".toolbar .mode-button.active",
      expectedElementClass: "is-demo-selected",
    },
  ];
  const searchChecks = [
    {
      name: "dashboard search filters project rows",
      startPage: DASHBOARD_PAGE_ID,
      selector: ".toolbar .search-box input",
      query: "Stellar",
      visibleSelector: ".project-card:not(.is-filter-hidden), .project-row:not(.is-filter-hidden)",
      expectedText: "Stellar Outpost",
    },
    {
      name: "new project template search filters template rows",
      startPage: "hub-projects-new",
      selector: ".template-toolbar input",
      query: "Worker",
      visibleSelector: ".template-row:not(.is-filter-hidden)",
      expectedText: "Worker",
    },
    {
      name: "assets search filters catalog rows",
      startPage: "hub-assets",
      selector: ".toolbar .search-box input",
      query: "Door",
      visibleSelector: ".catalog-row:not(.is-filter-hidden)",
      expectedText: "HangarDoor_Normal",
    },
  ];
  const stateChecks = [
    {
      name: "filter menu applies Windows project filter",
      startPage: "hub-projects-browser-filter-menu",
      selector: ".menu-panel.filter button:nth-of-type(4)",
      expectedPage: "hub-projects-browser",
      expression: `(() => {
        const visible = [...document.querySelectorAll(".browser-row:not(.is-state-hidden)")].map((row) => row.textContent);
        const filterLabel = [...document.querySelectorAll(".toolbar .select-button")][0]?.innerText ?? "";
        const hidden = [...document.querySelectorAll(".browser-row.is-state-hidden")].map((row) => row.textContent);
        return JSON.stringify({ ok: visible.length > 0 && visible.every((text) => text.includes("Windows")) && hidden.some((text) => text.includes("Linux")) && filterLabel.includes("Windows"), filterLabel, visible });
      })()`,
    },
    {
      name: "sort menu applies platform ordering",
      startPage: "hub-projects-browser-sort-menu",
      selector: ".menu-panel.sort button:nth-of-type(4)",
      expectedPage: "hub-projects-browser",
      expression: `(() => {
        const rows = [...document.querySelectorAll(".browser-row:not(.is-state-hidden)")].map((row) => row.innerText);
        const sortLabel = [...document.querySelectorAll(".toolbar .select-button")][1]?.innerText ?? "";
        return JSON.stringify({ ok: rows[0]?.includes("Sands of Time") && sortLabel.includes("Platform"), sortLabel, rows: rows.slice(0, 3) });
      })()`,
    },
    {
      name: "browser pagination applies visible page slice",
      startPage: "hub-projects-browser",
      selector: ".browser-footer button:nth-child(2)",
      expectedPage: "hub-projects-browser",
      expression: `(() => {
        const visible = [...document.querySelectorAll(".browser-row:not(.is-state-hidden)")].map((row) => row.innerText);
        const footer = document.querySelector(".browser-footer > span")?.innerText ?? "";
        return JSON.stringify({ ok: visible.length === 2 && footer.includes("Showing 3-4 of 6 projects"), footer, visible });
      })()`,
    },
    {
      name: "source engine popup selection updates topbar engine",
      startPage: "hub-source-engine-popup",
      selector: ".source-popover .engine-pop-row:nth-of-type(2)",
      expectedPage: "projects-dashboard",
      expression: `(() => {
        const label = document.querySelector(".engine-select span")?.innerText ?? "";
        return JSON.stringify({ ok: label.includes("Zircon Engine 1.8.1"), label });
      })()`,
    },
    {
      name: "new project engine option updates selected source",
      startPage: "hub-projects-new",
      selector: ".engine-selector-list .engine-option:nth-of-type(3)",
      expectedPage: "hub-projects-new",
      expression: `(() => {
        const selected = document.querySelector(".engine-option.selected strong")?.innerText ?? "";
        const field = document.querySelector(".field-box.active")?.innerText ?? "";
        return JSON.stringify({ ok: selected.includes("Engine Beta") && field.includes("Engine Beta"), selected, field });
      })()`,
    },
    {
      name: "browser grid mode returns dashboard grid view",
      startPage: "hub-projects-browser",
      selector: ".toolbar .mode-button",
      expectedPage: "projects-dashboard",
      expression: `(() => {
        const active = document.querySelector(".toolbar .mode-button.active");
        const heading = document.querySelector(".page-heading h2")?.innerText ?? "";
        return JSON.stringify({ ok: heading === "Projects" && Boolean(active), heading, activeClass: active?.className ?? "" });
      })()`,
    },
  ];

  const unknownRoutes = [];
  const unknownPageLinks = [];
  const unhandledButtons = [];
  for (const pageId of knownPageIds) {
    await cdp.send("Page.navigate", { url: `${referenceUrl}?page=${encodeURIComponent(pageId)}` });
    await waitForPage(cdp, pageId);
    const routeState = await evaluate(
      cdp,
      `JSON.stringify([...document.querySelectorAll("[data-route]")].map((node) => node.getAttribute("data-route")))`,
    );
    for (const route of JSON.parse(routeState)) {
      if (!knownPageIds.has(route)) {
        unknownRoutes.push(`${pageId} -> ${route}`);
      }
    }

    const linkState = await evaluate(
      cdp,
      `JSON.stringify([...document.querySelectorAll("a[href*='?page=']")].map((node) => new URL(node.getAttribute("href"), location.href).searchParams.get("page")))`,
    );
    for (const linkPageId of JSON.parse(linkState)) {
      if (!knownPageIds.has(linkPageId)) {
        unknownPageLinks.push(`${pageId} -> ${linkPageId}`);
      }
    }

    const buttonState = await evaluate(
      cdp,
      `JSON.stringify([...document.querySelectorAll("button:not([disabled])")]
        .filter((button) => !window.ZirconHubInteractions?.hasButtonHandler(button))
        .map((button) => ({
          label: button.innerText.replace(/\\s+/g, " ").trim(),
          className: button.className,
        })))`,
    );
    for (const button of JSON.parse(buttonState)) {
      unhandledButtons.push(`${pageId} -> ${button.label || button.className || "<unlabeled button>"}`);
    }
  }
  if (unknownRoutes.length > 0) {
    throw new Error(`Unknown Hub web-reference data-route targets: ${unknownRoutes.join(", ")}`);
  }
  if (unknownPageLinks.length > 0) {
    throw new Error(`Unknown Hub web-reference page href targets: ${unknownPageLinks.join(", ")}`);
  }
  if (unhandledButtons.length > 0) {
    throw new Error(`Unhandled Hub web-reference buttons: ${unhandledButtons.join(", ")}`);
  }

  for (const [pageId, outputName] of EXPORTS_LIST) {
    await cdp.send("Page.navigate", { url: `${referenceUrl}?page=${encodeURIComponent(pageId)}` });
    await waitForPage(cdp, pageId);
    const expectedState = await pageIdentity(cdp);

    await cdp.send("Page.navigate", { url: `${referenceUrl}?page=${encodeURIComponent(outputName)}` });
    await waitForPage(cdp, pageId);
    const normalizedState = await pageIdentity(cdp);
    if (
      expectedState.page !== pageId ||
      normalizedState.page !== pageId ||
      !normalizedState.title.endsWith(" - Zircon Hub Web Reference") ||
      !normalizedState.heading
    ) {
      throw new Error(
        `Output filename replay ${outputName} did not normalize to page ${pageId}: expected ${JSON.stringify(expectedState)}, normalized ${JSON.stringify(normalizedState)}.`,
      );
    }
  }

  const exportReplayRows = readExportReplayRows();
  for (const { pageId, replayPath } of exportReplayRows) {
    await cdp.send("Page.navigate", { url: replayPathToFileUrl(replayPath) });
    await waitForPage(cdp, pageId);
    const state = await pageIdentity(cdp);
    if (state.page !== pageId) {
      throw new Error(`EXPORTS.md replay path ${replayPath} rendered ${state.page}; expected ${pageId}.`);
    }
  }

  for (const check of checks) {
    const startPage = check.startPage ?? DASHBOARD_PAGE_ID;
    await cdp.send("Page.navigate", { url: `${referenceUrl}?page=${encodeURIComponent(startPage)}` });
    await waitForPage(cdp, startPage);
    const selectorExists = await evaluate(cdp, `Boolean(document.querySelector(${JSON.stringify(check.selector)}))`);
    if (!selectorExists) {
      throw new Error(`Missing selector for Hub web-reference click check "${check.name}": ${check.selector}`);
    }
    await evaluate(cdp, `document.querySelector(${JSON.stringify(check.selector)}).click()`);
    await waitForPage(cdp, check.expectedPage);
    const state = await evaluate(
      cdp,
      "JSON.stringify({ page: new URL(location.href).searchParams.get('page'), text: document.body.innerText })",
    );
    const parsed = JSON.parse(state);
    if (parsed.page !== check.expectedPage || !parsed.text.includes(check.expectedText)) {
      throw new Error(
        `${check.name} routed to ${parsed.page}; expected ${check.expectedPage} with text ${check.expectedText}.`,
      );
    }
  }

  for (const check of localChecks) {
    const startPage = check.startPage ?? DASHBOARD_PAGE_ID;
    await cdp.send("Page.navigate", { url: `${referenceUrl}?page=${encodeURIComponent(startPage)}` });
    await waitForPage(cdp, startPage);
    const selectorExists = await evaluate(cdp, `Boolean(document.querySelector(${JSON.stringify(check.selector)}))`);
    if (!selectorExists) {
      throw new Error(`Missing selector for Hub web-reference local check "${check.name}": ${check.selector}`);
    }
    await evaluate(cdp, `document.querySelector(${JSON.stringify(check.selector)}).click()`);
    await delay(100);
    const state = await evaluate(
      cdp,
      `JSON.stringify({
        shellHasClass: ${check.expectedShellClass ? `document.querySelector(".hub-shell").classList.contains(${JSON.stringify(check.expectedShellClass)})` : "true"},
        elementHasClass: ${check.expectedElementClass ? `document.querySelector(${JSON.stringify(check.selector)}).classList.contains(${JSON.stringify(check.expectedElementClass)})` : "true"},
        pressed: document.querySelector(${JSON.stringify(check.selector)})?.getAttribute("aria-pressed")
      })`,
    );
    const parsed = JSON.parse(state);
    if (!parsed.shellHasClass || !parsed.elementHasClass || parsed.pressed !== "true") {
      throw new Error(`Local interaction "${check.name}" did not set expected state: ${state}.`);
    }
  }

  for (const check of searchChecks) {
    await cdp.send("Page.navigate", { url: `${referenceUrl}?page=${encodeURIComponent(check.startPage)}` });
    await waitForPage(cdp, check.startPage);
    const selectorExists = await evaluate(cdp, `Boolean(document.querySelector(${JSON.stringify(check.selector)}))`);
    if (!selectorExists) {
      throw new Error(`Missing selector for Hub web-reference search check "${check.name}": ${check.selector}`);
    }
    await evaluate(
      cdp,
      `(() => {
        const input = document.querySelector(${JSON.stringify(check.selector)});
        input.value = ${JSON.stringify(check.query)};
        input.dispatchEvent(new Event("input", { bubbles: true }));
      })()`,
    );
    await delay(100);
    const state = await evaluate(
      cdp,
      `JSON.stringify({
        hiddenCount: document.querySelectorAll(".is-filter-hidden").length,
        visibleText: [...document.querySelectorAll(${JSON.stringify(check.visibleSelector)})].map((node) => node.innerText).join("\\n")
      })`,
    );
    const parsed = JSON.parse(state);
    if (parsed.hiddenCount < 1 || !parsed.visibleText.includes(check.expectedText)) {
      throw new Error(`Search interaction "${check.name}" did not filter to ${check.expectedText}: ${state}.`);
    }
  }

  for (const check of stateChecks) {
    await cdp.send("Page.navigate", { url: `${referenceUrl}?page=${encodeURIComponent(check.startPage)}` });
    await waitForPage(cdp, check.startPage);
    const selectorExists = await evaluate(cdp, `Boolean(document.querySelector(${JSON.stringify(check.selector)}))`);
    if (!selectorExists) {
      throw new Error(`Missing selector for Hub web-reference state check "${check.name}": ${check.selector}`);
    }
    await evaluate(cdp, `document.querySelector(${JSON.stringify(check.selector)}).click()`);
    await waitForPage(cdp, check.expectedPage);
    await delay(100);
    const state = JSON.parse(await evaluate(cdp, check.expression));
    if (!state.ok) {
      throw new Error(`State interaction "${check.name}" did not apply expected state: ${JSON.stringify(state)}.`);
    }
  }

  cdp.close();
  console.log(`validated ${knownPageIds.size} Hub web-reference pages for known data-route and page href targets`);
  console.log(`validated ${EXPORTS_LIST.length} Hub web-reference output filename replay paths`);
  console.log(`validated ${exportReplayRows.length} Hub web-reference EXPORTS.md replay paths`);
  console.log(`validated ${checks.length} Hub web-reference click routes`);
  console.log(`validated ${localChecks.length} Hub web-reference local UI state interactions`);
  console.log(`validated ${searchChecks.length} Hub web-reference search/filter interactions`);
  console.log(`validated ${stateChecks.length} Hub web-reference applied state interactions`);
} finally {
  await cleanup();
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForPage(cdp, pageId, attempts = 80) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    const state = await pageIdentity(cdp);
    if (
      state.page === pageId &&
      state.title?.endsWith(" - Zircon Hub Web Reference") &&
      state.heading &&
      state.interactionsReady
    ) {
      return state;
    }
    await delay(100);
  }
  const state = await pageIdentity(cdp);
  throw new Error(`Timed out waiting for Hub web-reference page ${pageId}; last state ${JSON.stringify(state)}.`);
}

function readExportReplayRows() {
  const rows = [...readFileSync(exportsIndex, "utf8").matchAll(/^- ([^:]+): ([^ ]+) \(([^)]+)\)$/gm)].map(
    (match) => ({
      outputName: match[1],
      pageId: match[2],
      replayPath: match[3],
    }),
  );
  if (rows.length !== EXPORTS_LIST.length) {
    throw new Error(`EXPORTS.md lists ${rows.length} replay paths; expected ${EXPORTS_LIST.length}.`);
  }
  return rows;
}

function replayPathToFileUrl(replayPath) {
  const [pathPart, queryPart = ""] = replayPath.split("?");
  const url = new URL(pathToFileURL(resolve(repoRoot, pathPart)).href);
  url.search = queryPart;
  return url.href;
}

async function pageIdentity(cdp) {
  const state = await evaluate(
    cdp,
    "JSON.stringify({ page: document.querySelector('.hub-shell')?.dataset.page, title: document.title, heading: document.querySelector('.page-heading h2')?.innerText, interactionsReady: Boolean(window.ZirconHubInteractions?.ready) })",
  );
  return JSON.parse(state);
}

async function waitForJson(url, attempts = 80) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        return response.json();
      }
    } catch (_) {
      // The debug endpoint takes a moment to open.
    }
    await delay(100);
  }
  throw new Error(`Timed out waiting for ${url}.`);
}

function connect(wsUrl) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(wsUrl);
    const pending = new Map();

    ws.addEventListener("open", () => {
      resolve({
        send(method, params = {}) {
          const id = nextId;
          nextId += 1;
          ws.send(JSON.stringify({ id, method, params }));
          return new Promise((resolveSend, rejectSend) => {
            pending.set(id, { method, resolveSend, rejectSend });
          });
        },
        close() {
          ws.close();
        },
      });
    });

    ws.addEventListener("message", (event) => {
      const message = JSON.parse(event.data);
      if (!message.id || !pending.has(message.id)) {
        return;
      }
      const request = pending.get(message.id);
      pending.delete(message.id);
      if (message.error) {
        request.rejectSend(new Error(`${request.method}: ${message.error.message}`));
      } else {
        request.resolveSend(message.result);
      }
    });

    ws.addEventListener("error", reject);
  });
}

async function evaluate(cdp, expression) {
  const result = await cdp.send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue: true,
  });
  if (result.exceptionDetails) {
    throw new Error(result.exceptionDetails.text ?? "Runtime.evaluate exception.");
  }
  return result.result.value;
}

async function cleanup() {
  await terminateBrowser();
  assertSafeTemporaryProfile(profile);
  for (let attempt = 0; attempt < 8; attempt += 1) {
    try {
      rmSync(profile, { recursive: true, force: true });
      if (!existsSync(profile)) {
        return;
      }
    } catch (error) {
      if (attempt === 7) {
        console.warn(`Temporary Edge profile cleanup skipped: ${profile}`);
        return;
      }
    }
    await delay(250);
  }
}

async function terminateBrowser() {
  if (process.platform === "win32") {
    if (browser.exitCode === null && browser.signalCode === null && browser.pid) {
      await taskkillProcessTree(browser.pid);
    }
    await stopWindowsEdgeProfileProcesses(profile);
  } else {
    if (browser.exitCode === null && browser.signalCode === null) {
      browser.kill();
    }
  }
  await waitForExit(browser);
}

async function taskkillProcessTree(pid) {
  await new Promise((resolve) => {
    const killer = spawn("taskkill", ["/pid", String(pid), "/T", "/F"], { stdio: "ignore" });
    const timer = setTimeout(resolve, 3000);
    killer.once("exit", () => {
      clearTimeout(timer);
      resolve();
    });
    killer.once("error", () => {
      clearTimeout(timer);
      browser.kill();
      resolve();
    });
  });
}

async function stopWindowsEdgeProfileProcesses(profilePath) {
  const escapedProfile = profilePath.replaceAll("'", "''");
  const command = [
    `$profile = '${escapedProfile}'`,
    "Get-CimInstance Win32_Process |",
      "Where-Object { $_.Name -like 'msedge*.exe' -and $_.CommandLine -and $_.CommandLine.Contains($profile) } |",
      "ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }",
  ].join(" ");
  await runProcess("powershell.exe", ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", command]);
  await delay(500);
}

async function runProcess(command, args) {
  await new Promise((resolve) => {
    const child = spawn(command, args, { stdio: "ignore" });
    const timer = setTimeout(resolve, 5000);
    child.once("exit", () => {
      clearTimeout(timer);
      resolve();
    });
    child.once("error", () => {
      clearTimeout(timer);
      resolve();
    });
  });
}

function assertSafeTemporaryProfile(profilePath) {
  const tempRoot = resolve(tmpdir());
  const target = resolve(profilePath);
  if (!target.startsWith(`${tempRoot}${sep}`) || !basename(target).startsWith("zircon-hub-cdp-")) {
    throw new Error(`Refusing to recursively delete unexpected Edge profile path: ${profilePath}`);
  }
}

async function waitForExit(child, timeoutMs = 3000) {
  if (child.exitCode !== null || child.signalCode !== null) {
    return;
  }
  await new Promise((resolve) => {
    const timer = setTimeout(resolve, timeoutMs);
    child.once("exit", () => {
      clearTimeout(timer);
      resolve();
    });
  });
}
