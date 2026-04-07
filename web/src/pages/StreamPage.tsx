import { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { Users, Radio } from 'lucide-react'
import { Avatar } from '../components/ui/Avatar'
import { Badge } from '../components/ui/Badge'
import { LiveIndicator } from '../components/ui/LiveIndicator'
import { Button } from '../components/ui/Button'
import { VideoPlayer } from '../components/stream/VideoPlayer'
import { ChatPanel } from '../components/stream/ChatPanel'
import { StructuredPanel } from '../components/structured/StructuredPanel'
import { useChatWs } from '../hooks/useChatWs'
import { useEventsWs } from '../hooks/useEventsWs'
import { useAuthStore } from '../stores/auth'
import { api, type StreamPublic } from '../lib/api'

export function StreamPage() {
  const { username } = useParams()
  const [stream, setStream] = useState<StreamPublic | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const user = useAuthStore((s) => s.user)

  useEffect(() => {
    if (!username) return
    setLoading(true)
    api
      .getStreamByUsername(username)
      .then(setStream)
      .catch(() => setError('Stream not found'))
      .finally(() => setLoading(false))
  }, [username])

  // Auto-refresh stream data every 15s
  useEffect(() => {
    if (!username) return
    const interval = setInterval(() => {
      api.getStreamByUsername(username).then(setStream).catch(() => {})
    }, 15000)
    return () => clearInterval(interval)
  }, [username])

  const isLive = stream?.status === 'live'
  const { messages, viewerCount, sendMessage, connected } = useChatWs(stream?.id, isLive)
  const events = useEventsWs(stream?.id, isLive)

  const displayViewerCount = isLive && viewerCount > 0 ? viewerCount : (stream?.viewer_count ?? 0)

  if (loading) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-3.5rem)]">
        <div className="animate-spin h-8 w-8 border-2 border-accent border-t-transparent rounded-full" />
      </div>
    )
  }

  if (error || !stream) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-3.5rem)]">
        <div className="text-center">
          <Radio className="h-12 w-12 text-border mx-auto mb-4" />
          <h2 className="text-lg font-semibold text-text-secondary">
            {error || 'Stream not found'}
          </h2>
          <p className="text-sm text-text-secondary mt-1">
            This user doesn't have a stream configured yet
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="h-[calc(100vh-3.5rem)] flex">
      {/* Main area: video + structured view */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Stream header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-border">
          <div className="flex items-center gap-3 min-w-0">
            <Avatar src={stream.avatar_url} alt={stream.display_name || username || '?'} size="md" />
            <div className="min-w-0">
              <div className="flex items-center gap-2">
                <h2 className="font-semibold truncate">{stream.title}</h2>
                {isLive && <LiveIndicator />}
              </div>
              <p className="text-sm text-text-secondary truncate">
                @{stream.username || username}
                {isLive && stream.display_name && ` \u00B7 ${stream.display_name}`}
              </p>
            </div>
          </div>
          <div className="flex items-center gap-3 shrink-0">
            <div className="flex items-center gap-1.5 text-sm text-text-secondary">
              <Users className="h-4 w-4" />
              <span>{displayViewerCount}</span>
            </div>
            <Button variant="primary" size="sm">Follow</Button>
          </div>
        </div>

        {/* Content area */}
        <div className="flex-1 flex min-h-0">
          {/* Video panel */}
          <div className="flex-1 flex flex-col min-w-0">
            <div className="flex-1 min-h-0">
              <VideoPlayer hlsUrl={stream.hls_url} isLive={isLive} />
            </div>
            {/* Stream info below video */}
            <div className="p-4 border-t border-border">
              {stream.tags.length > 0 && (
                <div className="flex gap-2 mb-2">
                  {stream.tags.map((tag) => (
                    <Badge key={tag} variant="accent">{tag}</Badge>
                  ))}
                </div>
              )}
              {stream.description && (
                <p className="text-sm text-text-secondary">{stream.description}</p>
              )}
            </div>
          </div>

          {/* Structured view panel — THE killer feature */}
          <StructuredPanel events={events} isLive={isLive} />
        </div>
      </div>

      {/* Chat panel */}
      <ChatPanel
        messages={messages}
        viewerCount={displayViewerCount}
        connected={connected}
        isLive={isLive}
        isLoggedIn={!!user}
        onSend={sendMessage}
      />
    </div>
  )
}
