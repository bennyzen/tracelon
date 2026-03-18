<script setup lang="ts">
import type { TraceMode, PipelineParams } from '~/composables/useTracer'
import { getCurrentWindow } from '@tauri-apps/api/window'

async function minimizeWindow() { await getCurrentWindow().minimize() }
async function toggleMaximize() { await getCurrentWindow().toggleMaximize() }
async function closeWindow() { await getCurrentWindow().close() }

const props = defineProps<{
  hasImage: boolean
  hasSvg: boolean
  loading: boolean
}>()

const emit = defineEmits<{
  open: []
  export: []
  trace: []
  pipelineChange: [params: PipelineParams]
}>()

const mode = defineModel<TraceMode>('mode', {
  default: () => ({ type: 'Monochrome' as const }),
})
const smoothness = defineModel<number>('smoothness', { default: 50 })
const colorCount = ref(6)
const showTune = ref(false)

// Individual pipeline params — derived from smoothness by default
const lineSnap = ref(1.5)
const cornerAngle = ref(135)

// When smoothness changes, update derived params and emit
function deriveParams(s: number): PipelineParams {
  const normalized = s / 100
  return {
    smoothness: normalized,
    lineSnap: 0.5 + normalized * 2.0,
    cornerAngle: 120.0 + normalized * 30.0,
  }
}

function currentParams(): PipelineParams {
  return {
    smoothness: smoothness.value / 100,
    lineSnap: lineSnap.value,
    cornerAngle: cornerAngle.value,
  }
}

const modeItems = [
  { label: 'Mono', value: 'Monochrome' },
  { label: 'Color', value: 'MultiColor' },
  { label: 'Outline', value: 'Outline' },
]
const selectedModeValue = ref('Monochrome')

watch(selectedModeValue, (val) => {
  if (val === 'MultiColor') {
    mode.value = { type: 'MultiColor', colors: colorCount.value }
  }
  else if (val === 'Outline') {
    mode.value = { type: 'Outline' }
  }
  else {
    mode.value = { type: 'Monochrome' }
  }
})

watch(colorCount, (val) => {
  if (selectedModeValue.value === 'MultiColor') {
    mode.value = { type: 'MultiColor', colors: val }
  }
})

let pipelineTimeout: ReturnType<typeof setTimeout> | null = null
function emitPipelineChange() {
  if (pipelineTimeout) clearTimeout(pipelineTimeout)
  pipelineTimeout = setTimeout(() => {
    emit('pipelineChange', currentParams())
  }, 150)
}

// When main smoothness slider changes, update derived params + emit
watch(smoothness, (val) => {
  const derived = deriveParams(val)
  lineSnap.value = Math.round(derived.lineSnap * 10) / 10
  cornerAngle.value = Math.round(derived.cornerAngle)
  emitPipelineChange()
})

// When individual params change, emit
watch([lineSnap, cornerAngle], () => {
  emitPipelineChange()
})
</script>

<template>
  <div>
    <div class="flex items-center gap-3 px-4 py-2 border-b border-zinc-800 bg-zinc-900" data-tauri-drag-region>
      <span class="text-sm font-semibold text-zinc-400 select-none pointer-events-none">Tracelon</span>
      <div class="w-px h-6 bg-zinc-700" />
      <UButton icon="i-lucide-folder-open" label="Open" variant="soft" @click="$emit('open')" />
      <div class="w-px h-6 bg-zinc-700" />
      <span class="text-xs text-zinc-500">Mode:</span>
      <UTabs v-model="selectedModeValue" :items="modeItems" variant="pill" size="xs" :content="false" />
      <template v-if="selectedModeValue === 'MultiColor'">
        <span class="text-xs text-zinc-500">Colors:</span>
        <input
          v-model.number="colorCount"
          type="number"
          min="2"
          max="16"
          class="w-14 px-2 py-1 text-xs bg-zinc-800 border border-zinc-700 rounded text-white"
        />
      </template>
      <div class="w-px h-6 bg-zinc-700" />
      <span class="text-xs text-zinc-500">Smoothness:</span>
      <USlider v-model="smoothness" :min="0" :max="100" :step="1" class="w-40" :disabled="!hasSvg" />
      <span class="text-xs text-zinc-400 w-8">{{ smoothness }}%</span>
      <UButton
        :icon="showTune ? 'i-lucide-chevron-up' : 'i-lucide-sliders-horizontal'"
        size="xs"
        variant="ghost"
        :color="showTune ? 'primary' : 'neutral'"
        title="Tune pipeline parameters"
        :disabled="!hasSvg"
        @click="showTune = !showTune"
      />
      <UButton icon="i-lucide-play" label="Trace" color="primary" :disabled="!hasImage" :loading="loading" @click="$emit('trace')" />
      <div class="flex-1" />
      <UButton icon="i-lucide-download" label="Export SVG" color="success" variant="soft" :disabled="!hasSvg" @click="$emit('export')" />
      <div class="w-px h-6 bg-zinc-700" />
      <div class="flex gap-1">
        <button class="w-7 h-7 flex items-center justify-center rounded hover:bg-zinc-700 text-zinc-400 hover:text-white transition-colors" @click="minimizeWindow">
          <svg width="10" height="1" viewBox="0 0 10 1"><rect fill="currentColor" width="10" height="1" /></svg>
        </button>
        <button class="w-7 h-7 flex items-center justify-center rounded hover:bg-zinc-700 text-zinc-400 hover:text-white transition-colors" @click="toggleMaximize">
          <svg width="10" height="10" viewBox="0 0 10 10"><rect fill="none" stroke="currentColor" stroke-width="1" x="0.5" y="0.5" width="9" height="9" /></svg>
        </button>
        <button class="w-7 h-7 flex items-center justify-center rounded hover:bg-red-600 text-zinc-400 hover:text-white transition-colors" @click="closeWindow">
          <svg width="10" height="10" viewBox="0 0 10 10"><line stroke="currentColor" stroke-width="1.2" x1="1" y1="1" x2="9" y2="9" /><line stroke="currentColor" stroke-width="1.2" x1="9" y1="1" x2="1" y2="9" /></svg>
        </button>
      </div>
    </div>
    <!-- Tune panel — collapsible second row -->
    <div v-if="showTune" class="flex items-center gap-4 px-4 py-1.5 border-b border-zinc-800 bg-zinc-900/80 text-xs">
      <span class="text-zinc-500 font-medium">Pipeline:</span>
      <div class="flex items-center gap-1.5">
        <span class="text-zinc-500">Curve simplify:</span>
        <USlider v-model="smoothness" :min="0" :max="100" :step="1" class="w-28" />
        <span class="text-zinc-400 w-8">{{ smoothness }}%</span>
      </div>
      <div class="flex items-center gap-1.5">
        <span class="text-zinc-500">Line snap:</span>
        <USlider v-model="lineSnap" :min="0" :max="5" :step="0.1" class="w-28" />
        <span class="text-zinc-400 w-10">{{ lineSnap }}px</span>
      </div>
      <div class="flex items-center gap-1.5">
        <span class="text-zinc-500">Corner angle:</span>
        <USlider v-model="cornerAngle" :min="90" :max="170" :step="1" class="w-28" />
        <span class="text-zinc-400 w-8">{{ cornerAngle }}&deg;</span>
      </div>
    </div>
  </div>
</template>
