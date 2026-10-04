import { Box, Checkbox, FormControlLabel, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

// 勾选值由页面或设置草稿持有；省略变更回调用于只读状态展示，文案应说明被表示的条件。
export interface HubCheckboxProps {
  checked: boolean;
  label: string;
  detail?: string;
  disabled?: boolean;
  onChange?: (checked: boolean) => void;
}

// 设置编辑与只读状态复用同一标签行；提交的新值只交回调用方，组件不保存草稿或调用后端。
export function HubCheckbox({ checked, label, detail, disabled = false, onChange }: HubCheckboxProps) {
  const isDisabled = disabled || !onChange;

  return (
    <FormControlLabel
      disabled={isDisabled}
      control={
        <Checkbox
          size="small"
          checked={checked}
          onChange={(event) => onChange?.(event.target.checked)}
          sx={{
            color: hubTokens.colors.textMuted,
            "&.Mui-checked": { color: hubTokens.colors.accent },
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
      sx={{
        m: 0,
        minHeight: 38,
        px: 0.8,
        borderRadius: `${hubTokens.radius.compact}px`,
        "&:hover": { backgroundColor: isDisabled ? "transparent" : "rgba(255,255,255,0.035)" },
      }}
    />
  );
}
