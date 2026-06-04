import type { ReactNode } from "react";
import { ToggleButton, ToggleButtonGroup, Tooltip } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface HubToggleOption {
  value: string;
  label: string;
  icon: ReactNode;
}

export interface HubToggleProps {
  value: string;
  options: HubToggleOption[];
  onChange: (value: string) => void;
}

export function HubToggle({ value, options, onChange }: HubToggleProps) {
  return (
    <ToggleButtonGroup
      exclusive
      value={value}
      onChange={(_, nextValue: string | null) => {
        if (nextValue) {
          onChange(nextValue);
        }
      }}
      sx={{ gap: 0.75 }}
    >
      {options.map((option) => (
        <Tooltip key={option.value} title={option.label}>
          <ToggleButton
            value={option.value}
            aria-label={option.label}
            sx={{
              width: 50,
              height: 42,
              p: 0,
              color: hubTokens.colors.textSoft,
              border: `1px solid ${hubTokens.colors.lineStrong}`,
              borderRadius: `${hubTokens.radius.compact}px !important`,
              backgroundColor: "rgba(31,31,31,0.72)",
              "&.Mui-selected": {
                color: hubTokens.colors.text,
                backgroundColor: "rgba(9,94,91,0.56)",
                borderColor: "rgba(45,212,207,0.48)",
              },
              "&.Mui-selected:hover, &:hover": {
                color: hubTokens.colors.text,
                backgroundColor: "rgba(11,112,109,0.68)",
              },
            }}
          >
            {option.icon}
          </ToggleButton>
        </Tooltip>
      ))}
    </ToggleButtonGroup>
  );
}
