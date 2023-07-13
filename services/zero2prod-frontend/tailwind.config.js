/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./static/index.html",
    "./crate/src/**/*.rs"
  ],
  theme: {
    extend: {},
    fontFamily: {
      text: ['ZillaSlab']
    }
  },
  plugins: [],
}
