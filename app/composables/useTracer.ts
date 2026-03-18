import { invoke } from '@tauri-apps/api/core'

export interface ImageInfo {
  width: number
  height: number
  thumbnailBase64: string
}

export interface SvgData {
  paths: string
  pathCount: number
  viewbox: string
  estimatedSize: number
}

export interface Rect {
  x: number
  y: number
  width: number
  height: number
}

export type TraceMode =
  | { type: 'Monochrome' }
  | { type: 'MultiColor'; colors: number }
  | { type: 'Outline' }

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
    // Let Vue render the loading state before blocking on IPC
    await new Promise(r => setTimeout(r, 50))
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

  async function simplify(smoothness: number) {
    loading.value = true
    error.value = null
    await new Promise(r => setTimeout(r, 50))
    try {
      svgData.value = await invoke<SvgData>('simplify', { smoothness })
    }
    catch (e) {
      error.value = String(e)
    }
    finally {
      loading.value = false
    }
  }

  async function exportSvg(outputPath: string) {
    if (!svgData.value) return
    error.value = null
    try {
      await invoke('export_svg', {
        svgData: svgData.value.paths,
        viewbox: svgData.value.viewbox,
        outputPath,
      })
    }
    catch (e) {
      error.value = String(e)
    }
  }

  return { imageInfo, svgData, loading, error, loadImage, trace, simplify, exportSvg }
}
