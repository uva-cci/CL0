import {
	CssBaseline,
	ThemeProvider as MUIThemeProvider,
	createTheme,
} from "@mui/material";
import { PropsWithChildren, useMemo } from "react";

export const ThemeProvider = ({ children }: PropsWithChildren) => {
	const theme = useMemo(
		() =>
			createTheme({
				palette: {
					mode: "dark",
					primary: { main: "#4f46e5" },
					secondary: { main: "#14b8a6" },
					background: { default: "#0a0a0a", paper: "#0f0f0f" },
				},
				shape: { borderRadius: 14 },
				typography: {
					fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif",
				},
			}),
		[],
	);

	return (
		<MUIThemeProvider theme={theme}>
			<CssBaseline />
			{children}
		</MUIThemeProvider>
	);
};
