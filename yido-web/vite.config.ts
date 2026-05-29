import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'

export default defineConfig({
  plugins: [solid()],
  server: {
    fs: {
      allow: [fileURLToPath(new URL('..', import.meta.url))],
    },
  },
})
