/** @type {import('tailwindcss').Config} */

import typography from "@tailwindcss/typography";
export default {
	content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
	theme: {
		extend: {
			colors: {
				brand: { 600: "#4f46e5" },
			},
			borderRadius: {
				xl: "14px",
			},
		},
	},
	plugins: [typography],
};
