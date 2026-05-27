import { NavLink } from 'react-router-dom'
import {
  LayoutDashboard,
  List,
  Search,
  Share2,
  Archive,
  Settings,
} from 'lucide-react'
import { useStore } from '../store'
import { formatSpeed } from '../lib/utils'

const links = [
  { to: '/', icon: LayoutDashboard, label: 'Dashboard' },
  { to: '/torrents', icon: List, label: 'Torrents' },
  { to: '/search', icon: Search, label: 'Search' },
  { to: '/sharing', icon: Share2, label: 'Sharing' },
  { to: '/extraction', icon: Archive, label: 'Extraction' },
  { to: '/settings', icon: Settings, label: 'Settings' },
]

export default function Sidebar() {
  const stats = useStore((s) => s.stats)

  return (
    <aside className="w-56 glass-panel border-r border-white/5 flex flex-col h-full shrink-0">
      <div className="p-5 border-b border-white/5">
        <h1 className="text-xl font-bold text-accent-neon tracking-tight">
          Cirnotorrent
        </h1>
      </div>

      <nav className="flex-1 p-3 space-y-1">
        {links.map((link) => (
          <NavLink
            key={link.to}
            to={link.to}
            end={link.to === '/'}
            className={({ isActive }) =>
              `flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all ${
                isActive
                  ? 'bg-white/10 text-accent-neon shadow-accent-neon/20'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-white/5'
              }`
            }
          >
            <link.icon size={18} />
            {link.label}
          </NavLink>
        ))}
      </nav>

      <div className="p-4 border-t border-white/5 space-y-2">
        <div className="flex items-center justify-between text-xs">
          <span className="text-emerald-400">↓ {formatSpeed(stats.download_speed)}</span>
          <span className="text-blue-400">↑ {formatSpeed(stats.upload_speed)}</span>
        </div>
        <div className="flex items-center justify-between text-xs text-slate-500">
          <span>Peers: {stats.active_peers}</span>
          <span>DHT: {stats.dht_nodes}</span>
        </div>
      </div>
    </aside>
  )
}
