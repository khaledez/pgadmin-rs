/**
 * Theme Manager
 *
 * Handles dark mode theme switching with localStorage persistence
 * Respects system preference (prefers-color-scheme) by default
 */

// Declare ToastManager as external (will be available from app.js)
/* global ToastManager */

const ThemeManager = (() => {
    const STORAGE_KEY = 'pgadmin-theme';
    const THEME_ATTRIBUTE = 'data-theme';
    const LIGHT_THEME = 'light';
    const DARK_THEME = 'dark';
    const AUTO_THEME = 'auto';

    /**
     * Get the current theme preference
     * Priority: localStorage > system preference > default (auto)
     */
    const getTheme = () => {
        const stored = localStorage.getItem(STORAGE_KEY);
        if (stored) {
            return stored;
        }
        // Check system preference
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
            return DARK_THEME;
        }
        return LIGHT_THEME;
    };

    /**
     * Get the effective theme (resolves 'auto' to actual theme)
     */
    const getEffectiveTheme = () => {
        const theme = getTheme();
        if (theme === AUTO_THEME) {
            return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches
                ? DARK_THEME
                : LIGHT_THEME;
        }
        return theme;
    };

    /**
     * Apply theme to document
     */
    const applyTheme = (theme) => {
        const effective = theme === AUTO_THEME
            ? getEffectiveTheme()
            : theme;

        document.documentElement.setAttribute(THEME_ATTRIBUTE, effective);

        // Update meta theme-color for mobile browsers
        const metaTheme = document.querySelector('meta[name="theme-color"]');
        if (metaTheme) {
            const colors = {
                light: '#ffffff',
                dark: '#1f2937',
            };
            metaTheme.setAttribute('content', colors[effective] || colors.light);
        }

        // Dispatch custom event for other components
        window.dispatchEvent(new CustomEvent('theme-change', {
            detail: { theme: effective }
        }));
    };

    /**
     * Set theme and persist to localStorage
     */
    const setTheme = (theme) => {
        if (![LIGHT_THEME, DARK_THEME, AUTO_THEME].includes(theme)) {
            console.warn(`Invalid theme: ${theme}. Using 'auto'.`);
            theme = AUTO_THEME;
        }

        localStorage.setItem(STORAGE_KEY, theme);
        applyTheme(theme);

        // Update theme toggle if it exists
        updateThemeToggle(theme);

        return theme;
    };

    /**
     * Toggle between light and dark themes
     */
    const toggleTheme = () => {
        const current = getTheme();
        const next = current === DARK_THEME ? LIGHT_THEME : DARK_THEME;
        return setTheme(next);
    };

    /**
     * Update the theme toggle button state
     */
    const updateThemeToggle = (theme) => {
        const toggle = document.getElementById('theme-toggle');
        if (!toggle) return;

        const icon = toggle.querySelector('span');
        if (icon) {
            const effective = theme === AUTO_THEME ? getEffectiveTheme() : theme;
            icon.textContent = effective === DARK_THEME ? '☀️' : '🌙';
        }

        toggle.setAttribute('aria-label',
            `Switch to ${theme === DARK_THEME ? 'light' : 'dark'} mode`);
    };

    /**
     * Create theme toggle button
     */
    const createToggleButton = () => {
        const button = document.createElement('button');
        button.id = 'theme-toggle';
        button.className = 'theme-toggle';
        button.type = 'button';
        button.title = 'Toggle dark/light mode';

        const theme = getTheme();
        const effective = theme === AUTO_THEME ? getEffectiveTheme() : theme;
        const icon = document.createElement('span');
        icon.textContent = effective === DARK_THEME ? '☀️' : '🌙';
        icon.className = 'theme-icon';

        button.appendChild(icon);
        button.addEventListener('click', () => {
            toggleTheme();
            if (typeof ToastManager !== 'undefined') {
                ToastManager.info(`Switched to ${getEffectiveTheme()} mode`);
            }
        });

        return button;
    };

    /**
     * Initialize theme system
     */
    const init = () => {
        const theme = getTheme();
        applyTheme(theme);

        // Listen for system theme changes
        if (window.matchMedia) {
            const darkModeQuery = window.matchMedia('(prefers-color-scheme: dark)');
            darkModeQuery.addEventListener('change', (e) => {
                const current = getTheme();
                if (current === AUTO_THEME) {
                    applyTheme(AUTO_THEME);
                }
            });
        }

        // Inject theme toggle into header if it exists
        const header = document.querySelector('header');
        if (header) {
            const existingToggle = document.getElementById('theme-toggle');
            if (!existingToggle) {
                const toggle = createToggleButton();
                header.appendChild(toggle);
            }
        }
    };

    return {
        getTheme,
        getEffectiveTheme,
        setTheme,
        toggleTheme,
        init,
        LIGHT_THEME,
        DARK_THEME,
        AUTO_THEME,
    };
})();

// Initialize theme when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => ThemeManager.init());
} else {
    ThemeManager.init();
}

// Expose globally
window.ThemeManager = ThemeManager;
