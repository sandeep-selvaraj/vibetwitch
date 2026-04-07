const BASE = '/api'

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = localStorage.getItem('token')
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...((options.headers as Record<string, string>) || {}),
  }
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }

  const res = await fetch(`${BASE}${path}`, { ...options, headers })

  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }))
    throw new ApiError(res.status, body.error || 'Request failed')
  }

  if (res.status === 204) return undefined as T
  return res.json()
}

export class ApiError extends Error {
  status: number
  constructor(status: number, message: string) {
    super(message)
    this.status = status
  }
}

export const api = {
  // Auth
  register(data: { username: string; email: string; password: string; display_name: string }) {
    return request<{ token: string; user: User }>('/auth/register', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  },

  login(data: { email: string; password: string }) {
    return request<{ token: string; user: User }>('/auth/login', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  },

  getMe() {
    return request<User>('/auth/me')
  },

  // Users
  getUser(username: string) {
    return request<User>(`/users/${username}`)
  },

  updateProfile(data: { display_name: string; bio?: string; avatar_url?: string }) {
    return request<User>('/users/me', {
      method: 'PATCH',
      body: JSON.stringify(data),
    })
  },

  // Streams
  createStream(data: { title: string; description?: string; tags?: string[] }) {
    return request<StreamWithKey>('/streams', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  },

  getMyStream() {
    return request<StreamWithKey | null>('/streams/mine')
  },

  getLiveStreams() {
    return request<StreamPublic[]>('/streams/live')
  },

  getStream(id: string) {
    return request<Stream>(`/streams/${id}`)
  },

  getStreamByUsername(username: string) {
    return request<StreamPublic>(`/streams/by-username/${username}`)
  },

  updateStream(id: string, data: { title: string; description?: string; tags?: string[] }) {
    return request<Stream>(`/streams/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    })
  },

  goLive(id: string) {
    return request<{ rtmp_url: string; stream_key: string }>(`/streams/${id}/go-live`, {
      method: 'POST',
    })
  },

  endStream(id: string) {
    return request<Stream>(`/streams/${id}/end`, { method: 'POST' })
  },
}

// Types
export interface User {
  id: string
  username: string
  display_name: string
  avatar_url?: string
  bio?: string
  created_at: string
}

export interface Stream {
  id: string
  user_id: string
  title: string
  description?: string
  status: string
  hls_url?: string
  thumbnail_url?: string
  tags: string[]
  viewer_count: number
  started_at?: string
  created_at: string
}

export interface StreamWithKey extends Stream {
  stream_key_visible: string
}

export interface StreamPublic {
  id: string
  user_id: string
  title: string
  description?: string
  status: string
  hls_url?: string
  thumbnail_url?: string
  tags: string[]
  viewer_count: number
  started_at?: string
  username?: string
  display_name?: string
  avatar_url?: string
}
