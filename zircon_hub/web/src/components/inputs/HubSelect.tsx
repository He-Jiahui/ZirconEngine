import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import type { SelectChangeEvent } from "@mui/material";
import { Box, MenuItem, Select, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface HubSelectOption {
  value: string;
  label: string;
}

export interface HubSelectProps {
  value: string;
  options: HubSelectOption[];
  minWidth?: number;
  onChange: (value: string) => void;
}

export function HubSelect({ value, options, minWidth = 183, onChange }: HubSelectProps) {
  const handleChange = (event: SelectChangeEvent) => {
    onChange(event.target.value);
  };

  return (
    <Select
      value={value}
      size="small"
      IconComponent={ExpandMoreIcon}
      onChange={handleChange}
      renderValue={(selected) => (
        <Typography variant="body2" color="text.secondary">
          {options.find((option) => option.value === selected)?.label ?? selected}
        </Typography>
      )}
      sx={{
        minWidth,
        height: 42,
        color: hubTokens.colors.textSoft,
        "& .MuiSelect-select": {
          display: "flex",
          alignItems: "center",
          py: 0,
        },
      }}
    >
      {options.map((option) => (
        <MenuItem key={option.value} value={option.value}>
          <Box component="span">{option.label}</Box>
        </MenuItem>
      ))}
    </Select>
  );
}
