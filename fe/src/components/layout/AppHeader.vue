<script setup lang="ts">
import { Bell, User } from 'lucide-vue-next'
import ThemeToggle from './ThemeToggle.vue'
import { LButton } from '@/components/ui'
import { useRouter } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'

const router = useRouter()

interface Props {
  title?: string
  back?: string
}

defineProps<Props>()
</script>

<template>
  <header class="h-16 flex items-center justify-between px-6 border-b border-border bg-surface">
    <!-- Page title / breadcrumbs -->
    <div class="flex items-center gap-4">
      <LButton v-if="back" variant="ghost" size="sm" @click="router.push({ name: back })" square>
        <ArrowLeft :size="16" />
      </LButton>

      <h1 v-if="title" class="text-xl font-semibold text-text">
        {{ title }}
      </h1>
      <slot name="left" />
    </div>

    <!-- Actions -->
    <slot name="actions" />

    <!-- User menu -->
    <div class="flex items-center gap-3">
      <ThemeToggle />

      <LButton variant="ghost" size="sm" square>
        <Bell :size="16" />
      </LButton>

      <LButton variant="ghost" size="sm" square>
        <User :size="16" />
      </LButton>
    </div>
  </header>
</template>
