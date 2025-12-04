import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: "#00ffcc",
          dark: "#00d4aa",
        },
        secondary: {
          DEFAULT: "#7b61ff",
          dark: "#6347e8",
        },
        brand: {
          primary: "#00ffcc",
          secondary: "#7b61ff",
        },
        bg: {
          DEFAULT: "#0a0a0f",
          secondary: "#14141f",
          tertiary: "#1a1a2e",
        },
        background: {
          DEFAULT: "#0a0a0f",
          secondary: "#14141f",
          tertiary: "#1a1a2e",
        },
        text: {
          primary: "#ffffff",
          secondary: "#a0a0b0",
          tertiary: "#6b6b80",
        },
      },
      fontFamily: {
        sans: ["var(--font-inter)", "system-ui", "sans-serif"],
        mono: ["var(--font-jetbrains-mono)", "monospace"],
      },
      animation: {
        "float": "float 6s ease-in-out infinite",
        "pulse-slow": "pulse 4s cubic-bezier(0.4, 0, 0.6, 1) infinite",
      },
      keyframes: {
        float: {
          "0%, 100%": { transform: "translateY(0)" },
          "50%": { transform: "translateY(-20px)" },
        },
      },
    },
  },
  plugins: [],
};

export default config;
