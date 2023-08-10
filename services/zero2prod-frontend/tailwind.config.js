/** @type {import('tailwindcss').Config} */
export default {
	content: [
		"./index.html",
		"./src/**/*.{vue,js,ts,jsx,tsx}",
	],
	theme: {
		fontFamily: {
			text: ['FiraSans']
		},
		extend: {
      backgroundImage: {
        'yili': "url('/assets/img/zeng-yili-c9ZQDFwn-pk-unsplash.jpg')"
      }
    }
	},
	plugins: [],
}

