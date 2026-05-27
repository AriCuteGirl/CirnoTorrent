import { create } from 'zustand'
import { api, type TorrentStatus, type Settings, type GlobalStats } from '../api/client'

interface SpeedEntry {
  time: number
  download: number
  upload: number
}

interface StoreState {
  torrents: TorrentStatus[]
  stats: GlobalStats
  settings: Settings | null
  speedHistory: SpeedEntry[]
  selectedTorrentId: string | null
  wsConnected: boolean

  fetchTorrents: () => Promise<void>
  fetchSettings: () => Promise<void>
  connect: () => void
  addTorrent: (magnet: string, category?: string, tags?: string) => Promise<void>
  pauseTorrent: (id: string) => Promise<void>
  resumeTorrent: (id: string) => Promise<void>
  deleteTorrent: (id: string, deleteFiles?: boolean) => Promise<void>
  setCategory: (id: string, category: string) => Promise<void>
  toggleSequential: (id: string) => Promise<void>
  updateSettings: (settings: Settings) => Promise<void>
  selectTorrent: (id: string | null) => void
}

const defaultSettings: Settings = {
  downloads_dir: '',
  watch_dir: '',
  max_download_speed_kb: 0,
  max_upload_speed_kb: 0,
  max_connections: 200,
  max_connections_per_torrent: 50,
  dht_enabled: true,
  pex_enabled: true,
  lsd_enabled: true,
  utp_enabled: true,
  listen_port: 6881,
  webui_enabled: false,
  webui_port: 8080,
  webui_username: 'admin',
  webui_password_hash: '',
  auto_extract: false,
  extract_path: '',
  extract_passwords: '',
  rss_poll_interval_secs: 900,
  jackett_url: '',
  jackett_api_key: '',
  accent_color: 'blue',
  theme: 'dark',
  sequential_default: false,
  ratio_limit: 0,
  seeding_time_limit_mins: 0,
  notifications_enabled: true,
  blocklist_url: '',
}

export const useStore = create<StoreState>((set, get) => ({
  torrents: [],
  stats: {
    download_speed: 0,
    upload_speed: 0,
    active_peers: 0,
    dht_nodes: 0,
    total_downloaded: 0,
    total_uploaded: 0,
  },
  settings: null,
  speedHistory: [],
  selectedTorrentId: null,
  wsConnected: false,

  fetchTorrents: async () => {
    try {
      const torrents = await api.getTorrents()
      set({ torrents })
    } catch {}
  },

  fetchSettings: async () => {
    try {
      const settings = await api.getSettings()
      set({ settings })
      applyAccentColor(settings.accent_color)
      applyTheme(settings.theme)
    } catch {
      set({ settings: defaultSettings })
    }
  },

  connect: () => {
    const isTauri = typeof window !== 'undefined' && '__TAURI__' in window

    if (isTauri) {
      const tauri = (window as any).__TAURI__
      tauri.event.listen('tick', (event: any) => {
        const data = event.payload
        const now = Date.now()
        set((state) => ({
          torrents: data.torrents || state.torrents,
          stats: {
            ...state.stats,
            download_speed: data.global_download_speed || 0,
            upload_speed: data.global_upload_speed || 0,
          },
          speedHistory: [
            ...state.speedHistory.slice(-59),
            {
              time: now,
              download: data.global_download_speed || 0,
              upload: data.global_upload_speed || 0,
            },
          ],
        }))
      })
      return
    }

    const ws = api.createWebSocket((data) => {
      if (data.type === 'tick') {
        const now = Date.now()
        set((state) => ({
          torrents: data.data.torrents || state.torrents,
          stats: {
            ...state.stats,
            download_speed: data.data.global_download_speed || 0,
            upload_speed: data.data.global_upload_speed || 0,
          },
          speedHistory: [
            ...state.speedHistory.slice(-59),
            {
              time: now,
              download: data.data.global_download_speed || 0,
              upload: data.data.global_upload_speed || 0,
            },
          ],
          wsConnected: true,
        }))
      }
    })
    if (ws) set({ wsConnected: true })
  },

  addTorrent: async (magnet, category = '', tags = '') => {
    await api.addTorrent(magnet, category, tags)
    await get().fetchTorrents()
  },

  pauseTorrent: async (id) => {
    await api.pauseTorrent(id)
    await get().fetchTorrents()
  },

  resumeTorrent: async (id) => {
    await api.resumeTorrent(id)
    await get().fetchTorrents()
  },

  deleteTorrent: async (id, deleteFiles = false) => {
    await api.deleteTorrent(id, deleteFiles)
    await get().fetchTorrents()
  },

  setCategory: async (id, category) => {
    await api.setTorrentCategory(id, category)
    await get().fetchTorrents()
  },

  toggleSequential: async (id) => {
    await api.toggleSequential(id)
    await get().fetchTorrents()
  },

  updateSettings: async (settings) => {
    await api.updateSettings(settings)
    set({ settings })
    applyAccentColor(settings.accent_color)
    applyTheme(settings.theme)
  },

  selectTorrent: (id) => set({ selectedTorrentId: id }),
}))

function applyAccentColor(color: string) {
  const colors: Record<string, { main: string; glow: string }> = {
    blue: { main: '#00e5ff', glow: 'rgba(0, 229, 255, 0.35)' },
    purple: { main: '#d500f9', glow: 'rgba(213, 0, 249, 0.35)' },
    green: { main: '#00e676', glow: 'rgba(0, 230, 118, 0.35)' },
    red: { main: '#ff1744', glow: 'rgba(255, 23, 68, 0.35)' },
  }
  const c = colors[color] || colors.blue
  document.documentElement.style.setProperty('--accent-color', c.main)
  document.documentElement.style.setProperty('--accent-glow', c.glow)
}

function applyTheme(theme: string) {
  if (theme === 'dark') {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
  localStorage.setItem('cirnotorrent-theme', theme)
}
