import { useState } from 'react'
import { motion } from 'framer-motion'
import { useStore } from '../store'
import type { Settings } from '../api/client'
import { cn } from '../lib/utils'
import {
  Download, Gauge, Network, Settings as SettingsIcon,
  Archive, Monitor, Palette, Bell, Save, Loader2,
} from 'lucide-react'

const pageVariants = {
  initial: { opacity: 0, x: 40 },
  animate: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: -40 },
}

const tabs = [
  { id: 'downloads', label: 'Downloads', icon: Download },
  { id: 'speed', label: 'Speed', icon: Gauge },
  { id: 'connections', label: 'Connections', icon: Network },
  { id: 'bittorrent', label: 'BitTorrent', icon: SettingsIcon },
  { id: 'extraction', label: 'Extraction', icon: Archive },
  { id: 'webui', label: 'Web UI', icon: Monitor },
  { id: 'appearance', label: 'Appearance', icon: Palette },
  { id: 'notifications', label: 'Notifications', icon: Bell },
]

export default function SettingsPage() {
  const settings = useStore((s) => s.settings)
  const updateSettings = useStore((s) => s.updateSettings)
  const [activeTab, setActiveTab] = useState('downloads')
  const [localSettings, setLocalSettings] = useState<Settings | null>(settings)
  const [saving, setSaving] = useState(false)
  const [saved, setSaved] = useState(false)

  const s = localSettings || settings
  if (!s) return <div className="text-slate-500 p-8">Loading settings...</div>

  const update = (key: keyof Settings, value: any) => {
    setLocalSettings({ ...s, [key]: value })
  }

  const handleSave = async () => {
    if (!localSettings) return
    setSaving(true)
    await updateSettings(localSettings)
    setSaving(false)
    setSaved(true)
    setTimeout(() => setSaved(false), 2000)
  }

  return (
    <motion.div
      variants={pageVariants}
      initial="initial"
      animate="animate"
      exit="exit"
      transition={{ duration: 0.3 }}
      className="flex gap-6 h-full"
    >
      <div className="w-48 shrink-0 space-y-1">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={cn(
              'w-full flex items-center gap-2.5 px-3 py-2.5 rounded-lg text-sm font-medium transition-all text-left',
              activeTab === tab.id
                ? 'bg-white/10 text-accent-neon'
                : 'text-slate-400 hover:text-slate-200 hover:bg-white/5'
            )}
          >
            <tab.icon size={16} />
            {tab.label}
          </button>
        ))}
      </div>

      <div className="flex-1 glass-panel rounded-xl p-6 overflow-y-auto">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-bold">{tabs.find((t) => t.id === activeTab)?.label}</h2>
          <button
            onClick={handleSave}
            disabled={saving}
            className={cn(
              'flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition',
              saved
                ? 'bg-emerald-500/20 text-emerald-400'
                : 'bg-accent-neon/20 text-accent-neon hover:bg-accent-neon/30'
            )}
          >
            {saving ? <Loader2 size={14} className="animate-spin" /> : <Save size={14} />}
            {saved ? 'Saved' : 'Save'}
          </button>
        </div>

        {activeTab === 'downloads' && (
          <div className="space-y-4">
            <Field label="Download Directory">
              <input value={s.downloads_dir} onChange={(e) => update('downloads_dir', e.target.value)} className={inputCls} />
            </Field>
            <Field label="Watch Directory (auto-add .torrent files)">
              <input value={s.watch_dir} onChange={(e) => update('watch_dir', e.target.value)} className={inputCls} />
            </Field>
            <Field label="Default Sequential Download">
              <Toggle checked={s.sequential_default} onChange={(v) => update('sequential_default', v)} />
            </Field>
          </div>
        )}

        {activeTab === 'speed' && (
          <div className="space-y-4">
            <Field label="Max Download Speed (KB/s, 0 = unlimited)">
              <input type="number" value={s.max_download_speed_kb} onChange={(e) => update('max_download_speed_kb', +e.target.value)} className={inputCls} />
            </Field>
            <Field label="Max Upload Speed (KB/s, 0 = unlimited)">
              <input type="number" value={s.max_upload_speed_kb} onChange={(e) => update('max_upload_speed_kb', +e.target.value)} className={inputCls} />
            </Field>
            <Field label="Ratio Limit (0 = unlimited)">
              <input type="number" step="0.1" value={s.ratio_limit} onChange={(e) => update('ratio_limit', +e.target.value)} className={inputCls} />
            </Field>
            <Field label="Seeding Time Limit (minutes, 0 = unlimited)">
              <input type="number" value={s.seeding_time_limit_mins} onChange={(e) => update('seeding_time_limit_mins', +e.target.value)} className={inputCls} />
            </Field>
          </div>
        )}

        {activeTab === 'connections' && (
          <div className="space-y-4">
            <Field label="Max Global Connections">
              <input type="number" value={s.max_connections} onChange={(e) => update('max_connections', +e.target.value)} className={inputCls} />
            </Field>
            <Field label="Max Connections Per Torrent">
              <input type="number" value={s.max_connections_per_torrent} onChange={(e) => update('max_connections_per_torrent', +e.target.value)} className={inputCls} />
            </Field>
            <Field label="Listen Port">
              <input type="number" value={s.listen_port} onChange={(e) => update('listen_port', +e.target.value)} className={inputCls} />
            </Field>
          </div>
        )}

        {activeTab === 'bittorrent' && (
          <div className="space-y-4">
            <Field label="DHT"><Toggle checked={s.dht_enabled} onChange={(v) => update('dht_enabled', v)} /></Field>
            <Field label="Peer Exchange (PEX)"><Toggle checked={s.pex_enabled} onChange={(v) => update('pex_enabled', v)} /></Field>
            <Field label="Local Service Discovery"><Toggle checked={s.lsd_enabled} onChange={(v) => update('lsd_enabled', v)} /></Field>
            <Field label="uTP Protocol"><Toggle checked={s.utp_enabled} onChange={(v) => update('utp_enabled', v)} /></Field>
            <Field label="IP Blocklist URL">
              <input value={s.blocklist_url} onChange={(e) => update('blocklist_url', e.target.value)} className={inputCls} placeholder="https://example.com/blocklist.txt" />
            </Field>
          </div>
        )}

        {activeTab === 'extraction' && (
          <div className="space-y-4">
            <Field label="Auto-Extract Completed Archives">
              <Toggle checked={s.auto_extract} onChange={(v) => update('auto_extract', v)} />
            </Field>
            <Field label="Extraction Output Directory">
              <input value={s.extract_path} onChange={(e) => update('extract_path', e.target.value)} className={inputCls} placeholder="Leave empty for same directory" />
            </Field>
            <Field label="Default Passwords (comma-separated)">
              <input value={s.extract_passwords} onChange={(e) => update('extract_passwords', e.target.value)} className={inputCls} />
            </Field>
          </div>
        )}

        {activeTab === 'webui' && (
          <div className="space-y-4">
            <Field label="Enable Web UI"><Toggle checked={s.webui_enabled} onChange={(v) => update('webui_enabled', v)} /></Field>
            <Field label="Web UI Port">
              <input type="number" value={s.webui_port} onChange={(e) => update('webui_port', +e.target.value)} className={inputCls} />
            </Field>
            <Field label="Username">
              <input value={s.webui_username} onChange={(e) => update('webui_username', e.target.value)} className={inputCls} />
            </Field>
            <Field label="Password">
              <input type="password" value={s.webui_password_hash} onChange={(e) => update('webui_password_hash', e.target.value)} className={inputCls} placeholder="Enter new password" />
            </Field>
            <Field label="Jackett/Prowlarr URL">
              <input value={s.jackett_url} onChange={(e) => update('jackett_url', e.target.value)} className={inputCls} placeholder="http://localhost:9117" />
            </Field>
            <Field label="Jackett/Prowlarr API Key">
              <input value={s.jackett_api_key} onChange={(e) => update('jackett_api_key', e.target.value)} className={inputCls} />
            </Field>
          </div>
        )}

        {activeTab === 'appearance' && (
          <div className="space-y-4">
            <Field label="Theme">
              <div className="flex gap-2">
                {['dark', 'light'].map((t) => (
                  <button
                    key={t}
                    onClick={() => update('theme', t)}
                    className={cn(
                      'px-4 py-2 rounded-lg text-sm font-medium capitalize transition',
                      s.theme === t ? 'bg-accent-neon/20 text-accent-neon' : 'bg-white/5 text-slate-400 hover:text-slate-200'
                    )}
                  >
                    {t}
                  </button>
                ))}
              </div>
            </Field>
            <Field label="Accent Color">
              <div className="flex gap-3">
                {(['blue', 'purple', 'green', 'red'] as const).map((c) => {
                  const colors = { blue: '#00e5ff', purple: '#d500f9', green: '#00e676', red: '#ff1744' }
                  return (
                    <button
                      key={c}
                      onClick={() => update('accent_color', c)}
                      className={cn(
                        'w-10 h-10 rounded-full border-2 transition-all',
                        s.accent_color === c ? 'border-white scale-110' : 'border-transparent'
                      )}
                      style={{ backgroundColor: colors[c] }}
                    />
                  )
                })}
              </div>
            </Field>
          </div>
        )}

        {activeTab === 'notifications' && (
          <div className="space-y-4">
            <Field label="Enable Notifications">
              <Toggle checked={s.notifications_enabled} onChange={(v) => update('notifications_enabled', v)} />
            </Field>
          </div>
        )}
      </div>
    </motion.div>
  )
}

const inputCls = "w-full px-3 py-2.5 rounded-lg bg-white/5 border border-white/10 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:border-accent-neon/50"

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <label className="text-sm text-slate-400 shrink-0">{label}</label>
      <div className="flex-1 max-w-xs">{children}</div>
    </div>
  )
}

function Toggle({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={cn(
        'w-11 h-6 rounded-full transition-colors relative',
        checked ? 'bg-accent-neon' : 'bg-white/10'
      )}
    >
      <div
        className={cn(
          'w-4 h-4 rounded-full bg-white absolute top-1 transition-transform',
          checked ? 'translate-x-6' : 'translate-x-1'
        )}
      />
    </button>
  )
}
