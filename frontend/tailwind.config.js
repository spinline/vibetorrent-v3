/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.{rs,html}"],
  theme: {
    extend: {
      colors: {
        gray: {
          900: '#111827',
          800: '#1f2937',
          700: '#374151',
        }
      }
    },
  },
  plugins: [],
  daisyui: {
    themes: ["light", "dark", "cupcake", "dracula", "cyberpunk", "emerald", "luxury", "nord", "sunset", "winter", "night", "synthwave", "retro", "forest"],
  },
}
