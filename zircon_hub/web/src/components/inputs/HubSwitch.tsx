import { Box, FormControlLabel, Switch, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

// 页面持有开关值并解释其业务含义；没有变更回调表示只读条件，不能呈现为可修改设置。
export interface HubSwitchProps {
  checked: boolean;
  label: string;
  detail?: string;
  disabled?: boolean;
  onChange?: (checked: boolean) => void;
}

// 开关只表达下一次期望值，保存、成员权限及失败恢复由各自调用方处理，不在控件内执行副作用。
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
