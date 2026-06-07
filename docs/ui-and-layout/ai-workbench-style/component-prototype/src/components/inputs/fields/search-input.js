import { icon } from "../../../foundation/icons.js";
import { input } from "./input.js";

export function searchInput(placeholder) {
  return `<label class="zr-search">${icon("search")}${input(placeholder)}</label>`;
}
