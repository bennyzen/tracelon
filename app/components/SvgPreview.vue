<script setup lang="ts">
import type { SvgData } from '~/composables/useTracer'

const props = defineProps<{
  svgData: SvgData | null
  thumbnailBase64: string | null
  loading: boolean
}>()

const showOverlay = ref(true)

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  return `${(bytes / 1024).toFixed(1)} KB`
}
</script>

<template>
  <div class="flex flex-col flex-1">
    <div class="px-3 py-1.5 bg-zinc-900 border-b border-zinc-800 text-xs uppercase tracking-wider flex justify-between">
      <span class="text-zinc-500">SVG Preview</span>
      <span v-if="svgData" class="text-emerald-500">
        {{ svgData.pathCount }} paths &bull; {{ formatSize(svgData.estimatedSize) }}
      </span>
    </div>
    <div class="flex-1 relative overflow-hidden bg-zinc-950 flex items-center justify-center">
      <div v-if="loading" class="text-zinc-500">
        <UButton loading variant="ghost" label="Tracing..." />
      </div>
      <div v-else-if="svgData" class="relative max-w-full max-h-full">
        <img
          v-if="showOverlay && thumbnailBase64"
          :src="`data:image/jpeg;base64,${thumbnailBase64}`"
          class="absolute inset-0 w-full h-full object-contain opacity-20 pointer-events-none"
        />
        <div
          class="relative bg-white rounded"
          v-html="`<svg xmlns='http://www.w3.org/2000/svg' viewBox='${svgData.viewbox}' style='max-width:100%;max-height:70vh;'>${svgData.paths}</svg>`"
        />
      </div>
      <div v-else class="text-zinc-600">
        Trace an image to see the preview
      </div>
    </div>
    <div class="px-3 py-1.5 bg-zinc-900 border-t border-zinc-800 text-xs text-zinc-600 flex gap-4">
      <label class="flex items-center gap-1.5 cursor-pointer">
        <input v-model="showOverlay" type="checkbox" class="accent-violet-500" />
        Show overlay
      </label>
      <label class="flex items-center gap-1.5 cursor-pointer opacity-50" title="Coming soon">
        <input type="checkbox" class="accent-violet-500" disabled />
        Control points
      </label>
    </div>
  </div>
</template>
