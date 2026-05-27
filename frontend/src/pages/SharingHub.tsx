import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { api, type SharedLinkRecord, type TunnelStatus } from '../api/client'
import { formatBytes } from '../lib/utils'
import {
  Globe, Link, Copy, Trash2, Loader2, Shield, Wifi, WifiOff,
} from 'lucide-react'

const pageVariants = {
  initial: { opacity: 0, x: 20 },
  animate: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: -20 },
}

export default function SharingHub() {
  const [links, setLinks] = useState<SharedLinkRecord[]>([])
  const [tunnel, setTunnel] = useState<TunnelStatus>({ active: false, url: null, error: null })
  const [loading, setLoading] = useState(true)
  const [tunnelLoading, setTunnelLoading] = useState(false)

  const fetch = async () => {
    try {
      const [l, t] = await Promise.all([api.getSharedLinks(), api.getTunnelStatus()])
      setLinks(l)
      setTunnel(t)
    } catch {}
    setLoading(false)
  }

  useEffect(() => { fetch() }, [])

  const handleStartTunnel = async () => {
    setTunnelLoading(true)
    try {
      const url = await api.startTunnel()
      setTunnel({ active: true, url, error: null })
    } catch (e: any) {
      setTunnel({ active: false, url: null, error: e.message })
    }
    setTunnelLoading(false)
  }

  const handleStopTunnel = async () => {
    await api.stopTunnel()
    setTunnel({ active: false, url: null, error: null })
  }

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text)
  }

  const handleDelete = async (id: string) => {
    await api.deleteSharedLink(id)
    setLinks(links.filter((l) => l.id !== id))
  }

  return (
    <motion.div
      variants={pageVariants}
      initial="initial"
      animate="animate"
      exit="exit"
      transition={{ duration: 0.3 }}
      className="space-y-6"
    >
      <h2 className="text-2xl font-bold">Sharing Hub</h2>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div className="glass-panel rounded-xl p-5 space-y-4">
          <div className="flex items-center gap-3">
            <Globe size={20} className="text-accent-neon" />
            <h3 className="font-semibold">Remote Tunnel</h3>
          </div>
          <div className="flex items-center gap-2">
            {tunnel.active ? (
              <Wifi size={16} className="text-emerald-400" />
            ) : (
              <WifiOff size={16} className="text-slate-500" />
            )}
            <span className="text-sm text-slate-400">
              {tunnel.active ? 'Active' : 'Inactive'}
            </span>
          </div>
          {tunnel.url && (
            <div className="flex items-center gap-2">
              <input
                readOnly
                value={tunnel.url}
                className="flex-1 px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-xs text-slate-300"
              />
              <button
                onClick={() => handleCopy(tunnel.url!)}
                className="p-2 rounded-lg hover:bg-white/10 text-slate-400"
              >
                <Copy size={14} />
              </button>
            </div>
          )}
          {tunnel.error && <p className="text-xs text-red-400">{tunnel.error}</p>}
          <div className="flex gap-2">
            {!tunnel.active ? (
              <button
                onClick={handleStartTunnel}
                disabled={tunnelLoading}
                className="px-4 py-2 rounded-lg bg-accent-neon/20 text-accent-neon text-sm font-medium hover:bg-accent-neon/30 disabled:opacity-50 flex items-center gap-2"
              >
                {tunnelLoading && <Loader2 size={14} className="animate-spin" />}
                Start Tunnel
              </button>
            ) : (
              <button
                onClick={handleStopTunnel}
                className="px-4 py-2 rounded-lg bg-red-500/20 text-red-400 text-sm font-medium hover:bg-red-500/30"
              >
                Stop Tunnel
              </button>
            )}
          </div>
        </div>

        <div className="glass-panel rounded-xl p-5 space-y-4">
          <div className="flex items-center gap-3">
            <Shield size={20} className="text-accent-neon" />
            <h3 className="font-semibold">UPnP Status</h3>
          </div>
          <p className="text-sm text-slate-400">
            UPnP port mapping allows automatic port forwarding for incoming peer connections.
          </p>
          <div className="flex items-center gap-2">
            <span className="px-2 py-1 rounded-full text-xs bg-slate-500/20 text-slate-400">
              Not configured
            </span>
          </div>
        </div>
      </div>

      <div className="glass-panel rounded-xl p-5">
        <div className="flex items-center justify-between mb-4">
          <h3 className="font-semibold flex items-center gap-2">
            <Link size={18} /> Shared Links ({links.length})
          </h3>
        </div>
        <div className="space-y-3">
          {links.map((link) => (
            <div key={link.id} className="flex items-center justify-between py-2 border-b border-white/5">
              <div className="flex-1 min-w-0">
                <p className="text-sm text-slate-200 truncate">{link.file_path}</p>
                <div className="flex items-center gap-3 text-xs text-slate-500 mt-1">
                  <span>Access: {link.access_count}</span>
                  <span>Bandwidth: {formatBytes(link.bandwidth_used_bytes)}</span>
                  {link.password_hash && <span className="text-amber-400">Password protected</span>}
                </div>
              </div>
              <div className="flex items-center gap-2 ml-4">
                <button
                  onClick={() => handleCopy(`${window.location.origin}/shared/download/${link.id}`)}
                  className="p-1.5 rounded hover:bg-white/10 text-slate-400"
                >
                  <Copy size={14} />
                </button>
                <button
                  onClick={() => handleDelete(link.id)}
                  className="p-1.5 rounded hover:bg-white/10 text-red-400"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
          ))}
          {links.length === 0 && !loading && (
            <p className="text-slate-500 text-sm text-center py-4">No shared links</p>
          )}
        </div>
      </div>
    </motion.div>
  )
}
