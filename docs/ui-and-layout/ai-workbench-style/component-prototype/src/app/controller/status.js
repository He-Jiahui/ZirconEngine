export function updateStatusMessage(message) {
  const nextResponseCount = Number.parseInt(document.documentElement.dataset.zrResponseCount ?? "0", 10) + 1;
  document.documentElement.dataset.zrResponseCount = String(nextResponseCount);
  document.documentElement.dataset.zrLastResponse = message;
  const target = document.querySelector("[data-status-message]");
  if (!target) return;
  target.textContent = message;
  target.classList.remove("zr-action-flash");
  requestAnimationFrame(() => target.classList.add("zr-action-flash"));
}
