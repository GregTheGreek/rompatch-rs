import type { Config } from 'tailwindcss';

const config: Config = {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // Neutral light-grey palette. No blue tint - reads closer to macOS
        // native dark mode. Steps preserve clear contrast between layers.
        bg: {
          DEFAULT: '#1e1e20',
          raised: '#2a2a2d',
          input: '#37373b',
          border: '#48484d',
        },
        fg: {
          DEFAULT: '#ebebec',
          muted: '#a5a5a8',
          subtle: '#707075',
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
        'pulse-glow': 'pulseGlow 1400ms ease-in-out infinite',
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
        pulseGlow: {
          '0%, 100%': {
            boxShadow: '0 0 0 0 rgba(124,92,255,0.55), 0 0 24px rgba(124,92,255,0.25)',
          },
          '50%': {
            boxShadow: '0 0 0 6px rgba(124,92,255,0), 0 0 36px rgba(124,92,255,0.45)',
          },
        },
      },
    },
  },
  plugins: [],
};

export default config;
