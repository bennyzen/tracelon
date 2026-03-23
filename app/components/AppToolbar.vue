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
  exportMode: boolean
}>()

const emit = defineEmits<{
  open: []
  export: []
  save: []
  back: []
  trace: []
  pipelineChange: [params: PipelineParams]
}>()

const mode = defineModel<TraceMode>('mode', {
  default: () => ({ type: 'Monochrome' as const }),
})
const smoothness = defineModel<number>('smoothness', { default: 50 })
const colorCount = ref(6)
const colorCutout = ref(false)
const colorSpeckle = ref(8)
const colorPrecision = ref(8)
const showTune = ref(false)

// Individual pipeline params — derived from smoothness by default
const lineSnap = ref(1.5)

// When smoothness changes, update derived params and emit
function deriveParams(s: number): PipelineParams {
  const normalized = s / 100
  return {
    smoothness: normalized,
    lineSnap: 0.5 + normalized * 2.0,
  }
}

function currentParams(): PipelineParams {
  return {
    smoothness: smoothness.value / 100,
    lineSnap: lineSnap.value,
  }
}

const modeItems = [
  { label: 'Mono', value: 'Monochrome' },
  { label: 'Color', value: 'MultiColor' },
  { label: 'Outline', value: 'Outline' },
]

function buildMultiColorMode(): TraceMode {
  return {
    type: 'MultiColor',
    colors: colorCount.value,
    cutout: colorCutout.value,
    filterSpeckle: colorSpeckle.value,
    colorPrecision: colorPrecision.value,
  }
}

const selectedModeValue = computed({
  get: () => mode.value.type,
  set: (val: string) => {
    if (val === 'MultiColor') {
      mode.value = buildMultiColorMode()
    } else if (val === 'Outline') {
      mode.value = { type: 'Outline' }
    } else {
      mode.value = { type: 'Monochrome' }
    }
  },
})

watch([colorCount, colorCutout, colorSpeckle, colorPrecision], () => {
  if (selectedModeValue.value === 'MultiColor') {
    mode.value = buildMultiColorMode()
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
  emitPipelineChange()
})

// When individual params change, emit
watch(lineSnap, () => {
  emitPipelineChange()
})
</script>

<template>
  <div>
    <div class="flex items-center gap-3 px-4 py-2 border-b border-zinc-800 bg-zinc-900" data-tauri-drag-region>
      <img src="/logo.png" alt="Tracelon" class="w-5 h-5 pointer-events-none" />
      <span class="text-sm font-semibold text-zinc-400 select-none pointer-events-none">Tracelon</span>
      <div class="w-px h-6 bg-zinc-700" />
      <UButton icon="i-lucide-folder-open" label="Open" variant="outline" color="primary" size="sm" @click="$emit('open')" />
      <div class="w-px h-6 bg-zinc-700" />
      <span class="text-xs text-zinc-500">Mode:</span>
      <UTabs v-model="selectedModeValue" :items="modeItems" variant="pill" size="xs" :content="false" />
      <template v-if="selectedModeValue === 'MultiColor'">
        <span class="text-xs text-zinc-500">Colors:</span>
        <UInputNumber v-model="colorCount" :min="2" :max="32" :step="1" size="xs" class="w-24" />
        <UCheckbox v-model="colorCutout" label="Cutout" />
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
      <div class="flex-1" />
      <div class="flex gap-1">
        <UButton
          icon="i-lucide-route"
          label="Trace"
          size="sm"
          :color="!exportMode ? 'primary' : 'neutral'"
          variant="outline"
          :disabled="!hasImage"
          :loading="loading"
          @click="exportMode ? $emit('back') : $emit('trace')"
        />
        <UButton
          icon="i-lucide-sparkles"
          label="Optimize"
          size="sm"
          :color="exportMode ? 'primary' : 'neutral'"
          variant="outline"
          :disabled="!hasSvg"
          @click="$emit('export')"
        />
      </div>
      <UButton
        icon="i-lucide-save"
        label="Export"
        size="sm"
        color="primary"
        variant="outline"
        :disabled="!exportMode"
        @click="$emit('save')"
      />
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
      <template v-if="selectedModeValue === 'MultiColor'">
        <div class="w-px h-4 bg-zinc-700" />
        <div class="flex items-center gap-1.5">
          <span class="text-zinc-500">Speckle:</span>
          <USlider v-model="colorSpeckle" :min="1" :max="50" :step="1" class="w-28" />
          <span class="text-zinc-400 w-8">{{ colorSpeckle }}px</span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="text-zinc-500">Precision:</span>
          <USlider v-model="colorPrecision" :min="1" :max="8" :step="1" class="w-28" />
          <span class="text-zinc-400 w-4">{{ colorPrecision }}</span>
        </div>
      </template>
    </div>
  </div>
</template>
