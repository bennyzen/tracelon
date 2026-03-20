<script setup lang="ts">
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { Rect, TraceMode, PipelineParams } from '~/composables/useTracer'

const { imageInfo, svgData, loading, error, loadImage, trace, simplify, exportSvg } = useTracer()
const svgo = useSvgo()
const toast = useToast()

const selection = ref<Rect | null>(null)
const mode = ref<TraceMode>({ type: 'Monochrome' })
const smoothness = ref(0)
const hasTraced = ref(false)
const filename = ref<string | null>(null)

/** Whether we're in the export/optimize view (panes 2+3) */
const exportMode = ref(false)

async function handleOpen() {
  const path = await openDialog({
    filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp'] }],
  })
  if (path) {
    selection.value = null
    hasTraced.value = false
    exportMode.value = false
    svgo.reset()
    filename.value = (path as string).split('/').pop() || null
    await loadImage(path as string)
    if (error.value) {
      toast.add({ title: 'Error', description: error.value, color: 'error' })
    }
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
}

async function handlePipelineChange(params: PipelineParams) {
  if (!hasTraced.value) return
  await simplify(params)
  if (error.value) {
    toast.add({ title: 'Simplify failed', description: error.value, color: 'error' })
  }
}

function handleExport() {
  if (!svgData.value) return
  // Run initial SVGO optimization and slide to export view
  svgo.doOptimize(svgData.value.paths, svgData.value.viewbox)
  exportMode.value = true
}

function handleBack() {
  exportMode.value = false
}

function handleReoptimize() {
  if (!svgData.value) return
  svgo.doOptimize(svgData.value.paths, svgData.value.viewbox)
}

// Debounced re-optimization when plugins change (instant mode)
let pluginDebounce: ReturnType<typeof setTimeout> | null = null
function handlePluginsUpdate(plugins: typeof svgo.plugins.value) {
  svgo.plugins.value = plugins
  if (svgo.isManualMode.value) return
  if (pluginDebounce) clearTimeout(pluginDebounce)
  pluginDebounce = setTimeout(() => {
    handleReoptimize()
  }, 200)
}

async function handleSave() {
  if (!svgo.optimizedSvg.value) return
  const path = await saveDialog({
    filters: [{ name: 'SVG', extensions: ['svg'] }],
    defaultPath: 'traced.svg',
  })
  if (path) {
    try {
      await exportSvg(path as string, svgo.optimizedSvg.value)
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
  await currentWindow.onDragDropEvent(async (event) => {
    if (event.payload.type === 'drop' && event.payload.paths.length > 0) {
      const path = event.payload.paths[0]
      if (/\.(png|jpe?g|webp)$/i.test(path)) {
        selection.value = null
        hasTraced.value = false
        exportMode.value = false
        svgo.reset()
        filename.value = path.split('/').pop() || null
        await loadImage(path)
        if (error.value) {
          toast.add({ title: 'Error', description: error.value, color: 'error' })
        }
      }
    }
  })
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
          :thumbnail-base64="imageInfo?.thumbnailBase64 ?? null"
          :image-width="imageInfo?.width ?? 0"
          :image-height="imageInfo?.height ?? 0"
        />
        <SvgPreview
          class="!flex-none h-full"
          style="width: 33.333%;"
          :svg-data="svgData"
          :thumbnail-base64="imageInfo?.thumbnailBase64 ?? null"
          :loading="loading"
        />
        <OptimizedPreview
          class="!flex-none h-full"
          style="width: 33.333%;"
          :optimized-svg="svgo.optimizedSvg.value"
          :original-size="svgo.originalSize.value"
          :optimized-size="svgo.optimizedSize.value"
          :savings="svgo.savings.value"
          :plugins="svgo.plugins.value"
          :is-manual-mode="svgo.isManualMode.value"
          :optimizing="svgo.optimizing.value"
          @update:plugins="handlePluginsUpdate"
          @reoptimize="handleReoptimize"
        />
      </div>
    </div>
  </div>
</template>
