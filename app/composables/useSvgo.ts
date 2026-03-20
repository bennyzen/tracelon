import { optimize } from 'svgo'
import type { Config } from 'svgo'

export interface SvgoPlugin {
  id: string
  label: string
  enabled: boolean
  /** For plugins with floatPrecision param */
  precision?: number
}

const DEFAULT_PLUGINS: SvgoPlugin[] = [
  { id: 'convertPathData', label: 'Round path data', enabled: true, precision: 2 },
  { id: 'cleanupNumericValues', label: 'Round attributes', enabled: true, precision: 2 },
  { id: 'mergePaths', label: 'Merge paths', enabled: true },
  { id: 'collapseGroups', label: 'Collapse groups', enabled: true },
  { id: 'convertTransform', label: 'Simplify transforms', enabled: true },
]

export function useSvgo() {
  const plugins = ref<SvgoPlugin[]>(DEFAULT_PLUGINS.map(p => ({ ...p })))
  const optimizedSvg = ref<string | null>(null)
  const originalSize = ref(0)
  const optimizedSize = ref(0)
  const isManualMode = ref(false)
  const optimizing = ref(false)

  function buildConfig(): Config {
    const activePlugins: Config['plugins'] = []
    for (const p of plugins.value) {
      if (!p.enabled) continue
      if (p.precision !== undefined) {
        activePlugins.push({
          name: p.id,
          params: { floatPrecision: p.precision },
        } as any)
      }
      else {
        activePlugins.push(p.id as any)
      }
    }
    return {
      multipass: true,
      plugins: activePlugins,
    }
  }

  function runOptimize(svgString: string): string {
    const result = optimize(svgString, buildConfig())
    return result.data
  }

  /** Build a full SVG document from paths + viewbox (mirrors export.rs logic) */
  function buildSvgDocument(paths: string, viewbox: string): string {
    return `<?xml version="1.0" encoding="UTF-8"?>\n<svg xmlns="http://www.w3.org/2000/svg" viewBox="${viewbox}">\n${paths}\n</svg>`
  }

  function doOptimize(paths: string, viewbox: string) {
    optimizing.value = true
    try {
      const fullSvg = buildSvgDocument(paths, viewbox)
      originalSize.value = new Blob([fullSvg]).size

      const start = performance.now()
      const result = runOptimize(fullSvg)
      const elapsed = performance.now() - start

      optimizedSvg.value = result
      optimizedSize.value = new Blob([result]).size

      // Switch to manual mode if optimization is slow
      if (elapsed > 500) {
        isManualMode.value = true
      }
    }
    finally {
      optimizing.value = false
    }
  }

  function reset() {
    optimizedSvg.value = null
    originalSize.value = 0
    optimizedSize.value = 0
    isManualMode.value = false
  }

  const savings = computed(() => {
    if (!originalSize.value) return 0
    return Math.round((1 - optimizedSize.value / originalSize.value) * 100)
  })

  return {
    plugins,
    optimizedSvg,
    originalSize,
    optimizedSize,
    savings,
    isManualMode,
    optimizing,
    doOptimize,
    reset,
  }
}
