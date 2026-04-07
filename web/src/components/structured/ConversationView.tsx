import { useEffect, useRef } from 'react'
import { User, Bot, Wrench } from 'lucide-react'
import type { ParsedEvent } from '../../hooks/useEventsWs'
import { CodeBlock } from './CodeBlock'

interface ConversationViewProps {
  events: ParsedEvent[]
}

export function ConversationView({ events }: ConversationViewProps) {
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [events.length])

  // Filter to conversation-type events
  const conversationEvents = events.filter((e) =>
    ['ai_message', 'ai_response', 'tool_use'].includes(e.eventType)
  )

  if (conversationEvents.length === 0) {
    return (
      <div className="text-center py-12 text-text-secondary">
        <Bot className="h-10 w-10 text-border mx-auto mb-3" />
        <p className="font-medium">Waiting for AI activity...</p>
        <p className="text-sm mt-1">
          Messages will appear here as the streamer interacts with their AI assistant.
        </p>
      </div>
    )
  }

  return (
    <div className="space-y-3">
      {conversationEvents.map((event) => (
        <ConversationBubble key={event.sequence} event={event} />
      ))}
      <div ref={bottomRef} />
    </div>
  )
}

function ConversationBubble({ event }: { event: ParsedEvent }) {
  const payload = event.payload

  switch (event.eventType) {
    case 'ai_message':
      return (
        <div className="rounded-lg p-3 bg-ai-user/15 border border-ai-user/20">
          <div className="flex items-center gap-1.5 text-xs font-medium text-accent mb-2">
            <User className="h-3 w-3" />
            <span>Developer</span>
          </div>
          <div className="text-sm text-text-primary leading-relaxed whitespace-pre-wrap">
            {payload.content as string}
          </div>
        </div>
      )

    case 'ai_response':
      return (
        <div className="rounded-lg p-3 bg-ai-assistant border border-border">
          <div className="flex items-center gap-1.5 text-xs font-medium text-text-secondary mb-2">
            <Bot className="h-3 w-3" />
            <span>Claude</span>
            {typeof payload.model === 'string' && (
              <span className="text-text-secondary/50 ml-1">
                ({payload.model})
              </span>
            )}
          </div>
          <div className="text-sm text-text-primary leading-relaxed">
            <FormattedResponse content={payload.content as string} />
          </div>
        </div>
      )

    case 'tool_use':
      return (
        <div className="rounded-lg p-2.5 bg-bg-tertiary border border-border">
          <div className="flex items-center gap-2 text-xs">
            <Wrench className="h-3 w-3 text-accent shrink-0" />
            <span className="font-medium text-accent">
              {payload.tool_name as string}
            </span>
            {typeof payload.file_path === 'string' && (
              <span className="text-text-secondary font-mono truncate">
                {payload.file_path}
              </span>
            )}
          </div>
          {typeof payload.summary === 'string' && (
            <p className="text-xs text-text-secondary mt-1 ml-5">
              {payload.summary}
            </p>
          )}
        </div>
      )

    default:
      return null
  }
}

function FormattedResponse({ content }: { content: string }) {
  // Simple markdown-like parsing: detect code blocks
  const parts = content.split(/(```[\s\S]*?```)/g)

  return (
    <>
      {parts.map((part, i) => {
        if (part.startsWith('```')) {
          const lines = part.split('\n')
          const langLine = lines[0].replace('```', '').trim()
          const code = lines.slice(1, -1).join('\n')
          return <CodeBlock key={i} code={code} language={langLine || undefined} />
        }
        return (
          <span key={i} className="whitespace-pre-wrap">
            {part}
          </span>
        )
      })}
    </>
  )
}
