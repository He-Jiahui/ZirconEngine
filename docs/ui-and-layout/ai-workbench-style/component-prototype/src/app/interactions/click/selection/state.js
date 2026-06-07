import { icon } from "../../../../foundation/icons.js";

export function applyToggleSelectionState(toggle) {
  if (toggle.dataset.toggle === "switch") {
    toggle.classList.toggle("is-on");
    return;
  }

  const checked = toggle.classList.toggle("is-checked");
  const box = toggle.querySelector(".zr-check-box");
  if (box) box.innerHTML = checked ? icon("check") : "";
}

export function applyRadioSelectionState(radio) {
  const group = radio.closest(".zr-check-stack") ?? radio.parentElement;
  group?.querySelectorAll("[data-radio]").forEach((item) => item.classList.remove("is-checked"));
  radio.classList.add("is-checked");
}
