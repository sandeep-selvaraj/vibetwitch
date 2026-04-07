type MessageHandler = (data: unknown) => void

/**
 * Managed WebSocket connection with auto-reconnect.
 */
export class WsConnection {
  private ws: WebSocket | null = null
  private url: string
  private handlers: MessageHandler[] = []
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null
  private intentionallyClosed = false
  private reconnectDelay = 1000

  constructor(url: string) {
    this.url = url
  }

  connect() {
    this.intentionallyClosed = false
    this.doConnect()
  }

  private doConnect() {
    if (this.ws?.readyState === WebSocket.OPEN) return

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const fullUrl = `${protocol}//${window.location.host}${this.url}`

    this.ws = new WebSocket(fullUrl)

    this.ws.onopen = () => {
      this.reconnectDelay = 1000 // reset on successful connect
    }

    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)
        for (const handler of this.handlers) {
          handler(data)
        }
      } catch {
        // ignore invalid JSON
      }
    }

    this.ws.onclose = () => {
      if (!this.intentionallyClosed) {
        this.scheduleReconnect()
      }
    }

    this.ws.onerror = () => {
      this.ws?.close()
    }
  }

  private scheduleReconnect() {
    if (this.reconnectTimer) return
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null
      this.reconnectDelay = Math.min(this.reconnectDelay * 1.5, 10000)
      this.doConnect()
    }, this.reconnectDelay)
  }

  onMessage(handler: MessageHandler) {
    this.handlers.push(handler)
  }

  send(data: unknown) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data))
    }
  }

  close() {
    this.intentionallyClosed = true
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
    this.ws?.close()
    this.ws = null
    this.handlers = []
  }
}
