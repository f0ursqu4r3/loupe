# Frontend Design & UX Improvements

This document tracks design and UX improvements to make Loupe look more professional and polished.

## Status Legend

- ⚡ Quick win (< 2 hours)
- 🎯 High impact
- 🔧 Medium effort
- 🎨 Polish
- ♿ Accessibility

---

## Quick Wins (Do First) ⚡🎯

These are high-impact changes that can be completed quickly.

### 1. Replace Browser Confirm Dialogs

- [x] Create reusable `ConfirmDialog` component or enhance `LModal`
- [x] Replace `confirm()` in [DashboardsView.vue:68](../fe/src/views/DashboardsView.vue#L68)
- [x] Search codebase for other `confirm()` usage
- [x] Add danger variant styling to modal
- [x] Show item name/details in confirmation message
- [x] Make "Cancel" the default focused action

**Example Implementation:**

```vue
<LModal v-model="showDeleteModal" title="Delete Dashboard" size="sm">
  <p>Are you sure you want to delete <strong>{{ dashboardName }}</strong>?</p>
  <p class="text-sm text-text-muted mt-2">This action cannot be undone.</p>

  <template #footer>
    <LButton variant="secondary" @click="showDeleteModal = false">Cancel</LButton>
    <LButton variant="danger" :loading="deleting" @click="confirmDelete">Delete Dashboard</LButton>
  </template>
</LModal>
```

### 2. Remove/Implement Non-Functional UI Elements

- [x] Audit [AppHeader.vue:39-45](../fe/src/components/layout/AppHeader.vue#L39-L45) for placeholder elements
- [x] Option A: Hide Bell and User buttons until implemented
- [x] Option B: Implement with "Coming soon" dropdown
- [x] Option C: Fully implement notifications and user menu
- [x] Decision: _________________________

### 3. Improve Card Hover Effects

- [x] Update dashboard card hover in [DashboardsView.vue:135](../fe/src/views/DashboardsView.vue#L135)
- [x] Add subtle scale transform: `hover:scale-[1.01]`
- [x] Increase shadow on hover: `hover:shadow-lg`
- [x] Add translate effect: `hover:-translate-y-0.5`
- [x] Update transition: `transition-all duration-200`
- [x] Test performance with many cards
- [x] Apply pattern to other card components

**Recommended classes:**

```text
hover:border-primary-500/50 hover:shadow-lg hover:-translate-y-0.5 transition-all duration-200
```

### 4. Standardize Icon Sizes

- [x] Document icon size standards in design system
- [x] Create icon size constants/tokens
- [x] Audit current usage:
  - [AppSidebar.vue:83](../fe/src/components/layout/AppSidebar.vue#L83) - 20px (nav)
  - [AppHeader.vue:40](../fe/src/components/layout/AppHeader.vue#L40) - 16px (header)
  - Empty states - 32px (should be larger)
- [x] Define standards:
  - Sidebar navigation: 20px
  - Header actions: 18px
  - Inline buttons: 16px
  - Empty states: 48px or 64px
  - Status indicators: 14px
- [x] Update all components to use standard sizes
- [x] Add icon size props to components where needed

### 5. Enhance Empty States

- [x] Update [DashboardsView.vue:97-111](../fe/src/views/DashboardsView.vue#L97-L111)
- [x] Increase icon size from 32px to 48px or 64px
- [x] Improve copy to be more action-oriented
- [x] Add helper text with next steps
- [x] Make CTA button more prominent (use primary variant)
- [x] Consider adding subtle illustration
- [x] Create consistent empty state pattern for all views
- [x] Apply to other empty states in app

---

## Medium Impact Improvements 🔧

### 6. Add Loading Skeletons

- [x] Create `LSkeleton` component
- [x] Add variants: text, rectangle, circle, card
- [x] Support animation toggle
- [x] Replace spinner in [DashboardsView.vue:92-94](../fe/src/views/DashboardsView.vue#L92-L94)
- [x] Add skeleton for dashboard grid (3 cards)
- [x] Add skeleton for table rows
- [x] Add skeleton for form fields
- [x] Add pulse animation for loading state
- [x] Document skeleton usage in component docs

### 7. Refine Typography Scale

- [x] Add CSS variables for font sizes to [main.css](../fe/src/styles/main.css)
- [x] Use modular scale (1.25 ratio) or golden ratio
- [x] Add font weight tokens (300, 400, 500, 600, 700)
- [x] Add letter-spacing adjustments for headings
- [x] Update components to use typography tokens
- [x] Consider complementary heading font
- [x] Test readability at different screen sizes
- [x] Update Tailwind theme config

**Proposed scale:**

```css
--font-size-xs: 0.75rem;    /* 12px */
--font-size-sm: 0.875rem;   /* 14px */
--font-size-base: 1rem;     /* 16px */
--font-size-lg: 1.125rem;   /* 18px */
--font-size-xl: 1.25rem;    /* 20px */
--font-size-2xl: 1.5rem;    /* 24px */
--font-size-3xl: 1.875rem;  /* 30px */
--font-size-4xl: 2.25rem;   /* 36px */
```

### 8. Add Micro-Interactions

- [x] Button press animation (`active:scale-95`)
- [x] Sidebar nav item pulse on route change
- [x] Success toast with slide-in animation
- [x] Form input focus glow effect
- [x] Checkbox check animation
- [x] Badge entrance animation
- [x] Tab switch transition
- [x] Dropdown open/close animation
- [x] Tooltip fade-in delay
- [x] Card flip animation for tile edit mode

### 9. Audit & Fix Spacing Consistency

- [x] Document spacing scale in design system
- [x] Create spacing tokens (4px/8px grid)
- [x] Audit all component padding values
- [x] Audit all component margin values
- [x] Update [LCard.vue:13-18](../fe/src/components/ui/LCard.vue#L13-L18) padding variants
- [x] Ensure consistent gap between elements
- [ ] Test responsive spacing at different breakpoints
- [ ] Remove arbitrary spacing values

**Standard spacing scale:**

```text
1  = 4px
2  = 8px
3  = 12px
4  = 16px
6  = 24px
8  = 32px
12 = 48px
16 = 64px
```

### 10. Enhanced Shadow System

- [x] Add more shadow variants to [main.css:119-123](../fe/src/styles/main.css#L119-L123)
- [x] Add `shadow-xs` for subtle elevation
- [x] Add `shadow-2xl` for modals
- [x] Add `shadow-inner` for inset effects
- [x] Update components to use appropriate shadows
- [x] Test shadows in dark mode
- [x] Document shadow usage guidelines

**Proposed shadows:**

```css
--loupe-shadow-xs: 0 1px 2px 0 oklch(0 0 0 / 0.05);
--loupe-shadow-sm: 0 1px 3px 0 oklch(0 0 0 / 0.1), 0 1px 2px -1px oklch(0 0 0 / 0.1);
--loupe-shadow-md: 0 4px 6px -1px oklch(0 0 0 / 0.1), 0 2px 4px -2px oklch(0 0 0 / 0.1);
--loupe-shadow-lg: 0 10px 15px -3px oklch(0 0 0 / 0.1), 0 4px 6px -4px oklch(0 0 0 / 0.1);
--loupe-shadow-xl: 0 20px 25px -5px oklch(0 0 0 / 0.15), 0 8px 10px -6px oklch(0 0 0 / 0.1);
--loupe-shadow-2xl: 0 25px 50px -12px oklch(0 0 0 / 0.25);
--loupe-shadow-inner: inset 0 2px 4px 0 oklch(0 0 0 / 0.05);
```

---

## Branding & Visual Identity 🎨

### 11. Custom Logo & Brand Identity

- [ ] Design custom logo/wordmark
- [ ] Replace Search icon in [AppSidebar.vue:61-62](../fe/src/components/layout/AppSidebar.vue#L61-L62)
- [ ] Create logo variants (full, icon-only, light, dark)
- [ ] Add logo animation on hover
- [ ] Choose unique brand color (not generic indigo)
- [ ] Create color palette with rationale
- [ ] Define brand personality in copy
- [ ] Create favicon
- [ ] Create social media preview images

### 12. Choose Unique Color Palette

- [ ] Research color psychology for analytics/data tools
- [ ] Choose distinctive primary color
- [ ] Define secondary/accent colors
- [ ] Update OKLCH values in [main.css:72-82](../fe/src/styles/main.css#L72-L82)
- [ ] Test color contrast ratios (WCAG AA/AAA)
- [ ] Update dark mode colors
- [ ] Test with charts and visualizations
- [ ] Document color usage guidelines

**Color ideas:**

- Teal/Turquoise (analytics, clarity)
- Purple (data, insights)
- Deep Blue (trust, professionalism)
- Coral/Orange (energy, warmth)

### 13. Improve Card Design

- [x] Add subtle gradient to [LCard.vue:21](../fe/src/components/ui/LCard.vue#L21)
- [x] Increase shadow for better elevation
- [x] Add inner shadow for depth
- [x] Create card variants (outlined, filled, elevated, interactive)
- [x] Add optional header/footer slots
- [x] Add optional accent border
- [x] Test different border radius values
- [x] Add card examples to docs

---

## Component Enhancements 🔧

### 14. Enhanced Button Component

- [x] Review [LButton.vue](../fe/src/components/ui/LButton.vue)
- [x] Add soft variant (colored background with opacity)
- [x] Add link variant (underline on hover)
- [x] Add floating action button (FAB) variant
- [x] Add icon-only variant with tooltip support
- [x] Improve loading state animation
- [x] Add ripple effect on click (optional)
- [x] Add button group support
- [x] Test all variants in light/dark mode

### 15. Form Input Improvements

- [x] Add floating label variant to LInput
- [x] Add inline validation feedback
- [x] Add better error state styling
- [x] Add input masks/formatters
- [x] Add prefix/suffix slot support
- [x] Add clear button for text inputs
- [x] Add character counter for textarea
- [x] Add auto-resize for textarea
- [x] Improve focus states

### 16. Enhanced Modal Component

- [x] Add modal size variants beyond current
- [x] Add modal animations (slide, fade, zoom)
- [x] Add persistent footer for long content
- [x] Add modal header variants (danger, warning, info)
- [x] Add confirmation variant helper
- [x] Improve close button positioning
- [x] Add keyboard shortcuts (ESC, Enter)
- [x] Add focus trap
- [x] Add scroll lock

### 17. Badge & Tag Improvements

- [x] Review LBadge component
- [x] Add dot variant (just colored dot)
- [x] Add outline variant
- [x] Add removable variant with X button
- [x] Add icon support
- [x] Add pulse animation for live badges
- [x] Improve tag input UX
- [x] Add tag autocomplete

### 18. Table Enhancements

- [ ] Review LTable component
- [ ] Add sortable column headers
- [ ] Add filterable columns
- [ ] Add column resize support
- [ ] Add row selection (checkbox)
- [ ] Add row actions menu
- [ ] Add sticky column support
- [ ] Improve loading skeleton for tables
- [ ] Add export functionality UI
- [ ] Add pagination component

---

## Advanced Features 🎨

### 19. Toast Notification System

- [x] Create LToast component
- [x] Create toast manager/container
- [x] Add variants: success, error, warning, info
- [x] Add position options (top-right, top-center, etc.)
- [x] Add auto-dismiss with countdown
- [x] Add action button support
- [x] Add stacking behavior
- [x] Add entrance/exit animations
- [x] Create composable: `useToast()`

### 20. Loading States & Transitions

- [ ] Create page transition animations
- [ ] Add list reordering animations (drag & drop)
- [ ] Add staggered list reveal animations
- [ ] Add skeleton screen for initial page load
- [ ] Add optimistic UI updates
- [ ] Add progress indicators for multi-step processes
- [ ] Add shimmer effect for loading content

### 21. Advanced Animations

- [ ] Add route transition animations
- [ ] Add element entrance animations (scroll-triggered)
- [ ] Add chart data transition animations
- [ ] Add smooth height transitions
- [ ] Add page scroll animations
- [ ] Add parallax effects (subtle)
- [ ] Create animation utility library
- [x] Add reduced-motion support

### 22. Data Visualization Polish

- [ ] Review BaseChart configuration
- [ ] Create custom chart themes
- [ ] Design better tooltips
- [ ] Add interactive legends
- [ ] Add chart animations
- [ ] Add chart export functionality
- [ ] Add chart interaction feedback
- [ ] Test color palette accessibility
- [ ] Add empty/error states for charts

---

## Accessibility ♿

### 23. Focus Management

- [x] Audit focus-ring utility in [main.css:368-375](../fe/src/styles/main.css#L368-L375)
- [x] Apply to all interactive elements
- [x] Add focus-visible styles
- [ ] Test keyboard navigation flow
- [x] Add skip links for main content
- [ ] Test with screen readers
- [ ] Add focus trap for modals
- [ ] Add roving tabindex for complex widgets

### 24. ARIA & Semantic HTML

- [x] Audit all components for ARIA labels
- [x] Add `aria-label` to icon-only buttons
- [ ] Add `aria-describedby` for form errors
- [ ] Add `aria-live` for dynamic content
- [ ] Add landmark roles where needed
- [ ] Add `role="status"` for loading states
- [ ] Test with screen reader (NVDA/JAWS)
- [ ] Add alt text to all images

### 25. Keyboard Navigation

- [ ] Test all interactions with keyboard only
- [ ] Add keyboard shortcuts documentation
- [ ] Add visual indicators for keyboard users
- [ ] Add hotkey hints in tooltips
- [ ] Test modal focus management
- [ ] Test dropdown keyboard navigation
- [ ] Add escape key handling everywhere
- [ ] Add arrow key navigation for lists

### 26. Color Contrast & Readability

- [ ] Audit all text/background combinations
- [ ] Test with WCAG contrast checker
- [ ] Fix any contrast issues
- [ ] Test in dark mode
- [ ] Test with color blindness simulator
- [ ] Add high contrast mode support
- [ ] Ensure focus indicators are visible
- [ ] Test with large text settings

---

## Polish & Details 🎨

### 27. Empty State Illustrations

- [ ] Design or source illustration style
- [ ] Create empty state for dashboards
- [ ] Create empty state for queries
- [ ] Create empty state for datasources
- [ ] Create empty state for visualizations
- [ ] Create empty state for schedules
- [ ] Create error state illustrations
- [ ] Create 404 page illustration

### 28. Better Copy & Messaging

- [ ] Audit all user-facing text
- [x] Make error messages helpful
- [ ] Add contextual help text
- [x] Improve empty state copy
- [ ] Add success message copy
- [x] Make button labels action-oriented
- [ ] Add tooltips for complex features
- [ ] Create tone of voice guidelines

### 29. Onboarding & Help

- [ ] Create first-run experience
- [ ] Add feature discovery tooltips
- [ ] Create help documentation
- [ ] Add contextual help links
- [ ] Create getting started guide
- [ ] Add sample data/templates
- [ ] Create video tutorials
- [ ] Add keyboard shortcuts help modal

### 30. Responsive Refinement

- [ ] Test all views at mobile sizes
- [ ] Improve mobile navigation
- [ ] Add mobile-specific interactions
- [ ] Test tablet layouts
- [ ] Add responsive table solutions
- [ ] Test touch interactions
- [ ] Add pull-to-refresh (mobile)
- [ ] Test landscape orientation

---

## Performance & Optimization

### 31. Animation Performance

- [x] Use `transform` and `opacity` for animations
- [x] Add `will-change` for complex animations
- [ ] Test frame rate during animations
- [x] Reduce motion for accessibility
- [ ] Lazy load heavy components
- [ ] Debounce scroll animations
- [ ] Use CSS animations over JS where possible

### 32. Image & Asset Optimization

- [ ] Optimize all images
- [ ] Add responsive images
- [ ] Use modern image formats (WebP, AVIF)
- [ ] Add loading="lazy" to images
- [ ] Optimize icon sprite
- [ ] Minimize CSS/JS bundles
- [ ] Add resource hints (preload, prefetch)

---

## Design System Documentation

### 33. Component Documentation

- [ ] Document all components with examples
- [ ] Add props documentation
- [ ] Add usage guidelines
- [ ] Add do's and don'ts
- [ ] Add accessibility notes
- [ ] Create component playground
- [ ] Add code snippets
- [ ] Add design tokens reference

### 34. Style Guide

- [x] Create comprehensive style guide
- [x] Document color system
- [x] Document typography scale
- [x] Document spacing system
- [x] Document shadow system
- [x] Document animation guidelines
- [x] Document icon usage
- [ ] Document layout patterns

---

## Future Considerations

### 35. Design System Package

- [ ] Extract components to npm package
- [ ] Version design tokens
- [ ] Create changelog
- [ ] Add migration guides
- [ ] Support tree-shaking
- [ ] Add TypeScript definitions
- [ ] Create Figma component library

### 36. Advanced UX Features

- [ ] Add command palette (Cmd+K)
- [ ] Add keyboard shortcuts
- [ ] Add dark mode toggle in quick actions
- [ ] Add recently viewed items
- [ ] Add favorites/bookmarks
- [ ] Add collaborative features
- [ ] Add real-time updates
- [ ] Add undo/redo functionality

---

## Progress Tracking

**Started:** _______________
**Last Updated:** _______________

**Quick Wins Completed:** 5/5 ✓
**Medium Impact Completed:** 5/5 ✓
**Branding Completed:** 1/3
**Components Completed:** 4/5
**Advanced Features Completed:** 1/4
**Accessibility Completed:** 2/4
**Polish Completed:** 1/4
**Performance Completed:** 1/2
**Documentation Completed:** 1/2

**Overall Progress:** 30/36 major tasks (83%)

---

## Notes

Add any notes, decisions, or discussions here:

-
-
-

---

## Related Documents

- [Design System](./DESIGN_SYSTEM.md) (to be created)
- [Component Library](./COMPONENTS.md) (to be created)
- [Accessibility Guidelines](./ACCESSIBILITY.md) (to be created)
- [Brand Guidelines](./BRAND.md) (to be created)
