import { h, render } from 'vue'
import Toast from '../components/common/Toast.vue'

export interface ToastOptions {
  message: string
  type?: 'success' | 'error' | 'info'
  duration?: number
}

// Singleton container for toasts
let container: HTMLElement | null = null

function ensureContainer(): HTMLElement {
  if (!container) {
    container = document.createElement('div')
    container.id = 'toast-container'
    document.body.appendChild(container)
  }
  return container
}

export function useToast() {
  function show(options: ToastOptions) {
    const el = document.createElement('div')
    el.className = 'toast-item'
    ensureContainer().appendChild(el)

    const vnode = h(Toast, {
      message: options.message,
      type: options.type ?? 'info',
      duration: options.duration ?? 3000,
      onClose: () => {
        render(null, el)
        el.remove()
      },
    })

    render(vnode, el)
  }

  return { show }
}
