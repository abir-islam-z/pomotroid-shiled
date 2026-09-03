// Theme store.
// Applies theme colors to CSS custom properties on :root with native glassmorphic transparency.

import type { Theme } from '$lib/types';

/** Parse hex color string to RGBA string with custom alpha */
function hexToRgba(hex: string, alpha: number): string {
  let c = hex.replace('#', '').trim();
  if (c.length === 3) {
    c = c.split('').map((x) => x + x).join('');
  }
  if (c.length === 6) {
    const num = parseInt(c, 16);
    const r = (num >> 16) & 255;
    const g = (num >> 8) & 255;
    const b = num & 255;
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
  }
  return hex;
}

/** Apply a theme's colors to the document root CSS custom properties.
 *  Theme keys already include the `--` prefix (e.g. "--color-background"). */
export function applyTheme(theme: Theme): void {
  const root = document.documentElement;
  for (const [key, value] of Object.entries(theme.colors)) {
    root.style.setProperty(key, value);
  }

  // Derive glassmorphic translucency tokens for ANY theme color
  const bgHex = theme.colors['--color-background'];
  if (bgHex) {
    // 0.76 opacity allows macOS native hudWindow / vibrancy blur to show through vividly
    root.style.setProperty('--color-theme-glass', hexToRgba(bgHex, 0.76));
    root.style.setProperty('--color-theme-glass-subtle', hexToRgba(bgHex, 0.5));
    root.style.setProperty('--color-theme-glass-deep', hexToRgba(bgHex, 0.88));
  }
}
