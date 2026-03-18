<script setup lang="ts">
import type { TraceMode } from '~/composables/useTracer'
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
  smoothnessChange: [value: number]
}>()

const mode = defineModel<TraceMode>('mode', {
  default: () => ({ type: 'Monochrome' as const }),
})
const smoothness = defineModel<number>('smoothness', { default: 50 })
const colorCount = ref(6)

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

let smoothnessTimeout: ReturnType<typeof setTimeout> | null = null
watch(smoothness, (val) => {
  if (smoothnessTimeout) clearTimeout(smoothnessTimeout)
  smoothnessTimeout = setTimeout(() => {
    emit('smoothnessChange', val)
  }, 100)
})
</script>

<template>
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
    <UButton icon="i-lucide-play" label="Trace" color="primary" :disabled="!hasImage" :loading="loading" @click="$emit('trace')" />
    <div class="flex-1" />
    <UButton icon="i-lucide-download" label="Export SVG" color="success" variant="soft" :disabled="!hasSvg" @click="$emit('export')" />
    <div class="w-px h-6 bg-zinc-700" />
    <div class="flex gap-1">
      <button class="w-7 h-7 flex items-center justify-center rounded hover:bg-zinc-700 text-zinc-400 hover:text-white transition-colors" @click="minimizeWindow">
        <svg width="10" height="1" viewBox="0 0 10 1"><rect fill="currentColor" width="10" height="1"/></svg>
      </button>
      <button class="w-7 h-7 flex items-center justify-center rounded hover:bg-zinc-700 text-zinc-400 hover:text-white transition-colors" @click="toggleMaximize">
        <svg width="10" height="10" viewBox="0 0 10 10"><rect fill="none" stroke="currentColor" stroke-width="1" x="0.5" y="0.5" width="9" height="9"/></svg>
      </button>
      <button class="w-7 h-7 flex items-center justify-center rounded hover:bg-red-600 text-zinc-400 hover:text-white transition-colors" @click="closeWindow">
        <svg width="10" height="10" viewBox="0 0 10 10"><line stroke="currentColor" stroke-width="1.2" x1="1" y1="1" x2="9" y2="9"/><line stroke="currentColor" stroke-width="1.2" x1="9" y1="1" x2="1" y2="9"/></svg>
      </button>
    </div>
  </div>
</template>
