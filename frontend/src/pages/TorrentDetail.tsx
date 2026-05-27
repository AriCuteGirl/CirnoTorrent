import { useParams, useNavigate } from 'react-router-dom'
import { motion } from 'framer-motion'
import { useStore } from '../store'
import { formatBytes, formatSpeed, formatEta, formatProgress } from '../lib/utils'
import { ArrowLeft, Pause, Play, Trash2, FolderTree } from 'lucide-react'

const pageVariants = {
  initial: { opacity: 0, x: 20 },
  animate: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: -20 },
}

export default function TorrentDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const torrents = useStore((s) => s.torrents)
  const pauseTorrent = useStore((s) => s.pauseTorrent)
  const resumeTorrent = useStore((s) => s.resumeTorrent)
  const deleteTorrent = useStore((s) => s.deleteTorrent)

  const torrent = torrents.find((t) => t.id === id)

  if (!torrent) {
    return (
      <div className="flex items-center justify-center h-64 text-slate-500">
        Torrent not found
      </div>
    )
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
      <div className="flex items-center gap-4">
        <button onClick={() => navigate('/torrents')} className="p-2 rounded-lg hover:bg-white/10">
          <ArrowLeft size={18} />
        </button>
        <h2 className="text-xl font-bold truncate flex-1">{torrent.name}</h2>
        <div className="flex items-center gap-2">
          {torrent.status === 'paused' ? (
            <button onClick={() => resumeTorrent(torrent.id)} className="p-2 rounded-lg bg-emerald-500/20 text-emerald-400 hover:bg-emerald-500/30">
              <Play size={16} />
            </button>
          ) : (
            <button onClick={() => pauseTorrent(torrent.id)} className="p-2 rounded-lg bg-amber-500/20 text-amber-400 hover:bg-amber-500/30">
              <Pause size={16} />
            </button>
          )}
          <button onClick={() => { deleteTorrent(torrent.id); navigate('/torrents') }} className="p-2 rounded-lg bg-red-500/20 text-red-400 hover:bg-red-500/30">
            <Trash2 size={16} />
          </button>
        </div>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <InfoCard label="Status" value={torrent.status} />
        <InfoCard label="Progress" value={formatProgress(torrent.progress)} />
        <InfoCard label="Download" value={formatSpeed(torrent.download_speed)} />
        <InfoCard label="Upload" value={formatSpeed(torrent.upload_speed)} />
        <InfoCard label="Size" value={formatBytes(torrent.total_bytes)} />
        <InfoCard label="Downloaded" value={formatBytes(torrent.downloaded_bytes)} />
        <InfoCard label="Uploaded" value={formatBytes(torrent.uploaded_bytes)} />
        <InfoCard label="ETA" value={formatEta(torrent.eta_secs)} />
        <InfoCard label="Peers" value={torrent.peers_connected.toString()} />
        <InfoCard label="Seeds" value={torrent.seeds_connected.toString()} />
        <InfoCard label="Category" value={torrent.category || 'None'} />
        <InfoCard label="Sequential" value={torrent.sequential ? 'Yes' : 'No'} />
      </div>

      <div className="glass-panel rounded-xl p-5">
        <h3 className="text-sm font-semibold text-slate-400 mb-4 flex items-center gap-2">
          <FolderTree size={16} /> Files ({torrent.files.length})
        </h3>
        <div className="space-y-2 max-h-64 overflow-y-auto">
          {torrent.files.map((file) => (
            <div key={file.index} className="flex items-center justify-between text-sm py-1.5 border-b border-white/5">
              <span className="truncate max-w-[60%] text-slate-300">{file.path}</span>
              <div className="flex items-center gap-3 text-xs text-slate-400">
                <span>{formatBytes(file.size)}</span>
                <span className="px-1.5 py-0.5 rounded bg-white/5">{file.priority}</span>
              </div>
            </div>
          ))}
          {torrent.files.length === 0 && (
            <p className="text-slate-500 text-sm">No file information available</p>
          )}
        </div>
      </div>

      {torrent.trackers.length > 0 && (
        <div className="glass-panel rounded-xl p-5">
          <h3 className="text-sm font-semibold text-slate-400 mb-4">Trackers</h3>
          <div className="space-y-1">
            {torrent.trackers.map((tracker, i) => (
              <p key={i} className="text-xs text-slate-400 truncate">{tracker}</p>
            ))}
          </div>
        </div>
      )}

      {torrent.piece_count > 0 && (
        <div className="glass-panel rounded-xl p-5">
          <h3 className="text-sm font-semibold text-slate-400 mb-4">
            Piece Map ({torrent.piece_count} pieces, {formatBytes(torrent.piece_length)} each)
          </h3>
          <div className="flex flex-wrap gap-px">
            {torrent.piece_map.slice(0, 500).map((piece, i) => (
              <div
                key={i}
                className="w-1.5 h-1.5 rounded-sm"
                style={{
                  backgroundColor: piece > 0 ? 'var(--accent-color)' : 'rgba(255,255,255,0.05)',
                }}
              />
            ))}
          </div>
        </div>
      )}
    </motion.div>
  )
}

function InfoCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="glass-panel rounded-lg p-3">
      <p className="text-[10px] text-slate-500 uppercase tracking-wider">{label}</p>
      <p className="text-sm font-medium text-slate-200 mt-0.5">{value}</p>
    </div>
  )
}
