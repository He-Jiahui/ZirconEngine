import { icon } from "../../foundation/icons.js";

export function alerts(items) {
  return `<div class="zr-alert-stack">${items.map(([tone, label]) => `<div class="zr-alert is-${tone}"><span class="zr-alert-status">${alertMark(tone)}</span><span>${label}</span>${icon("x")}</div>`).join("")}</div>`;
}

function alertMark(tone) {
  if (tone === "success") return icon("check");
  if (tone === "warning") return `<span>!</span>`;
  if (tone === "error") return `<span>x</span>`;
  return `<span>i</span>`;
}
