<script setup lang="ts">
import type { SvgData } from '~/composables/useTracer'

const props = defineProps<{
  svgData: SvgData | null
  thumbnailBase64: string | null
  loading: boolean
}>()

const showOverlay = ref(true)

// Zoom & pan state
const zoom = ref(1)
const panX = ref(0)
const panY = ref(0)
const isPanning = ref(false)
const lastMouse = ref({ x: 0, y: 0 })
const containerRef = ref<HTMLElement | null>(null)

const MIN_ZOOM = 0.1
const MAX_ZOOM = 20

const svgHtml = computed(() => {
  if (!props.svgData) return ''
  const vb = props.svgData.viewbox.split(' ')
  const w = vb[2] || '800'
  const h = vb[3] || '600'
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${props.svgData.viewbox}" width="${w}" height="${h}" style="display:block;">${props.svgData.paths}</svg>`
})

// Reset zoom/pan when new SVG data arrives
watch(() => props.svgData, () => {
  zoom.value = 1
  panX.value = 0
  panY.value = 0
})

function onWheel(e: WheelEvent) {
  e.preventDefault()
  const container = containerRef.value
  if (!container) return

  const rect = container.getBoundingClientRect()
  // Mouse position relative to container center
  const mx = e.clientX - rect.left - rect.width / 2
  const my = e.clientY - rect.top - rect.height / 2

  const oldZoom = zoom.value
  const factor = e.deltaY < 0 ? 1.15 : 1 / 1.15
  const newZoom = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, oldZoom * factor))

  // Zoom toward mouse position
  const scale = newZoom / oldZoom
  panX.value = mx - scale * (mx - panX.value)
  panY.value = my - scale * (my - panY.value)
  zoom.value = newZoom
}

function onMouseDown(e: MouseEvent) {
  if (e.button === 0 || e.button === 1) {
    isPanning.value = true
    lastMouse.value = { x: e.clientX, y: e.clientY }
    e.preventDefault()
  }
}

function onMouseMove(e: MouseEvent) {
  if (!isPanning.value) return
  panX.value += e.clientX - lastMouse.value.x
  panY.value += e.clientY - lastMouse.value.y
  lastMouse.value = { x: e.clientX, y: e.clientY }
}

function onMouseUp() {
  isPanning.value = false
}

function resetView() {
  zoom.value = 1
  panX.value = 0
  panY.value = 0
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  return `${(bytes / 1024).toFixed(1)} KB`
}

const zoomPercent = computed(() => Math.round(zoom.value * 100))
</script>

<template>
  <div class="flex flex-col flex-1">
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
      <!-- Loading overlay -->
      <div v-if="loading" class="absolute inset-0 z-10 flex flex-col items-center justify-center bg-zinc-950/80 backdrop-blur-sm">
        <div class="w-8 h-8 border-2 border-violet-500 border-t-transparent rounded-full animate-spin mb-3" />
        <span class="text-sm text-violet-400">Tracing...</span>
      </div>
      <div
        v-if="svgData"
        class="absolute inset-0 flex items-center justify-center"
      >
        <div
          :style="{
            transform: `translate(${panX}px, ${panY}px) scale(${zoom})`,
            transformOrigin: 'center center',
          }"
        >
          <div class="relative">
            <img
              v-if="showOverlay && thumbnailBase64"
              :src="`data:image/jpeg;base64,${thumbnailBase64}`"
              class="absolute inset-0 w-full h-full object-contain opacity-20 pointer-events-none"
            />
            <div
              class="relative rounded"
              style="background: repeating-conic-gradient(#e5e5e5 0% 25%, #fff 0% 50%) 0 0 / 16px 16px;"
              v-html="svgHtml"
            />
          </div>
        </div>
      </div>
      <div v-else class="absolute inset-0 flex items-center justify-center text-zinc-600">
        Trace an image to see the preview
      </div>
    </div>
    <div class="px-3 py-1.5 bg-zinc-900 border-t border-zinc-800 text-xs text-zinc-600 flex gap-4 items-center">
      <label class="flex items-center gap-1.5 cursor-pointer">
        <input v-model="showOverlay" type="checkbox" class="accent-violet-500" />
        Show overlay
      </label>
      <label class="flex items-center gap-1.5 cursor-pointer opacity-50" title="Coming soon">
        <input type="checkbox" class="accent-violet-500" disabled />
        Control points
      </label>
      <div class="flex-1" />
      <span v-if="svgData" class="text-zinc-500">{{ zoomPercent }}%</span>
      <button
        v-if="svgData && (zoom !== 1 || panX !== 0 || panY !== 0)"
        class="text-zinc-500 hover:text-white transition-colors"
        @click="resetView"
      >
        Reset view
      </button>
    </div>
  </div>
</template>
