import { Autocomplete, Box, TextField, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface HubComboBoxOption {
  value: string;
  label: string;
  detail?: string;
  disabled?: boolean;
}

export interface HubComboBoxProps {
  value: string;
  options: HubComboBoxOption[];
  placeholder?: string;
  minWidth?: number;
  onChange: (value: string) => void;
}

export function HubComboBox({ value, options, placeholder, minWidth = 176, onChange }: HubComboBoxProps) {
  const selected = options.find((option) => option.value === value) ?? null;

  return (
    <Autocomplete
      size="small"
      value={selected}
      options={options}
      clearOnBlur
      disableClearable={options.length > 0}
      getOptionLabel={(option) => option.label}
      getOptionDisabled={(option) => option.disabled ?? false}
      isOptionEqualToValue={(option, current) => option.value === current.value}
      onChange={(_, option) => {
        if (option) {
          onChange(option.value);
        }
      }}
      sx={{
        minWidth,
        "& .MuiInputBase-root": {
          height: 42,
          color: hubTokens.colors.textSoft,
          backgroundColor: "rgba(31,31,31,0.72)",
        },
      }}
      renderInput={(params) => <TextField {...params} placeholder={placeholder} />}
      renderOption={(props, option) => (
        <Box component="li" {...props} sx={{ display: "grid", gap: 0.3 }}>
          <Typography variant="body2">{option.label}</Typography>
          {option.detail ? (
            <Typography variant="caption" sx={{ color: hubTokens.colors.textMuted }}>
              {option.detail}
            </Typography>
          ) : null}
        </Box>
      )}
    />
  );
}
