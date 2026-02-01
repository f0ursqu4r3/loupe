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
- [x] Document color usage guidelines

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

- [x] Review LTable component
- [x] Add sortable column headers
- [ ] Add filterable columns
- [ ] Add column resize support
- [x] Add row selection (checkbox)
- [ ] Add row actions menu
- [ ] Add sticky column support
- [x] Improve loading skeleton for tables
- [ ] Add export functionality UI
- [x] Add pagination component

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

- [x] Create page transition animations
- [ ] Add list reordering animations (drag & drop)
- [ ] Add staggered list reveal animations
- [ ] Add skeleton screen for initial page load
- [ ] Add optimistic UI updates
- [ ] Add progress indicators for multi-step processes
- [x] Add shimmer effect for loading content

### 21. Advanced Animations

- [x] Add route transition animations
- [ ] Add element entrance animations (scroll-triggered)
- [ ] Add chart data transition animations
- [ ] Add smooth height transitions
- [ ] Add page scroll animations
- [ ] Add parallax effects (subtle)
- [x] Create animation utility library
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

## Authentication & Authorization 🔒

### 23. RBAC Support for Backend Permissions ✅

**Backend Context:** The backend now enforces role-based permissions (Admin, Editor, Viewer). The frontend needs to handle these appropriately.

**See:** [RBAC_IMPLEMENTATION.md](./RBAC_IMPLEMENTATION.md) for backend details.

- [x] Add user role to auth state/store
- [x] Fetch and store current user's role on login
- [x] Create `usePermissions()` composable for permission checks
- [x] Handle 403 Forbidden errors gracefully
- [x] Show user-friendly error messages when permission denied
- [x] Add error ID display for debugging (from backend error responses)

**Error Handling:**

```typescript
// Example: Handle 403 errors
if (error.response?.status === 403) {
  toast.error('You don\'t have permission to perform this action')
  // Optionally show error ID for support
  console.error('Error ID:', error.response.data?.error?.error_id)
}
```

### 24. Conditional UI Based on Roles

**Implement UI visibility based on user permissions:**

**Dashboards:**

- [x] Hide "New Dashboard" button for Viewers
- [x] Hide edit/delete actions for Viewers
- [ ] Show read-only badge for Viewers

**Queries:**

- [x] Hide "New Query" button for Viewers
- [x] Hide "Import" button for Viewers
- [ ] Disable "Ad-hoc SQL" tab for Viewers (Editor+ only)
- [ ] Show permission indicator on query execution

**Datasources:**

- [ ] Hide "New Datasource" button for non-Admins
- [ ] Hide edit/delete actions for non-Admins
- [ ] Show "Admin only" badge on sensitive actions

**Visualizations:**

- [ ] Hide "New Visualization" button for Viewers
- [ ] Hide edit/delete actions for Viewers

**Schedules:**

- [ ] Hide "New Schedule" button for Viewers
- [ ] Hide enable/disable/trigger actions for Viewers
- [ ] Show permission indicator on schedule management

**Canvases:**

- [ ] Hide "New Canvas" button for Viewers
- [ ] Hide edit actions (nodes, edges) for Viewers
- [ ] Show read-only mode for Viewers

**Example Implementation:**

```vue
<template>
  <!-- Only show for Editors and Admins -->
  <LButton
    v-if="canEdit"
    @click="createDashboard"
  >
    New Dashboard
  </LButton>

  <!-- Show disabled state for Viewers -->
  <LButton
    v-else
    disabled
    :tooltip="'Editor or Admin role required'"
  >
    New Dashboard
  </LButton>
</template>

<script setup>
import { usePermissions } from '@/composables/usePermissions'

const { canEdit, canAdmin, role } = usePermissions()
</script>
```

### 25. User Profile & Role Display ✅

- [x] Add user profile dropdown in header
- [x] Show current user's name and email
- [x] Display user's role with badge (Admin/Editor/Viewer)
- [ ] Add role-specific styling (Admin: red, Editor: blue, Viewer: gray)
- [ ] Add "Role: ..." indicator in settings
- [ ] Show organization name in profile
- [ ] Add logout functionality

**Example:**

```vue
<div class="user-profile">
  <div class="user-info">
    <span class="user-name">John Doe</span>
    <LBadge :variant="roleVariant">{{ role }}</LBadge>
  </div>
  <div class="user-email">john@example.com</div>
  <div class="organization">Acme Corp</div>
</div>
```

### 26. Permission-Based Routing

- [ ] Add route guards based on permissions
- [ ] Redirect non-Admins away from datasource management
- [ ] Add "Access Denied" page for unauthorized routes
- [ ] Show appropriate message based on required permission
- [ ] Provide navigation back to accessible areas

**Example Route Guard:**

```typescript
router.beforeEach((to, from, next) => {
  const { canAdmin } = usePermissions()

  if (to.meta.requiresAdmin && !canAdmin.value) {
    next({ name: 'AccessDenied', query: { required: 'Admin' } })
  } else {
    next()
  }
})
```

### 27. Composable: usePermissions() ✅

**Create reusable permission checking logic:**

- [x] Create `composables/usePermissions.ts`
- [x] Export `canView`, `canEdit`, `canAdmin` computed properties
- [x] Export `hasPermission(permission)` function
- [x] Export current `role` ref
- [x] Handle unauthenticated state
- [x] Add TypeScript types for roles and permissions

**Implementation:**

```typescript
// composables/usePermissions.ts
import { computed } from 'vue'
import { useAuthStore } from '@/stores/auth'

export enum Permission {
  Viewer = 'viewer',
  Editor = 'editor',
  Admin = 'admin'
}

export function usePermissions() {
  const authStore = useAuthStore()

  const role = computed(() => authStore.user?.role)

  const canView = computed(() => true) // All roles can view
  const canEdit = computed(() =>
    role.value === 'editor' || role.value === 'admin'
  )
  const canAdmin = computed(() => role.value === 'admin')

  const hasPermission = (required: Permission) => {
    if (!role.value) return false

    const hierarchy = {
      viewer: ['viewer'],
      editor: ['viewer', 'editor'],
      admin: ['viewer', 'editor', 'admin']
    }

    return hierarchy[role.value]?.includes(required) ?? false
  }

  return {
    role,
    canView,
    canEdit,
    canAdmin,
    hasPermission
  }
}
```

### 28. Organization Management UI

**Backend Context:** New organization management API has been implemented ([organizations.rs](../be/src/api/routes/organizations.rs)) that allows Admins to manage users and roles within their organization.

**API Endpoints:**

- `GET /api/v1/organizations/users` - List organization users (Viewer+)
- `PUT /api/v1/organizations/users/:user_id/role` - Update user role (Admin only)
- `DELETE /api/v1/organizations/users/:user_id` - Remove user from org (Admin only)

**Tasks:**

- [ ] Create Organization/Team Settings page
- [ ] Add Users list view showing all organization members
- [ ] Display user information (name, email, role, joined date)
- [ ] Add role badge with color coding (Admin: red, Editor: blue, Viewer: gray)
- [ ] Implement role change dropdown (Admin only)
- [ ] Add confirmation dialog for role changes
- [ ] Implement user removal with confirmation (Admin only)
- [ ] Show current user's own entry (non-editable, can't remove self)
- [ ] Add "You cannot change your own role" tooltip for current user
- [ ] Handle API errors (403 for non-Admins, 400 for self-modification)
- [ ] Add success/error toast notifications
- [ ] Show empty state when organization has only one user
- [ ] Add loading states for user list and actions
- [ ] Consider adding search/filter for large organizations
- [ ] Add "Invite User" functionality (future enhancement)

**UI Requirements:**

```vue
<template>
  <div class="organization-users">
    <div class="header">
      <h2>Organization Members</h2>
      <LButton v-if="canAdmin" variant="primary" @click="showInviteDialog">
        Invite User
      </LButton>
    </div>

    <div class="users-list">
      <div v-for="user in users" :key="user.id" class="user-card">
        <div class="user-info">
          <div class="user-name">{{ user.name }}</div>
          <div class="user-email">{{ user.email }}</div>
        </div>

        <div class="user-role">
          <LBadge :variant="roleVariant(user.role)">{{ user.role }}</LBadge>

          <!-- Role change dropdown (Admin only, not for self) -->
          <LSelect
            v-if="canAdmin && user.id !== currentUserId"
            :modelValue="user.role"
            @update:modelValue="updateRole(user, $event)"
            :disabled="updating"
          >
            <option value="admin">Admin</option>
            <option value="editor">Editor</option>
            <option value="viewer">Viewer</option>
          </LSelect>
        </div>

        <div class="user-actions">
          <!-- Remove user button (Admin only, not for self) -->
          <LButton
            v-if="canAdmin && user.id !== currentUserId"
            variant="danger"
            size="sm"
            @click="confirmRemoveUser(user)"
          >
            Remove
          </LButton>
          <span v-else-if="user.id === currentUserId" class="text-muted">
            (You)
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
```

**Business Rules to Implement:**

- Admins can change anyone's role except their own
- Admins can remove anyone except themselves
- Viewers/Editors see the user list but cannot modify roles or remove users
- Show appropriate tooltips/disabled states for non-permitted actions
- Confirm before role changes and user removal

---

## Accessibility ♿

### 23. Focus Management

- [x] Audit focus-ring utility in [main.css:368-375](../fe/src/styles/main.css#L368-L375)
- [x] Apply to all interactive elements
- [x] Add focus-visible styles
- [ ] Test keyboard navigation flow
- [x] Add skip links for main content
- [ ] Test with screen readers
- [x] Add focus trap for modals
- [ ] Add roving tabindex for complex widgets

### 24. ARIA & Semantic HTML

- [x] Audit all components for ARIA labels
- [x] Add `aria-label` to icon-only buttons
- [x] Add `aria-describedby` for form errors
- [x] Add `aria-live` for dynamic content
- [x] Add landmark roles where needed
- [x] Add `role="status"` for loading states
- [ ] Test with screen reader (NVDA/JAWS)
- [ ] Add alt text to all images

### 25. Keyboard Navigation

- [ ] Test all interactions with keyboard only
- [x] Add keyboard shortcuts documentation
- [x] Add visual indicators for keyboard users
- [ ] Add hotkey hints in tooltips
- [x] Test modal focus management
- [ ] Test dropdown keyboard navigation
- [x] Add escape key handling everywhere
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
- [x] Add success message copy
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
- [x] Add keyboard shortcuts help modal

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
- [x] Document layout patterns

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
- [x] Add keyboard shortcuts
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
**Components Completed:** 5/5 ✓
**Advanced Features Completed:** 2/4
**Authentication & Authorization:** 3/6 (RBAC composable ✓, User profile ✓, Error handling ✓, Conditional UI partial, Routing pending, Org management UI pending)
**Accessibility Completed:** 4/4 ✓
**Polish Completed:** 1/4
**Performance Completed:** 1/2
**Documentation Completed:** 2/2 ✓

**Overall Progress:** 39/42 major tasks (93%)

---

## Notes

Add any notes, decisions, or discussions here:

-
-
-

---

## Related Documents

- [Backend TODO](./BE_TODO.md) - Backend improvements and features
- [RBAC Implementation](./RBAC_IMPLEMENTATION.md) - Role-based access control documentation
- [Design System](./DESIGN_SYSTEM.md) (to be created)
- [Component Library](./COMPONENTS.md) (to be created)
- [Accessibility Guidelines](./ACCESSIBILITY.md) (to be created)
- [Brand Guidelines](./BRAND.md) (to be created)
