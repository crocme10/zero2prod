/** @type {import('tailwindcss').Config} */
module.exports = {
  content: { 
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {},
    fontFamily: {
      FiraSans: ["FiraSans, sans-serif"]
    }
  },
  plugins: [],
}
