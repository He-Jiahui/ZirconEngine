import { DESIGNS, HEIGHT, WIDTH } from "./design-manifest.mjs";

const COLUMN_COUNT = 6;
const KIND_LABELS = new Map([
  ["full", "Full Pages"],
  ["focus", "Core Focus"],
  ["tool", "Tool Pages"],
  ["editor-page", "Editor Pages"],
  ["tool-focus", "Tool Focus"],
  ["layout-spec", "Layout Specs"],
  ["state-spec", "State Specs"],
  ["content-spec", "Content Specs"],
  ["overlay-spec", "Overlay Specs"],
  ["workflow-spec", "Workflow Specs"],
  ["floating-window", "Floating Windows"],
]);

const KIND_SHORT_LABELS = new Map([
  ["full", "FULL"],
  ["focus", "FOC"],
  ["tool", "TOOL"],
  ["editor-page", "PAGE"],
  ["tool-focus", "TFOC"],
  ["layout-spec", "LAY"],
  ["state-spec", "STATE"],
  ["content-spec", "CONT"],
  ["overlay-spec", "OVR"],
  ["workflow-spec", "FLOW"],
  ["floating-window", "WIN"],
]);

export function previewSheetEntries(renderedDesigns) {
  const renderedById = new Map(renderedDesigns.map((design) => [design.id, design]));
  return DESIGNS.map((manifest, index) => {
    const rendered = renderedById.get(manifest.id) ?? {};
    return {
      id: manifest.id,
      output: manifest.output,
      kind: manifest.kind,
      index: index + 1,
      title: rendered.title ?? titleFromId(manifest.id),
      description: rendered.description ?? KIND_LABELS.get(manifest.kind) ?? manifest.kind,
    };
  });
}

export function previewSheetCoverageIds(renderedDesigns) {
  return previewSheetEntries(renderedDesigns).map((entry) => entry.id);
}

export function renderPreviewSheet(renderedDesigns) {
  const entries = previewSheetEntries(renderedDesigns);
  const columns = splitColumns(entries, COLUMN_COUNT);
  const sheet = el("div", "preview-sheet manifest-sheet");
  sheet.dataset.previewDesignCount = String(entries.length);
  sheet.dataset.previewCanvas = `${WIDTH}x${HEIGHT}`;

  const title = el("div", "sheet-title");
  title.innerHTML = `<div><strong>Editor Workbench Design Manifest</strong><div class="muted">${entries.length} design entries plus preview-sheet.png, all locked to ${WIDTH} x ${HEIGHT}.</div></div><div class="pill">${entries.length + 1} PNG exports</div>`;

  const summary = el("div", "sheet-summary");
  for (const [kind, count] of kindCounts(entries)) {
    const card = el("div", "sheet-summary-card");
    card.innerHTML = `<strong>${count}</strong><span>${KIND_LABELS.get(kind) ?? kind}</span>`;
    summary.append(card);
  }

  const grid = el("div", "sheet-coverage-grid");
  columns.forEach((column, columnIndex) => {
    const panel = el("div", "sheet-column");
    const first = column[0]?.index ?? 0;
    const last = column[column.length - 1]?.index ?? 0;
    const header = el("div", "sheet-column-title");
    header.innerHTML = `<span>Column ${columnIndex + 1}</span><strong>${String(first).padStart(3, "0")} - ${String(last).padStart(3, "0")}</strong>`;
    panel.append(header);
    column.forEach((entry) => panel.append(renderEntry(entry)));
    grid.append(panel);
  });

  sheet.append(title, summary, grid);
  return sheet;
}

function renderEntry(entry) {
  const row = el("div", "sheet-row");
  row.dataset.previewDesignId = entry.id;
  row.dataset.previewOutput = entry.output;
  row.innerHTML = `<span class="sheet-index">${String(entry.index).padStart(3, "0")}</span><strong title="${entry.title}">${entry.id}</strong><span class="sheet-kind">${KIND_SHORT_LABELS.get(entry.kind) ?? entry.kind}</span>`;
  return row;
}

function kindCounts(entries) {
  const counts = new Map();
  entries.forEach((entry) => counts.set(entry.kind, (counts.get(entry.kind) ?? 0) + 1));
  return [...counts.entries()].sort((left, right) => right[1] - left[1]);
}

function splitColumns(entries, columnCount) {
  const perColumn = Math.ceil(entries.length / columnCount);
  const columns = [];
  for (let index = 0; index < entries.length; index += perColumn) {
    columns.push(entries.slice(index, index + perColumn));
  }
  return columns;
}

function titleFromId(id) {
  return id
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function el(tag, className = "") {
  const node = document.createElement(tag);
  if (className) node.className = className;
  return node;
}
