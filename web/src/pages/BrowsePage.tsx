import { useEffect, useState } from 'react'
import { Search, Radio } from 'lucide-react'
import { StreamCard } from '../components/browse/StreamCard'
import { api, type StreamPublic } from '../lib/api'

export function BrowsePage() {
  const [streams, setStreams] = useState<StreamPublic[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    api
      .getLiveStreams()
      .then(setStreams)
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [])

  return (
    <div className="mx-auto max-w-[1400px] px-4 py-8">
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 mb-8">
        <div>
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <Radio className="h-6 w-6 text-accent" />
            Browse Streams
          </h1>
          <p className="text-sm text-text-secondary mt-1">
            {streams.length} stream{streams.length !== 1 ? 's' : ''} live now
          </p>
        </div>
        <div className="relative w-full sm:w-72">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-text-secondary" />
          <input
            type="text"
            placeholder="Search streams..."
            className="w-full pl-9 pr-3 py-2 text-sm"
          />
        </div>
      </div>

      {loading ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          {Array.from({ length: 8 }).map((_, i) => (
            <div key={i} className="animate-pulse rounded-xl bg-bg-secondary border border-border">
              <div className="aspect-video bg-bg-tertiary rounded-t-xl" />
              <div className="p-3 space-y-2">
                <div className="h-4 bg-bg-tertiary rounded w-3/4" />
                <div className="h-3 bg-bg-tertiary rounded w-1/2" />
              </div>
            </div>
          ))}
        </div>
      ) : streams.length === 0 ? (
        <div className="text-center py-20">
          <Radio className="h-12 w-12 text-border mx-auto mb-4" />
          <h2 className="text-lg font-semibold text-text-secondary">No streams live right now</h2>
          <p className="text-sm text-text-secondary mt-1">Check back later or start your own stream!</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          {streams.map((s) => (
            <StreamCard key={s.id} stream={s} />
          ))}
        </div>
      )}
    </div>
  )
}
