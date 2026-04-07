import { useEffect, useRef, useState, useCallback } from 'react'
import { WsConnection } from '../lib/ws'

export interface CodeEventMessage {
  type: 'code_event'
  sequence: number
  event: {
    event_type: string
    payload: Record<string, unknown>
  }
}

export interface ParsedEvent {
  sequence: number
  eventType: string
  payload: Record<string, unknown>
  timestamp: number
}

export function useEventsWs(streamId: string | undefined, isLive: boolean) {
  const [events, setEvents] = useState<ParsedEvent[]>([])
  const wsRef = useRef<WsConnection | null>(null)

  // Backfill existing events on connect
  const backfill = useCallback(async (id: string) => {
    try {
      const res = await fetch(`/api/streams/${id}/events?limit=200`)
      if (!res.ok) return
      const data = await res.json()
      const parsed: ParsedEvent[] = data.map((row: { sequence: number; event_type: string; payload: Record<string, unknown>; created_at: string }) => ({
        sequence: row.sequence,
        eventType: row.event_type,
        payload: row.payload,
        timestamp: new Date(row.created_at).getTime(),
      }))
      setEvents(parsed)
    } catch {
      // ignore backfill errors
    }
  }, [])

  useEffect(() => {
    if (!streamId || !isLive) {
      setEvents([])
      return
    }

    backfill(streamId)

    const url = `/ws/events/${streamId}`
    const ws = new WsConnection(url)
    wsRef.current = ws

    ws.onMessage((data: unknown) => {
      const msg = data as CodeEventMessage
      if (msg.type === 'code_event') {
        const parsed: ParsedEvent = {
          sequence: msg.sequence,
          eventType: msg.event.event_type,
          payload: msg.event.payload,
          timestamp: Date.now(),
        }
        setEvents((prev) => {
          // Deduplicate by sequence
          if (prev.some((e) => e.sequence === parsed.sequence)) return prev
          const next = [...prev, parsed]
          return next.length > 1000 ? next.slice(-1000) : next
        })
      }
    })

    ws.connect()

    return () => {
      ws.close()
      wsRef.current = null
    }
  }, [streamId, isLive, backfill])

  return events
}
