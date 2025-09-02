import axios from 'axios'

export const http = axios.create({
  baseURL: import.meta.env.VITE_API_BASE || '/api',
  timeout: 15_000,
})

http.interceptors.response.use(
  (r) => r,
  (err) => {
    // Centralized error normalization or toast handling can live here
    throw err
  }
)