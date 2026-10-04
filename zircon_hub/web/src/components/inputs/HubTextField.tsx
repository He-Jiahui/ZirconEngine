import type { TextFieldProps } from "@mui/material";
import { TextField } from "@mui/material";

// 继承表单的值、标签、事件及验证属性，只统一输入外观；最小宽度由所在表单布局选择。
export interface HubTextFieldProps extends Omit<TextFieldProps, "variant" | "size"> {
  minWidth?: number;
}

// 设置和创建表单共用紧凑输入入口；文本是否为有效路径或项目名仍须由表单及后端验证。
export function HubTextField({ minWidth = 0, sx, ...props }: HubTextFieldProps) {
  return (
    <TextField
      {...props}
      variant="outlined"
      size="small"
      sx={[
        {
          minWidth,
          "& .MuiInputBase-root": { minHeight: 42 },
        },
        ...(Array.isArray(sx) ? sx : sx ? [sx] : []),
      ]}
    />
  );
}
