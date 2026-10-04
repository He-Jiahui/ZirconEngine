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
  label: string;
  minWidth?: number;
  onChange: (value: string) => void;
}

export function HubSelect({ value, options, label, minWidth = 183, onChange }: HubSelectProps) {
  const handleChange = (event: SelectChangeEvent) => {
    onChange(event.target.value);
  };

  return (
    <Select
      value={value}
      inputProps={{ "aria-label": label }}
      size="small"
      IconComponent={ExpandMoreIcon}
      onChange={handleChange}
      renderValue={(selected) => (
        <Typography variant="body2" color="text.secondary">
          {options.find((option) => option.value === selected)?.label ?? selected}
        </Typography>
      )}
      sx={{
        width: "100%",
        minWidth: 0,
        maxWidth: "100%",
        "@media (min-width: 761px)": { width: "auto", minWidth },
        height: 42,
        color: hubTokens.colors.textSoft,
        "& .MuiSelect-select": {
          display: "flex",
          alignItems: "center",
          py: 0,
          minWidth: 0,
          "& .MuiTypography-root": { overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" },
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
