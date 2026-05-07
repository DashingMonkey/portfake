<script setup lang="ts">
defineProps<{
  title: string
  message?: string
}>()
</script>

<template>
  <div class="flex flex-col items-center justify-center text-center p-6">
    <svg class="w-12 h-12 text-text-muted/30 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
      <path stroke-linecap="round" stroke-linejoin="round" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"/>
    </svg>
    <h3 class="text-sm font-medium text-text-secondary">{{ title }}</h3>
    <p v-if="message" class="text-xs text-text-muted mt-1 max-w-xs">{{ message }}</p>
  </div>
</template>
