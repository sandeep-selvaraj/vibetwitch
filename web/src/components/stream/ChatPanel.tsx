import { useEffect, useRef, useState } from 'react'
import { MessageSquare, Send, Wifi, WifiOff } from 'lucide-react'
import type { ChatMessage } from '../../hooks/useChatWs'

interface ChatPanelProps {
  messages: ChatMessage[]
  viewerCount: number
  connected: boolean
  isLive: boolean
  isLoggedIn: boolean
  onSend: (content: string) => void
}

export function ChatPanel({ messages, viewerCount, connected, isLive, isLoggedIn, onSend }: ChatPanelProps) {
  const [input, setInput] = useState('')
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const [autoScroll, setAutoScroll] = useState(true)

  // Auto-scroll to bottom on new messages
  useEffect(() => {
    if (autoScroll) {
      messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
    }
  }, [messages, autoScroll])

  // Detect if user scrolled up (disable auto-scroll)
  const handleScroll = () => {
    const el = containerRef.current
    if (!el) return
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 60
    setAutoScroll(atBottom)
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    const content = input.trim()
    if (!content) return
    onSend(content)
    setInput('')
  }

  return (
    <div className="w-[300px] border-l border-border flex flex-col bg-bg-secondary">
      {/* Header */}
      <div className="px-4 py-3 border-b border-border flex items-center justify-between">
        <div className="flex items-center gap-2">
          <MessageSquare className="h-4 w-4 text-accent" />
          <h3 className="font-semibold text-sm">Stream Chat</h3>
        </div>
        <div className="flex items-center gap-1.5 text-xs text-text-secondary">
          {isLive && viewerCount > 0 && <span>{viewerCount}</span>}
          {isLive && (
            connected ? (
              <Wifi className="h-3 w-3 text-success" />
            ) : (
              <WifiOff className="h-3 w-3 text-danger" />
            )
          )}
        </div>
      </div>

      {/* Messages */}
      <div
        ref={containerRef}
        className="flex-1 overflow-y-auto p-3 space-y-1.5"
        onScroll={handleScroll}
      >
        {!isLive ? (
          <div className="text-center py-12 text-text-secondary">
            <MessageSquare className="h-8 w-8 text-border mx-auto mb-2" />
            <p className="text-sm">Chat is only available during live streams</p>
          </div>
        ) : messages.length === 0 ? (
          <div className="text-center py-12 text-text-secondary">
            <MessageSquare className="h-8 w-8 text-border mx-auto mb-2" />
            <p className="text-sm">No messages yet. Say something!</p>
          </div>
        ) : (
          messages.map((msg, i) => (
            <ChatMessageBubble key={`${msg.timestamp}-${i}`} message={msg} />
          ))
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* "Scroll to bottom" indicator */}
      {!autoScroll && messages.length > 0 && (
        <button
          onClick={() => {
            setAutoScroll(true)
            messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
          }}
          className="mx-3 mb-1 py-1 text-xs text-accent bg-accent/10 rounded-lg hover:bg-accent/20 transition-colors"
        >
          New messages below
        </button>
      )}

      {/* Input */}
      <form onSubmit={handleSubmit} className="p-3 border-t border-border flex gap-2">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={
            !isLive ? 'Stream is offline' :
            !isLoggedIn ? 'Log in to chat' :
            'Send a message...'
          }
          disabled={!isLive || !isLoggedIn}
          maxLength={500}
          className="flex-1 text-sm disabled:opacity-50"
        />
        <button
          type="submit"
          disabled={!isLive || !isLoggedIn || !input.trim()}
          className="p-2 rounded-lg bg-accent text-white hover:bg-accent-hover disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
        >
          <Send className="h-4 w-4" />
        </button>
      </form>
    </div>
  )
}

function ChatMessageBubble({ message }: { message: ChatMessage }) {
  return (
    <div className="text-sm leading-relaxed break-words">
      <span className="font-medium text-accent hover:underline cursor-pointer">
        {message.display_name}
      </span>
      <span className="text-text-primary ml-1.5">{message.content}</span>
    </div>
  )
}
