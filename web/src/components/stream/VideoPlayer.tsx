import { useEffect, useRef, useState } from 'react'
import Hls from 'hls.js'
import { Volume2, VolumeX, Maximize, Minimize, AlertCircle } from 'lucide-react'

interface VideoPlayerProps {
  hlsUrl: string | null | undefined
  isLive: boolean
}

export function VideoPlayer({ hlsUrl, isLive }: VideoPlayerProps) {
  const videoRef = useRef<HTMLVideoElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const [muted, setMuted] = useState(true)
  const [fullscreen, setFullscreen] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    const video = videoRef.current
    if (!video || !hlsUrl || !isLive) return

    setError(null)
    setLoading(true)

    if (Hls.isSupported()) {
      const hls = new Hls({
        enableWorker: true,
        lowLatencyMode: true,
        liveSyncDurationCount: 3,
        liveMaxLatencyDurationCount: 6,
      })

      hls.loadSource(hlsUrl)
      hls.attachMedia(video)

      hls.on(Hls.Events.MANIFEST_PARSED, () => {
        setLoading(false)
        video.play().catch(() => {})
      })

      hls.on(Hls.Events.ERROR, (_event, data) => {
        if (data.fatal) {
          setLoading(false)
          if (data.type === Hls.ErrorTypes.NETWORK_ERROR) {
            setError('Stream unavailable. The streamer may be setting up.')
            // Retry after a few seconds
            setTimeout(() => hls.startLoad(), 3000)
          } else if (data.type === Hls.ErrorTypes.MEDIA_ERROR) {
            hls.recoverMediaError()
          } else {
            setError('Playback error')
            hls.destroy()
          }
        }
      })

      return () => hls.destroy()
    } else if (video.canPlayType('application/vnd.apple.mpegurl')) {
      // Safari native HLS
      video.src = hlsUrl
      video.addEventListener('loadedmetadata', () => {
        setLoading(false)
        video.play().catch(() => {})
      })
    } else {
      setError('HLS playback is not supported in this browser')
    }
  }, [hlsUrl, isLive])

  const toggleMute = () => {
    if (videoRef.current) {
      videoRef.current.muted = !muted
      setMuted(!muted)
    }
  }

  const toggleFullscreen = () => {
    if (!containerRef.current) return
    if (document.fullscreenElement) {
      document.exitFullscreen()
      setFullscreen(false)
    } else {
      containerRef.current.requestFullscreen()
      setFullscreen(true)
    }
  }

  // Not live — show placeholder
  if (!isLive || !hlsUrl) {
    return (
      <div className="w-full h-full bg-black flex items-center justify-center">
        <div className="text-center text-text-secondary">
          <div className="w-16 h-16 rounded-full bg-bg-tertiary flex items-center justify-center mx-auto mb-4">
            <AlertCircle className="h-8 w-8 text-border" />
          </div>
          <p className="text-lg font-medium">Stream Offline</p>
          <p className="text-sm mt-1">This streamer isn't live right now</p>
        </div>
      </div>
    )
  }

  return (
    <div ref={containerRef} className="relative w-full h-full bg-black group">
      <video
        ref={videoRef}
        className="w-full h-full object-contain"
        muted={muted}
        playsInline
        autoPlay
      />

      {/* Loading overlay */}
      {loading && (
        <div className="absolute inset-0 flex items-center justify-center bg-black/50">
          <div className="animate-spin h-10 w-10 border-2 border-accent border-t-transparent rounded-full" />
        </div>
      )}

      {/* Error overlay */}
      {error && (
        <div className="absolute inset-0 flex items-center justify-center bg-black/70">
          <div className="text-center">
            <AlertCircle className="h-8 w-8 text-danger mx-auto mb-2" />
            <p className="text-sm text-text-secondary">{error}</p>
          </div>
        </div>
      )}

      {/* Controls — visible on hover */}
      <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent p-3 opacity-0 group-hover:opacity-100 transition-opacity">
        <div className="flex items-center justify-between">
          <button onClick={toggleMute} className="p-1.5 rounded hover:bg-white/10 transition-colors">
            {muted ? (
              <VolumeX className="h-5 w-5 text-white" />
            ) : (
              <Volume2 className="h-5 w-5 text-white" />
            )}
          </button>
          <button onClick={toggleFullscreen} className="p-1.5 rounded hover:bg-white/10 transition-colors">
            {fullscreen ? (
              <Minimize className="h-5 w-5 text-white" />
            ) : (
              <Maximize className="h-5 w-5 text-white" />
            )}
          </button>
        </div>
      </div>
    </div>
  )
}
