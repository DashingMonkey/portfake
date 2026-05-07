const isDev = import.meta.env.DEV

export const logger = {
  error: (message: string, error: unknown) => {
    if (isDev) {
      console.error(message, error)
    }
  },
}
