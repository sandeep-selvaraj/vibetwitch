import { useState } from 'react'
import { Copy, Check } from 'lucide-react'

interface CodeBlockProps {
  code: string
  language?: string
}

export function CodeBlock({ code, language }: CodeBlockProps) {
  const [copied, setCopied] = useState(false)

  const handleCopy = () => {
    navigator.clipboard.writeText(code)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="my-2 rounded-lg border border-border overflow-hidden bg-bg-primary">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-1.5 bg-bg-tertiary border-b border-border">
        <span className="text-xs text-text-secondary font-mono">
          {language || 'code'}
        </span>
        <button
          onClick={handleCopy}
          className="p-1 rounded hover:bg-border/50 text-text-secondary hover:text-text-primary transition-colors"
        >
          {copied ? (
            <Check className="h-3.5 w-3.5 text-success" />
          ) : (
            <Copy className="h-3.5 w-3.5" />
          )}
        </button>
      </div>
      {/* Code */}
      <pre className="p-3 overflow-x-auto text-xs leading-relaxed">
        <code className="text-text-primary">{code}</code>
      </pre>
    </div>
  )
}
