/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        // Dark Theme (Primary)
        "dark-bg": "#0f1115",
        "dark-surface": "#161920",
        "dark-surface-hover": "#1c212c",
        "dark-surface-active": "#232836",
        "dark-border": "#2a303c",
        "dark-border-hover": "#3b4354",

        // Light Theme (Optional/Future)
        "light-bg": "#ffffff",
        "light-surface": "#f8fafc",
        "light-border": "#e2e8f0",

        // Accents
        primary: "#6366f1", // Indigo-500
        "primary-hover": "#818cf8", // Indigo-400
        success: "#10b981", // Emerald-500
        warning: "#f59e0b", // Amber-500
        error: "#ef4444", // Red-500
        info: "#3b82f6", // Blue-500

        // Text
        "text-primary": "#f8fafc", // Slate-50
        "text-secondary": "#94a3b8", // Slate-400
        "text-tertiary": "#64748b", // Slate-500
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "monospace"],
      },
      boxShadow: {
        'glow': '0 0 15px rgba(99, 102, 241, 0.3)',
        'glow-sm': '0 0 8px rgba(99, 102, 241, 0.2)',
      },
      keyframes: {
        "fade-in": {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        "slide-up": {
          "0%": { transform: "translateY(10px)", opacity: "0" },
          "100%": { transform: "translateY(0)", opacity: "1" },
        },
        "slide-in-right": {
          "0%": { transform: "translateX(100%)", opacity: "0" },
          "100%": { transform: "translateX(0)", opacity: "1" },
        },
        "scale-in": {
          "0%": { transform: "scale(0.95)", opacity: "0" },
          "100%": { transform: "scale(1)", opacity: "1" },
        },
      },
      animation: {
        "fade-in": "fade-in 0.2s ease-out",
        "slide-up": "slide-up 0.3s ease-out",
        "slide-in-right": "slide-in-right 0.3s ease-out",
        "scale-in": "scale-in 0.2s ease-out",
      },
    },
  },
  plugins: [],
};
