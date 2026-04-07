interface BadgeProps {
  children: React.ReactNode
  variant?: 'default' | 'live' | 'accent'
  className?: string
}

const variantClasses = {
  default: 'bg-bg-tertiary text-text-secondary border border-border',
  live: 'bg-live/20 text-live border border-live/30',
  accent: 'bg-accent/10 text-accent border border-accent/30',
}

export function Badge({ children, variant = 'default', className = '' }: BadgeProps) {
  return (
    <span
      className={`inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-medium ${variantClasses[variant]} ${className}`}
    >
      {children}
    </span>
  )
}
