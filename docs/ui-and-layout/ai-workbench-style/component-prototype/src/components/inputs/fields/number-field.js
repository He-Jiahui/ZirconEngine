import { esc } from "../input-utils.js";

export function numberField(value, options = {}) {
  const classes = ["zr-number"];
  if (options.stepper) classes.push("has-stepper");
  if (options.className) classes.push(options.className);
  return `<span class="${classes.join(" ")}">${esc(value)}${options.stepper ? '<span class="zr-number-stepper"><span></span><span></span></span>' : ""}</span>`;
}
