<script setup lang="ts">
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { convertFileSrc } from '@tauri-apps/api/core'
import type { Rect, TraceMode, PipelineParams } from '~/composables/useTracer'

const { imageInfo, svgData, loading, error, loadImage, trace, simplify, exportSvg } = useTracer()
const previewUrl = computed(() => imageInfo.value ? convertFileSrc(imageInfo.value.path) : null)
const { optimizedSvg, originalSize, optimizedSize, savings, plugins, isManualMode, optimizing, doOptimize, reset: resetSvgo } = useSvgo()
const toast = useToast()

const selection = ref<Rect | null>(null)
const mode = ref<TraceMode>({ type: 'Monochrome' })
const smoothness = ref(0)
const hasTraced = ref(false)
const filename = ref<string | null>(null)

/** Whether we're in the export/optimize view (panes 2+3) */
const exportMode = ref(false)

async function resetAndLoad(path: string) {
  selection.value = null
  hasTraced.value = false
  exportMode.value = false
  resetSvgo()
  filename.value = path.split(/[\\/]/).pop() || null
  await loadImage(path)
  if (error.value) {
    toast.add({ title: 'Error', description: error.value, color: 'error' })
  }
}

async function handleOpen() {
  const path = await openDialog({
    filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp'] }],
  })
  if (path) {
    await resetAndLoad(path)
  }
}

async function handleTrace() {
  if (!imageInfo.value) return
  const sel = selection.value ?? {
    x: 0, y: 0,
    width: imageInfo.value.width,
    height: imageInfo.value.height,
  }
  await trace(sel, mode.value, smoothness.value / 100)
  hasTraced.value = true
  if (error.value) {
    toast.add({ title: 'Trace failed', description: error.value, color: 'error' })
  }
  // Re-optimize if in export view so the user sees updated results immediately
  if (exportMode.value && svgData.value) {
    doOptimize(svgData.value.paths, svgData.value.viewbox)
  }
}

// Auto-retrace when mode changes (if already traced), debounced to avoid queue flooding
let modeDebounce: ReturnType<typeof setTimeout> | null = null
watch(mode, () => {
  if (!hasTraced.value) return
  if (modeDebounce) clearTimeout(modeDebounce)
  modeDebounce = setTimeout(handleTrace, 300)
})

async function handlePipelineChange(params: PipelineParams) {
  if (!hasTraced.value) return
  await simplify(params)
  if (error.value) {
    toast.add({ title: 'Simplify failed', description: error.value, color: 'error' })
  }
  // Re-optimize if in export mode
  if (exportMode.value && svgData.value) {
    doOptimize(svgData.value.paths, svgData.value.viewbox)
  }
}

function handleExport() {
  if (!svgData.value) return
  // Run initial SVGO optimization and slide to export view
  doOptimize(svgData.value.paths, svgData.value.viewbox)
  exportMode.value = true
}

function handleBack() {
  exportMode.value = false
}

function handleReoptimize() {
  if (!svgData.value) return
  doOptimize(svgData.value.paths, svgData.value.viewbox)
}

// Debounced re-optimization when plugins change (instant mode)
let pluginDebounce: ReturnType<typeof setTimeout> | null = null
function handlePluginsUpdate(newPlugins: typeof plugins.value) {
  plugins.value = newPlugins
  if (isManualMode.value) return
  if (pluginDebounce) clearTimeout(pluginDebounce)
  pluginDebounce = setTimeout(() => {
    handleReoptimize()
  }, 200)
}

async function handleSave() {
  if (!optimizedSvg.value) return
  const path = await saveDialog({
    filters: [{ name: 'SVG', extensions: ['svg'] }],
    defaultPath: 'traced.svg',
  })
  if (path) {
    try {
      await exportSvg(path, optimizedSvg.value)
      toast.add({ title: 'Exported', description: 'Optimized SVG saved successfully', color: 'success' })
    }
    catch (e) {
      toast.add({ title: 'Export failed', description: String(e), color: 'error' })
    }
  }
}

// Drag and drop support
onMounted(async () => {
  const currentWindow = getCurrentWindow()
  const unlisten = await currentWindow.onDragDropEvent(async (event) => {
    if (event.payload.type === 'drop' && event.payload.paths.length > 0) {
      const path = event.payload.paths[0]
      if (/\.(png|jpe?g|webp)$/i.test(path)) {
        await resetAndLoad(path)
      }
    }
  })
  onUnmounted(unlisten)
})
</script>

<template>
  <div class="h-screen flex flex-col bg-zinc-950 text-white">
    <AppToolbar
      v-model:mode="mode"
      v-model:smoothness="smoothness"
      :has-image="!!imageInfo"
      :has-svg="!!svgData"
      :loading="loading"
      :export-mode="exportMode"
      @open="handleOpen"
      @trace="handleTrace"
      @export="handleExport"
      @save="handleSave"
      @back="handleBack"
      @pipeline-change="handlePipelineChange"
    />
    <div class="flex-1 min-h-0 overflow-hidden relative">
      <div
        class="absolute inset-0 flex transition-transform duration-400 ease-in-out"
        :style="{
          width: '150%',
          transform: exportMode ? 'translateX(-33.333%)' : 'translateX(0)',
        }"
      >
        <SourceCanvas
          class="!flex-none h-full"
          style="width: 33.333%;"
          v-model:selection="selection"
          :preview-url="previewUrl"
          :image-width="imageInfo?.width ?? 0"
          :image-height="imageInfo?.height ?? 0"
          :loading="loading"
        />
        <SvgPreview
          class="!flex-none h-full"
          style="width: 33.333%;"
          :svg-data="svgData"
          :preview-url="previewUrl"
          :image-width="imageInfo?.width ?? 0"
          :image-height="imageInfo?.height ?? 0"
          :selection="selection"
          :loading="loading"
        />
        <OptimizedPreview
          class="!flex-none h-full"
          style="width: 33.333%;"
          :optimized-svg="optimizedSvg"
          :original-size="originalSize"
          :optimized-size="optimizedSize"
          :savings="savings"
          :plugins="plugins"
          :is-manual-mode="isManualMode"
          :optimizing="optimizing"
          @update:plugins="handlePluginsUpdate"
          @reoptimize="handleReoptimize"
        />
      </div>
    </div>
  </div>
</template>
