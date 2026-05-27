import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { api, type ExtractionRecord } from '../api/client'
import { Archive, Loader2, Lock, CheckCircle, XCircle, Clock } from 'lucide-react'

const pageVariants = {
  initial: { opacity: 0, x: 20 },
  animate: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: -20 },
}

export default function ExtractionQueue() {
  const [queue, setQueue] = useState<ExtractionRecord[]>([])
  const [loading, setLoading] = useState(true)
  const [passwordModal, setPasswordModal] = useState<ExtractionRecord | null>(null)
  const [passwordInput, setPasswordInput] = useState('')

  const fetch = async () => {
    try {
      const q = await api.getExtractionQueue()
      setQueue(q)
    } catch {}
    setLoading(false)
  }

  useEffect(() => {
    fetch()
    const interval = setInterval(fetch, 5000)
    return () => clearInterval(interval)
  }, [])

  const handleSubmitPassword = async () => {
    if (!passwordModal || !passwordInput.trim()) return
    await api.submitExtractionPassword(passwordModal.id, passwordInput.trim())
    setPasswordModal(null)
    setPasswordInput('')
    fetch()
  }

  const statusIcon = (status: string) => {
    switch (status) {
      case 'completed': return <CheckCircle size={16} className="text-emerald-400" />
      case 'error': return <XCircle size={16} className="text-red-400" />
      case 'processing': return <Loader2 size={16} className="text-accent-neon animate-spin" />
      case 'needs_password': return <Lock size={16} className="text-amber-400" />
      default: return <Clock size={16} className="text-slate-500" />
    }
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
      <h2 className="text-2xl font-bold">Extraction Queue</h2>

      <div className="space-y-3">
        {queue.map((item, i) => (
          <motion.div
            key={item.id}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.05 }}
            className="glass-panel rounded-xl p-4"
          >
            <div className="flex items-center gap-4">
              <motion.div
                animate={item.status === 'processing' ? { rotate: 360 } : {}}
                transition={{ duration: 2, repeat: Infinity, ease: 'linear' }}
              >
                <Archive size={24} className="text-accent-neon" />
              </motion.div>
              <div className="flex-1 min-w-0">
                <p className="text-sm text-slate-200 truncate">{item.archive_path}</p>
                <div className="flex items-center gap-3 mt-1">
                  <span className="flex items-center gap-1 text-xs text-slate-400">
                    {statusIcon(item.status)}
                    {item.status}
                  </span>
                  {item.error_message && (
                    <span className="text-xs text-red-400">{item.error_message}</span>
                  )}
                </div>
              </div>
              {item.status === 'needs_password' && (
                <button
                  onClick={() => setPasswordModal(item)}
                  className="px-3 py-1.5 rounded-lg bg-amber-500/20 text-amber-400 text-xs font-medium hover:bg-amber-500/30"
                >
                  Enter Password
                </button>
              )}
            </div>
            {(item.status === 'processing' || item.status === 'queued') && (
              <div className="mt-3 h-1.5 bg-white/10 rounded-full overflow-hidden">
                <motion.div
                  className="h-full bg-accent-neon shimmer-bar rounded-full"
                  initial={{ width: 0 }}
                  animate={{ width: `${item.progress}%` }}
                  transition={{ duration: 0.5 }}
                />
              </div>
            )}
          </motion.div>
        ))}
        {queue.length === 0 && !loading && (
          <div className="glass-panel rounded-xl p-12 text-center">
            <Archive size={40} className="mx-auto text-slate-600 mb-3" />
            <p className="text-slate-500">No extraction jobs in queue</p>
          </div>
        )}
      </div>

      {passwordModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50" onClick={() => setPasswordModal(null)}>
          <motion.div
            initial={{ scale: 0.95, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            className="glass-panel rounded-xl p-6 w-full max-w-sm"
            onClick={(e) => e.stopPropagation()}
          >
            <h3 className="text-lg font-bold mb-2 flex items-center gap-2">
              <Lock size={18} /> Archive Password
            </h3>
            <p className="text-xs text-slate-400 mb-4 truncate">{passwordModal.archive_path}</p>
            <input
              type="password"
              value={passwordInput}
              onChange={(e) => setPasswordInput(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSubmitPassword()}
              placeholder="Enter archive password..."
              className="w-full px-3 py-2.5 rounded-lg bg-white/5 border border-white/10 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:border-accent-neon/50"
              autoFocus
            />
            <div className="flex justify-end gap-2 mt-4">
              <button onClick={() => setPasswordModal(null)} className="px-4 py-2 rounded-lg text-sm text-slate-400 hover:text-slate-200">
                Cancel
              </button>
              <button onClick={handleSubmitPassword} className="px-4 py-2 rounded-lg bg-accent-neon/20 text-accent-neon text-sm font-medium hover:bg-accent-neon/30">
                Submit
              </button>
            </div>
          </motion.div>
        </div>
      )}
    </motion.div>
  )
}
