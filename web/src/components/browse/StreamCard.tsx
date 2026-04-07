import { Link } from 'react-router-dom'
import { Users } from 'lucide-react'
import { Avatar } from '../ui/Avatar'
import { Badge } from '../ui/Badge'
import { LiveIndicator } from '../ui/LiveIndicator'
import type { StreamPublic } from '../../lib/api'

export function StreamCard({ stream }: { stream: StreamPublic }) {
  return (
    <Link
      to={`/stream/${stream.username}`}
      className="group flex flex-col rounded-xl overflow-hidden bg-bg-secondary border border-border hover:border-accent/40 transition-all hover:shadow-lg hover:shadow-accent/5"
    >
      {/* Thumbnail */}
      <div className="relative aspect-video bg-bg-tertiary">
        {stream.thumbnail_url ? (
          <img src={stream.thumbnail_url} alt={stream.title} className="w-full h-full object-cover" />
        ) : (
          <div className="w-full h-full flex items-center justify-center">
            <div className="text-4xl font-bold text-border">
              {stream.display_name?.[0] || '?'}
            </div>
          </div>
        )}
        <div className="absolute top-2 left-2">
          <LiveIndicator />
        </div>
        <div className="absolute bottom-2 right-2 flex items-center gap-1 rounded bg-black/70 px-1.5 py-0.5">
          <Users className="h-3 w-3 text-text-secondary" />
          <span className="text-xs text-text-secondary">{stream.viewer_count}</span>
        </div>
      </div>

      {/* Info */}
      <div className="p-3 flex gap-3">
        <Avatar src={stream.avatar_url} alt={stream.display_name || 'Unknown'} size="sm" />
        <div className="min-w-0">
          <h3 className="font-semibold text-sm text-text-primary truncate group-hover:text-accent transition-colors">
            {stream.title}
          </h3>
          <p className="text-xs text-text-secondary truncate">{stream.display_name}</p>
          {stream.tags.length > 0 && (
            <div className="mt-1.5 flex flex-wrap gap-1">
              {stream.tags.slice(0, 3).map((tag) => (
                <Badge key={tag}>{tag}</Badge>
              ))}
            </div>
          )}
        </div>
      </div>
    </Link>
  )
}
