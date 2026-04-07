import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Radio, Copy, Eye, EyeOff, Settings, Play, Square } from 'lucide-react'
import { Button } from '../components/ui/Button'
import { Input } from '../components/ui/Input'
import { Badge } from '../components/ui/Badge'
import { LiveIndicator } from '../components/ui/LiveIndicator'
import { useAuthStore } from '../stores/auth'
import { api, type StreamWithKey, ApiError } from '../lib/api'

export function DashboardPage() {
  const user = useAuthStore((s) => s.user)
  const loading = useAuthStore((s) => s.loading)
  const navigate = useNavigate()
  const [stream, setStream] = useState<StreamWithKey | null>(null)
  const [title, setTitle] = useState('')
  const [description, setDescription] = useState('')
  const [tags, setTags] = useState('')
  const [showKey, setShowKey] = useState(false)
  const [error, setError] = useState('')
  const [saving, setSaving] = useState(false)
  const [streamLoading, setStreamLoading] = useState(true)

  useEffect(() => {
    if (!loading && !user) navigate('/login')
  }, [user, loading, navigate])

  useEffect(() => {
    if (!user) return
    api
      .getMyStream()
      .then((s) => {
        if (s) {
          setStream(s)
          setTitle(s.title)
          setDescription(s.description || '')
          setTags(s.tags.join(', '))
        }
      })
      .catch(() => {})
      .finally(() => setStreamLoading(false))
  }, [user])

  const handleSave = async () => {
    setError('')
    setSaving(true)
    try {
      const tagArray = tags.split(',').map((t) => t.trim()).filter(Boolean)
      if (stream) {
        const updated = await api.updateStream(stream.id, {
          title,
          description: description || undefined,
          tags: tagArray,
        })
        setStream({ ...stream, ...updated })
      } else {
        const created = await api.createStream({
          title,
          description: description || undefined,
          tags: tagArray,
        })
        setStream(created)
      }
    } catch (err) {
      setError(err instanceof ApiError ? err.message : 'Failed to save')
    } finally {
      setSaving(false)
    }
  }

  const handleGoLive = async () => {
    if (!stream) return
    try {
      await api.goLive(stream.id)
      const updated = await api.getMyStream()
      if (updated) setStream(updated)
    } catch (err) {
      setError(err instanceof ApiError ? err.message : 'Failed to go live')
    }
  }

  const handleEndStream = async () => {
    if (!stream) return
    try {
      await api.endStream(stream.id)
      const updated = await api.getMyStream()
      if (updated) setStream(updated)
    } catch (err) {
      setError(err instanceof ApiError ? err.message : 'Failed to end stream')
    }
  }

  const copyKey = () => {
    if (stream) navigator.clipboard.writeText(stream.stream_key_visible)
  }

  if (loading || streamLoading) {
    return (
      <div className="flex items-center justify-center py-20">
        <div className="animate-spin h-8 w-8 border-2 border-accent border-t-transparent rounded-full" />
      </div>
    )
  }

  const isLive = stream?.status === 'live'

  return (
    <div className="mx-auto max-w-3xl px-4 py-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <Settings className="h-6 w-6 text-accent" />
            Stream Dashboard
          </h1>
          <p className="text-sm text-text-secondary mt-1">Configure and manage your stream</p>
        </div>
        {isLive && <LiveIndicator />}
      </div>

      {error && (
        <div className="rounded-lg bg-danger/10 border border-danger/20 px-4 py-2 text-sm text-danger mb-6">
          {error}
        </div>
      )}

      {/* Stream config */}
      <div className="rounded-xl border border-border bg-bg-secondary p-6 space-y-5">
        <h2 className="font-semibold flex items-center gap-2">
          <Radio className="h-4 w-4 text-accent" />
          Stream Settings
        </h2>

        <Input
          label="Stream title"
          id="title"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="Building a full-stack app with Claude"
        />

        <div className="flex flex-col gap-1.5">
          <label htmlFor="desc" className="text-sm font-medium text-text-secondary">Description</label>
          <textarea
            id="desc"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="What are you building?"
            rows={3}
            className="w-full resize-none"
          />
        </div>

        <Input
          label="Tags (comma-separated)"
          id="tags"
          value={tags}
          onChange={(e) => setTags(e.target.value)}
          placeholder="rust, react, claude-code"
        />

        <Button onClick={handleSave} disabled={saving || !title}>
          {saving ? 'Saving...' : stream ? 'Update stream' : 'Create stream'}
        </Button>
      </div>

      {/* Stream key */}
      {stream && (
        <div className="mt-6 rounded-xl border border-border bg-bg-secondary p-6 space-y-4">
          <h2 className="font-semibold">Stream Key</h2>
          <div className="flex items-center gap-2">
            <div className="flex-1 rounded-lg bg-bg-tertiary border border-border px-3 py-2 font-mono text-sm">
              {showKey ? stream.stream_key_visible : '\u2022'.repeat(20)}
            </div>
            <Button variant="ghost" size="sm" onClick={() => setShowKey(!showKey)}>
              {showKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </Button>
            <Button variant="ghost" size="sm" onClick={copyKey}>
              <Copy className="h-4 w-4" />
            </Button>
          </div>
          <p className="text-xs text-text-secondary">
            Use this key in OBS or the VibeTwitch agent. Never share it publicly.
          </p>
        </div>
      )}

      {/* Go live / end */}
      {stream && (
        <div className="mt-6 rounded-xl border border-border bg-bg-secondary p-6">
          <h2 className="font-semibold mb-4">Stream Control</h2>
          {stream.tags.length > 0 && (
            <div className="flex gap-2 mb-4">
              {stream.tags.map((t) => (
                <Badge key={t} variant="accent">{t}</Badge>
              ))}
            </div>
          )}
          {isLive ? (
            <Button variant="danger" onClick={handleEndStream}>
              <Square className="h-4 w-4" />
              End Stream
            </Button>
          ) : (
            <Button onClick={handleGoLive}>
              <Play className="h-4 w-4" />
              Go Live
            </Button>
          )}
        </div>
      )}

      {/* Agent setup */}
      {stream && (
        <div className="mt-6 rounded-xl border border-border bg-bg-secondary p-6">
          <h2 className="font-semibold mb-2">Agent Setup</h2>
          <p className="text-sm text-text-secondary mb-4">
            Run the VibeTwitch agent alongside your coding session to stream structured AI events.
          </p>
          <div className="rounded-lg bg-bg-primary border border-border p-4 font-mono text-sm text-text-secondary">
            <p className="text-text-secondary/60"># Install the agent (coming soon)</p>
            <p>cargo install vibetwitch-agent</p>
            <p className="mt-2 text-text-secondary/60"># Start streaming events</p>
            <p>vibetwitch-agent start \</p>
            <p>&nbsp; --project-dir . \</p>
            <p>&nbsp; --stream-key {showKey ? stream.stream_key_visible : '<your-stream-key>'}</p>
          </div>
        </div>
      )}
    </div>
  )
}
