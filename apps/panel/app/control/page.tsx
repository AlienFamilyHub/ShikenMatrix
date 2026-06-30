import { SiteHeader } from '@/components/site-header'
import { ControlPanel } from '@/components/control-panel'
import '../dashboard.css'
import '../control.css'

export default function ControlPage() {
  return (
    <>
      <SiteHeader />
      <main className="page">
        <ControlPanel />
      </main>
    </>
  )
}
