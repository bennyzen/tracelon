import { invoke } from '@tauri-apps/api/core'

export interface ImageInfo {
  width: number
  height: number
  thumbnailBase64: string
}

export interface SvgData {
  paths: string
  pathCount: number
  segmentCount: number
  rawSegmentCount: number
  viewbox: string
  estimatedSize: number
}

export interface Rect {
  x: number
  y: number
  width: number
  height: number
}

export interface PipelineParams {
  smoothness: number
  lineSnap: number
  cornerAngle: number
}

export type TraceMode =
  | { type: 'Monochrome' }
  | { type: 'MultiColor'; colors: number }
  | { type: 'Outline' }

// Force the browser to paint the loading state before invoke() blocks the JS thread.
async function waitForPaint(): Promise<void> {
  // Flush Vue DOM updates
  await nextTick()
  // Force synchronous layout recalculation
  void document.body.offsetHeight
  // Wait two frames + extra time for the compositor to actually render
  await new Promise<void>(resolve => {
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        setTimeout(resolve, 32)
      })
    })
  })
}

export function useTracer() {
  const imageInfo = ref<ImageInfo | null>(null)
  const svgData = ref<SvgData | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadImage(path: string) {
    loading.value = true
    error.value = null
    try {
      imageInfo.value = await invoke<ImageInfo>('load_image', { path })
      svgData.value = null
    }
    catch (e) {
      error.value = String(e)
    }
    finally {
      loading.value = false
    }
  }

  async function trace(selection: Rect, mode: TraceMode, smoothness: number) {
    loading.value = true
    error.value = null
    await waitForPaint()
    const minDelay = new Promise(resolve => setTimeout(resolve, 400))
    try {
      const [result] = await Promise.all([
        invoke<SvgData>('trace', { selection, mode, smoothness }),
        minDelay,
      ])
      svgData.value = result
    }
    catch (e) {
      error.value = String(e)
    }
    finally {
      loading.value = false
    }
  }

  async function simplify(params: PipelineParams) {
    loading.value = true
    error.value = null
    await waitForPaint()
    const minDelay = new Promise(resolve => setTimeout(resolve, 300))
    try {
      const [result] = await Promise.all([
        invoke<SvgData>('simplify', { params }),
        minDelay,
      ])
      svgData.value = result
    }
    catch (e) {
      error.value = String(e)
    }
    finally {
      loading.value = false
    }
  }

  async function exportSvg(outputPath: string, optimizedSvgContent?: string) {
    error.value = null
    try {
      if (optimizedSvgContent) {
        // Export pre-optimized SVG directly
        await invoke('export_optimized_svg', {
          svgContent: optimizedSvgContent,
          outputPath,
        })
      }
      else {
        // Legacy: export from traced paths
        if (!svgData.value) return
        await invoke('export_svg', {
          svgData: svgData.value.paths,
          viewbox: svgData.value.viewbox,
          outputPath,
        })
      }
    }
    catch (e) {
      error.value = String(e)
    }
  }

  return { imageInfo, svgData, loading, error, loadImage, trace, simplify, exportSvg }
}
