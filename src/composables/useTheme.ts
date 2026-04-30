import { ref } from 'vue';

type Theme = 'light' | 'dark';

const theme = ref<Theme>('light');

export function useTheme() {
  function applyTheme(t: Theme) {
    if (t === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
    theme.value = t;
  }

  function toggleTheme() {
    const next = theme.value === 'light' ? 'dark' : 'light';
    applyTheme(next);
    localStorage.setItem('omnilink-theme', next);
  }

  function initTheme() {
    const saved = localStorage.getItem('omnilink-theme') as Theme | null;
    if (saved) {
      applyTheme(saved);
    } else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
      applyTheme('dark');
    } else {
      applyTheme('light');
    }
  }

  return { theme, toggleTheme, initTheme };
}
