# Loupe Frontend Style Guide

This guide documents the design system, component library, and patterns used in the Loupe frontend. Follow these conventions to ensure consistency across the application.

---

## Table of Contents

1. [Tech Stack](#tech-stack)
2. [Color System](#color-system)
3. [Typography](#typography)
4. [Spacing & Layout](#spacing--layout)
5. [Components](#components)
6. [Icons](#icons)
7. [Patterns & Best Practices](#patterns--best-practices)

---

## Tech Stack

| Tool            | Version | Purpose          |
| --------------- | ------- | ---------------- |
| Vue             | 3.5+    | Framework        |
| TypeScript      | 5.x     | Type safety      |
| Vite            | latest  | Build tool       |
| TailwindCSS     | 4.x     | Styling          |
| Pinia           | 3.x     | State management |
| Vue Router      | 4.x     | Routing          |
| Lucide Vue Next | latest  | Icons            |

---

## Color System

We use CSS custom properties with OKLCH color space for better perceptual uniformity. All colors are defined in `src/styles/main.css` and exposed as Tailwind utilities.

### Primary Colors (Indigo)

Use for primary actions, links, and focus states.

```html
<div class="bg-primary-600 text-white">Primary button</div>
<div class="text-primary-600">Primary link</div>
<div class="border-primary-500">Focused input</div>
```

| Class                         | Usage                           |
| ----------------------------- | ------------------------------- |
| `primary-50` - `primary-200`  | Light backgrounds, hover states |
| `primary-500` - `primary-600` | Primary actions, links          |
| `primary-700` - `primary-900` | Dark mode accents               |

### Semantic Surface Colors

```html
<div class="bg-surface">Page background</div>
<div class="bg-surface-raised">Cards, elevated content</div>
<div class="bg-surface-sunken">Input backgrounds, hover states</div>
<div class="bg-surface-overlay">Modals, popovers</div>
```

### Text Colors

```html
<p class="text-text">Primary body text</p>
<p class="text-text-muted">Secondary text, labels</p>
<p class="text-text-subtle">Placeholder, disabled text</p>
<p class="text-text-inverted">Text on dark backgrounds</p>
```

### Border Colors

```html
<div class="border border-border">Default border</div>
<div class="border border-border-strong">Emphasized border</div>
<div class="border border-border-muted">Subtle border</div>
```

### Status Colors

For alerts, badges, and status indicators:

```html
<!-- Success -->
<div class="bg-success-muted text-success">Success message</div>

<!-- Warning -->
<div class="bg-warning-muted text-warning">Warning message</div>

<!-- Error -->
<div class="bg-error-muted text-error">Error message</div>

<!-- Info -->
<div class="bg-info-muted text-info">Info message</div>
```

### Chart Colors

For data visualizations, use the chart palette:

```css
--color-chart-1  /* Primary (indigo) */
--color-chart-2  /* Teal */
--color-chart-3  /* Amber */
--color-chart-4  /* Pink */
--color-chart-5  /* Red */
--color-chart-6  /* Cyan */
```

---

## Typography

We use the Inter font family with system fallbacks.

### Font Sizes

```html
<p class="text-xs">12px - Badges, captions</p>
<p class="text-sm">14px - Body text, inputs</p>
<p class="text-base">16px - Large body text</p>
<p class="text-lg">18px - Section headers</p>
<p class="text-xl">20px - Page titles</p>
<p class="text-2xl">24px - Hero text</p>
```

### Font Weights

```html
<p class="font-normal">Regular - Body text</p>
<p class="font-medium">Medium - Labels, buttons</p>
<p class="font-semibold">Semibold - Headings</p>
```

### Text Patterns

```html
<!-- Page title -->
<h1 class="text-2xl font-semibold text-text">Dashboard</h1>

<!-- Section header -->
<h2 class="text-lg font-semibold text-text">Settings</h2>

<!-- Card header -->
<h3 class="text-base font-medium text-text">Account Details</h3>

<!-- Label -->
<label class="text-sm font-medium text-text">Email</label>

<!-- Helper text -->
<p class="text-sm text-text-muted">Enter your email address</p>

<!-- Caption -->
<span class="text-xs text-text-subtle">Last updated 2 hours ago</span>
```

---

## Spacing & Layout

### Spacing Scale

Use Tailwind's default spacing scale (0.25rem increments):

| Class          | Value | Usage            |
| -------------- | ----- | ---------------- |
| `gap-1`, `p-1` | 4px   | Tight spacing    |
| `gap-2`, `p-2` | 8px   | Compact elements |
| `gap-3`, `p-3` | 12px  | Default spacing  |
| `gap-4`, `p-4` | 16px  | Card padding     |
| `gap-6`, `p-6` | 24px  | Section spacing  |
| `gap-8`, `p-8` | 32px  | Page sections    |

### Border Radius

```html
<div class="rounded-sm">4px - Small elements</div>
<div class="rounded-md">6px - Buttons, inputs</div>
<div class="rounded-lg">8px - Cards, modals</div>
<div class="rounded-xl">12px - Large containers</div>
<div class="rounded-full">Circular - Avatars, badges</div>
```

### Shadows

```html
<div class="shadow-sm">Subtle elevation</div>
<div class="shadow-md">Cards, dropdowns</div>
<div class="shadow-lg">Modals, popovers</div>
<div class="shadow-xl">Prominent dialogs</div>
```

### Layout Constants

```css
--loupe-sidebar-width: 16rem;  /* 256px */
--loupe-header-height: 4rem;   /* 64px */
```

---

## Components

All components are prefixed with `L` (for Loupe) and located in `src/components/ui/`.

### LButton

```vue
<script setup>
import { LButton } from '@/components/ui'
</script>

<!-- Variants -->
<LButton variant="primary">Primary</LButton>
<LButton variant="secondary">Secondary</LButton>
<LButton variant="ghost">Ghost</LButton>
<LButton variant="outline">Outline</LButton>
<LButton variant="danger">Danger</LButton>

<!-- Sizes -->
<LButton size="sm">Small (h-8)</LButton>
<LButton size="md">Medium (h-10)</LButton>
<LButton size="lg">Large (h-12)</LButton>

<!-- States -->
<LButton :loading="true">Loading...</LButton>
<LButton :disabled="true">Disabled</LButton>

<!-- With icon -->
<LButton>
  <Plus class="w-4 h-4" />
  Add Item
</LButton>
```

### LInput

```vue
<script setup>
import { LInput } from '@/components/ui'
const email = ref('')
</script>

<LInput v-model="email" type="email" placeholder="Enter email" />
<LInput type="password" placeholder="Password" />
<LInput :error="true" placeholder="Invalid input" />
<LInput :disabled="true" value="Disabled" />
```

### LTextarea

```vue
<LTextarea v-model="description" placeholder="Enter description" :rows="4" />
<LTextarea :error="true" placeholder="Invalid" />
```

### LSelect

```vue
<script setup>
import { LSelect } from '@/components/ui'

const options = [
  { value: 'opt1', label: 'Option 1' },
  { value: 'opt2', label: 'Option 2' },
]
const selected = ref('opt1')
</script>

<LSelect v-model="selected" :options="options" placeholder="Select..." />
```

### LCheckbox

```vue
<script setup>
import { LCheckbox } from '@/components/ui'
const checked = ref(false)
</script>

<label class="flex items-center gap-2 cursor-pointer">
  <LCheckbox v-model="checked" />
  <span class="text-sm text-text">Enable feature</span>
</label>
```

### LBadge

```vue
<script setup>
import { LBadge } from '@/components/ui'
</script>

<LBadge>Default</LBadge>
<LBadge variant="success">Success</LBadge>
<LBadge variant="warning">Warning</LBadge>
<LBadge variant="error">Error</LBadge>
<LBadge variant="info">Info</LBadge>

<!-- Sizes -->
<LBadge size="sm">Small</LBadge>
<LBadge size="md">Medium</LBadge>
```

### LCard

```vue
<script setup>
import { LCard } from '@/components/ui'
</script>

<LCard>
  <h3 class="text-lg font-semibold text-text mb-4">Card Title</h3>
  <p class="text-text-muted">Card content</p>
</LCard>

<!-- Padding variants -->
<LCard padding="none">No padding</LCard>
<LCard padding="sm">Small (p-3)</LCard>
<LCard padding="md">Medium (p-4, default)</LCard>
<LCard padding="lg">Large (p-6)</LCard>
```

### LModal

```vue
<script setup>
import { LModal, LButton } from '@/components/ui'
const open = ref(false)
</script>

<LButton @click="open = true">Open Modal</LButton>

<LModal v-model="open" title="Modal Title" size="md">
  <p>Modal content goes here</p>
  
  <template #footer>
    <LButton variant="secondary" @click="open = false">Cancel</LButton>
    <LButton @click="save">Save</LButton>
  </template>
</LModal>

<!-- Sizes: sm (max-w-sm), md (max-w-lg), lg (max-w-2xl), xl (max-w-4xl) -->
```

### LAlert

```vue
<script setup>
import { LAlert } from '@/components/ui'
</script>

<LAlert variant="info" title="Information">
  This is an informational message.
</LAlert>

<LAlert variant="success" title="Success!">
  Your changes have been saved.
</LAlert>

<LAlert variant="warning">
  Please review before continuing.
</LAlert>

<LAlert variant="error" title="Error" :dismissible="true" @dismiss="hideAlert">
  Something went wrong.
</LAlert>
```

### LSpinner

```vue
<script setup>
import { LSpinner } from '@/components/ui'
</script>

<LSpinner size="sm" />  <!-- 16px -->
<LSpinner size="md" />  <!-- 24px, default -->
<LSpinner size="lg" />  <!-- 32px -->
```

### LTable

```vue
<script setup>
import { LTable } from '@/components/ui'

const columns = [
  { key: 'name', label: 'Name' },
  { key: 'email', label: 'Email' },
  { key: 'role', label: 'Role', align: 'center' },
]
const data = [
  { id: 1, name: 'John', email: 'john@example.com', role: 'Admin' },
]
</script>

<LTable :columns="columns" :data="data" :loading="false">
  <template #cell-role="{ value }">
    <LBadge>{{ value }}</LBadge>
  </template>
</LTable>
```

### LEmptyState

```vue
<script setup>
import { LEmptyState, LButton } from '@/components/ui'
import { Database } from 'lucide-vue-next'
</script>

<LEmptyState
  title="No datasources"
  description="Get started by adding your first datasource."
  :icon="Database"
>
  <LButton>
    <Plus class="w-4 h-4" />
    Add Datasource
  </LButton>
</LEmptyState>
```

### LTooltip

```vue
<script setup>
import { LTooltip } from '@/components/ui'
</script>

<LTooltip content="Helpful information">
  <button>Hover me</button>
</LTooltip>

<!-- Positions: top (default), bottom, left, right -->
<LTooltip content="Bottom tooltip" position="bottom">
  <button>Hover</button>
</LTooltip>
```

---

## Icons

We use [Lucide Vue Next](https://lucide.dev/) for icons. Import icons individually:

```vue
<script setup>
import { 
  Plus, 
  Settings, 
  Search, 
  X, 
  ChevronDown,
  Database,
  BarChart3,
  Table2,
  Calendar,
  Play,
  Trash2,
  Edit,
  Copy,
  Check,
  AlertCircle,
  Info,
  Loader2,
} from 'lucide-vue-next'
</script>

<!-- Standard sizes -->
<Plus class="w-4 h-4" />  <!-- Small: buttons, inline -->
<Plus class="w-5 h-5" />  <!-- Medium: list items -->
<Plus class="w-6 h-6" />  <!-- Large: empty states -->
<Plus class="w-8 h-8" />  <!-- XL: hero sections -->

<!-- With text color -->
<Settings class="w-5 h-5 text-text-muted" />
<AlertCircle class="w-5 h-5 text-error" />
```

### Common Icon Patterns

```vue
<!-- Button with icon -->
<LButton>
  <Plus class="w-4 h-4" />
  Add Item
</LButton>

<!-- Icon-only button -->
<button class="p-2 rounded-md hover:bg-surface-sunken transition-colors">
  <Settings class="w-5 h-5 text-text-muted" />
</button>

<!-- Loading state -->
<Loader2 class="w-4 h-4 animate-spin" />

<!-- Status indicator -->
<CheckCircle class="w-5 h-5 text-success" />
```

---

## Patterns & Best Practices

### Page Structure

```vue
<template>
  <AppLayout title="Page Title">
    <!-- Optional header actions slot -->
    <template #header-actions>
      <LButton>
        <Plus class="w-4 h-4" />
        Add New
      </LButton>
    </template>

    <!-- Page content -->
    <div class="space-y-6">
      <LCard>
        <!-- Card content -->
      </LCard>
    </div>
  </AppLayout>
</template>
```

### Form Layout

```vue
<template>
  <form @submit.prevent="onSubmit" class="space-y-4">
    <div>
      <label class="block text-sm font-medium text-text mb-1.5">
        Field Label
      </label>
      <LInput v-model="form.field" placeholder="Placeholder" />
      <p v-if="errors.field" class="mt-1 text-sm text-error">
        {{ errors.field }}
      </p>
    </div>

    <div class="flex justify-end gap-3 pt-4">
      <LButton variant="secondary" type="button" @click="cancel">
        Cancel
      </LButton>
      <LButton type="submit" :loading="saving">
        Save
      </LButton>
    </div>
  </form>
</template>
```

### List with Empty State

```vue
<template>
  <div>
    <LSpinner v-if="loading" class="mx-auto" />
    
    <LEmptyState
      v-else-if="items.length === 0"
      title="No items"
      description="Create your first item to get started."
      :icon="FolderIcon"
    >
      <LButton @click="openCreateModal">
        <Plus class="w-4 h-4" />
        Create Item
      </LButton>
    </LEmptyState>

    <div v-else class="space-y-4">
      <LCard v-for="item in items" :key="item.id">
        <!-- Item content -->
      </LCard>
    </div>
  </div>
</template>
```

### Info Row Pattern

For displaying key-value pairs:

```vue
<div class="flex items-center gap-3 p-3 bg-surface-alt rounded-lg">
  <Mail class="w-5 h-5 text-text-muted shrink-0" />
  <div class="flex-1 min-w-0">
    <p class="text-xs text-text-muted">Label</p>
    <p class="text-sm text-text truncate">Value</p>
  </div>
</div>
```

### Loading States

```vue
<!-- Button loading -->
<LButton :loading="saving">Save</LButton>

<!-- Full page loading -->
<div class="flex items-center justify-center h-64">
  <LSpinner size="lg" />
</div>

<!-- Inline loading -->
<span class="inline-flex items-center gap-2 text-text-muted">
  <Loader2 class="w-4 h-4 animate-spin" />
  Loading...
</span>
```

### Transitions

Use Vue's `<Transition>` with our animation classes:

```vue
<Transition
  enter-active-class="animate-fade-in"
  leave-active-class="animate-fade-in"
  leave-to-class="opacity-0"
>
  <div v-if="visible">Content</div>
</Transition>
```

Available animations:

- `animate-fade-in`
- `animate-slide-in-right`
- `animate-slide-in-left`
- `animate-slide-in-top`
- `animate-slide-in-bottom`
- `animate-spin`
- `animate-pulse`

### Focus Management

All interactive elements should have visible focus states:

```html
<!-- Use the focus-ring utility -->
<button class="focus-ring rounded-md">Focusable</button>
<input class="focus-ring rounded-md" />
```

### Dark Mode

Dark mode is handled automatically via CSS variables. Toggle with the `dark` class on `<html>`:

```ts
// Toggle dark mode
document.documentElement.classList.toggle('dark')
```

No component changes needed—all colors adapt automatically.

### API Calls

Use the services in `src/services/api/`:

```ts
import { datasourcesApi, queriesApi } from '@/services/api'

// List
const datasources = await datasourcesApi.list()

// Create
const newDs = await datasourcesApi.create({
  name: 'My DB',
  type: 'postgres',
  connection_string: '...'
})

// Delete
await datasourcesApi.delete(id)
```

### Auth State

Access auth state via the Pinia store:

```ts
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore()

// Access user
authStore.user?.name
authStore.user?.email
authStore.user?.org_id

// Check auth
if (authStore.isAuthenticated) {
  // ...
}

// Logout
authStore.logout()
```

---

## File Organization

```text
src/
├── components/
│   ├── layout/          # App shell components
│   │   ├── AppLayout.vue
│   │   ├── AppSidebar.vue
│   │   ├── AppHeader.vue
│   │   └── ThemeToggle.vue
│   └── ui/              # Reusable UI components
│       ├── LButton.vue
│       ├── LInput.vue
│       └── ...
├── services/
│   └── api/             # API client and services
├── stores/              # Pinia stores
├── types/               # TypeScript types
├── views/               # Page components
└── styles/
    └── main.css         # Theme and global styles
```

---

## Quick Reference

### Common Class Combinations

```html
<!-- Card header -->
<h2 class="text-lg font-semibold text-text mb-4">Title</h2>

<!-- Form label -->
<label class="block text-sm font-medium text-text mb-1.5">Label</label>

<!-- Helper text -->
<p class="text-sm text-text-muted">Description</p>

<!-- Error text -->
<p class="text-sm text-error">Error message</p>

<!-- Clickable list item -->
<button class="w-full flex items-center gap-3 p-3 rounded-lg hover:bg-surface-sunken transition-colors text-left">

<!-- Icon button -->
<button class="p-2 rounded-md hover:bg-surface-sunken transition-colors text-text-muted hover:text-text">
```

### Color Cheat Sheet

| Need                    | Class               |
| ----------------------- | ------------------- |
| Page background         | `bg-surface`        |
| Card background         | `bg-surface-raised` |
| Hover/active background | `bg-surface-sunken` |
| Primary text            | `text-text`         |
| Secondary text          | `text-text-muted`   |
| Disabled/placeholder    | `text-text-subtle`  |
| Default border          | `border-border`     |
| Primary action          | `bg-primary-600`    |
| Danger action           | `bg-error`          |
