<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, shallowRef } from 'vue'
import * as monaco from 'monaco-editor'
import { format as formatSql } from 'sql-formatter'

interface Props {
  modelValue: string
  readonly?: boolean
  minimap?: boolean
  lineNumbers?: boolean
  height?: string
}

const props = withDefaults(defineProps<Props>(), {
  readonly: false,
  minimap: false,
  lineNumbers: true,
  height: '300px',
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  run: []
}>()

const containerRef = ref<HTMLDivElement | null>(null)
const editor = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null)

// Get theme from document
function getMonacoTheme(): string {
  return document.documentElement.classList.contains('dark') ? 'loupe-dark' : 'loupe-light'
}

// Define custom themes
function defineThemes() {
  monaco.editor.defineTheme('loupe-light', {
    base: 'vs',
    inherit: true,
    rules: [
      { token: 'keyword', foreground: '6366f1', fontStyle: 'bold' },
      { token: 'string', foreground: '16a34a' },
      { token: 'number', foreground: 'ea580c' },
      { token: 'comment', foreground: '9ca3af', fontStyle: 'italic' },
      { token: 'operator', foreground: 'dc2626' },
      { token: 'identifier', foreground: '1f2937' },
    ],
    colors: {
      'editor.background': '#ffffff',
      'editor.foreground': '#1f2937',
      'editor.lineHighlightBackground': '#f3f4f6',
      'editor.selectionBackground': '#e0e7ff',
      'editorCursor.foreground': '#6366f1',
      'editorLineNumber.foreground': '#9ca3af',
      'editorLineNumber.activeForeground': '#4b5563',
    },
  })

  monaco.editor.defineTheme('loupe-dark', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      { token: 'keyword', foreground: 'a5b4fc', fontStyle: 'bold' },
      { token: 'string', foreground: '4ade80' },
      { token: 'number', foreground: 'fb923c' },
      { token: 'comment', foreground: '6b7280', fontStyle: 'italic' },
      { token: 'operator', foreground: 'f87171' },
      { token: 'identifier', foreground: 'f3f4f6' },
    ],
    colors: {
      'editor.background': '#1a1a2e',
      'editor.foreground': '#f3f4f6',
      'editor.lineHighlightBackground': '#262640',
      'editor.selectionBackground': '#4338ca50',
      'editorCursor.foreground': '#818cf8',
      'editorLineNumber.foreground': '#6b7280',
      'editorLineNumber.activeForeground': '#9ca3af',
    },
  })
}

// Watch for theme changes
const themeObserver = new MutationObserver(() => {
  if (editor.value) {
    monaco.editor.setTheme(getMonacoTheme())
  }
})

onMounted(() => {
  if (!containerRef.value) return

  defineThemes()

  editor.value = monaco.editor.create(containerRef.value, {
    value: props.modelValue,
    language: 'sql',
    theme: getMonacoTheme(),
    readOnly: props.readonly,
    minimap: { enabled: props.minimap },
    lineNumbers: props.lineNumbers ? 'on' : 'off',
    fontSize: 14,
    fontFamily: 'JetBrains Mono, Menlo, Monaco, Consolas, monospace',
    fontLigatures: true,
    tabSize: 2,
    wordWrap: 'on',
    scrollBeyondLastLine: false,
    automaticLayout: true,
    padding: { top: 12, bottom: 12 },
    renderLineHighlight: 'line',
    cursorBlinking: 'smooth',
    cursorSmoothCaretAnimation: 'on',
    smoothScrolling: true,
    bracketPairColorization: { enabled: true },
    guides: {
      bracketPairs: true,
      indentation: true,
    },
    suggest: {
      showKeywords: true,
      showSnippets: true,
    },
  })

  // Listen for content changes
  editor.value.onDidChangeModelContent(() => {
    const value = editor.value?.getValue() ?? ''
    emit('update:modelValue', value)
  })

  // Add keyboard shortcut for running query (Cmd/Ctrl + Enter)
  editor.value.addAction({
    id: 'run-query',
    label: 'Run Query',
    keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter],
    run: () => {
      emit('run')
    },
  })

  // Add keyboard shortcut for formatting (Cmd/Ctrl + I)
  editor.value.addAction({
    id: 'format-sql',
    label: 'Format SQL',
    keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyI],
    contextMenuGroupId: 'modification',
    contextMenuOrder: 1.5,
    run: (ed) => {
      const position = ed.getPosition()
      const value = ed.getValue()
      const formatted = formatSql(value, {
        language: 'postgresql',
        tabWidth: 2,
        keywordCase: 'upper',
        linesBetweenQueries: 2,
      })
      ed.setValue(formatted)
      if (position) {
        // Clamp position to new content bounds
        const model = ed.getModel()
        if (model) {
          const maxLine = model.getLineCount()
          const line = Math.min(position.lineNumber, maxLine)
          const maxCol = model.getLineMaxColumn(line)
          const col = Math.min(position.column, maxCol)
          ed.setPosition({ lineNumber: line, column: col })
        }
      }
    },
  })

  // Observe theme changes
  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['class'],
  })
})

// Sync external modelValue changes to editor
watch(
  () => props.modelValue,
  (newValue) => {
    if (editor.value && editor.value.getValue() !== newValue) {
      editor.value.setValue(newValue)
    }
  },
)

onBeforeUnmount(() => {
  themeObserver.disconnect()
  editor.value?.dispose()
})

// Expose methods for parent components
defineExpose({
  focus: () => editor.value?.focus(),
  format: () => {
    if (editor.value) {
      const position = editor.value.getPosition()
      const value = editor.value.getValue()
      const formatted = formatSql(value, {
        language: 'postgresql',
        tabWidth: 2,
        keywordCase: 'upper',
        linesBetweenQueries: 2,
      })
      editor.value.setValue(formatted)
      if (position) {
        // Clamp position to new content bounds
        const model = editor.value.getModel()
        if (model) {
          const maxLine = model.getLineCount()
          const line = Math.min(position.lineNumber, maxLine)
          const maxCol = model.getLineMaxColumn(line)
          const col = Math.min(position.column, maxCol)
          editor.value.setPosition({ lineNumber: line, column: col })
        }
      }
    }
  },
  getSelectedText: () => {
    const selection = editor.value?.getSelection()
    if (selection && editor.value) {
      return editor.value.getModel()?.getValueInRange(selection) ?? ''
    }
    return ''
  },
})
</script>

<template>
  <div
    ref="containerRef"
    class="rounded-md border border-border overflow-hidden"
    :style="{ height }"
  />
</template>
