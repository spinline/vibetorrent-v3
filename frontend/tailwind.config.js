/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.{rs,html}"],
  theme: {
    extend: {
      spacing: {
        "safe-top": "env(safe-area-inset-top)",
        "safe-bottom": "env(safe-area-inset-bottom)",
      },
      colors: {
        gray: {
          900: "#111827",
          800: "#1f2937",
          700: "#374151",
        },
      },
    },
  },
};
