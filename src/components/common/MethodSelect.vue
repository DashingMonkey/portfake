<script setup lang="ts">
import { Listbox, ListboxButton, ListboxOptions, ListboxOption } from '@headlessui/vue'
import { computed } from 'vue'

type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'OPTIONS'

const props = defineProps<{
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [method: string]
}>()

const methods: HttpMethod[] = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS']

const methodColors: Record<string, string> = {
  GET: 'text-success',
  POST: 'text-info',
  PUT: 'text-warning',
  DELETE: 'text-error',
  PATCH: 'text-purple-400',
  OPTIONS: 'text-text-muted',
}

const methodBgColors: Record<string, string> = {
  GET: 'bg-success/10',
  POST: 'bg-info/10',
  PUT: 'bg-warning/10',
  DELETE: 'bg-error/10',
  PATCH: 'bg-purple-400/10',
  OPTIONS: 'bg-surface-base',
}

function handleUpdate(method: HttpMethod) {
  emit('update:modelValue', method)
}

const currentMethod = computed(() => props.modelValue.toUpperCase())
</script>

<template>
  <Listbox :model-value="modelValue.toUpperCase()" @update:model-value="handleUpdate">
    <div class="relative">
      <ListboxButton
        class="flex items-center gap-2 px-2 py-1.5 text-xs font-semibold rounded-l border border-border-default bg-surface-base focus:outline-hidden min-w-[80px]"
        :class="[methodBgColors[currentMethod] || methodBgColors.GET, methodColors[currentMethod] || methodColors.GET]"
      >
        <span class="font-mono text-[11px]">{{ modelValue.toUpperCase() }}</span>
        <svg class="w-4 h-4 ml-auto opacity-60" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </ListboxButton>

      <transition
        leave-active-class="transition duration-100 ease-in"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <ListboxOptions
          class="absolute z-10 mt-1 w-full bg-surface-base shadow-xl max-h-60 rounded-md border border-border-default focus:outline-hidden overflow-hidden"
        >
          <ListboxOption
            v-for="method in methods"
            :key="method"
            :value="method"
            v-slot="{ active, selected }"
          >
            <li
              class="cursor-pointer select-none relative py-2 pl-3 pr-9 flex items-center gap-2 transition-colors duration-100 text-xs"
              :class="[
                active ? 'bg-accent/10 text-text-primary' : 'text-text-secondary',
                selected ? 'font-semibold' : 'font-normal'
              ]"
            >
              <span :class="methodColors[method]" class="font-mono font-semibold w-16">{{ method }}</span>
              <span v-if="selected" class="ml-auto text-accent">
                <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              </span>
            </li>
          </ListboxOption>
        </ListboxOptions>
      </transition>
    </div>
  </Listbox>
</template>
