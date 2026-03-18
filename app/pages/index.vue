<script setup lang="ts">
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { Rect, TraceMode, PipelineParams } from '~/composables/useTracer'

const { imageInfo, svgData, loading, error, loadImage, trace, simplify, exportSvg } = useTracer()
const toast = useToast()

const selection = ref<Rect | null>(null)
const mode = ref<TraceMode>({ type: 'Monochrome' })
const smoothness = ref(0)
const hasTraced = ref(false)
const filename = ref<string | null>(null)

async function handleOpen() {
  const path = await openDialog({
    filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp'] }],
  })
  if (path) {
    selection.value = null
    hasTraced.value = false
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

async function handleExport() {
  const path = await saveDialog({
    filters: [{ name: 'SVG', extensions: ['svg'] }],
    defaultPath: 'traced.svg',
  })
  if (path) {
    await exportSvg(path as string)
    if (error.value) {
      toast.add({ title: 'Export failed', description: error.value, color: 'error' })
    }
    else {
      toast.add({ title: 'Exported', description: 'SVG saved successfully', color: 'success' })
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
      @open="handleOpen"
      @trace="handleTrace"
      @export="handleExport"
      @pipeline-change="handlePipelineChange"
    />
    <div class="flex-1 flex min-h-0">
      <SourceCanvas
        v-model:selection="selection"
        :thumbnail-base64="imageInfo?.thumbnailBase64 ?? null"
        :image-width="imageInfo?.width ?? 0"
        :image-height="imageInfo?.height ?? 0"
      />
      <SvgPreview
        :svg-data="svgData"
        :thumbnail-base64="imageInfo?.thumbnailBase64 ?? null"
        :loading="loading"
      />
    </div>
  </div>
</template>
