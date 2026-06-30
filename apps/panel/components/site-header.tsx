'use client'

import { ThemeToggle } from './theme-toggle'

export function SiteHeader() {
  return (
    <header className="site-header">
      <div className="site-header-inner">
        <a
          href="/"
          className="wordmark md-typescale-title-large"
          aria-label="Presence 主页"
        >
          <md-icon class="wordmark-icon">devices</md-icon>
          <span className="wordmark-text">Presence</span>
        </a>
        <div className="site-header-actions">
          <a href="/control" aria-label="控制面板">
            <md-icon-button>
              <md-icon>settings</md-icon>
            </md-icon-button>
          </a>
          <ThemeToggle />
        </div>
      </div>
    </header>
  )
}
