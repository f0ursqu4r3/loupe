<script setup lang="ts">
interface Props {
  text: string
}

defineProps<Props>()

const emit = defineEmits<{
  'update:text': [text: string]
}>()

function onInput(event: Event) {
  emit('update:text', (event.target as HTMLTextAreaElement).value)
}
</script>

<template>
  <div class="h-full min-h-0 grid grid-rows-[40px_1fr]">
    <div class="flex items-center px-2.5 py-2 border-b border-border font-semibold text-xs">
      <span>Note</span>
    </div>
    <textarea
      class="w-full h-full p-3 border-none outline-none resize-none bg-transparent text-sm leading-relaxed"
      :value="text"
      @input="onInput"
      placeholder="Write context, assumptions, conclusions..."
    ></textarea>
  </div>
</template>
