import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { Zap, Code, Eye, ArrowRight } from 'lucide-react'
import { Button } from '../components/ui/Button'
import { StreamCard } from '../components/browse/StreamCard'
import { api, type StreamPublic } from '../lib/api'

export function HomePage() {
  const [streams, setStreams] = useState<StreamPublic[]>([])

  useEffect(() => {
    api.getLiveStreams().then(setStreams).catch(() => {})
  }, [])

  return (
    <div>
      {/* Hero */}
      <section className="relative overflow-hidden border-b border-border">
        <div className="absolute inset-0 bg-gradient-to-br from-accent/5 via-transparent to-accent/5" />
        <div className="relative mx-auto max-w-[1200px] px-4 py-20 text-center">
          <div className="inline-flex items-center gap-2 rounded-full bg-accent/10 border border-accent/20 px-4 py-1.5 text-sm text-accent mb-6">
            <Zap className="h-4 w-4" />
            Watch AI build software in real-time
          </div>
          <h1 className="text-4xl sm:text-5xl lg:text-6xl font-bold tracking-tight">
            Where developers stream
            <br />
            <span className="text-accent">the vibe</span>
          </h1>
          <p className="mt-4 text-lg text-text-secondary max-w-2xl mx-auto">
            Watch AI-assisted coding sessions live. See the conversation, the code diffs,
            and the magic happen in real-time. Not just screen capture &mdash; structured, interactive, copyable.
          </p>
          <div className="mt-8 flex items-center justify-center gap-4">
            <Link to="/browse">
              <Button size="lg">
                <Eye className="h-5 w-5" />
                Browse streams
              </Button>
            </Link>
            <Link to="/register">
              <Button variant="secondary" size="lg">
                <Code className="h-5 w-5" />
                Start streaming
              </Button>
            </Link>
          </div>
        </div>
      </section>

      {/* Feature highlights */}
      <section className="mx-auto max-w-[1200px] px-4 py-16">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <FeatureCard
            icon={<Code className="h-6 w-6" />}
            title="Structured AI View"
            description="See the full AI conversation, tool calls, and code diffs rendered natively. Copy code, search history, scroll back."
          />
          <FeatureCard
            icon={<Eye className="h-6 w-6" />}
            title="Live Screen Capture"
            description="Watch the developer's screen in real-time alongside the structured view. Best of both worlds."
          />
          <FeatureCard
            icon={<Zap className="h-6 w-6" />}
            title="Built for Developers"
            description="Dark theme, syntax highlighting, diff views. This isn't a general streaming platform &mdash; it's built for code."
          />
        </div>
      </section>

      {/* Live streams */}
      {streams.length > 0 && (
        <section className="mx-auto max-w-[1200px] px-4 pb-16">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-xl font-bold">Live Now</h2>
            <Link to="/browse" className="text-sm text-accent hover:text-accent-hover flex items-center gap-1">
              View all <ArrowRight className="h-4 w-4" />
            </Link>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
            {streams.slice(0, 8).map((s) => (
              <StreamCard key={s.id} stream={s} />
            ))}
          </div>
        </section>
      )}
    </div>
  )
}

function FeatureCard({ icon, title, description }: { icon: React.ReactNode; title: string; description: string }) {
  return (
    <div className="rounded-xl border border-border bg-bg-secondary p-6 hover:border-accent/30 transition-colors">
      <div className="inline-flex items-center justify-center rounded-lg bg-accent/10 p-2.5 text-accent mb-4">
        {icon}
      </div>
      <h3 className="font-semibold text-text-primary mb-2">{title}</h3>
      <p className="text-sm text-text-secondary leading-relaxed">{description}</p>
    </div>
  )
}
