import { menuItems } from "../../foundation/data.js";
import { menu } from "./menu.js";

export function popups() {
  return `<div id="popup-layer" class="zr-popup-layer">${menu(menuItems)}</div>`;
}
