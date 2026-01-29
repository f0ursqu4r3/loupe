/// <reference types="vite/client" />

declare module 'splitpanes' {
  import type { DefineComponent } from 'vue'
  export const Splitpanes: DefineComponent<{
    horizontal?: boolean
    pushOtherPanes?: boolean
    dblClickSplitter?: boolean
    firstSplitter?: boolean
  }, {}, {}, {}, {}, {}, {}, {
    ready: () => void
    resize: (panes: { min: number; max: number; size: number }[]) => void
    resized: (panes: { min: number; max: number; size: number }[]) => void
    'pane-click': (pane: { min: number; max: number; size: number }) => void
    'pane-maximize': (pane: { min: number; max: number; size: number }) => void
    'pane-add': (payload: { index: number; panes: { min: number; max: number; size: number }[] }) => void
    'pane-remove': (payload: { removed: { min: number; max: number; size: number }; panes: { min: number; max: number; size: number }[] }) => void
    'splitter-click': (payload: { index: number; next: { min: number; max: number; size: number } }) => void
  }>
  export const Pane: DefineComponent<{
    size?: number
    minSize?: number
    maxSize?: number
  }>
}
