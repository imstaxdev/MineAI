import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["Inter", "Segoe UI", "system-ui", "sans-serif"],
      },
      colors: {
        ink: "#e8eefc",
        panel: "#07111f",
        surface: "#0b1b31",
        surface2: "#102640",
        hero: "#071426",
        line: "#1f3b5d",
        blue: "#2563eb",
        blue2: "#38bdf8",
        grass: "#4ade80",
        ember: "#f59e0b",
        muted: "#91a7c7",
      },
      boxShadow: {
        soft: "0 18px 50px rgba(0, 0, 0, 0.35)",
      },
    },
  },
  plugins: [],
} satisfies Config;
