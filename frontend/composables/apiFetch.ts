import { $fetch } from 'ofetch'
import { useAuthStore } from '~/stores/auth'
import { useRuntimeConfig } from '#app'
import { useRouter } from 'vue-router';

export function apiFetch(url, options = {}) {
  const authStore = useAuthStore()
  const router = useRouter();
  const config = useRuntimeConfig()
  const apiBase = config.public.apiBase
  const headers = {
    ...options.headers,
  }
  if (authStore.accessToken) {
    headers['Authorization'] = `Bearer ${authStore.accessToken}`
  }

  console.log('Requesting:', `${apiBase}${url}`, 'Method:', options.method || 'GET', 'Headers:', headers, 'Body:', options.body);

  return $fetch(`${apiBase}${url}`, {
    ...options,
    headers,
    credentials: 'include',
  }).catch(err => {
    console.error('Fetch error:', err);
    console.log('Error details:', {
      status: err.response?.status,
      data: err.response?._data,
      headers: err.response?.headers,
    });
    if (err.response?.status === 401) {
      authStore.clearAccessToken();
      router.push('/login');
      return Promise.reject(new Error('Unauthorized - Redirecting to login'));
    }

    throw err;
  });
}