import { Box, FormControlLabel, Switch, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface HubSwitchProps {
  checked: boolean;
  label: string;
  detail?: string;
  disabled?: boolean;
  onChange?: (checked: boolean) => void;
}

export function HubSwitch({ checked, label, detail, disabled = false, onChange }: HubSwitchProps) {
  const isDisabled = disabled || !onChange;

  return (
    <FormControlLabel
      disabled={isDisabled}
      control={
        <Switch
          size="small"
          checked={checked}
          onChange={(event) => onChange?.(event.target.checked)}
          sx={{
            "& .MuiSwitch-switchBase.Mui-checked": { color: hubTokens.colors.accent },
            "& .MuiSwitch-switchBase.Mui-checked + .MuiSwitch-track": {
              backgroundColor: "rgba(33,213,207,0.44)",
            },
          }}
        />
      }
      label={
        <Box sx={{ minWidth: 0 }}>
          <Typography variant="body2" noWrap sx={{ color: isDisabled ? hubTokens.colors.textMuted : hubTokens.colors.text }}>
            {label}
          </Typography>
          {detail ? (
            <Typography variant="caption" noWrap sx={{ display: "block", color: hubTokens.colors.textMuted }}>
              {detail}
            </Typography>
          ) : null}
        </Box>
      }
      sx={{ m: 0, minHeight: 38, justifyContent: "space-between", gap: 1 }}
    />
  );
}
