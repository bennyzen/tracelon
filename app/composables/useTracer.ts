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

// Wait for Vue to flush DOM updates AND the browser to paint a frame
function waitForPaint(): Promise<void> {
  return new Promise(resolve => {
    requestAnimationFrame(() => {
      requestAnimationFrame(() => resolve())
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
    try {
      svgData.value = await invoke<SvgData>('trace', { selection, mode, smoothness })
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
    try {
      svgData.value = await invoke<SvgData>('simplify', { params })
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
