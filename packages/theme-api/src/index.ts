/**
 * NeonShell Theme API
 * 
 * This package provides TypeScript types for building NeonShell themes.
 */

export interface Theme {
  id: string;
  name: string;
  version: string;
  author?: string;
  description?: string;
  colors: ThemeColors;
  terminal?: TerminalTheme;
  ui?: UITheme;
  css_file?: string;
}

export interface ThemeColors {
  /** Main background color */
  background: string;
  /** Main text color */
  foreground: string;
  /** Primary accent color */
  accent: string;
  /** Muted accent color */
  accent_muted?: string;
  /** Darkest surface (usually same as background) */
  surface_0?: string;
  /** Slightly lighter surface */
  surface_1?: string;
  /** Medium surface color */
  surface_2?: string;
  /** Lightest surface color */
  surface_3?: string;
  /** Border color */
  border?: string;
  /** Terminal cursor color */
  cursor?: string;
  /** Selection highlight color */
  selection?: string;
  /** Error color */
  error?: string;
  /** Warning color */
  warning?: string;
  /** Success color */
  success?: string;
}

export interface TerminalTheme {
  /** Terminal font family */
  font_family?: string;
  /** Terminal font size */
  font_size?: number;
  /** ANSI color palette */
  ansi_colors?: AnsiColors;
}

export interface AnsiColors {
  black: string;
  red: string;
  green: string;
  yellow: string;
  blue: string;
  magenta: string;
  cyan: string;
  white: string;
  bright_black: string;
  bright_red: string;
  bright_green: string;
  bright_yellow: string;
  bright_blue: string;
  bright_magenta: string;
  bright_cyan: string;
  bright_white: string;
}

export interface UITheme {
  /** Border radius for UI elements */
  border_radius?: string;
  /** Enable drop shadows */
  shadows?: boolean;
  /** Enable animations */
  animations?: boolean;
  /** Enable backdrop blur effects */
  blur?: boolean;
}

/**
 * Create a theme object with defaults
 */
export function createTheme(partial: Partial<Theme> & { id: string; name: string }): Theme {
  return {
    version: '1.0.0',
    colors: {
      background: '#0a0a0f',
      foreground: '#e0e0e0',
      accent: '#ff0080',
      ...partial.colors,
    },
    terminal: {
      font_family: 'JetBrains Mono',
      font_size: 14,
      ...partial.terminal,
    },
    ui: {
      border_radius: '8px',
      shadows: true,
      animations: true,
      blur: true,
      ...partial.ui,
    },
    ...partial,
  };
}

/**
 * Generate CSS variables from a theme
 */
export function generateCSSVariables(theme: Theme): string {
  const vars: string[] = [];
  
  // Colors
  vars.push(`--background: ${theme.colors.background};`);
  vars.push(`--foreground: ${theme.colors.foreground};`);
  vars.push(`--accent: ${theme.colors.accent};`);
  
  if (theme.colors.accent_muted) {
    vars.push(`--accent-muted: ${theme.colors.accent_muted};`);
  }
  
  for (let i = 0; i <= 3; i++) {
    const key = `surface_${i}` as keyof ThemeColors;
    if (theme.colors[key]) {
      vars.push(`--surface-${i}: ${theme.colors[key]};`);
    }
  }
  
  if (theme.colors.border) vars.push(`--border: ${theme.colors.border};`);
  if (theme.colors.cursor) vars.push(`--cursor: ${theme.colors.cursor};`);
  if (theme.colors.selection) vars.push(`--selection: ${theme.colors.selection};`);
  if (theme.colors.error) vars.push(`--error: ${theme.colors.error};`);
  if (theme.colors.warning) vars.push(`--warning: ${theme.colors.warning};`);
  if (theme.colors.success) vars.push(`--success: ${theme.colors.success};`);
  
  return `:root {\n  ${vars.join('\n  ')}\n}`;
}

/**
 * Validate a theme object
 */
export function validateTheme(theme: unknown): theme is Theme {
  if (typeof theme !== 'object' || theme === null) return false;
  
  const t = theme as Record<string, unknown>;
  
  if (typeof t.id !== 'string' || !t.id) return false;
  if (typeof t.name !== 'string' || !t.name) return false;
  if (typeof t.colors !== 'object' || t.colors === null) return false;
  
  const colors = t.colors as Record<string, unknown>;
  if (typeof colors.background !== 'string') return false;
  if (typeof colors.foreground !== 'string') return false;
  if (typeof colors.accent !== 'string') return false;
  
  return true;
}

