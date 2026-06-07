export function activateActionGroupState(action) {
  const group = action.closest("[data-action-group]");
  if (!group) return;
  group.querySelectorAll(".is-active").forEach((item) => item.classList.remove("is-active"));
  action.classList.add("is-active");
}
