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

const { zoom, panX, panY, isPanning, containerRef, zoomPercent, isViewModified, onWheel, onMouseDown, onMouseMove, onMouseUp, resetView } = useZoomPan()

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
            class="relative rounded bg-checkerboard"
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
            <UButton
              v-if="isManualMode"
              size="xs"
              color="success"
              label="Re-optimize"
              @click="$emit('reoptimize')"
            />
          </div>
          <div class="grid grid-cols-2 gap-x-4 gap-y-1">
            <div
              v-for="(plugin, i) in plugins"
              :key="plugin.id"
              class="flex items-center gap-1.5 text-xs"
              :class="plugin.enabled ? 'text-zinc-300' : 'text-zinc-600'"
            >
              <UCheckbox
                :model-value="plugin.enabled"
                :label="plugin.label"
                @update:model-value="togglePlugin(i)"
              />
              <UInputNumber
                v-if="plugin.precision !== undefined && plugin.enabled"
                :model-value="plugin.precision"
                :min="0"
                :max="6"
                :step="1"
                size="xs"
                class="w-16 ml-1"
                @update:model-value="updatePrecision(i, Number($event))"
              />
            </div>
          </div>
        </div>
        <div class="px-3 h-8 bg-zinc-900/90 backdrop-blur-sm border-t border-zinc-800 text-xs text-zinc-600 flex gap-4 items-center">
          <UButton
            variant="link"
            color="neutral"
            size="xs"
            :label="showPlugins ? 'Hide plugins' : 'Show plugins'"
            @click="showPlugins = !showPlugins"
          />
          <div class="flex-1" />
          <span v-if="optimizedSvg" class="text-zinc-500">{{ zoomPercent }}%</span>
          <UButton
            v-if="optimizedSvg && isViewModified"
            variant="link"
            color="neutral"
            size="xs"
            label="Reset view"
            @click="resetView"
          />
        </div>
      </div>
    </div>
  </div>
</template>
