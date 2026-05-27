import { useState } from 'react'
import { motion } from 'framer-motion'
import { api, type SearchResult } from '../api/client'
import { useStore } from '../store'
import { formatBytes } from '../lib/utils'
import { Search as SearchIcon, Download, Loader2 } from 'lucide-react'

const pageVariants = {
  initial: { opacity: 0, x: 20 },
  animate: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: -20 },
}

export default function Search() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchResult[]>([])
  const [loading, setLoading] = useState(false)
  const [addedIdx, setAddedIdx] = useState<number | null>(null)
  const addTorrent = useStore((s) => s.addTorrent)

  const handleSearch = async () => {
    if (!query.trim()) return
    setLoading(true)
    try {
      const res = await api.searchTorrents(query.trim())
      setResults(res)
    } catch {
      setResults([])
    }
    setLoading(false)
  }

  const handleAdd = async (result: SearchResult, idx: number) => {
    try {
      await addTorrent(result.magnet_link)
      setAddedIdx(idx)
      setTimeout(() => setAddedIdx(null), 2000)
    } catch {}
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
      <h2 className="text-2xl font-bold">Search</h2>

      <div className="flex gap-3">
        <div className="relative flex-1">
          <SearchIcon size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            placeholder="Search for torrents..."
            className="w-full pl-10 pr-4 py-2.5 rounded-lg bg-white/5 border border-white/10 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:border-accent-neon/50"
          />
        </div>
        <button
          onClick={handleSearch}
          disabled={loading}
          className="px-5 py-2.5 rounded-lg bg-accent-neon/20 text-accent-neon text-sm font-medium hover:bg-accent-neon/30 disabled:opacity-50 flex items-center gap-2"
        >
          {loading ? <Loader2 size={16} className="animate-spin" /> : <SearchIcon size={16} />}
          Search
        </button>
      </div>

      {results.length > 0 && (
        <div className="glass-panel rounded-xl overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-white/5 text-slate-400 text-xs">
                <th className="px-4 py-3 text-left font-medium">Title</th>
                <th className="px-4 py-3 text-left font-medium">Size</th>
                <th className="px-4 py-3 text-left font-medium">Seeds</th>
                <th className="px-4 py-3 text-left font-medium">Peers</th>
                <th className="px-4 py-3 text-left font-medium">Indexer</th>
                <th className="px-4 py-3 text-left font-medium">Action</th>
              </tr>
            </thead>
            <tbody>
              {results.map((r, i) => (
                <motion.tr
                  key={i}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: i * 0.05 }}
                  className="border-b border-white/5 hover:bg-white/5"
                >
                  <td className="px-4 py-3 text-slate-200 max-w-[400px] truncate">{r.title}</td>
                  <td className="px-4 py-3 text-slate-400 text-xs">{formatBytes(r.size)}</td>
                  <td className="px-4 py-3 text-emerald-400 text-xs">{r.seeds}</td>
                  <td className="px-4 py-3 text-red-400 text-xs">{r.peers}</td>
                  <td className="px-4 py-3 text-slate-500 text-xs">{r.indexer}</td>
                  <td className="px-4 py-3">
                    <button
                      onClick={() => handleAdd(r, i)}
                      disabled={addedIdx === i}
                      className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-accent-neon/20 text-accent-neon text-xs font-medium hover:bg-accent-neon/30 disabled:opacity-50"
                    >
                      <Download size={12} />
                      {addedIdx === i ? 'Added' : 'Add'}
                    </button>
                  </td>
                </motion.tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {!loading && results.length === 0 && query && (
        <p className="text-slate-500 text-sm text-center py-8">No results found</p>
      )}
    </motion.div>
  )
}
