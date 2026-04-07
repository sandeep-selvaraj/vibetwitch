import { useEffect } from 'react'
import { Outlet } from 'react-router-dom'
import { Navbar } from './Navbar'
import { useAuthStore } from '../../stores/auth'

export function AppShell() {
  const loadUser = useAuthStore((s) => s.loadUser)

  useEffect(() => {
    loadUser()
  }, [loadUser])

  return (
    <div className="min-h-screen bg-bg-primary">
      <Navbar />
      <main>
        <Outlet />
      </main>
    </div>
  )
}
