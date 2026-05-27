import { motion } from 'framer-motion'
import { useStore } from '../store'
import { formatSpeed, formatBytes } from '../lib/utils'
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from 'recharts'
import { Download, Upload, Users, HardDrive } from 'lucide-react'

const pageVariants = {
  initial: { opacity: 0, x: 20 },
  animate: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: -20 },
}

export default function Dashboard() {
  const stats = useStore((s) => s.stats)
  const torrents = useStore((s) => s.torrents)
  const speedHistory = useStore((s) => s.speedHistory)

  const activeTorrents = torrents.filter(
    (t) => t.status === 'downloading' || t.status === 'seeding'
  )

  const chartData = speedHistory.map((entry, i) => ({
    name: i,
    download: entry.download / 1048576,
    upload: entry.upload / 1048576,
  }))

  return (
    <motion.div
      variants={pageVariants}
      initial="initial"
      animate="animate"
      exit="exit"
      transition={{ duration: 0.3 }}
      className="space-y-6"
    >
      <h2 className="text-2xl font-bold">Dashboard</h2>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          icon={<Download size={20} />}
          label="Download Speed"
          value={formatSpeed(stats.download_speed)}
          color="text-emerald-400"
        />
        <StatCard
          icon={<Upload size={20} />}
          label="Upload Speed"
          value={formatSpeed(stats.upload_speed)}
          color="text-blue-400"
        />
        <StatCard
          icon={<Users size={20} />}
          label="Active Peers"
          value={stats.active_peers.toString()}
          color="text-purple-400"
        />
        <StatCard
          icon={<HardDrive size={20} />}
          label="Total Downloaded"
          value={formatBytes(stats.total_downloaded)}
          color="text-amber-400"
        />
      </div>

      <div className="glass-panel rounded-xl p-5">
        <h3 className="text-sm font-semibold text-slate-400 mb-4">Speed History</h3>
        <div className="h-48">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={chartData}>
              <defs>
                <linearGradient id="dlGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#10b981" stopOpacity={0.4} />
                  <stop offset="95%" stopColor="#10b981" stopOpacity={0} />
                </linearGradient>
                <linearGradient id="ulGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.4} />
                  <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
                </linearGradient>
              </defs>
              <XAxis dataKey="name" hide />
              <YAxis
                tick={{ fill: '#64748b', fontSize: 11 }}
                tickFormatter={(v) => `${v.toFixed(1)}`}
                width={50}
              />
              <Tooltip
                contentStyle={{
                  background: 'rgba(17, 24, 39, 0.9)',
                  border: '1px solid rgba(255,255,255,0.1)',
                  borderRadius: 8,
                  color: 'white',
                }}
                formatter={(value: number) => [`${value.toFixed(2)} MB/s`]}
              />
              <Area
                type="monotone"
                dataKey="download"
                stroke="#10b981"
                fill="url(#dlGrad)"
                strokeWidth={2}
              />
              <Area
                type="monotone"
                dataKey="upload"
                stroke="#3b82f6"
                fill="url(#ulGrad)"
                strokeWidth={2}
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </div>

      <div className="glass-panel rounded-xl p-5">
        <h3 className="text-sm font-semibold text-slate-400 mb-4">
          Active Torrents ({activeTorrents.length})
        </h3>
        <div className="space-y-3">
          {activeTorrents.slice(0, 5).map((t) => (
            <div key={t.id} className="flex items-center justify-between text-sm">
              <span className="truncate max-w-[60%] text-slate-200">{t.name}</span>
              <div className="flex items-center gap-4 text-xs text-slate-400">
                <span className="text-emerald-400">↓ {formatSpeed(t.download_speed)}</span>
                <span className="text-blue-400">↑ {formatSpeed(t.upload_speed)}</span>
                <span>{t.progress.toFixed(1)}%</span>
              </div>
            </div>
          ))}
          {activeTorrents.length === 0 && (
            <p className="text-slate-500 text-sm">No active torrents</p>
          )}
        </div>
      </div>
    </motion.div>
  )
}

function StatCard({
  icon,
  label,
  value,
  color,
}: {
  icon: React.ReactNode
  label: string
  value: string
  color: string
}) {
  return (
    <div className="glass-panel rounded-xl p-4 flex items-center gap-4">
      <div className={`${color}`}>{icon}</div>
      <div>
        <p className="text-xs text-slate-400">{label}</p>
        <p className={`text-lg font-bold ${color}`}>{value}</p>
      </div>
    </div>
  )
}
