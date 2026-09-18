import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// В разработке фронтенд ходит к ядру (`stratum play … --port 8765`) через прокси;
// в сборке ядро само отдаёт app/dist.
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:8765',
      '/frame': 'http://127.0.0.1:8765',
      '/event': 'http://127.0.0.1:8765',
    },
  },
  build: { chunkSizeWarningLimit: 4000 },
})
