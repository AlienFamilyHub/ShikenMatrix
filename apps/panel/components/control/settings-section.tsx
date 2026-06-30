import type { ReactNode } from 'react'

export function SettingsSection({
  icon,
  title,
  description,
  children,
}: {
  icon: string
  title: string
  description?: string
  children: ReactNode
}) {
  return (
    <md-outlined-card className="settings-card">
      <header className="settings-card-head">
        <span className="settings-card-icon" aria-hidden="true">
          <md-icon>{icon}</md-icon>
        </span>
        <div className="settings-card-titles">
          <h2 className="md-typescale-title-medium">{title}</h2>
          {description ? (
            <p className="md-typescale-body-small settings-card-desc">
              {description}
            </p>
          ) : null}
        </div>
      </header>
      <div className="settings-card-body">{children}</div>
    </md-outlined-card>
  )
}

export function FieldRow({
  label,
  hint,
  control,
}: {
  label: string
  hint?: string
  control: ReactNode
}) {
  return (
    <label className="toggle-row">
      <span className="toggle-row-text">
        <span className="md-typescale-body-large">{label}</span>
        {hint ? (
          <span className="md-typescale-body-small toggle-row-hint">{hint}</span>
        ) : null}
      </span>
      {control}
    </label>
  )
}
