'use client'

import { useEffect } from 'react'

/**
 * Registers all @material/web custom element definitions on the client,
 * after hydration. Importing the definitions at module top level would
 * upgrade the elements before React hydrates, and Lit elements mutate
 * their own attributes on upgrade (e.g. aria-label -> data-aria-label),
 * which triggers hydration mismatch errors. Loading in an effect means
 * React hydrates the plain server-rendered tags first, then the elements
 * upgrade in place.
 *
 * `all.js` registers every non-labs component. Labs components (cards,
 * segmented buttons, etc.) must be added here individually, e.g.:
 *   import("@material/web/labs/card/elevated-card.js")
 */
export function MaterialWebLoader() {
  useEffect(() => {
    import('@material/web/all.js')
    // Labs components are not part of all.js — register the ones we use.
    import('@material/web/labs/card/elevated-card.js')
    import('@material/web/labs/card/outlined-card.js')
    import('@material/web/labs/card/filled-card.js')
  }, [])

  return null
}
