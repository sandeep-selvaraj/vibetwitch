import { useEffect, useRef, useState, useCallback } from 'react'
import { WsConnection } from '../lib/ws'

export interface ChatMessage {
  type: 'chat_message'
  username: string
  display_name: string
  content: string
  timestamp: string
}

interface UseChatWsResult {
  messages: ChatMessage[]
  viewerCount: number
  sendMessage: (content: string) => void
  connected: boolean
}

export function useChatWs(streamId: string | undefined, isLive: boolean): UseChatWsResult {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [viewerCount, setViewerCount] = useState(0)
  const [connected, setConnected] = useState(false)
  const wsRef = useRef<WsConnection | null>(null)

  useEffect(() => {
    if (!streamId || !isLive) {
      setMessages([])
      setViewerCount(0)
      setConnected(false)
      return
    }

    const token = localStorage.getItem('token') || ''
    const url = `/ws/chat/${streamId}${token ? `?token=${token}` : ''}`
    const ws = new WsConnection(url)
    wsRef.current = ws

    ws.onMessage((data: unknown) => {
      const msg = data as { type: string; [key: string]: unknown }
      if (msg.type === 'chat_message') {
        setMessages((prev) => {
          const next = [...prev, msg as unknown as ChatMessage]
          // Keep last 500 messages in memory
          return next.length > 500 ? next.slice(-500) : next
        })
      } else if (msg.type === 'viewer_count') {
        setViewerCount(msg.count as number)
      }
    })

    ws.connect()
    setConnected(true)

    return () => {
      ws.close()
      wsRef.current = null
      setConnected(false)
    }
  }, [streamId, isLive])

  const sendMessage = useCallback((content: string) => {
    wsRef.current?.send({ type: 'chat_message', content })
  }, [])

  return { messages, viewerCount, sendMessage, connected }
}
