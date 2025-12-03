// pgAdmin-rs Application JavaScript
// Handles UI interactions, notifications, and enhancements

// Import HTMX
import htmx from 'htmx.org';

// Make htmx available globally
window.htmx = htmx;

/**
 * Toast Notification System
 */
const ToastManager = (() => {
    const showToast = (message, type = 'info', duration = 3000) => {
        const container = getOrCreateContainer();
        const toast = createToastElement(message, type);

        container.appendChild(toast);

        // Auto-remove after duration
        setTimeout(() => {
            removeToast(toast);
        }, duration);

        return toast;
    };

    const createToastElement = (message, type) => {
        const toast = document.createElement('div');
        toast.className = `toast toast-${type}`;
        toast.textContent = message;
        toast.style.animation = 'slideIn 0.3s ease-out';

        // Add click to dismiss
        toast.addEventListener('click', () => removeToast(toast));

        return toast;
    };

    const removeToast = (toast) => {
        toast.classList.add('removing');
        setTimeout(() => {
            toast.remove();
        }, 300);
    };

    const getOrCreateContainer = () => {
        let container = document.querySelector('.toast-container');
        if (!container) {
            container = document.createElement('div');
            container.className = 'toast-container';
            document.body.appendChild(container);
        }
        return container;
    };

    return {
        success: (msg, duration) => showToast(msg, 'success', duration),
        error: (msg, duration) => showToast(msg, 'error', duration),
        warning: (msg, duration) => showToast(msg, 'warning', duration),
        info: (msg, duration) => showToast(msg, 'info', duration),
    };
})();

/**
 * Modal Dialog System
 */
const ModalManager = (() => {
    const openModal = (id) => {
        const modal = document.getElementById(id);
        if (modal) {
            modal.classList.add('open');
            return modal;
        }
        return null;
    };

    const closeModal = (id) => {
        const modal = document.getElementById(id);
        if (modal) {
            modal.classList.remove('open');
        }
    };

    const closeAllModals = () => {
        document.querySelectorAll('.modal.open').forEach(modal => {
            modal.classList.remove('open');
        });
    };

    // Click backdrop to close
    document.addEventListener('click', (e) => {
        if (e.target.classList.contains('modal-backdrop')) {
            e.target.closest('.modal')?.classList.remove('open');
        }
    });

    // Close button handling
    document.addEventListener('click', (e) => {
        if (e.target.closest('.modal-header button')) {
            e.target.closest('.modal')?.classList.remove('open');
        }
    });

    return {
        open: openModal,
        close: closeModal,
        closeAll: closeAllModals,
    };
})();

/**
 * Tree/Sidebar Navigation Functions
 */
const toggleElement = (id) => {
    const content = document.getElementById(id);
    const icon = document.getElementById('icon-' + id);
    if (content && icon) {
        if (content.style.display === 'none') {
            content.style.display = 'block';
            icon.classList.add('expanded');
        } else {
            content.style.display = 'none';
            icon.classList.remove('expanded');
        }
    }
};

// Handle data-toggle attributes for tree navigation
document.addEventListener('click', (e) => {
    const toggleTarget = e.target.closest('[data-toggle]');
    if (toggleTarget) {
        const id = toggleTarget.getAttribute('data-toggle');
        toggleElement(id);
    }
});

// Handle data-tab attributes for tab switching
document.addEventListener('click', (e) => {
    const tabBtn = e.target.closest('[data-tab]');
    if (tabBtn) {
        const tabName = tabBtn.getAttribute('data-tab');

        // Hide all tabs
        document.querySelectorAll('.tab-content').forEach(tab => {
            tab.classList.remove('active');
        });
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.classList.remove('active');
        });

        // Show selected tab
        const tabContent = document.getElementById(tabName + '-tab');
        if (tabContent) {
            tabContent.classList.add('active');
        }
        tabBtn.classList.add('active');
    }
});

// Keep global functions for backward compatibility
window.toggleSchema = toggleElement;
window.toggleGroup = toggleElement;
window.switchTab = function(tabName) {
    // Dispatch a click event on the corresponding tab button
    const tabBtn = document.querySelector(`[data-tab="${tabName}"]`);
    if (tabBtn) {
        tabBtn.click();
    }
};

/**
 * HTMX Event Handling
 */
document.addEventListener('htmx:responseError', (evt) => {
    const error = evt.detail.xhr.responseText || 'An error occurred';
    ToastManager.error(`Request failed: ${error}`);
});

document.addEventListener('htmx:sendError', (evt) => {
    ToastManager.error('Failed to send request');
});

/**
 * Keyboard Shortcuts
 */
const setupKeyboardShortcuts = () => {
    document.addEventListener('keydown', (e) => {
        // Ctrl/Cmd + K: Focus query editor
        if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
            e.preventDefault();
            const queryInput = document.getElementById('sql-input');
            if (queryInput) {
                queryInput.focus();
            }
        }

        // Ctrl/Cmd + Enter: Execute query
        if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
            const queryForm = document.getElementById('query-form');
            if (queryForm && document.activeElement === document.getElementById('sql-input')) {
                e.preventDefault();
                queryForm.dispatchEvent(new Event('submit'));
            }
        }

        // Escape: Close all modals
        if (e.key === 'Escape') {
            ModalManager.closeAll();
        }
    });
};

/**
 * Initialize Application
 */
const initApp = () => {
    console.log('Initializing pgAdmin-rs...');
    setupKeyboardShortcuts();

    // Log successful HTMX requests for debugging (optional)
    document.addEventListener('htmx:afterRequest', (evt) => {
        if (evt.detail.xhr.status >= 200 && evt.detail.xhr.status < 300) {
            console.log('Request successful:', evt.detail.pathInfo.requestPath);
        }
    });
};

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initApp);
} else {
    initApp();
}

// Expose globally for template usage
window.ToastManager = ToastManager;
window.ModalManager = ModalManager;
