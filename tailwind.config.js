/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        accent: {
          blue: "#007aff",
          "blue-hover": "#0066d6",
        },
        surface: {
          dark: "rgba(24, 24, 27, 0.75)",
          "dark-solid": "#18181b",
          "dark-border": "rgba(255, 255, 255, 0.15)",
          "dark-selection": "rgba(255, 255, 255, 0.10)",
        },
        text: {
          primary: "#f8fafc",
          secondary: "#94a3b8",
          muted: "#64748b",
        },
      },
      fontFamily: {
        sans: ['"Segoe UI Variable Text"', "-apple-system", "system-ui", "sans-serif"],
      },
      borderRadius: {
        spotlight: "12px",
      },
      backdropBlur: {
        spotlight: "25px",
      },
      animation: {
        "slide-down": "slideDown 150ms ease-out",
        "slide-up": "slideUp 100ms ease-in",
        "fade-in": "fadeIn 150ms ease-out",
      },
      keyframes: {
        slideDown: {
          "0%": { transform: "translateY(-10px)", opacity: "0" },
          "100%": { transform: "translateY(0)", opacity: "1" },
        },
        slideUp: {
          "0%": { transform: "translateY(0)", opacity: "1" },
          "100%": { transform: "translateY(-10px)", opacity: "0" },
        },
        fadeIn: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
      },
    },
  },
  plugins: [],
};
