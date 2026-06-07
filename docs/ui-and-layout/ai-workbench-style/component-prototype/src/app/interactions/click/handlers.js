import { handleActionClick } from "./actions.js";
import { handleDropdownClick, handlePopupDismissal } from "./dropdowns.js";
import { handleGenericCommandClick } from "./generic.js";
import { handleModuleNavigation } from "./navigation.js";
import { handleDataRowClick, handleTreeRowClick } from "./rows.js";
import { handleRadioClick, handleToggleClick } from "./selection.js";
import { handleTabClick } from "./tabs.js";
import { handleRailClick, handleToolClick } from "./toolbar.js";

export const clickHandlers = [
  handleModuleNavigation,
  handleActionClick,
  handleToggleClick,
  handleRadioClick,
  handleTabClick,
  handleTreeRowClick,
  handleDataRowClick,
  handleRailClick,
  handleToolClick,
  handleDropdownClick,
  handlePopupDismissal,
  handleGenericCommandClick
];
