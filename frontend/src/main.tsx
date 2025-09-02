import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "./styles/global.css";
import { ThemeProvider } from "./providers/ThemeProvider";
import { Toaster } from "react-hot-toast";

createRoot(document.getElementById("root")!).render(
	<StrictMode>
		<ThemeProvider>
			<App />
			<Toaster position="top-right" toastOptions={{ duration: 3000 }} />
		</ThemeProvider>
	</StrictMode>,
);
