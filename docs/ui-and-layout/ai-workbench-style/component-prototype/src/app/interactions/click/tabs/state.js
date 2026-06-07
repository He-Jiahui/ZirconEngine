export function activateTabState(tab) {
  [...tab.parentElement.children].forEach((item) => {
    item.classList.remove("is-active");
    item.setAttribute("aria-selected", "false");
  });
  tab.classList.add("is-active");
  tab.setAttribute("aria-selected", "true");
}
