export interface TorrentStatus {
  id: string
  info_hash: string
  name: string
  status: string
  progress: number
  download_speed: number
  upload_speed: number
  downloaded_bytes: number
  uploaded_bytes: number
  total_bytes: number
  peers_connected: number
  seeds_connected: number
  eta_secs: number
  category: string
  tags: string
  sequential: boolean
  save_path: string
  added_at: string
  files: FileEntry[]
  trackers: string[]
  piece_count: number
  piece_length: number
  piece_map: number[]
}

export interface FileEntry {
  index: number
  path: string
  size: number
  priority: string
  progress: number
}

export interface Settings {
  downloads_dir: string
  watch_dir: string
  max_download_speed_kb: number
  max_upload_speed_kb: number
  max_connections: number
  max_connections_per_torrent: number
  dht_enabled: boolean
  pex_enabled: boolean
  lsd_enabled: boolean
  utp_enabled: boolean
  listen_port: number
  webui_enabled: boolean
  webui_port: number
  webui_username: string
  webui_password_hash: string
  auto_extract: boolean
  extract_path: string
  extract_passwords: string
  rss_poll_interval_secs: number
  jackett_url: string
  jackett_api_key: string
  accent_color: string
  theme: string
  sequential_default: boolean
  ratio_limit: number
  seeding_time_limit_mins: number
  notifications_enabled: boolean
  blocklist_url: string
}

export interface ExtractionRecord {
  id: string
  torrent_id: string
  archive_path: string
  output_dir: string
  status: string
  progress: number
  password: string | null
  error_message: string | null
  started_at: string
  completed_at: string | null
}

export interface SharedLinkRecord {
  id: string
  torrent_hash: string
  file_path: string
  password_hash: string | null
  expiry_at: number | null
  created_at: string
  access_count: number
  bandwidth_used_bytes: number
}

export interface RssFeedRecord {
  id: string
  name: string
  url: string
  last_polled_at: string | null
  last_etag: string | null
}

export interface RssRuleRecord {
  id: string
  name: string
  pattern: string
  feed_id: string | null
  category: string
  save_path: string
  last_matched_at: string | null
}

export interface TunnelStatus {
  active: boolean
  url: string | null
  error: string | null
}

export interface SearchResult {
  title: string
  size: number
  seeds: number
  peers: number
  magnet_link: string
  indexer: string
}

export interface GlobalStats {
  download_speed: number
  upload_speed: number
  active_peers: number
  dht_nodes: number
  total_downloaded: number
  total_uploaded: number
}

const isTauri = (): boolean => {
  return typeof window !== 'undefined' && '__TAURI__' in window
}

let authToken: string | null = localStorage.getItem('cirnotorrent-token')

export function setAuthToken(token: string | null) {
  authToken = token
  if (token) {
    localStorage.setItem('cirnotorrent-token', token)
  } else {
    localStorage.removeItem('cirnotorrent-token')
  }
}

export function getAuthToken(): string | null {
  return authToken
}

async function httpFetch(path: string, options: RequestInit = {}): Promise<Response> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string> || {}),
  }
  if (authToken) {
    headers['Authorization'] = `Bearer ${authToken}`
  }
  return fetch(`/api${path}`, { ...options, headers })
}

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const tauri = (window as any).__TAURI__
  return tauri.core.invoke(cmd, args)
}

export const api = {
  async login(username: string, password: string): Promise<{ token: string; username: string }> {
    if (isTauri()) {
      return { token: 'desktop', username }
    }
    const res = await httpFetch('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    })
    if (!res.ok) throw new Error('Login failed')
    return res.json()
  },

  async getTorrents(): Promise<TorrentStatus[]> {
    if (isTauri()) return tauriInvoke('get_torrents')
    const res = await httpFetch('/torrents')
    return res.json()
  },

  async addTorrent(magnetOrUrl: string, category = '', tags = ''): Promise<string> {
    if (isTauri()) return tauriInvoke('add_torrent', { magnetOrUrl, category, tags })
    const res = await httpFetch('/torrent/add', {
      method: 'POST',
      body: JSON.stringify({ magnet_or_url: magnetOrUrl, category, tags }),
    })
    if (!res.ok) throw new Error('Failed to add torrent')
    return res.json()
  },

  async pauseTorrent(hash: string): Promise<void> {
    if (isTauri()) return tauriInvoke('pause_torrent', { hash })
    await httpFetch(`/torrent/${hash}/pause`, { method: 'POST' })
  },

  async resumeTorrent(hash: string): Promise<void> {
    if (isTauri()) return tauriInvoke('resume_torrent', { hash })
    await httpFetch(`/torrent/${hash}/resume`, { method: 'POST' })
  },

  async deleteTorrent(hash: string, deleteFiles = false): Promise<void> {
    if (isTauri()) return tauriInvoke('delete_torrent', { hash, deleteFiles })
    await httpFetch(`/torrent/${hash}/delete?delete_files=${deleteFiles}`, { method: 'DELETE' })
  },

  async setTorrentLimit(hash: string, downloadKb: number, uploadKb: number): Promise<void> {
    if (isTauri()) return tauriInvoke('set_torrent_limit', { hash, downloadKb, uploadKb })
    await httpFetch(`/torrent/${hash}/limit`, {
      method: 'POST',
      body: JSON.stringify({ download_kb: downloadKb, upload_kb: uploadKb }),
    })
  },

  async setTorrentCategory(hash: string, category: string): Promise<void> {
    if (isTauri()) return tauriInvoke('set_torrent_category', { hash, category })
    await httpFetch(`/torrent/${hash}/category`, {
      method: 'POST',
      body: JSON.stringify({ category }),
    })
  },

  async toggleSequential(hash: string): Promise<boolean> {
    if (isTauri()) return tauriInvoke('toggle_sequential_mode', { hash })
    const res = await httpFetch(`/torrent/${hash}/sequential`, { method: 'POST' })
    return res.json()
  },

  async getSettings(): Promise<Settings> {
    if (isTauri()) return tauriInvoke('get_settings')
    const res = await httpFetch('/settings')
    return res.json()
  },

  async updateSettings(settings: Settings): Promise<void> {
    if (isTauri()) return tauriInvoke('update_settings', { settings })
    await httpFetch('/settings', {
      method: 'PUT',
      body: JSON.stringify(settings),
    })
  },

  async getExtractionQueue(): Promise<ExtractionRecord[]> {
    if (isTauri()) return tauriInvoke('get_extraction_queue')
    const res = await httpFetch('/extraction/queue')
    return res.json()
  },

  async submitExtractionPassword(id: string, password: string): Promise<void> {
    if (isTauri()) return tauriInvoke('submit_extraction_password', { id, password })
    await httpFetch(`/extraction/${id}/password`, {
      method: 'POST',
      body: JSON.stringify({ password }),
    })
  },

  async getSharedLinks(): Promise<SharedLinkRecord[]> {
    if (isTauri()) return tauriInvoke('get_shared_links')
    const res = await httpFetch('/sharing/links')
    return res.json()
  },

  async createSharedLink(torrentHash: string, filePath: string, password?: string, expiryMins?: number): Promise<SharedLinkRecord> {
    if (isTauri()) return tauriInvoke('create_shared_link', { torrentHash, filePath, password, expiryMins })
    const res = await httpFetch('/sharing/links', {
      method: 'POST',
      body: JSON.stringify({ torrent_hash: torrentHash, file_path: filePath, password, expiry_mins: expiryMins }),
    })
    return res.json()
  },

  async deleteSharedLink(id: string): Promise<void> {
    if (isTauri()) return tauriInvoke('delete_shared_link', { id })
    await httpFetch(`/sharing/links/${id}`, { method: 'DELETE' })
  },

  async getTunnelStatus(): Promise<TunnelStatus> {
    if (isTauri()) return tauriInvoke('get_tunnel_status')
    const res = await httpFetch('/sharing/tunnel')
    return res.json()
  },

  async startTunnel(): Promise<string> {
    if (isTauri()) return tauriInvoke('start_tunnel')
    const res = await httpFetch('/sharing/tunnel', { method: 'POST' })
    return res.json()
  },

  async stopTunnel(): Promise<void> {
    if (isTauri()) return tauriInvoke('stop_tunnel')
    await httpFetch('/sharing/tunnel', { method: 'DELETE' })
  },

  async getRssFeeds(): Promise<RssFeedRecord[]> {
    if (isTauri()) return tauriInvoke('get_rss_feeds')
    const res = await httpFetch('/rss/feeds')
    return res.json()
  },

  async addRssFeed(name: string, url: string): Promise<string> {
    if (isTauri()) return tauriInvoke('add_rss_feed', { name, url })
    const res = await httpFetch('/rss/feeds', {
      method: 'POST',
      body: JSON.stringify({ name, url }),
    })
    return res.json()
  },

  async deleteRssFeed(id: string): Promise<void> {
    if (isTauri()) return tauriInvoke('delete_rss_feed', { id })
    await httpFetch(`/rss/feeds/${id}`, { method: 'DELETE' })
  },

  async getRssRules(): Promise<RssRuleRecord[]> {
    if (isTauri()) return tauriInvoke('get_rss_rules')
    const res = await httpFetch('/rss/rules')
    return res.json()
  },

  async addRssRule(name: string, pattern: string, feedId: string | null, category: string, savePath: string): Promise<string> {
    if (isTauri()) return tauriInvoke('add_rss_rule', { name, pattern, feedId, category, savePath })
    const res = await httpFetch('/rss/rules', {
      method: 'POST',
      body: JSON.stringify({ name, pattern, feed_id: feedId, category, save_path: savePath }),
    })
    return res.json()
  },

  async deleteRssRule(id: string): Promise<void> {
    if (isTauri()) return tauriInvoke('delete_rss_rule', { id })
    await httpFetch(`/rss/rules/${id}`, { method: 'DELETE' })
  },

  async searchTorrents(query: string): Promise<SearchResult[]> {
    if (isTauri()) return tauriInvoke('search_torrents', { query })
    const res = await httpFetch(`/search?query=${encodeURIComponent(query)}`)
    return res.json()
  },

  async getStats(): Promise<GlobalStats> {
    const res = await httpFetch('/stats')
    return res.json()
  },

  createWebSocket(onMessage: (data: any) => void): WebSocket | null {
    if (isTauri()) return null
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const ws = new WebSocket(`${protocol}//${window.location.host}/api/ws?token=${authToken}`)
    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)
        onMessage(data)
      } catch {}
    }
    ws.onclose = () => {
      setTimeout(() => {
        api.createWebSocket(onMessage)
      }, 3000)
    }
    return ws
  },
}
