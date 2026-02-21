# FluentLM — Component Reference

Compact reference for LLM-driven HTML generation. Each component shows 1–3 examples followed by its class list.

---

## Global Classes

Apply to any element.

| Class | Effect |
|-------|--------|
| `flm-text--secondary` | Color: `--bodySubtext` |
| `flm-text--disabled` | Color: `--disabledText` |
| `flm-text--error` | Color: `--errorText` |
| `flm-text--success` | Color: `--successText` |
| `flm-text--nowrap` | Truncate with ellipsis |
| `flm-text--block` | `display: block` |
| `flm-text--semibold` | Weight 600 |
| `flm-text--bold` | Weight 700 |
| `flm-sr-only` | Visually hidden, screen-reader accessible |

---

## Button

```html
<button class="flm-button">Default</button>
<button class="flm-button flm-button--primary" data-icon="Save">Save</button>
<button class="flm-button flm-button--icon" data-icon="Settings" aria-label="Settings"></button>
```

| Class | Effect |
|-------|--------|
| `flm-button` | Base button |
| `flm-button--primary` | Brand-colored fill |
| `flm-button--compound` | Two-line button (needs `.flm-button-label` + `.flm-button-description` children) |
| `flm-button--icon` | Icon-only, square (requires `aria-label`) |
| `flm-button--subtle` | No border, transparent background |

| Attribute | Effect |
|-----------|--------|
| `data-icon="Name"` | JS injects icon before label |
| `data-split` | JS adds dropdown caret |
| `disabled` | Native disabled state |

| Child class | Where |
|-------------|-------|
| `flm-button-label` | Primary text in compound button |
| `flm-button-description` | Secondary text in compound button |

---

## Text

```html
<span class="flm-text">Default (medium)</span>
<span class="flm-text flm-text--xLarge">Heading</span>
<p class="flm-text flm-text--small flm-text--secondary flm-text--block">Caption paragraph.</p>
```

| Class | Size |
|-------|------|
| `flm-text--tiny` | 10px |
| `flm-text--xSmall` | 10px |
| `flm-text--small` | 12px |
| `flm-text--smallPlus` | 12px |
| `flm-text--medium` | 14px (default) |
| `flm-text--mediumPlus` | 16px |
| `flm-text--large` | 18px |
| `flm-text--xLarge` | 20px / semibold |
| `flm-text--xxLarge` | 28px / semibold |
| `flm-text--superLarge` | 42px / semibold |
| `flm-text--mega` | 68px / semibold |

---

## Label

```html
<label class="flm-label" for="f1">Name</label>
<label class="flm-label flm-label--required" for="f2">Email</label>
```

| Class | Effect |
|-------|--------|
| `flm-label` | Base label |
| `flm-label--required` | Adds red asterisk |
| `flm-label--disabled` | Dimmed text |

---

## Link

```html
<a class="flm-link" href="/page">Go to page</a>
```

| Class | Effect |
|-------|--------|
| `flm-link` | Base link |
| `flm-link--disabled` | Dimmed, no pointer events |

---

## Icon

```html
<i class="flm-icon" data-icon="Mail"></i>
<i class="flm-icon flm-icon--large" data-icon="ChevronDown"></i>
```

| Class | Size |
|-------|------|
| `flm-icon--small` | 12px |
| (default) | 16px |
| `flm-icon--large` | 20px |

| Attribute | Effect |
|-----------|--------|
| `data-icon="Name"` | JS resolves to SVG/glyph |

---

## TextField

```html
<div class="flm-textfield">
  <label class="flm-label" for="f1">Name</label>
  <input class="flm-textfield-input" id="f1" placeholder="Enter name">
</div>

<div class="flm-textfield flm-textfield--error">
  <label class="flm-label" for="f2">Age</label>
  <input class="flm-textfield-input" id="f2" value="-1">
  <span class="flm-textfield-error">Must be positive</span>
</div>

<div class="flm-textfield">
  <label class="flm-label" for="f3">Notes</label>
  <textarea class="flm-textfield-input" id="f3" rows="4"></textarea>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-textfield` | Base wrapper |
| `flm-textfield--required` | Required field styling |
| `flm-textfield--error` | Error border + enables `.flm-textfield-error` |
| `flm-textfield--disabled` | Dimmed, pair with `disabled` on input |
| `flm-textfield--underlined` | Bottom border only |
| `flm-textfield--borderless` | No border |

| Child class | Element |
|-------------|---------|
| `flm-textfield-input` | `<input>` or `<textarea>` |
| `flm-textfield-error` | `<span>` error message |
| `flm-textfield-wrapper` | `<div>` wraps input when using prefix/suffix |
| `flm-textfield-prefix` | `<span>` before input |
| `flm-textfield-suffix` | `<span>` after input |

---

## Checkbox

```html
<label class="flm-checkbox">
  <input type="checkbox" class="flm-checkbox-input">
  <span class="flm-checkbox-mark"></span>
  <span class="flm-checkbox-label">Accept terms</span>
</label>
```

| Class | Effect |
|-------|--------|
| `flm-checkbox` | Base wrapper (is a `<label>`) |
| `flm-checkbox--disabled` | Dimmed, pair with `disabled` on input |

| Child class | Element |
|-------------|---------|
| `flm-checkbox-input` | Hidden native `<input type="checkbox">` |
| `flm-checkbox-mark` | Visual checkmark box |
| `flm-checkbox-label` | Text label |

Native: `checked`, `disabled`, `indeterminate` (via JS).

---

## Toggle

```html
<label class="flm-toggle">
  <span class="flm-toggle-label">Notifications</span>
  <input type="checkbox" class="flm-toggle-input">
  <span class="flm-toggle-track"><span class="flm-toggle-thumb"></span></span>
  <span class="flm-toggle-state" data-on="On" data-off="Off"></span>
</label>
```

| Class | Effect |
|-------|--------|
| `flm-toggle` | Base wrapper (is a `<label>`) |
| `flm-toggle--disabled` | Dimmed, pair with `disabled` on input |
| `flm-toggle--inline` | Label and track on same line |

| Child class | Element |
|-------------|---------|
| `flm-toggle-input` | Hidden native `<input type="checkbox">` |
| `flm-toggle-label` | Text label |
| `flm-toggle-track` | Sliding rail |
| `flm-toggle-thumb` | Circle knob inside track |
| `flm-toggle-state` | Shows `data-on`/`data-off` text based on state |

---

## Dropdown

```html
<div class="flm-dropdown">
  <label class="flm-label">Country</label>
  <button class="flm-dropdown-trigger">
    <span class="flm-dropdown-title flm-dropdown-title--placeholder">Select…</span>
    <span class="flm-dropdown-caret" data-icon="ChevronDown"></span>
  </button>
  <div class="flm-dropdown-listbox">
    <div class="flm-dropdown-option" data-value="us">United States</div>
    <div class="flm-dropdown-option" data-value="gb">United Kingdom</div>
  </div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-dropdown` | Container |
| `flm-dropdown--disabled` | Disabled state |
| `flm-dropdown--error` | Error border |

| Attribute | Effect |
|-----------|--------|
| `data-value="val"` | On `.flm-dropdown-option` — option value. Auto-set on root when selected |

| Child class | Element |
|-------------|---------|
| `flm-dropdown-trigger` | Clickable button (styled like ComboBox wrapper) |
| `flm-dropdown-title` | Selected text display |
| `flm-dropdown-title--placeholder` | Placeholder color |
| `flm-dropdown-caret` | Chevron icon |
| `flm-dropdown-listbox` | Options popup |
| `flm-dropdown-option` | Individual option |
| `flm-dropdown-option--selected` | Selected option |
| `flm-dropdown-option--disabled` | Disabled option |
| `flm-dropdown-value` | Hidden `<input>` for form submission (optional) |
| `flm-dropdown-error` | `<span>` error message |

JS: keyboard nav (ArrowUp/Down/Enter/Space/Escape), click-outside dismiss, flip-above.

---

## Stack

```html
<div class="flm-stack" style="gap: var(--spacingS1)">
  <div>Row 1</div>
  <div>Row 2</div>
</div>

<div class="flm-stack flm-stack--horizontal" style="gap: var(--spacingM)">
  <div class="flm-stack-item flm-stack-item--grow">Fills space</div>
  <div>Fixed</div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-stack` | Vertical flexbox (default) |
| `flm-stack--horizontal` | Row direction |
| `flm-stack--center` | Center both axes |
| `flm-stack--end` | Align to end |
| `flm-stack--space-between` | Space between items |
| `flm-stack--wrap` | Allow wrapping |

| Child class | Effect |
|-------------|--------|
| `flm-stack-item--grow` | `flex-grow: 1` |
| `flm-stack-item--shrink` | `flex-shrink: 1` |
| `flm-stack-item--align-end` | Self-align to end |
| `flm-stack-item--align-center` | Self-align to center |

Inline `style="gap: var(--spacingS1)"` sets spacing per instance.

---

## Separator

```html
<hr class="flm-separator">
<hr class="flm-separator flm-separator--vertical">
```

| Class | Effect |
|-------|--------|
| `flm-separator` | Horizontal line |
| `flm-separator--vertical` | Vertical line (use in horizontal stacks) |

---

## Spinner

```html
<div class="flm-spinner"><div class="flm-spinner-circle"></div></div>
<div class="flm-spinner flm-spinner--large">
  <div class="flm-spinner-circle"></div>
  <span class="flm-spinner-label">Loading…</span>
</div>
```

| Class | Size |
|-------|------|
| `flm-spinner--xSmall` | 12px |
| `flm-spinner--small` | 16px |
| (default) | 20px |
| `flm-spinner--large` | 28px |

| Child class | Element |
|-------------|---------|
| `flm-spinner-circle` | Animated ring |
| `flm-spinner-label` | Optional text label |

---

## MessageBar

```html
<div class="flm-messagebar flm-messagebar--success">Operation completed.</div>
<div class="flm-messagebar flm-messagebar--error">Something went wrong.</div>
```

| Class | Type |
|-------|------|
| `flm-messagebar--info` | Neutral info (default) |
| `flm-messagebar--success` | Green success |
| `flm-messagebar--warning` | Yellow warning |
| `flm-messagebar--severeWarning` | Orange severe warning |
| `flm-messagebar--error` | Red error |
| `flm-messagebar--blocked` | Red blocked |

| Child class | Element |
|-------------|---------|
| `flm-messagebar-icon` | Leading icon (auto-injected by JS if absent) |
| `flm-messagebar-text` | Message content |
| `flm-messagebar-actions` | Action buttons container |
| `flm-messagebar-dismiss` | Dismiss/close button |

---

## Dialog

```html
<div class="flm-dialog-overlay" id="dlg1" data-light-dismiss>
  <div class="flm-dialog">
    <div class="flm-dialog-header">
      <h2 class="flm-dialog-title">Confirm</h2>
      <button class="flm-dialog-close" data-icon="Cancel" aria-label="Close"></button>
    </div>
    <div class="flm-dialog-body">Are you sure?</div>
    <div class="flm-dialog-footer">
      <button class="flm-button" data-dialog-close>Cancel</button>
      <button class="flm-button flm-button--primary" data-dialog-close>OK</button>
    </div>
  </div>
</div>
<button class="flm-button" data-dialog-open="dlg1">Open Dialog</button>
```

| Class | Effect |
|-------|--------|
| `flm-dialog-overlay` | Full-screen overlay container (give it an `id`) |
| `flm-dialog` | The dialog box |
| `flm-dialog--large-header` | Larger title font |

| Attribute | Effect |
|-----------|--------|
| `data-dialog-open="id"` | On trigger button — opens dialog |
| `data-dialog-close` | On any button — closes containing dialog |
| `data-light-dismiss` | On overlay — click outside to close |

| Child class | Element |
|-------------|---------|
| `flm-dialog-header` | Header row |
| `flm-dialog-title` | `<h2>` title |
| `flm-dialog-close` | Close X button |
| `flm-dialog-body` | Content area |
| `flm-dialog-footer` | Action buttons row |

---

## Panel

```html
<div class="flm-panel-overlay"></div>
<div class="flm-panel" id="panel1">
  <div class="flm-panel-header">
    <h2 class="flm-panel-title">Settings</h2>
    <button class="flm-panel-close" data-icon="Cancel" aria-label="Close"></button>
  </div>
  <div class="flm-panel-body">Panel content here.</div>
  <div class="flm-panel-footer">
    <button class="flm-button flm-button--primary" data-panel-close>Save</button>
  </div>
</div>
<button class="flm-button" data-panel-open="panel1">Open Panel</button>
```

| Class | Effect |
|-------|--------|
| `flm-panel` | Slide-in container (give it an `id`) |
| `flm-panel--small` | 272px width |
| `flm-panel--medium` | 592px width |
| `flm-panel--large` | 940px width |
| `flm-panel--extraLarge` | 1200px width |
| `flm-panel--fluid` | 100% width |
| `flm-panel--left` | Slides from left instead of right |

| Attribute | Effect |
|-----------|--------|
| `data-panel-open="id"` | On trigger — opens panel |
| `data-panel-close` | On button — closes containing panel |

| Child class | Element |
|-------------|---------|
| `flm-panel-header` | Header row |
| `flm-panel-title` | `<h2>` title |
| `flm-panel-close` | Close X button |
| `flm-panel-body` | Scrollable content |
| `flm-panel-footer` | Actions row (sticky bottom) |

---

## Modal

```html
<div class="flm-modal-overlay" id="modal1" data-light-dismiss>
  <div class="flm-modal" style="width: 480px; padding: var(--spacingL1);">
    Content here
    <button class="flm-button" data-modal-close>Close</button>
  </div>
</div>
<button class="flm-button" data-modal-open="modal1">Open Modal</button>
```

| Class | Effect |
|-------|--------|
| `flm-modal-overlay` | Full-screen overlay (give it an `id`) |
| `flm-modal` | Centered content box |

| Attribute | Effect |
|-----------|--------|
| `data-modal-open="id"` | Opens modal |
| `data-modal-close` | Closes containing modal |
| `data-light-dismiss` | On overlay — click outside to close |
| `data-blocking` | On overlay — prevents Escape/click dismiss |

---

## Callout

```html
<button class="flm-button" data-callout-toggle="co1">Show Info</button>
<div class="flm-callout" id="co1">
  <div class="flm-callout-beak"></div>
  <div class="flm-callout-body">Callout content here.</div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-callout` | Positioned popup (give it an `id`) |

| Attribute | Effect |
|-----------|--------|
| `data-callout-toggle="id"` | Toggle callout visibility |

| Child class | Element |
|-------------|---------|
| `flm-callout-beak` | Arrow pointing to target |
| `flm-callout-body` | Content area |

---

## CommandBar

```html
<div class="flm-commandbar">
  <div class="flm-commandbar-items">
    <button class="flm-commandbar-item" data-icon="Add">New</button>
    <button class="flm-commandbar-item" data-icon="Edit">Edit</button>
    <span class="flm-commandbar-divider"></span>
    <button class="flm-commandbar-item" data-icon="Delete">Delete</button>
  </div>
  <div class="flm-commandbar-far">
    <button class="flm-commandbar-item" data-icon="Filter">Filter</button>
  </div>
</div>
```

| Class | Element |
|-------|---------|
| `flm-commandbar` | Toolbar container |
| `flm-commandbar-items` | Left-side items |
| `flm-commandbar-far` | Right-side items |
| `flm-commandbar-item` | Individual command button |
| `flm-commandbar-divider` | Vertical separator |
| `flm-commandbar-overflow` | Overflow "..." button |

| Attribute | Effect |
|-----------|--------|
| `data-icon="Name"` | Icon before label |
| `disabled` | Disabled state |

---

## Nav

```html
<nav class="flm-nav">
  <div class="flm-nav-group">
    <button class="flm-nav-group-header">
      <i class="flm-nav-chevron flm-nav-chevron--expanded" data-icon="ChevronRight"></i>
      Section
    </button>
    <div class="flm-nav-group-items">
      <a class="flm-nav-link flm-nav-link--active" href="#">Home</a>
      <a class="flm-nav-link" href="#">Settings</a>
      <a class="flm-nav-link flm-nav-link--disabled">Disabled</a>
    </div>
  </div>
</nav>
```

| Class | Effect |
|-------|--------|
| `flm-nav` | Nav container |
| `flm-nav-group` | Section group |
| `flm-nav-group-header` | Collapsible group title |
| `flm-nav-group-items` | Links container |
| `flm-nav-group-items--collapsed` | Hidden (toggled by JS) |
| `flm-nav-link` | Navigation link |
| `flm-nav-link--active` | Active/selected link |
| `flm-nav-link--disabled` | Disabled link |
| `flm-nav-link--nested` | Extra indentation |
| `flm-nav-chevron` | Expand/collapse icon |
| `flm-nav-chevron--expanded` | Rotated 90deg |

---

## Breadcrumb

```html
<ol class="flm-breadcrumb">
  <li class="flm-breadcrumb-item"><a class="flm-breadcrumb-link" href="#">Home</a></li>
  <li class="flm-breadcrumb-item"><a class="flm-breadcrumb-link" href="#">Library</a></li>
  <li class="flm-breadcrumb-item"><a class="flm-breadcrumb-link">Current Page</a></li>
</ol>
```

| Class | Effect |
|-------|--------|
| `flm-breadcrumb` | `<ol>` container |
| `flm-breadcrumb--small` | Smaller font |
| `flm-breadcrumb--large` | Larger font + semibold |
| `flm-breadcrumb-item` | `<li>` wrapper |
| `flm-breadcrumb-link` | Clickable text |
| `flm-breadcrumb-overflow` | "..." overflow button |

---

## Pivot

```html
<div class="flm-pivot">
  <div class="flm-pivot-tabs" role="tablist">
    <button class="flm-pivot-tab flm-pivot-tab--active" data-panel="p1" role="tab">Tab 1</button>
    <button class="flm-pivot-tab" data-panel="p2" role="tab">Tab 2</button>
    <button class="flm-pivot-tab flm-pivot-tab--disabled" role="tab">Disabled</button>
  </div>
  <div class="flm-pivot-panel flm-pivot-panel--active" id="p1" role="tabpanel">Content 1</div>
  <div class="flm-pivot-panel" id="p2" role="tabpanel">Content 2</div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-pivot` | Container |
| `flm-pivot--links` | Link-style tabs (wider padding) |
| `flm-pivot--tabs` | Filled tab style |
| `flm-pivot-tabs` | Tab bar |
| `flm-pivot-tab` | Individual tab button |
| `flm-pivot-tab--active` | Selected tab |
| `flm-pivot-tab--disabled` | Disabled tab |
| `flm-pivot-panel` | Content panel |
| `flm-pivot-panel--active` | Visible panel |

| Attribute | Effect |
|-----------|--------|
| `data-panel="id"` | Links tab to panel by id |

---

## DetailsList

```html
<div class="flm-detailslist">
  <div class="flm-detailslist-header">
    <div class="flm-detailslist-check"><label class="flm-checkbox"><input type="checkbox" class="flm-checkbox-input"><span class="flm-checkbox-mark"></span></label></div>
    <div class="flm-detailslist-header-cell" style="width: 200px;">Name</div>
    <div class="flm-detailslist-header-cell" style="width: 120px;">Status</div>
    <div class="flm-detailslist-header-cell" style="flex: 1;">Date</div>
  </div>
  <div class="flm-detailslist-row">
    <div class="flm-detailslist-check"><label class="flm-checkbox"><input type="checkbox" class="flm-checkbox-input"><span class="flm-checkbox-mark"></span></label></div>
    <div class="flm-detailslist-cell" style="width: 200px;">Document.pdf</div>
    <div class="flm-detailslist-cell" style="width: 120px;">Active</div>
    <div class="flm-detailslist-cell" style="flex: 1;">Feb 20, 2026</div>
  </div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-detailslist` | Table container |
| `flm-detailslist--compact` | Reduced row height (32px) |
| `flm-detailslist-header` | Header row |
| `flm-detailslist-header-cell` | Column header |
| `flm-detailslist-row` | Data row |
| `flm-detailslist-row--selected` | Selected row highlight |
| `flm-detailslist-cell` | Data cell |
| `flm-detailslist-check` | Row checkbox column |
| `flm-detailslist-empty` | Empty state message |

Set column widths via inline `style="width: 200px"` or `style="flex: 1"`.

---

## SearchBox

```html
<div class="flm-searchbox">
  <input class="flm-searchbox-input" type="search" placeholder="Search…">
</div>
```

| Class | Effect |
|-------|--------|
| `flm-searchbox` | Container (JS injects icon + clear btn) |
| `flm-searchbox--underlined` | Bottom border only |
| `flm-searchbox--disabled` | Disabled state |

| Child class | Element |
|-------------|---------|
| `flm-searchbox-input` | `<input type="search">` |
| `flm-searchbox-icon` | Search icon (auto-injected) |
| `flm-searchbox-clear` | Clear button (auto-injected) |

---

## ContextualMenu

```html
<button class="flm-button" data-contextmenu-toggle="menu1">Menu</button>
<div class="flm-contextmenu" id="menu1">
  <div class="flm-contextmenu-header">Actions</div>
  <button class="flm-contextmenu-item"><span class="flm-contextmenu-item-text">Edit</span></button>
  <button class="flm-contextmenu-item"><span class="flm-contextmenu-item-text">Copy</span></button>
  <div class="flm-contextmenu-divider"></div>
  <button class="flm-contextmenu-item" disabled><span class="flm-contextmenu-item-text">Delete</span></button>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-contextmenu` | Menu container (give it an `id`) |
| `flm-contextmenu-item` | Menu item button |
| `flm-contextmenu-item--checked` | Shows checkmark |
| `flm-contextmenu-divider` | Horizontal rule |
| `flm-contextmenu-header` | Section header |

| Attribute | Effect |
|-----------|--------|
| `data-contextmenu-toggle="id"` | Click trigger |
| `data-contextmenu="id"` | Right-click trigger |
| `disabled` | Disabled item |

| Child class | Element |
|-------------|---------|
| `flm-contextmenu-item-icon` | Leading icon |
| `flm-contextmenu-item-text` | Item label |
| `flm-contextmenu-item-submenu` | Submenu chevron |

---

## ProgressIndicator

```html
<div class="flm-progress">
  <span class="flm-progress-label">Uploading…</span>
  <div class="flm-progress-track"><div class="flm-progress-bar" style="width: 60%"></div></div>
  <span class="flm-progress-description">60% complete</span>
</div>

<div class="flm-progress flm-progress--indeterminate">
  <div class="flm-progress-track"><div class="flm-progress-bar"></div></div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-progress` | Container |
| `flm-progress--indeterminate` | Animated looping bar |

| Child class | Element |
|-------------|---------|
| `flm-progress-label` | Title text |
| `flm-progress-track` | Background rail |
| `flm-progress-bar` | Filled portion (set `width` via style) |
| `flm-progress-description` | Subtitle text |

---

## Tooltip

```html
<span data-tooltip="Helpful info">Hover me</span>
```

| Class | Effect |
|-------|--------|
| `flm-tooltip-host` | Explicit wrapper (optional — `data-tooltip` on any element works) |
| `flm-tooltip` | Tooltip box (for pre-defined rich tooltips) |

| Attribute | Effect |
|-----------|--------|
| `data-tooltip="text"` | Simple text tooltip (auto-created by JS) |
| `data-tooltip-id="id"` | References a pre-defined `.flm-tooltip` element |

---

## Persona

```html
<div class="flm-persona flm-persona--size40">
  <div class="flm-persona-coin flm-persona-presence flm-persona-presence--available">JD</div>
  <div class="flm-persona-details">
    <span class="flm-persona-name">Jane Doe</span>
    <span class="flm-persona-secondary">Software Engineer</span>
  </div>
</div>
```

| Class | Size |
|-------|------|
| `flm-persona--size24` | 24px (default) |
| `flm-persona--size32` | 32px |
| `flm-persona--size40` | 40px |
| `flm-persona--size48` | 48px |
| `flm-persona--size56` | 56px |
| `flm-persona--size72` | 72px |
| `flm-persona--size100` | 100px |
| `flm-persona--size120` | 120px |

| Presence class | Status |
|----------------|--------|
| `flm-persona-presence--available` | Green |
| `flm-persona-presence--away` | Yellow |
| `flm-persona-presence--busy` | Red |
| `flm-persona-presence--dnd` | Red |
| `flm-persona-presence--offline` | Gray |

| Child class | Element |
|-------------|---------|
| `flm-persona-coin` | Avatar circle (initials or `<img>`) |
| `flm-persona-details` | Text wrapper |
| `flm-persona-name` | Primary name |
| `flm-persona-secondary` | Title / role |
| `flm-persona-tertiary` | Additional info |

---

## Image

```html
<img class="flm-image" src="photo.jpg" alt="Description">
<img class="flm-image flm-image--cover flm-image--round" src="avatar.jpg" alt="User" style="width:48px;height:48px">
```

| Class | Effect |
|-------|--------|
| `flm-image` | Base image |
| `flm-image--cover` | `object-fit: cover` |
| `flm-image--contain` | `object-fit: contain` |
| `flm-image--center` | Centered, no scaling |
| `flm-image--border` | Border + rounded corners |
| `flm-image--round` | Circular |
| `flm-image--maximize` | Fill parent 100% |

---

## ChoiceGroup

```html
<div class="flm-choicegroup">
  <label class="flm-choicegroup-option">
    <input type="radio" class="flm-choicegroup-input" name="g1" value="a" checked>
    <span class="flm-choicegroup-mark"></span>
    <span class="flm-choicegroup-label">Option A</span>
  </label>
  <label class="flm-choicegroup-option">
    <input type="radio" class="flm-choicegroup-input" name="g1" value="b">
    <span class="flm-choicegroup-mark"></span>
    <span class="flm-choicegroup-label">Option B</span>
  </label>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-choicegroup` | Radio group container |
| `flm-choicegroup--horizontal` | Row layout |
| `flm-choicegroup-option` | Individual radio (is a `<label>`) |
| `flm-choicegroup-option--disabled` | Dimmed, pair with `disabled` on input |

| Child class | Element |
|-------------|---------|
| `flm-choicegroup-input` | Hidden native `<input type="radio">` |
| `flm-choicegroup-mark` | Visual radio circle |
| `flm-choicegroup-label` | Text label |

Native: `checked`, `disabled`.

---

## Shimmer

```html
<div class="flm-shimmer">
  <div class="flm-shimmer-line"></div>
  <div class="flm-shimmer-line flm-shimmer-line--medium"></div>
  <div class="flm-shimmer-line flm-shimmer-line--short"></div>
</div>

<div class="flm-shimmer flm-shimmer--row">
  <div class="flm-shimmer-circle"></div>
  <div class="flm-shimmer" style="flex:1">
    <div class="flm-shimmer-line"></div>
    <div class="flm-shimmer-line flm-shimmer-line--medium"></div>
  </div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-shimmer` | Container (vertical) |
| `flm-shimmer--row` | Horizontal composition |

| Element class | Shape |
|---------------|-------|
| `flm-shimmer-line` | 16px bar (full width) |
| `flm-shimmer-line--short` | 40% width |
| `flm-shimmer-line--medium` | 70% width |
| `flm-shimmer-circle` | 40px circle |
| `flm-shimmer-circle--small` | 24px circle |
| `flm-shimmer-circle--large` | 56px circle |
| `flm-shimmer-rect` | 120×120px block |
| `flm-shimmer-rect--wide` | 200px wide |
| `flm-shimmer-rect--tall` | 200px tall |

---

## List

```html
<div class="flm-list flm-list--bordered flm-list--scrollable">
  <div class="flm-list-item flm-list-item--selected">Selected</div>
  <div class="flm-list-item">
    <div class="flm-list-item-content">
      <span class="flm-list-item-primary">Title</span>
      <span class="flm-list-item-secondary">Subtitle</span>
    </div>
  </div>
  <div class="flm-list-item flm-list-item--disabled">Disabled</div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-list` | List container |
| `flm-list--bordered` | Border + dividers |
| `flm-list--scrollable` | Scrollable (300px max-height) |
| `flm-list--compact` | Reduced row height |
| `flm-list-item` | Row (36px min-height) |
| `flm-list-item--selected` | Checked background |
| `flm-list-item--disabled` | Dimmed, no pointer events |

| Child class | Element |
|-------------|---------|
| `flm-list-item-icon` | Leading icon |
| `flm-list-item-content` | Wrapper for two-line layout |
| `flm-list-item-primary` | Primary text |
| `flm-list-item-secondary` | Secondary text |

---

## GroupedList

```html
<div class="flm-groupedlist">
  <div class="flm-groupedlist-group">
    <button class="flm-groupedlist-header">
      <i class="flm-groupedlist-chevron flm-groupedlist-chevron--expanded" data-icon="ChevronRight"></i>
      Group Name <span class="flm-groupedlist-count">(3)</span>
    </button>
    <div class="flm-groupedlist-items">
      <div class="flm-groupedlist-item">Item 1</div>
      <div class="flm-groupedlist-item">Item 2</div>
    </div>
  </div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-groupedlist` | Container |
| `flm-groupedlist-group` | Section group |
| `flm-groupedlist-header` | Collapsible group title |
| `flm-groupedlist-items` | Items container |
| `flm-groupedlist-items--collapsed` | Hidden (toggled by JS) |
| `flm-groupedlist-item` | Individual item |
| `flm-groupedlist-item--selected` | Selected item |
| `flm-groupedlist-item--disabled` | Disabled item |
| `flm-groupedlist-chevron` | Expand/collapse icon |
| `flm-groupedlist-chevron--expanded` | Rotated 90deg |
| `flm-groupedlist-count` | Item count badge |

---

## Rating

```html
<div class="flm-rating">
  <input type="radio" class="flm-rating-input" name="r" value="5"><label class="flm-rating-star"></label>
  <input type="radio" class="flm-rating-input" name="r" value="4"><label class="flm-rating-star"></label>
  <input type="radio" class="flm-rating-input" name="r" value="3" checked><label class="flm-rating-star"></label>
  <input type="radio" class="flm-rating-input" name="r" value="2"><label class="flm-rating-star"></label>
  <input type="radio" class="flm-rating-input" name="r" value="1"><label class="flm-rating-star"></label>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-rating` | Container (radio inputs ordered 5→1) |
| `flm-rating--readonly` | Non-interactive, use `--filled`/`--empty` on stars |
| `flm-rating--disabled` | Dimmed, no interaction |
| `flm-rating--large` | Larger stars (32px) |

| Child class | Element |
|-------------|---------|
| `flm-rating-input` | Hidden `<input type="radio">` |
| `flm-rating-star` | Star label (clip-path shape) |
| `flm-rating-star--filled` | Filled star (readonly mode) |
| `flm-rating-star--empty` | Empty star (readonly mode) |

JS stores `data-rating-value` on root.

---

## Facepile

```html
<div class="flm-facepile" data-max="3">
  <span class="flm-facepile-member"><span class="flm-persona-coin">JD</span></span>
  <span class="flm-facepile-member"><span class="flm-persona-coin">AB</span></span>
  <span class="flm-facepile-member"><span class="flm-persona-coin">MK</span></span>
  <span class="flm-facepile-member"><span class="flm-persona-coin">TW</span></span>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-facepile` | Container |
| `flm-facepile-member` | Overlapping coin wrapper |
| `flm-facepile-overflow` | +N chip (auto-injected by JS) |
| `flm-facepile-add` | Add person button |

| Attribute | Effect |
|-----------|--------|
| `data-max="N"` | JS hides excess members, shows +N overflow |

---

## SwatchColorPicker

```html
<div class="flm-swatchcolorpicker">
  <button class="flm-swatchcolorpicker-cell" style="background-color: #ff1744;" data-color="#ff1744"></button>
  <button class="flm-swatchcolorpicker-cell flm-swatchcolorpicker-cell--selected" style="background-color: #2979ff;" data-color="#2979ff"></button>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-swatchcolorpicker` | Grid container |
| `flm-swatchcolorpicker--circle` | Circular swatches |
| `flm-swatchcolorpicker--large` | 32px swatches |
| `flm-swatchcolorpicker-cell` | Color button (set `background-color` inline) |
| `flm-swatchcolorpicker-cell--selected` | Selection ring |
| `flm-swatchcolorpicker-cell--disabled` | Dimmed |

| Attribute | Effect |
|-----------|--------|
| `data-color="hex"` | Color value for JS selection tracking |

JS sets `data-selected` and `data-selected-color` on root.

---

## DocumentCard

```html
<div class="flm-documentcard">
  <div class="flm-documentcard-preview">
    <div class="flm-documentcard-preview-icon"><i class="flm-icon flm-icon--large" data-icon="Document"></i></div>
  </div>
  <div class="flm-documentcard-details">
    <span class="flm-documentcard-title">Report.docx</span>
  </div>
  <div class="flm-documentcard-activity">
    <span class="flm-persona-coin" style="width:24px;height:24px;font-size:10px;">JD</span>
    <div>
      <span class="flm-documentcard-activity-name">Jane Doe</span>
      <span class="flm-documentcard-activity-date">Feb 20</span>
    </div>
  </div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-documentcard` | Card container (212px) |
| `flm-documentcard--compact` | Horizontal layout |

| Child class | Element |
|-------------|---------|
| `flm-documentcard-preview` | Image/icon area |
| `flm-documentcard-preview-icon` | Centered icon |
| `flm-documentcard-details` | Title/text area |
| `flm-documentcard-title` | Document name |
| `flm-documentcard-activity` | Persona + timestamp row |
| `flm-documentcard-activity-name` | Author name |
| `flm-documentcard-activity-date` | Timestamp |
| `flm-documentcard-actions` | Action buttons row |

---

## SpinButton

```html
<div class="flm-spinbutton-wrapper">
  <label class="flm-label" for="sb1">Quantity</label>
  <div class="flm-spinbutton">
    <button class="flm-spinbutton-btn flm-spinbutton-btn--decrement" data-icon="ChevronDown" aria-label="Decrease"></button>
    <input class="flm-spinbutton-input" id="sb1" type="number" value="5" min="0" max="100" step="1">
    <button class="flm-spinbutton-btn flm-spinbutton-btn--increment" data-icon="ChevronUp" aria-label="Increase"></button>
  </div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-spinbutton` | Input + buttons container |
| `flm-spinbutton--disabled` | Disabled state |
| `flm-spinbutton-wrapper` | Outer wrapper with label |

| Child class | Element |
|-------------|---------|
| `flm-spinbutton-input` | `<input type="number">` (native spinners hidden) |
| `flm-spinbutton-btn` | Inc/dec button |
| `flm-spinbutton-btn--decrement` | Decrement button |
| `flm-spinbutton-btn--increment` | Increment button |

JS wires buttons to `stepUp()`/`stepDown()`.

---

## Slider

```html
<div class="flm-slider">
  <div class="flm-slider-label">
    <label class="flm-label" for="sl1">Volume</label>
  </div>
  <div class="flm-slider-container">
    <input class="flm-slider-input" id="sl1" type="range" min="0" max="100" value="60">
    <span class="flm-slider-value">60</span>
  </div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-slider` | Container |
| `flm-slider--disabled` | Disabled state |
| `flm-slider--vertical` | Vertical orientation |

| Child class | Element |
|-------------|---------|
| `flm-slider-container` | Flex wrapper for input + value |
| `flm-slider-input` | `<input type="range">` |
| `flm-slider-value` | Live value display |
| `flm-slider-label` | Label row |

JS sets `--flm-slider-fill` CSS var for track gradient.

---

## ComboBox

```html
<div class="flm-combobox">
  <label class="flm-label">Fruit</label>
  <div class="flm-combobox-wrapper">
    <input class="flm-combobox-input" placeholder="Select…">
    <button class="flm-combobox-caret" data-icon="ChevronDown" aria-label="Toggle"></button>
  </div>
  <div class="flm-combobox-listbox">
    <div class="flm-combobox-option" data-value="apple">Apple</div>
    <div class="flm-combobox-option" data-value="banana">Banana</div>
  </div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-combobox` | Container |
| `flm-combobox--disabled` | Disabled state |
| `flm-combobox--error` | Error border |

| Attribute | Effect |
|-----------|--------|
| `data-multiselect` | On `.flm-combobox` — enables multi-select mode |
| `data-value="val"` | On `.flm-combobox-option` — option value |

| Child class | Element |
|-------------|---------|
| `flm-combobox-wrapper` | Input + caret row |
| `flm-combobox-input` | Text input |
| `flm-combobox-caret` | Dropdown toggle button |
| `flm-combobox-listbox` | Options dropdown |
| `flm-combobox-option` | Individual option |
| `flm-combobox-option--selected` | Selected option |
| `flm-combobox-option--disabled` | Disabled option |
| `flm-combobox-option-check` | Checkbox (multi-select mode) |
| `flm-combobox-error` | Error message |

JS: filtering, keyboard nav (ArrowUp/Down/Enter/Escape), click-outside dismiss, flip-above.

---

## TeachingBubble

```html
<button data-teachingbubble-toggle="tb1">Learn more</button>
<div class="flm-teachingbubble" id="tb1">
  <div class="flm-teachingbubble-beak"></div>
  <div class="flm-teachingbubble-header">
    <h3 class="flm-teachingbubble-headline">Welcome!</h3>
    <button class="flm-teachingbubble-close" data-icon="Cancel" aria-label="Close"></button>
  </div>
  <div class="flm-teachingbubble-body">Guidance text here.</div>
  <div class="flm-teachingbubble-footer">
    <button class="flm-button flm-button--primary">Got it!</button>
  </div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-teachingbubble` | Inverted-color callout (themePrimary bg) |

| Attribute | Effect |
|-----------|--------|
| `data-teachingbubble-toggle="id"` | Toggle trigger |

| Child class | Element |
|-------------|---------|
| `flm-teachingbubble-beak` | Arrow pointing to target |
| `flm-teachingbubble-header` | Header row |
| `flm-teachingbubble-headline` | Title |
| `flm-teachingbubble-close` | Close button |
| `flm-teachingbubble-body` | Content text |
| `flm-teachingbubble-footer` | Action buttons |

Button colors automatically inverted inside `.flm-teachingbubble`.

---

## HoverCard

```html
<span data-hovercard-id="hc1" style="cursor:pointer">Hover me</span>
<div class="flm-hovercard" id="hc1">
  <div class="flm-hovercard-compact">
    <div class="flm-hovercard-title">Jane Doe</div>
    <div class="flm-hovercard-subtitle">Engineer</div>
  </div>
  <div class="flm-hovercard-expanded">
    <div class="flm-hovercard-body">Extended details…</div>
  </div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-hovercard` | Positioned card |

| Attribute | Effect |
|-----------|--------|
| `data-hovercard-id="id"` | On host — links to hovercard |

| Child class | Element |
|-------------|---------|
| `flm-hovercard-compact` | First phase (shows after 500ms) |
| `flm-hovercard-expanded` | Second phase (shows after 1500ms total) |
| `flm-hovercard-title` | Title |
| `flm-hovercard-subtitle` | Subtitle |
| `flm-hovercard-body` | Body text |

Card stays open while mouse is over it.

---

## Coachmark

```html
<div class="flm-coachmark" data-teachingbubble-toggle="tb1">
  <div class="flm-coachmark-dot"></div>
  <div class="flm-coachmark-ring"></div>
</div>
<div class="flm-teachingbubble" id="tb1">…</div>
```

| Class | Effect |
|-------|--------|
| `flm-coachmark` | Pulsing beacon |
| `flm-coachmark--hidden` | Hidden (auto-set when bubble dismissed) |
| `flm-coachmark--dark` | Dark theme variant |

| Attribute | Effect |
|-----------|--------|
| `data-teachingbubble-toggle="id"` | Click opens linked TeachingBubble |

| Child class | Element |
|-------------|---------|
| `flm-coachmark-dot` | Central dot |
| `flm-coachmark-ring` | Pulsing ring animation |

JS uses MutationObserver to auto-hide beacon when bubble is dismissed.

---

## DatePicker

```html
<div class="flm-datepicker" data-min-date="01/01/2026" data-max-date="12/31/2026">
  <label class="flm-label" for="dp1">Date</label>
  <div class="flm-datepicker-wrapper">
    <input class="flm-datepicker-input" id="dp1" placeholder="MM/DD/YYYY">
    <button class="flm-datepicker-icon" data-icon="Calendar" aria-label="Open calendar"></button>
  </div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-datepicker` | Container |
| `flm-datepicker--disabled` | Disabled state |
| `flm-datepicker--error` | Error border |

| Attribute | Effect |
|-----------|--------|
| `data-min-date="MM/DD/YYYY"` | Minimum selectable date |
| `data-max-date="MM/DD/YYYY"` | Maximum selectable date |

| Child class | Element |
|-------------|---------|
| `flm-datepicker-wrapper` | Input + icon row |
| `flm-datepicker-input` | Text input |
| `flm-datepicker-icon` | Calendar toggle button |
| `flm-datepicker-error` | Error message |

JS generates 42-cell calendar grid on open. Month nav via prev/next buttons. Day click formats as MM/DD/YYYY.

---

## TagPicker

```html
<div class="flm-tagpicker">
  <label class="flm-label">Tags</label>
  <div class="flm-tagpicker-well">
    <input class="flm-tagpicker-input" placeholder="Add tags…">
  </div>
  <div class="flm-tagpicker-listbox">
    <div class="flm-tagpicker-option" data-value="react">React</div>
    <div class="flm-tagpicker-option" data-value="vue">Vue</div>
  </div>
</div>
```

PeoplePicker variant:

```html
<div class="flm-tagpicker flm-tagpicker--people">
  <label class="flm-label">People</label>
  <div class="flm-tagpicker-well">
    <input class="flm-tagpicker-input" placeholder="Search people…">
  </div>
  <div class="flm-tagpicker-listbox">
    <div class="flm-tagpicker-option" data-value="jane" data-initials="JD" data-secondary="Engineer">
      <span class="flm-tagpicker-option-coin">JD</span>
      <span class="flm-tagpicker-option-details">
        <span class="flm-tagpicker-option-name">Jane Doe</span>
        <span class="flm-tagpicker-option-secondary">Engineer</span>
      </span>
    </div>
  </div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-tagpicker` | Container |
| `flm-tagpicker--people` | PeoplePicker variant (shows persona coins) |
| `flm-tagpicker--disabled` | Disabled state |
| `flm-tagpicker--error` | Error border |

| Attribute | Effect |
|-----------|--------|
| `data-max-tags="N"` | Limits number of selected tags |
| `data-value="val"` | On `.flm-tagpicker-option` — option value |
| `data-initials="XX"` | On option — initials for persona coin (people variant) |
| `data-secondary="text"` | On option — secondary text (people variant) |
| `data-selected-values` | Auto-maintained CSV of selected values on root |

| Child class | Element |
|-------------|---------|
| `flm-tagpicker-well` | Input area holding chips + input |
| `flm-tagpicker-input` | Text input |
| `flm-tagpicker-chip` | Selected tag pill |
| `flm-tagpicker-chip-text` | Chip label text |
| `flm-tagpicker-chip-remove` | Chip remove (X) button |
| `flm-tagpicker-chip-coin` | Small persona coin in chip (people variant) |
| `flm-tagpicker-listbox` | Options dropdown |
| `flm-tagpicker-option` | Individual option |
| `flm-tagpicker-option--selected` | Already-picked option (grayed out) |
| `flm-tagpicker-option-coin` | Persona coin in option (people variant) |
| `flm-tagpicker-option-details` | Name + secondary text wrapper |
| `flm-tagpicker-option-name` | Primary name |
| `flm-tagpicker-option-secondary` | Secondary text |
| `flm-tagpicker-error` | Error message |

JS: type-to-filter, keyboard nav (ArrowUp/Down/Enter/Escape), Backspace removes last chip, click X removes chip, click-outside dismiss, flip-above.

---

## OverflowSet

```html
<div class="flm-overflowset">
  <div class="flm-overflowset-items">
    <button class="flm-overflowset-item" data-label="New" data-icon="Add">New</button>
    <button class="flm-overflowset-item" data-label="Edit" data-icon="Edit">Edit</button>
  </div>
  <button class="flm-overflowset-overflow" aria-label="More items">...</button>
  <div class="flm-overflowset-far">
    <button class="flm-overflowset-item">Settings</button>
  </div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-overflowset` | Container (flex row, overflow hidden) |
| `flm-overflowset--has-overflow` | Auto-set when items overflow (shows "..." button) |
| `flm-overflowset-items` | Flex row holding the items |
| `flm-overflowset-item` | Individual item (`flex-shrink: 0`) |
| `flm-overflowset-item--hidden` | Hidden by JS (overflowed) |
| `flm-overflowset-overflow` | "..." overflow button |
| `flm-overflowset-far` | Right-side items (never overflow, `margin-left: auto`) |

| Attribute | Effect |
|-----------|--------|
| `data-overflow-order="N"` | Higher N overflows first (priority control) |
| `data-label="text"` | Label shown in overflow menu |
| `data-icon="Name"` | Icon shown in overflow menu |

JS: ResizeObserver measures available width, hides items that don't fit, builds a ContextMenu for hidden items. Clicking a menu item proxies the original item's click.

---

## TimePicker

```html
<div class="flm-timepicker" data-increment="30" data-use-12h data-min-time="09:00" data-max-time="17:00">
  <label class="flm-label" for="tp1">Time</label>
  <div class="flm-timepicker-wrapper">
    <input class="flm-timepicker-input" id="tp1" placeholder="Select a time…">
    <button class="flm-timepicker-icon" data-icon="Clock" aria-label="Open time picker"></button>
  </div>
</div>
```

| Class | Effect |
|-------|--------|
| `flm-timepicker` | Container |
| `flm-timepicker--disabled` | Disabled state |
| `flm-timepicker--error` | Error border |

| Attribute | Effect |
|-----------|--------|
| `data-increment="30"` | Minute increment (default 30) |
| `data-use-12h` | 12-hour format with AM/PM (default 24h) |
| `data-min-time="HH:MM"` | Earliest available time (24h format) |
| `data-max-time="HH:MM"` | Latest available time (24h format) |

| Child class | Element |
|-------------|---------|
| `flm-timepicker-wrapper` | Input + icon row |
| `flm-timepicker-input` | Text input (typeable, filters options) |
| `flm-timepicker-icon` | Clock toggle button |
| `flm-timepicker-error` | Error message |

JS generates time-slot options at the configured increment. Input text filters the list. Arrow keys navigate, Enter selects, Escape closes. Auto-scrolls to selected or nearest-to-current-time option on open.
