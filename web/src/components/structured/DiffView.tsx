import { useEffect, useRef } from 'react'
import { GitBranch, Plus, Minus } from 'lucide-react'
import type { ParsedEvent } from '../../hooks/useEventsWs'

interface DiffViewProps {
  events: ParsedEvent[]
}

export function DiffView({ events }: DiffViewProps) {
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [events.length])

  const diffEvents = events.filter((e) => e.eventType === 'code_diff')

  if (diffEvents.length === 0) {
    return (
      <div className="text-center py-12 text-text-secondary">
        <GitBranch className="h-10 w-10 text-border mx-auto mb-3" />
        <p className="font-medium">No diffs yet</p>
        <p className="text-sm mt-1">Code changes will appear here as files are modified.</p>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {diffEvents.map((event) => (
        <DiffCard key={event.sequence} event={event} />
      ))}
      <div ref={bottomRef} />
    </div>
  )
}

function DiffCard({ event }: { event: ParsedEvent }) {
  const payload = event.payload
  const filePath = payload.file_path as string
  const diff = payload.diff as string
  const additions = (payload.additions as number) || 0
  const deletions = (payload.deletions as number) || 0
  const language = payload.language as string | undefined

  return (
    <div className="rounded-lg border border-border overflow-hidden">
      {/* File header */}
      <div className="flex items-center justify-between px-3 py-2 bg-bg-tertiary border-b border-border">
        <span className="text-xs font-mono text-text-primary truncate">{filePath}</span>
        <div className="flex items-center gap-2 text-xs shrink-0 ml-2">
          {additions > 0 && (
            <span className="flex items-center gap-0.5 text-success">
              <Plus className="h-3 w-3" />
              {additions}
            </span>
          )}
          {deletions > 0 && (
            <span className="flex items-center gap-0.5 text-danger">
              <Minus className="h-3 w-3" />
              {deletions}
            </span>
          )}
          {language && (
            <span className="text-text-secondary">{language}</span>
          )}
        </div>
      </div>
      {/* Diff content */}
      <pre className="p-2 overflow-x-auto text-xs leading-relaxed max-h-64 overflow-y-auto">
        {diff.split('\n').map((line, i) => (
          <DiffLine key={i} line={line} />
        ))}
      </pre>
    </div>
  )
}

function DiffLine({ line }: { line: string }) {
  let className = 'text-text-secondary' // context line
  let bg = ''

  if (line.startsWith('+') && !line.startsWith('+++')) {
    className = 'text-success'
    bg = 'bg-success/5'
  } else if (line.startsWith('-') && !line.startsWith('---')) {
    className = 'text-danger'
    bg = 'bg-danger/5'
  } else if (line.startsWith('@@')) {
    className = 'text-accent'
    bg = 'bg-accent/5'
  }

  return (
    <div className={`${className} ${bg} px-2 -mx-2`}>
      {line || '\u00A0'}
    </div>
  )
}
