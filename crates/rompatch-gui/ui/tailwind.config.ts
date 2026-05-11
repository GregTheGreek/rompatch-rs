import type { Config } from 'tailwindcss';

const config: Config = {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // Sleek dark palette inspired by Linear/Vercel.
        bg: {
          DEFAULT: '#0b0b0f',
          raised: '#15151c',
          input: '#1c1c25',
          border: '#26262f',
        },
        fg: {
          DEFAULT: '#e8e8ec',
          muted: '#a0a0ad',
          subtle: '#6b6b78',
        },
        accent: {
          DEFAULT: '#7c5cff',
          hover: '#8d70ff',
          active: '#6849f0',
          subtle: '#2a1d52',
        },
        success: '#34d399',
        danger: '#f87171',
        warning: '#fbbf24',
      },
      fontFamily: {
        sans: [
          '-apple-system',
          'BlinkMacSystemFont',
          'Inter',
          'system-ui',
          'sans-serif',
        ],
        mono: [
          'ui-monospace',
          'SFMono-Regular',
          'Menlo',
          'Monaco',
          'Consolas',
          'monospace',
        ],
      },
      boxShadow: {
        soft: '0 1px 2px rgba(0,0,0,0.4), 0 4px 12px rgba(0,0,0,0.25)',
        glow: '0 0 0 1px rgba(124,92,255,0.4), 0 8px 24px rgba(124,92,255,0.18)',
      },
      animation: {
        'fade-in': 'fadeIn 120ms ease-out',
        'slide-up': 'slideUp 200ms cubic-bezier(0.16, 1, 0.3, 1)',
      },
      keyframes: {
        fadeIn: {
          from: { opacity: '0' },
          to: { opacity: '1' },
        },
        slideUp: {
          from: { opacity: '0', transform: 'translateY(6px)' },
          to: { opacity: '1', transform: 'translateY(0)' },
        },
      },
    },
  },
  plugins: [],
};

export default config;
