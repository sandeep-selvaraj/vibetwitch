import { useMemo } from 'react'
import { File, FilePlus, FileX, FileEdit, FolderOpen } from 'lucide-react'
import type { ParsedEvent } from '../../hooks/useEventsWs'

interface FileTreeViewProps {
  events: ParsedEvent[]
}

export function FileTreeView({ events }: FileTreeViewProps) {
  // Build a deduplicated map of file → latest change type
  const fileChanges = useMemo(() => {
    const map = new Map<string, string>()
    for (const event of events) {
      if (event.eventType === 'file_change') {
        const path = event.payload.file_path as string
        const changeType = event.payload.change_type as string
        map.set(path, changeType)
      }
      if (event.eventType === 'code_diff') {
        const path = event.payload.file_path as string
        map.set(path, 'modified')
      }
    }
    return map
  }, [events])

  if (fileChanges.size === 0) {
    return (
      <div className="text-center py-12 text-text-secondary">
        <FolderOpen className="h-10 w-10 text-border mx-auto mb-3" />
        <p className="font-medium">No file changes yet</p>
        <p className="text-sm mt-1">Changed files will be listed here.</p>
      </div>
    )
  }

  // Sort files: group by directory
  const sorted = [...fileChanges.entries()].sort(([a], [b]) => a.localeCompare(b))

  return (
    <div className="space-y-0.5">
      {sorted.map(([filePath, changeType]) => (
        <FileEntry key={filePath} filePath={filePath} changeType={changeType} />
      ))}
    </div>
  )
}

function FileEntry({ filePath, changeType }: { filePath: string; changeType: string }) {
  const Icon = changeType === 'created' ? FilePlus
    : changeType === 'deleted' ? FileX
    : changeType === 'modified' ? FileEdit
    : File

  const colorClass = changeType === 'created' ? 'text-success'
    : changeType === 'deleted' ? 'text-danger'
    : 'text-accent'

  return (
    <div className="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-bg-tertiary text-sm group">
      <Icon className={`h-4 w-4 shrink-0 ${colorClass}`} />
      <span className="font-mono text-xs text-text-primary truncate">{filePath}</span>
      <span className={`text-xs ${colorClass} ml-auto shrink-0 opacity-60 group-hover:opacity-100`}>
        {changeType}
      </span>
    </div>
  )
}
