<script setup lang="ts">
import type { SvgoPlugin } from '~/composables/useSvgo'

const props = defineProps<{
  optimizedSvg: string | null
  originalSize: number
  optimizedSize: number
  savings: number
  plugins: SvgoPlugin[]
  isManualMode: boolean
  optimizing: boolean
}>()

const emit = defineEmits<{
  'update:plugins': [plugins: SvgoPlugin[]]
  reoptimize: []
}>()

const showPlugins = ref(true)

// Zoom & pan state (same pattern as SvgPreview)
const zoom = ref(1)
const panX = ref(0)
const panY = ref(0)
const isPanning = ref(false)
const lastMouse = ref({ x: 0, y: 0 })
const containerRef = ref<HTMLElement | null>(null)

const MIN_ZOOM = 0.1
const MAX_ZOOM = 20

// Extract viewBox and paths from optimized SVG to render with zoom baked in
const svgHtml = computed(() => {
  if (!props.optimizedSvg) return ''
  // Parse viewBox from the optimized SVG
  const vbMatch = props.optimizedSvg.match(/viewBox="([^"]*)"/)
  const vb = vbMatch ? vbMatch[1] : '0 0 800 600'
  const dims = vb.split(/\s+/)
  const w = Number(dims[2]) || 800
  const h = Number(dims[3]) || 600
  const zw = w * zoom.value
  const zh = h * zoom.value
  // Extract inner content (everything between <svg> and </svg>)
  const innerMatch = props.optimizedSvg.match(/<svg[^>]*>([\s\S]*)<\/svg>/)
  const inner = innerMatch ? innerMatch[1] : ''
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${vb}" width="${zw}" height="${zh}" style="display:block;">${inner}</svg>`
})

// No zoom/pan reset on re-optimization — user is actively comparing

function onWheel(e: WheelEvent) {
  e.preventDefault()
  const container = containerRef.value
  if (!container) return
  const rect = container.getBoundingClientRect()
  const mx = e.clientX - rect.left - rect.width / 2
  const my = e.clientY - rect.top - rect.height / 2
  const oldZoom = zoom.value
  const factor = e.deltaY < 0 ? 1.15 : 1 / 1.15
  const newZoom = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, oldZoom * factor))
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

function togglePlugin(index: number) {
  const updated = props.plugins.map((p, i) =>
    i === index ? { ...p, enabled: !p.enabled } : { ...p },
  )
  emit('update:plugins', updated)
}

function updatePrecision(index: number, value: number) {
  const updated = props.plugins.map((p, i) =>
    i === index ? { ...p, precision: value } : { ...p },
  )
  emit('update:plugins', updated)
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  return `${(bytes / 1024).toFixed(1)} KB`
}

const zoomPercent = computed(() => Math.round(zoom.value * 100))
</script>

<template>
  <div class="flex flex-col w-full h-full border-l border-zinc-800">
    <div class="px-3 py-1.5 bg-zinc-900 border-b border-zinc-800 text-xs uppercase tracking-wider flex justify-between">
      <span class="text-zinc-500">Optimized SVG</span>
      <span v-if="optimizedSvg" class="text-emerald-500">
        {{ formatSize(optimizedSize) }}
        <span class="text-zinc-500">(was {{ formatSize(originalSize) }})</span>
        <span class="text-emerald-400 font-semibold">&minus;{{ savings }}%</span>
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
      <div v-if="optimizing" class="absolute inset-0 z-10 flex flex-col items-center justify-center bg-zinc-950/80 backdrop-blur-sm">
        <div class="w-8 h-8 border-2 border-emerald-500 border-t-transparent rounded-full animate-spin mb-3" />
        <span class="text-sm text-emerald-400">Optimizing...</span>
      </div>
      <div
        v-if="optimizedSvg"
        class="absolute inset-0 flex items-center justify-center"
      >
        <div :style="{ transform: `translate(${panX}px, ${panY}px)` }">
          <div
            class="relative rounded"
            style="background: repeating-conic-gradient(#e5e5e5 0% 25%, #fff 0% 50%) 0 0 / 16px 16px;"
            v-html="svgHtml"
          />
        </div>
      </div>
      <div v-else class="absolute inset-0 flex items-center justify-center text-zinc-600">
        Click Export to optimize
      </div>
      <!-- Plugin config drawer — overlays bottom of viewport -->
      <div class="absolute bottom-0 left-0 right-0 z-20">
        <div v-if="showPlugins" class="px-3 py-2 bg-zinc-900/90 backdrop-blur-sm border-t border-zinc-800">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-semibold text-zinc-400">SVGO Plugins</span>
            <button
              v-if="isManualMode"
              class="text-xs px-2 py-0.5 rounded bg-emerald-600 hover:bg-emerald-500 text-white transition-colors"
              @click="$emit('reoptimize')"
            >
              Re-optimize
            </button>
          </div>
          <div class="grid grid-cols-2 gap-x-4 gap-y-1">
            <label
              v-for="(plugin, i) in plugins"
              :key="plugin.id"
              class="flex items-center gap-1.5 text-xs cursor-pointer"
              :class="plugin.enabled ? 'text-zinc-300' : 'text-zinc-600'"
            >
              <input
                type="checkbox"
                :checked="plugin.enabled"
                class="accent-emerald-500"
                @change="togglePlugin(i)"
              />
              {{ plugin.label }}
              <input
                v-if="plugin.precision !== undefined && plugin.enabled"
                type="number"
                :value="plugin.precision"
                min="0"
                max="6"
                class="w-10 px-1 py-0 text-xs bg-zinc-800 border border-zinc-700 rounded text-white ml-1"
                @input="updatePrecision(i, Number(($event.target as HTMLInputElement).value))"
              />
            </label>
          </div>
        </div>
        <div class="px-3 py-1.5 bg-zinc-900/90 backdrop-blur-sm border-t border-zinc-800 text-xs text-zinc-600 flex gap-4 items-center">
          <button
            class="text-zinc-500 hover:text-white transition-colors"
            @click="showPlugins = !showPlugins"
          >
            {{ showPlugins ? 'Hide' : 'Show' }} plugins
          </button>
          <div class="flex-1" />
          <span v-if="optimizedSvg" class="text-zinc-500">{{ zoomPercent }}%</span>
          <button
            v-if="optimizedSvg && (zoom !== 1 || panX !== 0 || panY !== 0)"
            class="text-zinc-500 hover:text-white transition-colors"
            @click="resetView"
          >
            Reset view
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
