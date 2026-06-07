import { icon } from "../../../foundation/icons.js";
import { inspectorSections } from "../../../foundation/data.js";
import { panelGroup } from "../../../modules/shared/module-components.js";
import { button, checkbox, select } from "../../inputs/atoms.js";
import { drawerSurface } from "./drawer-surface.js";

export function inspector() {
  return drawerSurface({
    className: "zr-inspector",
    host: "inspector",
    kind: "window",
    children: [
      panelGroup("inspector", [
        { label: "Inspector", active: true, content: `<div class="zr-inspector-body"><div class="zr-object-header">${icon("cube")}<span>Props</span>${checkbox("Static", false)}${icon("more")}</div><div class="zr-form-row"><span>Tag</span>${select("Untagged")}<span>Layer</span>${select("Default")}</div>${inspectorSections.map(section).join("")}${button("Add Component", { icon: "plus" })}</div>` },
        { label: "History", content: historyView() }
      ])
    ]
  });
}

function section(sectionData) {
  const sectionClass = sectionData.title.toLowerCase().replace(/\s+/g, "-");
  const vectors = sectionData.fields?.map(vectorRow).join("") ?? "";
  const resources = sectionData.rows?.map((row) => `<div class="zr-resource-row ${row.count ? "has-count" : "is-single-resource"}"><span>${row.label}</span><span>${row.count ?? ""}</span>${select(row.value, row.swatch ? { swatch: true } : { icon: row.icon })}</div>`).join("") ?? "";
  const nested = sectionData.nested?.map(([label, value]) => {
    const isDisclosure = value === "";
    const content = isDisclosure ? `${icon("chevronDown")}<span>${label}</span>` : label;
    const control = value === "check" ? checkbox("", true) : value ? select(value) : "<span></span>";
    return `<div class="zr-resource-row is-nested-resource ${isDisclosure ? "is-disclosure-row" : ""}"><span>${content}</span><span></span>${control}</div>`;
  }).join("") ?? "";
  return `<section class="zr-section is-${sectionClass}"><div class="zr-section-title">${icon(sectionData.icon)}<span>${sectionData.title}</span>${checkbox("", sectionData.checked)}${icon("chevronUp")}</div>${vectors}${resources}${nested}</section>`;
}

function vectorRow(row) {
  if (!row.link) {
    return `<div class="zr-vector-row"><span>${row.label}</span><span>X</span>${row.values.map((value, index) => `${index > 0 ? `<span>${["Y", "Z"][index - 1]}</span>` : ""}<span class="zr-value-box">${value}</span>`).join("")}</div>`;
  }

  const axes = ["X", "Y", "Z"];
  const cells = row.values.map((value, index) => {
    if (index === 0) {
      return `<span class="zr-linked-axis">${icon("link")}<span class="zr-axis-x">${axes[index]}</span></span><span class="zr-value-box">${value}</span>`;
    }
    return `<span>${axes[index]}</span><span class="zr-value-box">${value}</span>`;
  }).join("");
  return `<div class="zr-vector-row has-linked-axis"><span>${row.label}</span>${cells}</div>`;
}

function historyView() {
  const entries = ["Selected Props", "Updated material", "Moved Box_01", "Saved scene"];
  return `<div class="zr-inspector-body zr-history-list">${entries.map((entry, index) => `<button class="zr-history-row ${index === 0 ? "is-active" : ""}" type="button">${icon(index === 0 ? "check" : "undo")}<span>${entry}</span><small>${index + 1}m</small></button>`).join("")}</div>`;
}
