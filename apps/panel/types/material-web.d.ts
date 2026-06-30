import type React from 'react'

type MdElement = React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement> & {
  [key: string]: unknown
}

type IntrinsicElementsMap = {
  'md-elevated-card': MdElement
  'md-outlined-card': MdElement
  'md-filled-card': MdElement
  'md-icon': MdElement
  'md-icon-button': MdElement
  'md-filled-button': MdElement
  'md-outlined-button': MdElement
  'md-text-button': MdElement
  'md-linear-progress': MdElement
  'md-divider': MdElement
  'md-chip-set': MdElement
  'md-filter-chip': MdElement
  'md-assist-chip': MdElement
  'md-list': MdElement
  'md-list-item': MdElement
  'md-fab': MdElement
  'md-outlined-text-field': MdElement
  'md-filled-text-field': MdElement
  'md-outlined-select': MdElement
  'md-select-option': MdElement
  'md-switch': MdElement
  'md-checkbox': MdElement
  'md-dialog': MdElement
  'md-circular-progress': MdElement
}

declare module 'react' {
  namespace JSX {
    interface IntrinsicElements extends IntrinsicElementsMap {}
  }
}

declare global {
  namespace JSX {
    interface IntrinsicElements extends IntrinsicElementsMap {}
  }
}

export {}
