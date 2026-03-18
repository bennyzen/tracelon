<script setup lang="ts">
import type { TraceMode } from '~/composables/useTracer'

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
  <div class="flex items-center gap-3 px-4 py-2 border-b border-zinc-800 bg-zinc-900">
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
  </div>
</template>
