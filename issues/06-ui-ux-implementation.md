# Issue #06: UI/UX Implementation

## Overview
Design and implement a clean, modern, and intuitive user interface with minimal JavaScript using HTMX for dynamic interactions and server-side rendering with Askama templates.

## Goals
- Create a responsive, modern interface
- Minimize JavaScript while maintaining sleek UX
- Implement consistent design system
- Ensure accessibility
- Optimize for performance

## Design Principles

### Keep It Simple
- Clear visual hierarchy
- Intuitive navigation
- Minimal cognitive load
- Consistent patterns throughout

### Progressive Enhancement
- Core functionality works without JavaScript
- HTMX enhances the experience
- Graceful degradation

### Performance First
- Fast page loads
- Minimal bundle sizes
- Efficient rendering
- Optimized assets

## UI Architecture

### Layout Structure
```
┌─────────────────────────────────────────────┐
│              Header / Nav Bar               │
├──────────┬──────────────────────────────────┤
│          │                                  │
│ Sidebar  │        Main Content Area        │
│          │                                  │
│ - Home   │  ┌──────────────────────────┐  │
│ - Query  │  │                          │  │
│ - Browse │  │      Content             │  │
│ - Tables │  │                          │  │
│          │  └──────────────────────────┘  │
│          │                                  │
└──────────┴──────────────────────────────────┘
```

## Tasks

### 1. Template System Setup

**Base template with common layout:**
```html
<!-- templates/base.html -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}pgAdmin-rs{% endblock %}</title>

    <!-- CSS -->
    <link rel="stylesheet" href="/static/css/main.css">

    <!-- HTMX -->
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>

    <!-- Optional: Alpine.js for lightweight interactivity -->
    <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js"></script>

    {% block head %}{% endblock %}
</head>
<body>
    {% include "components/header.html" %}

    <div class="layout">
        {% include "components/sidebar.html" %}

        <main class="content">
            {% block content %}{% endblock %}
        </main>
    </div>

    {% include "components/notifications.html" %}
    {% block scripts %}{% endblock %}
</body>
</html>
```

### 2. Component Library

**Build reusable components:**

**Button component:**
```html
<!-- templates/components/button.html -->
<button
    class="btn btn-{{ variant }}"
    {% if hx_get %}hx-get="{{ hx_get }}"{% endif %}
    {% if hx_post %}hx-post="{{ hx_post }}"{% endif %}
    {% if hx_target %}hx-target="{{ hx_target }}"{% endif %}
    {{ attributes }}>
    {{ text }}
</button>
```

**Table component:**
```html
<!-- templates/components/table.html -->
<div class="table-container">
    <table class="data-table">
        <thead>
            <tr>
                {% for column in columns %}
                <th>
                    {% if sortable %}
                    <a hx-get="{{ sort_url }}&sort={{ column }}"
                       hx-target="#table-data">
                        {{ column }}
                    </a>
                    {% else %}
                    {{ column }}
                    {% endif %}
                </th>
                {% endfor %}
            </tr>
        </thead>
        <tbody>
            {% for row in rows %}
            <tr>
                {% for cell in row %}
                <td>{{ cell }}</td>
                {% endfor %}
            </tr>
            {% endfor %}
        </tbody>
    </table>
</div>
```

**Modal component:**
```html
<!-- templates/components/modal.html -->
<div class="modal" id="{{ id }}" x-data="{ open: false }">
    <div class="modal-backdrop" x-show="open" @click="open = false"></div>
    <div class="modal-content" x-show="open">
        <div class="modal-header">
            <h3>{{ title }}</h3>
            <button @click="open = false">&times;</button>
        </div>
        <div class="modal-body">
            {% block modal_body %}{% endblock %}
        </div>
        <div class="modal-footer">
            {% block modal_footer %}{% endblock %}
        </div>
    </div>
</div>
```

**Code editor component:**
```html
<!-- templates/components/code_editor.html -->
<div class="code-editor">
    <textarea
        id="sql-editor"
        name="query"
        class="sql-input"
        placeholder="Enter SQL query..."
        spellcheck="false">{{ query }}</textarea>

    <div class="editor-toolbar">
        <button hx-post="/query/execute"
                hx-include="#sql-editor"
                hx-target="#query-results"
                class="btn btn-primary">
            Execute
        </button>

        <button hx-post="/query/explain"
                hx-include="#sql-editor"
                hx-target="#query-results"
                class="btn btn-secondary">
            Explain
        </button>

        <button onclick="clearEditor()"
                class="btn btn-secondary">
            Clear
        </button>
    </div>
</div>

<script>
    function clearEditor() {
        document.getElementById('sql-editor').value = '';
    }
</script>
```

### 3. CSS Architecture

**Minimal, maintainable CSS structure:**

```css
/* static/css/main.css */

/* 1. CSS Variables for theming */
:root {
    /* Colors */
    --color-primary: #3b82f6;
    --color-secondary: #6b7280;
    --color-success: #10b981;
    --color-danger: #ef4444;
    --color-warning: #f59e0b;

    /* Neutrals */
    --color-bg: #ffffff;
    --color-surface: #f9fafb;
    --color-border: #e5e7eb;
    --color-text: #111827;
    --color-text-muted: #6b7280;

    /* Spacing */
    --space-xs: 0.25rem;
    --space-sm: 0.5rem;
    --space-md: 1rem;
    --space-lg: 1.5rem;
    --space-xl: 2rem;

    /* Typography */
    --font-sans: system-ui, -apple-system, sans-serif;
    --font-mono: 'Courier New', monospace;

    /* Borders */
    --radius-sm: 0.25rem;
    --radius-md: 0.5rem;
    --radius-lg: 0.75rem;

    /* Shadows */
    --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
    --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
    --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
}

/* 2. Reset and base styles */
*, *::before, *::after {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: var(--font-sans);
    color: var(--color-text);
    background-color: var(--color-bg);
    line-height: 1.5;
}

/* 3. Layout */
.layout {
    display: flex;
    height: calc(100vh - 60px);
}

.sidebar {
    width: 250px;
    background: var(--color-surface);
    border-right: 1px solid var(--color-border);
    overflow-y: auto;
}

.content {
    flex: 1;
    padding: var(--space-lg);
    overflow-y: auto;
}

/* 4. Components */
.btn {
    padding: var(--space-sm) var(--space-md);
    border: none;
    border-radius: var(--radius-md);
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
}

.btn-primary {
    background: var(--color-primary);
    color: white;
}

.btn-primary:hover {
    background: #2563eb;
}

.data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
}

.data-table th,
.data-table td {
    padding: var(--space-sm) var(--space-md);
    text-align: left;
    border-bottom: 1px solid var(--color-border);
}

.data-table th {
    background: var(--color-surface);
    font-weight: 600;
    position: sticky;
    top: 0;
}

.data-table tr:hover {
    background: var(--color-surface);
}

/* 5. Utility classes */
.hidden { display: none; }
.flex { display: flex; }
.items-center { align-items: center; }
.justify-between { justify-content: space-between; }
.gap-md { gap: var(--space-md); }
.text-sm { font-size: 0.875rem; }
.text-muted { color: var(--color-text-muted); }
.font-mono { font-family: var(--font-mono); }
```

### 4. HTMX Patterns

**Common HTMX patterns to implement:**

**1. Lazy loading:**
```html
<div hx-get="/api/slow-data"
     hx-trigger="load"
     hx-indicator="#spinner">
    <div id="spinner" class="htmx-indicator">Loading...</div>
</div>
```

**2. Infinite scroll:**
```html
<div hx-get="/api/more-rows?page=2"
     hx-trigger="revealed"
     hx-swap="afterend">
    More content loads when scrolled into view
</div>
```

**3. Inline editing:**
```html
<div hx-get="/edit/{{ id }}"
     hx-trigger="click"
     hx-target="this"
     hx-swap="outerHTML">
    {{ value }}
</div>
```

**4. Polling for updates:**
```html
<div hx-get="/status"
     hx-trigger="every 2s"
     hx-target="this"
     hx-swap="innerHTML">
    Checking status...
</div>
```

**5. Form submission:**
```html
<form hx-post="/api/submit"
      hx-target="#result"
      hx-swap="innerHTML"
      hx-indicator="#spinner">
    <input type="text" name="value">
    <button type="submit">Submit</button>
</form>

<div id="result"></div>
<div id="spinner" class="htmx-indicator">Processing...</div>
```

### 5. SQL Editor Enhancement

**Lightweight syntax highlighting with Prism.js:**
```html
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism.min.css">
<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/prism.min.js"></script>
<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-sql.min.js"></script>

<div class="editor-container">
    <pre><code class="language-sql" contenteditable="true" id="sql-editor">{{ query }}</code></pre>
</div>
```

### 6. Responsive Design

**Mobile-friendly breakpoints:**
```css
/* Mobile first approach */
@media (max-width: 768px) {
    .layout {
        flex-direction: column;
    }

    .sidebar {
        width: 100%;
        height: auto;
        border-right: none;
        border-bottom: 1px solid var(--color-border);
    }

    .data-table {
        font-size: 0.75rem;
    }
}

@media (max-width: 480px) {
    .content {
        padding: var(--space-sm);
    }

    .data-table th,
    .data-table td {
        padding: var(--space-xs);
    }
}
```

### 7. Loading States and Feedback

**HTMX indicators:**
```css
.htmx-indicator {
    display: none;
}

.htmx-request .htmx-indicator {
    display: inline-block;
}

.htmx-request.htmx-indicator {
    display: inline-block;
}

.spinner {
    border: 2px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    width: 20px;
    height: 20px;
    animation: spin 1s linear infinite;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}
```

**Toast notifications:**
```html
<!-- templates/components/toast.html -->
<div class="toast toast-{{ type }}"
     x-data="{ show: true }"
     x-show="show"
     x-init="setTimeout(() => show = false, 3000)"
     x-transition>
    {{ message }}
</div>
```

### 8. Accessibility

**ARIA attributes and semantic HTML:**
```html
<!-- Good semantic structure -->
<nav aria-label="Main navigation">
    <ul role="list">
        <li><a href="/" aria-current="page">Home</a></li>
        <li><a href="/query">Query</a></li>
    </ul>
</nav>

<!-- Accessible forms -->
<label for="query-input">SQL Query</label>
<textarea id="query-input"
          aria-describedby="query-help"
          required></textarea>
<small id="query-help">Enter your PostgreSQL query</small>

<!-- Keyboard navigation -->
<button type="button"
        aria-label="Close modal"
        @keydown.escape="close()">
    &times;
</button>
```

### 9. Dark Mode (Optional)

**CSS variables for dark mode:**
```css
@media (prefers-color-scheme: dark) {
    :root {
        --color-bg: #111827;
        --color-surface: #1f2937;
        --color-border: #374151;
        --color-text: #f9fafb;
        --color-text-muted: #9ca3af;
    }
}

/* Or manual toggle */
[data-theme="dark"] {
    --color-bg: #111827;
    --color-surface: #1f2937;
    /* ... */
}
```

### 10. Performance Optimization

**Optimization checklist:**
- [ ] Minify CSS and JavaScript
- [ ] Use CSS containment for complex components
- [ ] Lazy load images and heavy components
- [ ] Use `will-change` sparingly for animations
- [ ] Implement virtual scrolling for large tables
- [ ] Cache static assets with proper headers
- [ ] Use font-display: swap for web fonts

## File Structure
```
static/
├── css/
│   ├── main.css
│   ├── components.css
│   └── utilities.css
├── js/
│   ├── app.js           # Minimal custom JS
│   └── editor.js        # SQL editor enhancements
└── images/
    └── logo.svg

templates/
├── base.html
├── components/
│   ├── header.html
│   ├── sidebar.html
│   ├── button.html
│   ├── table.html
│   ├── modal.html
│   ├── toast.html
│   └── code_editor.html
├── pages/
│   ├── dashboard.html
│   ├── query.html
│   ├── browser.html
│   └── table.html
└── partials/
    └── ...
```

## Testing Requirements
- [ ] UI renders correctly in major browsers
- [ ] Mobile responsive layout works
- [ ] HTMX interactions function properly
- [ ] Keyboard navigation works
- [ ] Screen reader compatible
- [ ] Fast page load times (<2s)
- [ ] CSS validates
- [ ] No console errors

## Accessibility Checklist
- [ ] Semantic HTML elements used
- [ ] Proper heading hierarchy
- [ ] Alt text for images
- [ ] ARIA labels where needed
- [ ] Keyboard navigable
- [ ] Focus indicators visible
- [ ] Color contrast ratio compliant (WCAG AA)
- [ ] Form labels associated properly

## Browser Support
- Chrome (latest 2 versions)
- Firefox (latest 2 versions)
- Safari (latest 2 versions)
- Edge (latest 2 versions)

## Acceptance Criteria
- [ ] All pages have consistent layout
- [ ] Components are reusable
- [ ] HTMX provides smooth interactions
- [ ] Minimal JavaScript footprint (<50KB total)
- [ ] Responsive on mobile devices
- [ ] Accessible to screen readers
- [ ] Fast and performant
- [ ] Documentation complete
