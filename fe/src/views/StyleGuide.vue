<script setup lang="ts">
import { ref } from 'vue'
import { AppLayout } from '@/components/layout'
import {
  LButton,
  LInput,
  LTextarea,
  LSelect,
  LCheckbox,
  LBadge,
  LCard,
  LModal,
  LSpinner,
  LSkeleton,
  LAlert,
  LTable,
  LPagination,
  LEmptyState,
  LTooltip,
  LToast,
} from '@/components/ui'
import {
  Heart,
  Star,
  AlertCircle,
  CheckCircle,
  Info,
  Trash2,
  Download,
  Upload,
  Plus,
  Settings,
  BarChart3,
} from 'lucide-vue-next'

const showModal = ref(false)
const inputValue = ref('')
const textareaValue = ref('')
const selectValue = ref('option1')
const checkboxValue = ref(false)

// Table demo data
const tableHeaders = ['Name', 'Role', 'Status', 'Email']
const tableRows = [
  ['Alice Johnson', 'Developer', 'Active', 'alice@example.com'],
  ['Bob Smith', 'Designer', 'Active', 'bob@example.com'],
  ['Carol White', 'Manager', 'Away', 'carol@example.com'],
  ['David Brown', 'Developer', 'Active', 'david@example.com'],
  ['Eve Davis', 'Designer', 'Inactive', 'eve@example.com'],
]
const selectedRows = ref<number[]>([])
const sortBy = ref<number | undefined>(undefined)
const sortDirection = ref<'asc' | 'desc' | null>(null)

function handleSort(column: number, direction: 'asc' | 'desc' | null) {
  sortBy.value = direction === null ? undefined : column
  sortDirection.value = direction
}

const selectOptions = [
  { value: 'option1', label: 'Option 1' },
  { value: 'option2', label: 'Option 2' },
  { value: 'option3', label: 'Option 3' },
]

const colors = {
  primary: [
    { name: 'primary-50', var: 'var(--loupe-primary-50)' },
    { name: 'primary-100', var: 'var(--loupe-primary-100)' },
    { name: 'primary-200', var: 'var(--loupe-primary-200)' },
    { name: 'primary-300', var: 'var(--loupe-primary-300)' },
    { name: 'primary-400', var: 'var(--loupe-primary-400)' },
    { name: 'primary-500', var: 'var(--loupe-primary-500)' },
    { name: 'primary-600', var: 'var(--loupe-primary-600)' },
    { name: 'primary-700', var: 'var(--loupe-primary-700)' },
    { name: 'primary-800', var: 'var(--loupe-primary-800)' },
    { name: 'primary-900', var: 'var(--loupe-primary-900)' },
    { name: 'primary-950', var: 'var(--loupe-primary-950)' },
  ],
  surface: [
    { name: 'surface', var: 'var(--loupe-surface)' },
    { name: 'surface-raised', var: 'var(--loupe-surface-raised)' },
    { name: 'surface-overlay', var: 'var(--loupe-surface-overlay)' },
    { name: 'surface-sunken', var: 'var(--loupe-surface-sunken)' },
  ],
  text: [
    { name: 'text', var: 'var(--loupe-text)' },
    { name: 'text-muted', var: 'var(--loupe-text-muted)' },
    { name: 'text-subtle', var: 'var(--loupe-text-subtle)' },
    { name: 'text-inverted', var: 'var(--loupe-text-inverted)' },
  ],
  status: [
    { name: 'success', var: 'var(--loupe-success)' },
    { name: 'success-muted', var: 'var(--loupe-success-muted)' },
    { name: 'warning', var: 'var(--loupe-warning)' },
    { name: 'warning-muted', var: 'var(--loupe-warning-muted)' },
    { name: 'error', var: 'var(--loupe-error)' },
    { name: 'error-muted', var: 'var(--loupe-error-muted)' },
    { name: 'info', var: 'var(--loupe-info)' },
    { name: 'info-muted', var: 'var(--loupe-info-muted)' },
  ],
}

const typography = [
  { name: 'xs', size: 'var(--loupe-font-size-xs)', example: '12px' },
  { name: 'sm', size: 'var(--loupe-font-size-sm)', example: '14px' },
  { name: 'base', size: 'var(--loupe-font-size-base)', example: '16px' },
  { name: 'lg', size: 'var(--loupe-font-size-lg)', example: '18px' },
  { name: 'xl', size: 'var(--loupe-font-size-xl)', example: '20px' },
  { name: '2xl', size: 'var(--loupe-font-size-2xl)', example: '24px' },
  { name: '3xl', size: 'var(--loupe-font-size-3xl)', example: '30px' },
  { name: '4xl', size: 'var(--loupe-font-size-4xl)', example: '36px' },
]

const shadows = [
  { name: 'xs', value: 'var(--loupe-shadow-xs)' },
  { name: 'sm', value: 'var(--loupe-shadow-sm)' },
  { name: 'md', value: 'var(--loupe-shadow-md)' },
  { name: 'lg', value: 'var(--loupe-shadow-lg)' },
  { name: 'xl', value: 'var(--loupe-shadow-xl)' },
  { name: '2xl', value: 'var(--loupe-shadow-2xl)' },
  { name: 'inner', value: 'var(--loupe-shadow-inner)' },
]
</script>

<template>
  <AppLayout title="Style Guide">
    <div class="space-y-12 pb-12">
      <!-- Introduction -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-3">Design System</h2>
        <p class="text-text-muted max-w-3xl">
          This style guide showcases the Loupe design system, including colors, typography,
          components, and interactive elements. All components are built with accessibility in mind
          and support both light and dark themes.
        </p>
      </section>

      <!-- Colors -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Colors</h2>

        <!-- Primary Colors -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Primary Palette</h3>
          <div class="grid grid-cols-11 gap-2">
            <div
              v-for="color in colors.primary"
              :key="color.name"
              class="space-y-2"
            >
              <div
                class="h-20 rounded-lg border border-border"
                :style="{ backgroundColor: color.var }"
              />
              <p class="text-xs text-text-muted text-center">{{ color.name.split('-')[1] }}</p>
            </div>
          </div>
        </div>

        <!-- Surface Colors -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Surface Colors</h3>
          <div class="grid grid-cols-4 gap-4">
            <div
              v-for="color in colors.surface"
              :key="color.name"
              class="space-y-2"
            >
              <div
                class="h-24 rounded-lg border border-border flex items-center justify-center"
                :style="{ backgroundColor: color.var }"
              >
                <span class="text-xs text-text">{{ color.name }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Text Colors -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Text Colors</h3>
          <div class="grid grid-cols-4 gap-4">
            <div
              v-for="color in colors.text"
              :key="color.name"
              class="p-4 rounded-lg bg-surface-sunken"
            >
              <p :style="{ color: color.var }" class="font-medium mb-1">{{ color.name }}</p>
              <p class="text-xs text-text-muted">Example text</p>
            </div>
          </div>
        </div>

        <!-- Status Colors -->
        <div>
          <h3 class="text-lg font-medium text-text mb-4">Status Colors</h3>
          <div class="grid grid-cols-4 gap-4">
            <div
              v-for="color in colors.status"
              :key="color.name"
              class="p-4 rounded-lg"
              :style="{ backgroundColor: color.var }"
            >
              <p class="text-xs font-medium">{{ color.name }}</p>
            </div>
          </div>
        </div>
      </section>

      <!-- Color Usage Guidelines -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Color Usage Guidelines</h2>
        <p class="text-text-muted mb-6">
          Best practices for using the color system consistently throughout the application.
        </p>

        <div class="space-y-6">
          <!-- Primary Colors -->
          <LCard>
            <h3 class="text-lg font-medium text-text mb-3">Primary Colors</h3>
            <p class="text-text-muted text-sm mb-4">
              Use for primary actions, links, and active states. Creates visual hierarchy and guides user attention.
            </p>
            <div class="space-y-2 text-sm">
              <div class="flex items-start gap-2">
                <CheckCircle :size="16" class="text-success mt-0.5 shrink-0" />
                <span class="text-text">Use for primary buttons, selected states, and important links</span>
              </div>
              <div class="flex items-start gap-2">
                <CheckCircle :size="16" class="text-success mt-0.5 shrink-0" />
                <span class="text-text">Use primary-600 for default, primary-700 for hover, primary-500 for focus rings</span>
              </div>
              <div class="flex items-start gap-2">
                <AlertCircle :size="16" class="text-error mt-0.5 shrink-0" />
                <span class="text-text">Avoid using for large backgrounds or body text</span>
              </div>
            </div>
          </LCard>

          <!-- Semantic Colors -->
          <LCard>
            <h3 class="text-lg font-medium text-text mb-3">Semantic Colors</h3>
            <p class="text-text-muted text-sm mb-4">
              Communicate meaning and status through color.
            </p>
            <div class="grid grid-cols-2 gap-4 text-sm">
              <div>
                <div class="flex items-center gap-2 mb-2">
                  <div class="w-4 h-4 rounded bg-success" />
                  <span class="font-medium text-text">Success</span>
                </div>
                <p class="text-text-muted">Confirmations, completed states</p>
              </div>
              <div>
                <div class="flex items-center gap-2 mb-2">
                  <div class="w-4 h-4 rounded bg-error" />
                  <span class="font-medium text-text">Error</span>
                </div>
                <p class="text-text-muted">Errors, destructive actions</p>
              </div>
              <div>
                <div class="flex items-center gap-2 mb-2">
                  <div class="w-4 h-4 rounded bg-warning" />
                  <span class="font-medium text-text">Warning</span>
                </div>
                <p class="text-text-muted">Cautions, non-critical issues</p>
              </div>
              <div>
                <div class="flex items-center gap-2 mb-2">
                  <div class="w-4 h-4 rounded bg-info" />
                  <span class="font-medium text-text">Info</span>
                </div>
                <p class="text-text-muted">Helpful information, tips</p>
              </div>
            </div>
          </LCard>

          <!-- Surface Colors -->
          <LCard>
            <h3 class="text-lg font-medium text-text mb-3">Surface & Text Colors</h3>
            <p class="text-text-muted text-sm mb-4">
              Create depth and hierarchy through layered surfaces.
            </p>
            <div class="space-y-2 text-sm">
              <div class="flex items-center gap-3">
                <div class="w-16 h-8 rounded bg-surface border border-border" />
                <span class="text-text">surface - Base background</span>
              </div>
              <div class="flex items-center gap-3">
                <div class="w-16 h-8 rounded bg-surface-sunken border border-border" />
                <span class="text-text">surface-sunken - Recessed areas (input backgrounds)</span>
              </div>
              <div class="flex items-center gap-3">
                <div class="w-16 h-8 rounded bg-surface-raised border border-border" />
                <span class="text-text">surface-raised - Elevated elements (cards, dropdowns)</span>
              </div>
              <div class="flex items-center gap-3">
                <div class="w-16 h-8 rounded bg-surface-overlay border border-border" />
                <span class="text-text">surface-overlay - Modals, popovers (highest elevation)</span>
              </div>
            </div>
          </LCard>

          <!-- Contrast Guidelines -->
          <LCard>
            <h3 class="text-lg font-medium text-text mb-3">Contrast & Accessibility</h3>
            <div class="space-y-2 text-sm">
              <div class="flex items-start gap-2">
                <CheckCircle :size="16" class="text-success mt-0.5 shrink-0" />
                <span class="text-text">All text meets WCAG AA contrast requirements (4.5:1 for normal text)</span>
              </div>
              <div class="flex items-start gap-2">
                <CheckCircle :size="16" class="text-success mt-0.5 shrink-0" />
                <span class="text-text">Use text-text for primary content, text-muted for secondary, text-subtle for tertiary</span>
              </div>
              <div class="flex items-start gap-2">
                <CheckCircle :size="16" class="text-success mt-0.5 shrink-0" />
                <span class="text-text">OKLCH color space ensures perceptually uniform colors across the spectrum</span>
              </div>
            </div>
          </LCard>
        </div>
      </section>

      <!-- Animation Reference -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Animation Reference</h2>
        <p class="text-text-muted mb-6">
          All available animations and transitions for consistent motion design.
        </p>

        <!-- Animation Classes -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Animation Utilities</h3>
          <div class="grid grid-cols-2 gap-4">
            <LCard>
              <code class="text-sm text-primary-600 font-mono">animate-fade-in</code>
              <p class="text-text-muted text-sm mt-2">Fade in from transparent to opaque</p>
              <p class="text-text-subtle text-xs mt-1">Duration: 200ms</p>
            </LCard>
            <LCard>
              <code class="text-sm text-primary-600 font-mono">animate-slide-in-right</code>
              <p class="text-text-muted text-sm mt-2">Slide in from the right</p>
              <p class="text-text-subtle text-xs mt-1">Duration: 300ms</p>
            </LCard>
            <LCard>
              <code class="text-sm text-primary-600 font-mono">animate-slide-in-left</code>
              <p class="text-text-muted text-sm mt-2">Slide in from the left</p>
              <p class="text-text-subtle text-xs mt-1">Duration: 300ms</p>
            </LCard>
            <LCard>
              <code class="text-sm text-primary-600 font-mono">animate-slide-in-top</code>
              <p class="text-text-muted text-sm mt-2">Slide in from the top</p>
              <p class="text-text-subtle text-xs mt-1">Duration: 300ms</p>
            </LCard>
            <LCard>
              <code class="text-sm text-primary-600 font-mono">animate-slide-in-bottom</code>
              <p class="text-text-muted text-sm mt-2">Slide in from the bottom</p>
              <p class="text-text-subtle text-xs mt-1">Duration: 300ms</p>
            </LCard>
            <LCard>
              <code class="text-sm text-primary-600 font-mono">animate-spin</code>
              <p class="text-text-muted text-sm mt-2">Continuous rotation</p>
              <p class="text-text-subtle text-xs mt-1">Duration: 1s infinite</p>
            </LCard>
            <LCard>
              <code class="text-sm text-primary-600 font-mono">animate-pulse</code>
              <p class="text-text-muted text-sm mt-2">Pulsing opacity effect</p>
              <p class="text-text-subtle text-xs mt-1">Duration: 2s infinite</p>
            </LCard>
            <LCard>
              <code class="text-sm text-primary-600 font-mono">animate-shimmer</code>
              <p class="text-text-muted text-sm mt-2">Shimmer loading effect</p>
              <p class="text-text-subtle text-xs mt-1">Duration: 2s infinite</p>
            </LCard>
          </div>
        </div>

        <!-- Transition Guidelines -->
        <div>
          <h3 class="text-lg font-medium text-text mb-4">Transition Guidelines</h3>
          <div class="space-y-4">
            <LCard>
              <h4 class="font-medium text-text mb-2">Timing</h4>
              <div class="space-y-2 text-sm text-text-muted">
                <p><strong>Fast (150ms):</strong> Small UI changes, hover states, focus indicators</p>
                <p><strong>Base (200ms):</strong> Button clicks, tab switches, most interactions</p>
                <p><strong>Slow (300ms):</strong> Page transitions, modal animations, large movements</p>
              </div>
            </LCard>
            <LCard>
              <h4 class="font-medium text-text mb-2">Easing</h4>
              <div class="space-y-2 text-sm text-text-muted">
                <p><strong>ease-out:</strong> Entrances, appearing elements (fast start, slow end)</p>
                <p><strong>ease-in:</strong> Exits, disappearing elements (slow start, fast end)</p>
                <p><strong>ease-in-out:</strong> Continuous animations, repeated motions</p>
              </div>
            </LCard>
            <LCard>
              <h4 class="font-medium text-text mb-2">Best Practices</h4>
              <div class="space-y-1 text-sm">
                <div class="flex items-start gap-2">
                  <CheckCircle :size="16" class="text-success mt-0.5 shrink-0" />
                  <span class="text-text">Keep animations subtle - users shouldn't wait for animations</span>
                </div>
                <div class="flex items-start gap-2">
                  <CheckCircle :size="16" class="text-success mt-0.5 shrink-0" />
                  <span class="text-text">Respect prefers-reduced-motion for accessibility</span>
                </div>
                <div class="flex items-start gap-2">
                  <CheckCircle :size="16" class="text-success mt-0.5 shrink-0" />
                  <span class="text-text">Use will-change sparingly for performance optimization</span>
                </div>
                <div class="flex items-start gap-2">
                  <CheckCircle :size="16" class="text-success mt-0.5 shrink-0" />
                  <span class="text-text">Transform and opacity are the most performant properties to animate</span>
                </div>
              </div>
            </LCard>
          </div>
        </div>
      </section>

      <!-- Typography -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Typography</h2>
        <div class="space-y-4">
          <div
            v-for="type in typography"
            :key="type.name"
            class="flex items-baseline gap-8 p-4 rounded-lg bg-surface-sunken"
          >
            <div class="w-24">
              <p class="text-sm text-text-muted">{{ type.name }}</p>
              <p class="text-xs text-text-subtle">{{ type.example }}</p>
            </div>
            <p :style="{ fontSize: type.size }" class="text-text font-medium">
              The quick brown fox jumps over the lazy dog
            </p>
          </div>
        </div>
      </section>

      <!-- Shadows -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Shadows</h2>
        <div class="grid grid-cols-4 gap-6">
          <div
            v-for="shadow in shadows"
            :key="shadow.name"
            class="text-center"
          >
            <div
              class="h-24 rounded-lg bg-surface-raised mb-3"
              :style="{ boxShadow: shadow.value }"
            />
            <p class="text-sm font-medium text-text">shadow-{{ shadow.name }}</p>
          </div>
        </div>
      </section>

      <!-- Buttons -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Buttons</h2>

        <!-- Button Variants -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Variants</h3>
          <div class="flex flex-wrap gap-3">
            <LButton variant="primary">Primary</LButton>
            <LButton variant="secondary">Secondary</LButton>
            <LButton variant="danger">Danger</LButton>
            <LButton variant="ghost">Ghost</LButton>
            <LButton variant="soft">Soft</LButton>
            <LButton variant="link">Link</LButton>
          </div>
        </div>

        <!-- Button Sizes -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Sizes</h3>
          <div class="flex flex-wrap items-center gap-3">
            <LButton size="xs">Extra Small</LButton>
            <LButton size="sm">Small</LButton>
            <LButton size="md">Medium</LButton>
            <LButton size="lg">Large</LButton>
          </div>
        </div>

        <!-- Button States -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">States</h3>
          <div class="flex flex-wrap gap-3">
            <LButton>Default</LButton>
            <LButton :loading="true">Loading</LButton>
            <LButton :disabled="true">Disabled</LButton>
          </div>
        </div>

        <!-- Buttons with Icons -->
        <div>
          <h3 class="text-lg font-medium text-text mb-4">With Icons</h3>
          <div class="flex flex-wrap gap-3">
            <LButton>
              <Plus :size="16" />
              Create
            </LButton>
            <LButton variant="secondary">
              <Download :size="16" />
              Download
            </LButton>
            <LButton variant="danger">
              <Trash2 :size="16" />
              Delete
            </LButton>
            <LButton variant="ghost" square aria-label="Favorite">
              <Heart :size="16" />
            </LButton>
          </div>
        </div>
      </section>

      <!-- Form Inputs -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Form Inputs</h2>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 max-w-4xl">
          <!-- Text Input -->
          <div>
            <label class="block text-sm font-medium text-text mb-2">Text Input</label>
            <LInput v-model="inputValue" placeholder="Enter text..." />
          </div>

          <!-- Input with Error -->
          <div>
            <label class="block text-sm font-medium text-text mb-2">Input with Error</label>
            <LInput
              v-model="inputValue"
              error="This field is required"
              placeholder="Enter text..."
            />
          </div>

          <!-- Input with Helper Text -->
          <div>
            <label class="block text-sm font-medium text-text mb-2">Input with Helper</label>
            <LInput
              v-model="inputValue"
              helper-text="We'll never share your email with anyone"
              placeholder="Enter email..."
            />
          </div>

          <!-- Clearable Input -->
          <div>
            <label class="block text-sm font-medium text-text mb-2">Clearable Input</label>
            <LInput v-model="inputValue" clearable placeholder="Enter text..." />
          </div>

          <!-- Textarea -->
          <div class="md:col-span-2">
            <label class="block text-sm font-medium text-text mb-2">Textarea</label>
            <LTextarea v-model="textareaValue" placeholder="Enter description..." rows="4" />
          </div>

          <!-- Select -->
          <div>
            <label class="block text-sm font-medium text-text mb-2">Select</label>
            <LSelect v-model="selectValue" :options="selectOptions" />
          </div>

          <!-- Checkbox -->
          <div>
            <label class="block text-sm font-medium text-text mb-2">Checkbox</label>
            <LCheckbox v-model="checkboxValue" label="I agree to the terms and conditions" />
          </div>
        </div>
      </section>

      <!-- Badges -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Badges</h2>

        <!-- Badge Variants -->
        <div class="mb-6">
          <h3 class="text-lg font-medium text-text mb-4">Variants</h3>
          <div class="flex flex-wrap gap-3">
            <LBadge variant="default">Default</LBadge>
            <LBadge variant="primary">Primary</LBadge>
            <LBadge variant="success">Success</LBadge>
            <LBadge variant="warning">Warning</LBadge>
            <LBadge variant="error">Error</LBadge>
            <LBadge variant="info">Info</LBadge>
          </div>
        </div>

        <!-- Badge Styles -->
        <div class="mb-6">
          <h3 class="text-lg font-medium text-text mb-4">Styles</h3>
          <div class="flex flex-wrap gap-3">
            <LBadge badge-style="filled" variant="primary">Filled</LBadge>
            <LBadge badge-style="outline" variant="primary">Outline</LBadge>
            <LBadge badge-style="dot" variant="success">Dot</LBadge>
            <LBadge badge-style="dot" variant="success" pulse>Pulse</LBadge>
          </div>
        </div>

        <!-- Badge Sizes -->
        <div class="mb-6">
          <h3 class="text-lg font-medium text-text mb-4">Sizes</h3>
          <div class="flex flex-wrap items-center gap-3">
            <LBadge size="xs">Extra Small</LBadge>
            <LBadge size="sm">Small</LBadge>
            <LBadge size="md">Medium</LBadge>
            <LBadge size="lg">Large</LBadge>
            <LBadge size="xl">Extra Large</LBadge>
          </div>
        </div>

        <!-- Removable Badges -->
        <div>
          <h3 class="text-lg font-medium text-text mb-4">Removable</h3>
          <div class="flex flex-wrap gap-3">
            <LBadge variant="primary" removable>Tag 1</LBadge>
            <LBadge variant="success" removable>Tag 2</LBadge>
            <LBadge variant="info" removable>
              <template #icon>
                <Star :size="12" />
              </template>
              Featured
            </LBadge>
          </div>
        </div>
      </section>

      <!-- Cards -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Cards</h2>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <!-- Default Card -->
          <LCard>
            <h3 class="font-semibold text-text mb-2">Default Card</h3>
            <p class="text-text-muted text-sm">
              This is a default card with standard padding and styling.
            </p>
          </LCard>

          <!-- Elevated Card -->
          <LCard variant="elevated">
            <h3 class="font-semibold text-text mb-2">Elevated Card</h3>
            <p class="text-text-muted text-sm">This card has enhanced shadow for more depth.</p>
          </LCard>

          <!-- Card with Header and Footer -->
          <LCard variant="outlined" class="md:col-span-2">
            <template #header>
              <div class="flex items-center justify-between">
                <h3 class="font-semibold text-text">Card with Header & Footer</h3>
                <LBadge variant="success" size="sm">Active</LBadge>
              </div>
            </template>
            <p class="text-text-muted">
              This card demonstrates the header and footer slots for organizing content.
            </p>
            <template #footer>
              <div class="flex justify-end gap-2">
                <LButton variant="secondary" size="sm">Cancel</LButton>
                <LButton size="sm">Save</LButton>
              </div>
            </template>
          </LCard>

          <!-- Card with Accent -->
          <LCard accent="primary">
            <h3 class="font-semibold text-text mb-2">Primary Accent</h3>
            <p class="text-text-muted text-sm">Card with a colored accent border.</p>
          </LCard>

          <LCard accent="success">
            <h3 class="font-semibold text-text mb-2">Success Accent</h3>
            <p class="text-text-muted text-sm">Card with a success accent border.</p>
          </LCard>
        </div>
      </section>

      <!-- Alerts -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Alerts</h2>
        <div class="space-y-4 max-w-3xl">
          <LAlert variant="info" title="Information">
            This is an informational alert message.
          </LAlert>
          <LAlert variant="success" title="Success">
            Your changes have been saved successfully.
          </LAlert>
          <LAlert variant="warning" title="Warning">
            Please review your settings before proceeding.
          </LAlert>
          <LAlert variant="error" title="Error">
            An error occurred while processing your request.
          </LAlert>
        </div>
      </section>

      <!-- Tables -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Tables</h2>

        <!-- Basic Table -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Basic Table</h3>
          <LTable :headers="tableHeaders" :rows="tableRows" striped hoverable />
        </div>

        <!-- Sortable Table -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Sortable Table</h3>
          <p class="text-text-muted text-sm mb-4">Click column headers to sort</p>
          <LTable
            :headers="tableHeaders"
            :rows="tableRows"
            sortable
            :sort-by="sortBy"
            :sort-direction="sortDirection"
            striped
            hoverable
            @sort="handleSort"
          />
        </div>

        <!-- Selectable Table -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Selectable Table</h3>
          <p class="text-text-muted text-sm mb-4">
            Selected rows: {{ selectedRows.length }}
          </p>
          <LTable
            :headers="tableHeaders"
            :rows="tableRows"
            selectable
            :selected-rows="selectedRows"
            striped
            hoverable
            @update:selected-rows="selectedRows = $event"
          />
        </div>

        <!-- Compact Table -->
        <div>
          <h3 class="text-lg font-medium text-text mb-4">Compact Table</h3>
          <LTable :headers="tableHeaders" :rows="tableRows" compact striped hoverable />
        </div>
      </section>

      <!-- Pagination -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Pagination</h2>

        <!-- Basic Pagination -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Basic Pagination</h3>
          <LCard>
            <LPagination :current-page="3" :total-pages="10" :total-items="247" />
          </LCard>
        </div>

        <!-- Without Page Size -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Without Page Size Selector</h3>
          <LCard>
            <LPagination :current-page="1" :total-pages="5" :total-items="48" :show-page-size="false" />
          </LCard>
        </div>

        <!-- Many Pages -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Many Pages</h3>
          <LCard>
            <LPagination :current-page="47" :total-pages="100" :total-items="1000" />
          </LCard>
        </div>

        <!-- Disabled State -->
        <div>
          <h3 class="text-lg font-medium text-text mb-4">Disabled</h3>
          <LCard>
            <LPagination :current-page="3" :total-pages="10" :total-items="247" disabled />
          </LCard>
        </div>
      </section>

      <!-- Loading States -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Loading States</h2>

        <!-- Spinners -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Spinners</h3>
          <div class="flex items-center gap-8">
            <div class="text-center">
              <LSpinner size="sm" />
              <p class="text-xs text-text-muted mt-2">Small</p>
            </div>
            <div class="text-center">
              <LSpinner size="md" />
              <p class="text-xs text-text-muted mt-2">Medium</p>
            </div>
            <div class="text-center">
              <LSpinner size="lg" />
              <p class="text-xs text-text-muted mt-2">Large</p>
            </div>
          </div>
        </div>

        <!-- Skeletons -->
        <div>
          <h3 class="text-lg font-medium text-text mb-4">Skeletons</h3>
          <div class="space-y-4 max-w-2xl">
            <LSkeleton variant="text" />
            <LSkeleton variant="text" lines="3" />
            <LSkeleton variant="rectangle" class="h-40" />
            <div class="flex items-center gap-4">
              <LSkeleton variant="circle" class="w-12 h-12" />
              <div class="flex-1">
                <LSkeleton variant="text" />
                <LSkeleton variant="text" class="w-2/3" />
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Empty States -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Empty States</h2>
        <LCard class="max-w-2xl">
          <LEmptyState
            title="No items found"
            description="Get started by creating your first item."
          >
            <template #icon>
              <AlertCircle :size="48" class="text-text-subtle" />
            </template>
            <template #action>
              <LButton>
                <Plus :size="16" />
                Create Item
              </LButton>
            </template>
          </LEmptyState>
        </LCard>
      </section>

      <!-- Tooltips -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Tooltips</h2>
        <div class="flex gap-4">
          <LTooltip content="This is a tooltip">
            <LButton>Hover me</LButton>
          </LTooltip>
          <LTooltip content="Tooltips work with any element">
            <LBadge variant="primary">Badge with tooltip</LBadge>
          </LTooltip>
        </div>
      </section>

      <!-- Modal -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Modal</h2>
        <LButton @click="showModal = true">Open Modal</LButton>

        <LModal v-model="showModal" title="Example Modal" size="md">
          <p class="text-text mb-4">
            This is an example modal demonstrating the modal component with header and footer.
          </p>
          <p class="text-text-muted text-sm">
            Modals support various sizes (sm, md, lg, xl, full) and can include custom headers and
            footers.
          </p>

          <template #footer>
            <LButton variant="secondary" @click="showModal = false">Cancel</LButton>
            <LButton @click="showModal = false">Confirm</LButton>
          </template>
        </LModal>
      </section>

      <!-- Spacing -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Spacing Scale</h2>
        <p class="text-text-muted mb-6">
          Loupe uses a consistent spacing scale based on a 4px/8px grid system.
        </p>
        <div class="space-y-3">
          <div v-for="(size, name) in { 1: '4px', 2: '8px', 3: '12px', 4: '16px', 6: '24px', 8: '32px', 12: '48px', 16: '64px' }" :key="name" class="flex items-center gap-4">
            <div class="w-16 text-sm text-text-muted">{{ name }} ({{ size }})</div>
            <div class="h-8 bg-primary-500 rounded" :style="{ width: size }" />
          </div>
        </div>
      </section>

      <!-- Accessibility -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Accessibility Features</h2>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <LCard>
            <div class="flex items-start gap-3">
              <CheckCircle :size="24" class="text-success shrink-0" />
              <div>
                <h3 class="font-semibold text-text mb-2">ARIA Labels</h3>
                <p class="text-text-muted text-sm">
                  All icon-only buttons include proper aria-label attributes for screen readers.
                </p>
              </div>
            </div>
          </LCard>

          <LCard>
            <div class="flex items-start gap-3">
              <CheckCircle :size="24" class="text-success shrink-0" />
              <div>
                <h3 class="font-semibold text-text mb-2">Reduced Motion</h3>
                <p class="text-text-muted text-sm">
                  Respects prefers-reduced-motion to disable animations for users with motion
                  sensitivity.
                </p>
              </div>
            </div>
          </LCard>

          <LCard>
            <div class="flex items-start gap-3">
              <CheckCircle :size="24" class="text-success shrink-0" />
              <div>
                <h3 class="font-semibold text-text mb-2">Keyboard Navigation</h3>
                <p class="text-text-muted text-sm">
                  All interactive elements are keyboard accessible with visible focus indicators.
                </p>
              </div>
            </div>
          </LCard>

          <LCard>
            <div class="flex items-start gap-3">
              <CheckCircle :size="24" class="text-success shrink-0" />
              <div>
                <h3 class="font-semibold text-text mb-2">Color Contrast</h3>
                <p class="text-text-muted text-sm">
                  OKLCH color space ensures perceptually uniform colors with proper contrast ratios.
                </p>
              </div>
            </div>
          </LCard>
        </div>
      </section>

      <!-- Layout Patterns -->
      <section>
        <h2 class="text-2xl font-semibold text-text mb-6">Layout Patterns</h2>
        <p class="text-text-muted mb-6">
          Common layout patterns used throughout the application for consistency.
        </p>

        <!-- Page Header Pattern -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Page Header with Actions</h3>
          <p class="text-text-muted text-sm mb-4">
            Standard page header layout with title and action buttons
          </p>
          <LCard>
            <div class="flex items-center justify-between mb-6">
              <div>
                <h2 class="text-2xl font-semibold text-text">Page Title</h2>
                <p class="text-text-muted mt-1">Optional page description goes here</p>
              </div>
              <div class="flex gap-2">
                <LButton variant="secondary" size="sm">
                  <Settings :size="16" />
                  Settings
                </LButton>
                <LButton size="sm">
                  <Heart :size="16" />
                  Primary Action
                </LButton>
              </div>
            </div>
          </LCard>
        </div>

        <!-- Card Grid Pattern -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Card Grid</h3>
          <p class="text-text-muted text-sm mb-4">
            Responsive grid layout for cards (grid-cols-1 md:grid-cols-2 lg:grid-cols-3)
          </p>
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <LCard>
              <div class="flex items-center gap-3 mb-3">
                <div class="w-10 h-10 rounded-lg bg-primary-100 dark:bg-primary-900 flex items-center justify-center">
                  <BarChart3 :size="20" class="text-primary-600" />
                </div>
                <h3 class="font-semibold text-text">Card Title</h3>
              </div>
              <p class="text-text-muted text-sm">Card content with description and details.</p>
            </LCard>
            <LCard>
              <div class="flex items-center gap-3 mb-3">
                <div class="w-10 h-10 rounded-lg bg-success-100 dark:bg-success-900 flex items-center justify-center">
                  <CheckCircle :size="20" class="text-success" />
                </div>
                <h3 class="font-semibold text-text">Card Title</h3>
              </div>
              <p class="text-text-muted text-sm">Card content with description and details.</p>
            </LCard>
            <LCard>
              <div class="flex items-center gap-3 mb-3">
                <div class="w-10 h-10 rounded-lg bg-warning-100 dark:bg-warning-900 flex items-center justify-center">
                  <AlertCircle :size="20" class="text-warning" />
                </div>
                <h3 class="font-semibold text-text">Card Title</h3>
              </div>
              <p class="text-text-muted text-sm">Card content with description and details.</p>
            </LCard>
          </div>
        </div>

        <!-- Stack Layout Pattern -->
        <div class="mb-8">
          <h3 class="text-lg font-medium text-text mb-4">Stack Layout</h3>
          <p class="text-text-muted text-sm mb-4">
            Vertical stacking with consistent spacing (space-y-4)
          </p>
          <div class="space-y-4 max-w-2xl">
            <LCard>
              <div class="flex items-center justify-between">
                <div>
                  <h3 class="font-medium text-text">Item Title</h3>
                  <p class="text-sm text-text-muted">Item description or metadata</p>
                </div>
                <LButton variant="secondary" size="sm">Action</LButton>
              </div>
            </LCard>
            <LCard>
              <div class="flex items-center justify-between">
                <div>
                  <h3 class="font-medium text-text">Item Title</h3>
                  <p class="text-sm text-text-muted">Item description or metadata</p>
                </div>
                <LButton variant="secondary" size="sm">Action</LButton>
              </div>
            </LCard>
            <LCard>
              <div class="flex items-center justify-between">
                <div>
                  <h3 class="font-medium text-text">Item Title</h3>
                  <p class="text-sm text-text-muted">Item description or metadata</p>
                </div>
                <LButton variant="secondary" size="sm">Action</LButton>
              </div>
            </LCard>
          </div>
        </div>

        <!-- Form Layout Pattern -->
        <div>
          <h3 class="text-lg font-medium text-text mb-4">Form Layout</h3>
          <p class="text-text-muted text-sm mb-4">
            Standard form layout with labels, inputs, and help text
          </p>
          <LCard class="max-w-2xl">
            <form class="space-y-4">
              <div>
                <label class="block text-sm font-medium text-text mb-1.5">Field Label</label>
                <LInput v-model="inputValue" placeholder="Enter value..." />
                <p class="mt-1 text-xs text-text-muted">Helper text for additional context</p>
              </div>
              <div>
                <label class="block text-sm font-medium text-text mb-1.5">Select Field</label>
                <LSelect v-model="selectValue" :options="selectOptions" />
              </div>
              <div>
                <label class="block text-sm font-medium text-text mb-1.5">Textarea</label>
                <textarea
                  v-model="textareaValue"
                  class="w-full px-3 py-2 rounded-md border border-border bg-surface text-text focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                  rows="3"
                  placeholder="Enter longer text..."
                />
              </div>
              <div class="flex justify-end gap-3 pt-4 border-t border-border">
                <LButton variant="secondary">Cancel</LButton>
                <LButton>Save Changes</LButton>
              </div>
            </form>
          </LCard>
        </div>
      </section>
    </div>
  </AppLayout>
</template>
