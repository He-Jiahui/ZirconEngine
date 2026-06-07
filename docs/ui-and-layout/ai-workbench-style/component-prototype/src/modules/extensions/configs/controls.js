import { checkbox, input, select } from "../../../components/inputs/atoms.js";

export function hydrateSettings(rows) {
  return rows.map(([label, control]) => [label, controlMarkup(control)]);
}

function controlMarkup(control) {
  if (!control || typeof control !== "object") {
    return control;
  }
  if (control.kind === "select") {
    return select(control.value);
  }
  if (control.kind === "input") {
    return input("", { value: control.value });
  }
  if (control.kind === "checkbox") {
    return checkbox("", control.value);
  }
  return String(control.value ?? "");
}
