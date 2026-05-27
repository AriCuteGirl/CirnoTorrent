import { useState, useCallback, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { motion } from 'framer-motion'
import { useStore } from '../store'
import { formatBytes, formatSpeed, formatEta, formatProgress, cn } from '../lib/utils'
import {
  Pause, Play, Trash2, Plus, ArrowUpDown,
} from 'lucide-react'

const pageVariants = {
  initial: { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -20 },
}

type SortKey = 'name' | 'status' | 'progress' | 'download_speed' | 'total_bytes' | 'eta_secs'

export default function TorrentList() {
  const torrents = useStore((s) => s.torrents)
  const pauseTorrent = useStore((s) => s.pauseTorrent)
  const resumeTorrent = useStore((s) => s.resumeTorrent)
  const deleteTorrent = useStore((s) => s.deleteTorrent)
  const addTorrent = useStore((s) => s.addTorrent)
  const navigate = useNavigate()

  const [sortKey, setSortKey] = useState<SortKey>('name')
  const [sortAsc, setSortAsc] = useState(true)
  const [filter, setFilter] = useState('')
  const [showAddModal, setShowAddModal] = useState(false)
  const [magnetInput, setMagnetInput] = useState('')
  const [dragOver, setDragOver] = useState(false)
  const [selectedId, setSelectedId] = useState<string | null>(null)

  const sorted = [...torrents]
    .filter((t) => t.name.toLowerCase().includes(filter.toLowerCase()))
    .sort((a, b) => {
      const av = a[sortKey]
      const bv = b[sortKey]
      if (typeof av === 'string' && typeof bv === 'string') {
        return sortAsc ? av.localeCompare(bv) : bv.localeCompare(av)
      }
      return sortAsc ? (av as number) - (bv as number) : (bv as number) - (av as number)
    })

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) setSortAsc(!sortAsc)
    else { setSortKey(key); setSortAsc(true) }
  }

  const handleAdd = async () => {
    if (!magnetInput.trim()) return
    try {
      await addTorrent(magnetInput.trim())
      setMagnetInput('')
      setShowAddModal(false)
    } catch {}
  }

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setDragOver(false)
    const text = e.dataTransfer.getData('text/plain') || e.dataTransfer.getData('text/uri-list')
    if (text && (text.startsWith('magnet:') || text.endsWith('.torrent'))) {
      addTorrent(text)
    }
  }, [addTorrent])

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === ' ' && selectedId) {
        e.preventDefault()
        const t = torrents.find((t) => t.id === selectedId)
        if (t) {
          t.status === 'paused' ? resumeTorrent(selectedId) : pauseTorrent(selectedId)
        }
      }
      if (e.key === 'Delete' && selectedId) {
        deleteTorrent(selectedId)
        setSelectedId(null)
      }
      if (e.ctrlKey && e.key === 'l') {
        e.preventDefault()
        setShowAddModal(true)
      }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [selectedId, torrents, pauseTorrent, resumeTorrent, deleteTorrent])

  return (
    <motion.div
      variants={pageVariants}
      initial="initial"
      animate="animate"
      exit="exit"
      transition={{ duration: 0.3 }}
      className="space-y-4"
    >
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">Torrents</h2>
        <button
          onClick={() => setShowAddModal(true)}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-accent-neon/20 text-accent-neon hover:bg-accent-neon/30 transition text-sm font-medium"
        >
          <Plus size={16} /> Add Torrent
        </button>
      </div>

      <div
        onDragOver={(e) => { e.preventDefault(); setDragOver(true) }}
        onDragLeave={() => setDragOver(false)}
        onDrop={handleDrop}
        className={cn(
          'relative rounded-xl border-2 border-dashed transition-all',
          dragOver ? 'border-accent-neon glow-border bg-accent-neon/5' : 'border-transparent'
        )}
      >
        <input
          type="text"
          placeholder="Filter torrents..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="w-full px-4 py-2.5 rounded-lg bg-white/5 border border-white/10 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:border-accent-neon/50 mb-3"
        />

        <div className="glass-panel rounded-xl overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-white/5 text-slate-400 text-xs">
                {([
                  ['name', 'Name'],
                  ['status', 'Status'],
                  ['progress', 'Progress'],
                  ['download_speed', 'Speed'],
                  ['total_bytes', 'Size'],
                  ['eta_secs', 'ETA'],
                ] as [SortKey, string][]).map(([key, label]) => (
                  <th
                    key={key}
                    onClick={() => toggleSort(key)}
                    className="px-4 py-3 text-left font-medium cursor-pointer hover:text-slate-200"
                  >
                    <span className="flex items-center gap-1">
                      {label}
                      {sortKey === key && <ArrowUpDown size={12} />}
                    </span>
                  </th>
                ))}
                <th className="px-4 py-3 text-left font-medium">Actions</th>
              </tr>
            </thead>
            <tbody>
              {sorted.map((t, i) => (
                <motion.tr
                  key={t.id}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: i * 0.03 }}
                  onClick={() => { setSelectedId(t.id); navigate(`/torrents/${t.id}`) }}
                  className={cn(
                    'border-b border-white/5 cursor-pointer transition-colors',
                    selectedId === t.id ? 'bg-accent-neon/10' : 'hover:bg-white/5'
                  )}
                >
                  <td className="px-4 py-3 max-w-[300px] truncate text-slate-200">
                    {t.name || t.info_hash.slice(0, 8)}
                    {t.category && (
                      <span className="ml-2 px-1.5 py-0.5 rounded text-[10px] bg-accent-neon/20 text-accent-neon">
                        {t.category}
                      </span>
                    )}
                  </td>
                  <td className="px-4 py-3">
                    <StatusBadge status={t.status} />
                  </td>
                  <td className="px-4 py-3 w-40">
                    <div className="flex items-center gap-2">
                      <div className="flex-1 h-1.5 bg-white/10 rounded-full overflow-hidden">
                        <div
                          className="h-full bg-accent-neon shimmer-bar rounded-full transition-all"
                          style={{ width: `${t.progress}%` }}
                        />
                      </div>
                      <span className="text-xs text-slate-400 w-12 text-right">
                        {formatProgress(t.progress)}
                      </span>
                    </div>
                  </td>
                  <td className="px-4 py-3 text-xs">
                    <span className="text-emerald-400">↓{formatSpeed(t.download_speed)}</span>
                    <span className="text-blue-400 ml-2">↑{formatSpeed(t.upload_speed)}</span>
                  </td>
                  <td className="px-4 py-3 text-xs text-slate-400">
                    {formatBytes(t.total_bytes)}
                  </td>
                  <td className="px-4 py-3 text-xs text-slate-400">
                    {formatEta(t.eta_secs)}
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-1" onClick={(e) => e.stopPropagation()}>
                      {t.status === 'paused' ? (
                        <button onClick={() => resumeTorrent(t.id)} className="p-1.5 rounded hover:bg-white/10 text-emerald-400">
                          <Play size={14} />
                        </button>
                      ) : (
                        <button onClick={() => pauseTorrent(t.id)} className="p-1.5 rounded hover:bg-white/10 text-amber-400">
                          <Pause size={14} />
                        </button>
                      )}
                      <button onClick={() => deleteTorrent(t.id)} className="p-1.5 rounded hover:bg-white/10 text-red-400">
                        <Trash2 size={14} />
                      </button>
                    </div>
                  </td>
                </motion.tr>
              ))}
              {sorted.length === 0 && (
                <tr>
                  <td colSpan={7} className="px-4 py-12 text-center text-slate-500">
                    No torrents. Drag & drop a .torrent file or magnet link, or click Add.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {showAddModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50" onClick={() => setShowAddModal(false)}>
          <motion.div
            initial={{ scale: 0.95, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            className="glass-panel rounded-xl p-6 w-full max-w-md"
            onClick={(e) => e.stopPropagation()}
          >
            <h3 className="text-lg font-bold mb-4">Add Torrent</h3>
            <textarea
              value={magnetInput}
              onChange={(e) => setMagnetInput(e.target.value)}
              placeholder="Paste magnet link or torrent URL..."
              className="w-full h-24 px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:border-accent-neon/50 resize-none"
              autoFocus
            />
            <div className="flex justify-end gap-2 mt-4">
              <button onClick={() => setShowAddModal(false)} className="px-4 py-2 rounded-lg text-sm text-slate-400 hover:text-slate-200">
                Cancel
              </button>
              <button onClick={handleAdd} className="px-4 py-2 rounded-lg bg-accent-neon/20 text-accent-neon text-sm font-medium hover:bg-accent-neon/30">
                Add
              </button>
            </div>
          </motion.div>
        </div>
      )}
    </motion.div>
  )
}

function StatusBadge({ status }: { status: string }) {
  const colors: Record<string, string> = {
    downloading: 'bg-emerald-500/20 text-emerald-400',
    seeding: 'bg-blue-500/20 text-blue-400',
    paused: 'bg-amber-500/20 text-amber-400',
    queued: 'bg-slate-500/20 text-slate-400',
    completed: 'bg-purple-500/20 text-purple-400',
    error: 'bg-red-500/20 text-red-400',
  }
  return (
    <span className={cn('px-2 py-0.5 rounded-full text-[10px] font-medium', colors[status] || colors.queued)}>
      {status}
    </span>
  )
}
