import { Routes, Route, Navigate } from 'react-router-dom'
import { AnimatePresence } from 'framer-motion'
import Layout from './components/Layout'
import Dashboard from './pages/Dashboard'
import TorrentList from './pages/TorrentList'
import TorrentDetail from './pages/TorrentDetail'
import Search from './pages/Search'
import SharingHub from './pages/SharingHub'
import ExtractionQueue from './pages/ExtractionQueue'
import Settings from './pages/Settings'
import { useStore } from './store'
import { useEffect } from 'react'

export default function App() {
  const connect = useStore((s) => s.connect)
  const fetchSettings = useStore((s) => s.fetchSettings)

  useEffect(() => {
    fetchSettings()
    connect()
  }, [connect, fetchSettings])

  return (
    <Layout>
      <AnimatePresence mode="wait">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/torrents" element={<TorrentList />} />
          <Route path="/torrents/:id" element={<TorrentDetail />} />
          <Route path="/search" element={<Search />} />
          <Route path="/sharing" element={<SharingHub />} />
          <Route path="/extraction" element={<ExtractionQueue />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </AnimatePresence>
    </Layout>
  )
}
