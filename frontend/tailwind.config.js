/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        accent: {
          blue: {
            DEFAULT: '#00e5ff',
            glow: 'rgba(0, 229, 255, 0.4)'
          },
          purple: {
            DEFAULT: '#d500f9',
            glow: 'rgba(213, 0, 249, 0.4)'
          },
          green: {
            DEFAULT: '#00e676',
            glow: 'rgba(0, 230, 118, 0.4)'
          },
          red: {
            DEFAULT: '#ff1744',
            glow: 'rgba(255, 23, 68, 0.4)'
          }
        }
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
      },
      boxShadow: {
        'glass': '0 8px 32px 0 rgba(0, 0, 0, 0.37)',
        'neon': '0 0 15px var(--accent-glow)',
      }
    },
  },
  plugins: [],
}
