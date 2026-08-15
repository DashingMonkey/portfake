<script setup lang="ts">
import { onMounted, ref } from 'vue'

const props = defineProps<{
  message: string
  type: 'success' | 'error' | 'info'
  duration: number
}>()

const emit = defineEmits<{
  close: []
}>()

const visible = ref(false)

onMounted(() => {
  requestAnimationFrame(() => {
    visible.value = true
  })
  setTimeout(() => {
    visible.value = false
    setTimeout(() => emit('close'), 200)
  }, props.duration)
})

const iconClass = {
  success: 'text-success',
  error: 'text-error',
  info: 'text-accent',
}

const bgClass = {
  success: 'bg-surface-elevated border-success/30',
  error: 'bg-surface-elevated border-error/30',
  info: 'bg-surface-elevated border-accent/30',
}
</script>

<template>
  <div
    class="transition-all duration-200"
    :class="visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-2'"
  >
      <div
        class="flex items-center gap-2 px-4 py-2.5 rounded-lg border text-sm shadow-lg min-w-[240px]"
        :class="bgClass[type]"
      >
        <svg
          v-if="type === 'success'"
          class="w-4 h-4 shrink-0"
          :class="iconClass[type]"
          fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7"/>
        </svg>
        <svg
          v-else-if="type === 'error'"
          class="w-4 h-4 shrink-0"
          :class="iconClass[type]"
          fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"/>
        </svg>
        <svg
          v-else
          class="w-4 h-4 shrink-0"
          :class="iconClass[type]"
          fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        <span class="text-text-primary">{{ message }}</span>
      </div>
    </div>
</template>
