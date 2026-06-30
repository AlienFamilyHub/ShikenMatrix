import { SiteHeader } from '@/components/site-header'
import { StatusDashboard } from '@/components/status-dashboard'
import './dashboard.css'

export default function HomePage() {
  return (
    <>
      <SiteHeader />
      <main className="page">
        <StatusDashboard />
      </main>
    </>
  )
}
