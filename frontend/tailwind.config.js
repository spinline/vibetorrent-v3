const path = require("path");
const os = require("os");

// Cargo registry'deki leptos-shadcn crate'lerini Tailwind'e taratmak için
const cargoRegistry = path.join(
  os.homedir(),
  ".cargo/registry/src/*/leptos-shadcn-*/src/**/*.rs"
);

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{rs,html}",
    cargoRegistry,
  ],
  theme: {
    extend: {
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
