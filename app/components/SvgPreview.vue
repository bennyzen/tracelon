<script setup lang="ts">
import type { SvgData } from '~/composables/useTracer'

const props = defineProps<{
  svgData: SvgData | null
  thumbnailBase64: string | null
  loading: boolean
}>()

const showOverlay = ref(true)

const svgHtml = computed(() => {
  if (!props.svgData) return ''
  const vb = props.svgData.viewbox.split(' ')
  const w = vb[2] || '800'
  const h = vb[3] || '600'
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${props.svgData.viewbox}" width="${w}" height="${h}" style="max-width:100%;height:auto;display:block;">${props.svgData.paths}</svg>`
})

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
    <div class="flex-1 relative overflow-hidden bg-zinc-950 flex items-center justify-center">
      <!-- Loading overlay — shown on top of any existing content -->
      <div v-if="loading" class="absolute inset-0 z-10 flex flex-col items-center justify-center bg-zinc-950/80 backdrop-blur-sm">
        <div class="w-8 h-8 border-2 border-violet-500 border-t-transparent rounded-full animate-spin mb-3" />
        <span class="text-sm text-violet-400">Tracing...</span>
      </div>
      <div v-if="svgData" class="relative flex items-center justify-center" style="width: 100%; height: 100%;">
        <div class="relative" style="max-width: 90%; max-height: 90%;">
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
