import { useState } from 'react'
import { MessageSquareCode, GitBranch, FolderOpen } from 'lucide-react'
import type { ParsedEvent } from '../../hooks/useEventsWs'
import { ConversationView } from './ConversationView'
import { DiffView } from './DiffView'
import { FileTreeView } from './FileTreeView'

interface StructuredPanelProps {
  events: ParsedEvent[]
  isLive: boolean
}

type Tab = 'conversation' | 'diffs' | 'files'

const tabs: { id: Tab; label: string; icon: React.ReactNode }[] = [
  { id: 'conversation', label: 'Conversation', icon: <MessageSquareCode className="h-3.5 w-3.5" /> },
  { id: 'diffs', label: 'Diffs', icon: <GitBranch className="h-3.5 w-3.5" /> },
  { id: 'files', label: 'Files', icon: <FolderOpen className="h-3.5 w-3.5" /> },
]

export function StructuredPanel({ events, isLive }: StructuredPanelProps) {
  const [activeTab, setActiveTab] = useState<Tab>('conversation')

  // Count badges
  const conversationCount = events.filter((e) =>
    ['ai_message', 'ai_response', 'tool_use'].includes(e.eventType)
  ).length
  const diffCount = events.filter((e) => e.eventType === 'code_diff').length
  const fileCount = new Set(
    events
      .filter((e) => e.eventType === 'file_change' || e.eventType === 'code_diff')
      .map((e) => e.payload.file_path as string)
  ).size

  const counts: Record<Tab, number> = {
    conversation: conversationCount,
    diffs: diffCount,
    files: fileCount,
  }

  return (
    <div className="w-[400px] border-l border-border flex flex-col bg-bg-secondary">
      {/* Tab bar */}
      <div className="flex border-b border-border">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex-1 flex items-center justify-center gap-1.5 px-3 py-2.5 text-xs font-medium transition-colors
              ${activeTab === tab.id
                ? 'text-accent border-b-2 border-accent bg-accent/5'
                : 'text-text-secondary hover:text-text-primary'
              }`}
          >
            {tab.icon}
            {tab.label}
            {counts[tab.id] > 0 && (
              <span className={`ml-1 rounded-full px-1.5 py-0.5 text-[10px] font-bold
                ${activeTab === tab.id ? 'bg-accent/20 text-accent' : 'bg-bg-tertiary text-text-secondary'}`}>
                {counts[tab.id]}
              </span>
            )}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4">
        {!isLive ? (
          <div className="text-center py-12 text-text-secondary">
            <MessageSquareCode className="h-10 w-10 text-border mx-auto mb-3" />
            <p className="text-sm">Stream is offline</p>
          </div>
        ) : activeTab === 'conversation' ? (
          <ConversationView events={events} />
        ) : activeTab === 'diffs' ? (
          <DiffView events={events} />
        ) : (
          <FileTreeView events={events} />
        )}
      </div>
    </div>
  )
}
