import type { ReactElement } from "react";
import { Tab, Tabs } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface HubTabOption {
  value: string;
  label: string;
  icon?: ReactElement;
}

export interface HubTabsProps {
  value: string;
  options: HubTabOption[];
  onChange: (value: string) => void;
}

export function HubTabs({ value, options, onChange }: HubTabsProps) {
  const tabListLabel = options.map((option) => option.label).join(", ");

  return (
    <Tabs
      aria-label={tabListLabel || undefined}
      value={value}
      onChange={(_, nextValue: string) => onChange(nextValue)}
      variant="scrollable"
      scrollButtons="auto"
      sx={{
        width: "100%",
        minWidth: 0,
        maxWidth: "100%",
        minHeight: 38,
        borderBottom: `1px solid ${hubTokens.colors.line}`,
        "& .MuiTabs-indicator": { backgroundColor: hubTokens.colors.accent },
      }}
    >
      {options.map((option) => (
        <Tab
          key={option.value}
          value={option.value}
          label={option.label}
          icon={option.icon}
          iconPosition="start"
          sx={{
            minHeight: 38,
            minWidth: 0,
            px: 1.4,
            color: hubTokens.colors.textSoft,
            "&.Mui-selected": { color: hubTokens.colors.accent },
          }}
        />
      ))}
    </Tabs>
  );
}
