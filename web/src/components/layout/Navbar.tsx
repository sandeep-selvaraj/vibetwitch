import { Link } from 'react-router-dom'
import { Monitor, Radio, LayoutDashboard, LogOut } from 'lucide-react'
import { Button } from '../ui/Button'
import { Avatar } from '../ui/Avatar'
import { useAuthStore } from '../../stores/auth'

export function Navbar() {
  const { user, logout } = useAuthStore()

  return (
    <nav className="sticky top-0 z-50 h-14 border-b border-border bg-bg-secondary/80 backdrop-blur-md">
      <div className="mx-auto flex h-full max-w-[1600px] items-center justify-between px-4">
        {/* Left */}
        <div className="flex items-center gap-6">
          <Link to="/" className="flex items-center gap-2 text-lg font-bold text-text-primary hover:text-accent transition-colors">
            <Monitor className="h-5 w-5 text-accent" />
            <span>VibeTwitch</span>
          </Link>
          <div className="hidden sm:flex items-center gap-1">
            <Link to="/browse">
              <Button variant="ghost" size="sm">
                <Radio className="h-4 w-4" />
                Browse
              </Button>
            </Link>
          </div>
        </div>

        {/* Right */}
        <div className="flex items-center gap-3">
          {user ? (
            <>
              <Link to="/dashboard">
                <Button variant="secondary" size="sm">
                  <LayoutDashboard className="h-4 w-4" />
                  Dashboard
                </Button>
              </Link>
              <div className="flex items-center gap-2">
                <Avatar src={user.avatar_url} alt={user.display_name} size="sm" />
                <span className="hidden sm:inline text-sm font-medium">{user.display_name}</span>
              </div>
              <Button variant="ghost" size="sm" onClick={logout}>
                <LogOut className="h-4 w-4" />
              </Button>
            </>
          ) : (
            <>
              <Link to="/login">
                <Button variant="ghost" size="sm">Log in</Button>
              </Link>
              <Link to="/register">
                <Button variant="primary" size="sm">Sign up</Button>
              </Link>
            </>
          )}
        </div>
      </div>
    </nav>
  )
}
