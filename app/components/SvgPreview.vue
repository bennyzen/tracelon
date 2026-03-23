<script setup lang="ts">
import type { SvgData, Rect } from '~/composables/useTracer'

const props = defineProps<{
  svgData: SvgData | null
  previewUrl: string | null
  imageWidth: number
  imageHeight: number
  selection: Rect | null
  loading: boolean
}>()

const showOverlay = ref(false)
const overlayCanvasRef = ref<HTMLCanvasElement | null>(null)
const overlayImg = ref<HTMLImageElement | null>(null)

const { zoom, panX, panY, isPanning, containerRef, zoomPercent, isViewModified, onWheel, onMouseDown, onMouseMove, onMouseUp, resetView } = useZoomPan()

// Parse viewBox dimensions once
const svgDims = computed(() => {
  if (!props.svgData) return { w: 800, h: 600 }
  const vb = props.svgData.viewbox.split(' ')
  return { w: Number(vb[2]) || 800, h: Number(vb[3]) || 600 }
})

// Bake zoom into SVG width/height so the browser renders vector paths at full resolution
const svgHtml = computed(() => {
  if (!props.svgData) return ''
  const { w, h } = svgDims.value
  const zw = w * zoom.value
  const zh = h * zoom.value
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${props.svgData.viewbox}" width="${zw}" height="${zh}" style="display:block;">${props.svgData.paths}</svg>`
})

// Load the source image for overlay
watch(() => props.previewUrl, (url) => {
  if (!url) { overlayImg.value = null; return }
  const img = new Image()
  img.onload = () => { overlayImg.value = img; drawOverlay() }
  img.src = url
})

// Redraw overlay when zoom, selection, or show state changes
watch([zoom, showOverlay, () => props.svgData?.viewbox], () => {
  nextTick(drawOverlay)
})

function drawOverlay() {
  const canvas = overlayCanvasRef.value
  const img = overlayImg.value
  if (!canvas || !img) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const { w, h } = svgDims.value
  const cw = w * zoom.value
  const ch = h * zoom.value
  canvas.width = cw
  canvas.height = ch

  // Source region: selection or full image
  const sx = props.selection?.x ?? 0
  const sy = props.selection?.y ?? 0
  const sw = props.selection?.width ?? props.imageWidth
  const sh = props.selection?.height ?? props.imageHeight

  ctx.clearRect(0, 0, cw, ch)
  ctx.drawImage(img, sx, sy, sw, sh, 0, 0, cw, ch)
}

// Reset zoom/pan only when a new image is traced (viewbox changes), not on re-simplification
watch(() => props.svgData?.viewbox, (newVb, oldVb) => {
  if (newVb !== oldVb) {
    resetView()
  }
})
</script>

<template>
  <div class="flex flex-col w-full h-full">
    <div class="px-3 py-1.5 bg-zinc-900 border-b border-zinc-800 text-xs uppercase tracking-wider flex justify-between">
      <span class="text-zinc-500">SVG Preview</span>
      <span v-if="svgData" class="text-emerald-500">
        {{ svgData.pathCount }} paths &bull;
        <template v-if="svgData.segmentCount < svgData.rawSegmentCount">
          {{ svgData.rawSegmentCount }} &rarr; {{ svgData.segmentCount }} segments
        </template>
        <template v-else>
          {{ svgData.segmentCount }} segments
        </template>
        &bull; {{ formatSize(svgData.estimatedSize) }}
      </span>
    </div>
    <div
      ref="containerRef"
      class="flex-1 relative overflow-hidden bg-zinc-950"
      :class="isPanning ? 'cursor-grabbing' : 'cursor-grab'"
      @wheel.prevent="onWheel"
      @mousedown="onMouseDown"
      @mousemove="onMouseMove"
      @mouseup="onMouseUp"
      @mouseleave="onMouseUp"
    >
      <!-- Loading overlay — always in DOM, instant show, fade out -->
      <div
        class="absolute inset-0 z-10 flex flex-col items-center justify-center bg-zinc-950/80"
        :class="loading ? 'opacity-100' : 'opacity-0 pointer-events-none transition-opacity duration-300'"
      >
        <div class="w-8 h-8 border-2 border-purple-500 border-t-transparent rounded-full animate-spin mb-3" />
        <span class="text-sm text-purple-400">Tracing...</span>
      </div>
      <div
        v-if="svgData"
        class="absolute inset-0 flex items-center justify-center"
      >
        <div
          :style="{
            transform: `translate(${panX}px, ${panY}px)`,
          }"
        >
          <div class="relative">
            <div
              class="rounded bg-checkerboard"
              v-html="svgHtml"
            />
            <canvas
              v-if="showOverlay && previewUrl"
              ref="overlayCanvasRef"
              class="absolute top-0 left-0 opacity-30 pointer-events-none"
              :style="{ width: `${svgDims.w * zoom}px`, height: `${svgDims.h * zoom}px` }"
            />
          </div>
        </div>
      </div>
      <div v-else class="absolute inset-0 flex items-center justify-center text-zinc-600">
        Trace an image to see the preview
      </div>
    </div>
    <div class="px-3 h-8 bg-zinc-900 border-t border-zinc-800 text-xs text-zinc-600 flex gap-4 items-center">
      <UCheckbox v-model="showOverlay" label="Show overlay" />
      <UCheckbox disabled label="Control points" class="opacity-50" title="Coming soon" />
      <div class="flex-1" />
      <span v-if="svgData" class="text-zinc-500">{{ zoomPercent }}%</span>
      <UButton
        v-if="svgData && isViewModified"
        variant="link"
        color="neutral"
        size="xs"
        label="Reset view"
        @click="resetView"
      />
    </div>
  </div>
</template>
