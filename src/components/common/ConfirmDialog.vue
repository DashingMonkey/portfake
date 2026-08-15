<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

defineProps<{
  visible: boolean
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  danger?: boolean
}>()

defineEmits<{
  confirm: []
  cancel: []
}>()
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity duration-150"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-150"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visible"
        class="fixed inset-0 z-90 flex items-center justify-center bg-black/40 backdrop-blur-xs"
        @click.self="$emit('cancel')"
        @keydown.esc="$emit('cancel')"
      >
        <div
          role="dialog"
          aria-modal="true"
          class="w-[360px] bg-surface-base border border-border-default rounded-xl shadow-2xl overflow-hidden"
        >
          <div class="px-5 py-4 border-b border-border-default flex items-center justify-between">
            <h2 class="text-sm font-semibold text-text-primary">{{ title }}</h2>
            <button
              @click="$emit('cancel')"
              class="w-6 h-6 flex items-center justify-center rounded-sm hover:bg-surface-elevated text-text-muted hover:text-text-primary transition-colors"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"/>
              </svg>
            </button>
          </div>
          <div class="px-5 py-4">
            <p class="text-sm text-text-secondary">{{ message }}</p>
          </div>
          <div class="px-5 py-3 border-t border-border-default flex justify-end gap-2">
            <button
              @click="$emit('cancel')"
              class="px-4 py-1.5 text-xs font-medium text-text-secondary hover:text-text-primary hover:bg-surface-elevated rounded-sm transition-colors"
            >
              {{ cancelText || t('common.cancel') }}
            </button>
            <button
              @click="$emit('confirm')"
              class="px-4 py-1.5 text-xs font-semibold text-white rounded-sm transition-colors"
              :class="danger ? 'bg-error hover:bg-error/90' : 'bg-accent hover:bg-accent/90'"
            >
              {{ confirmText || t('common.confirm') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
