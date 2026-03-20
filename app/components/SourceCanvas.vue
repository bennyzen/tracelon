<script setup lang="ts">
import type { Rect } from '~/composables/useTracer'

const props = defineProps<{
  thumbnailBase64: string | null
  imageWidth: number
  imageHeight: number
  loading: boolean
}>()

const selection = defineModel<Rect | null>('selection', { default: null })

const canvasRef = ref<HTMLCanvasElement | null>(null)
const containerRef = ref<HTMLDivElement | null>(null)
const displayScale = ref(1)
const img = ref<HTMLImageElement | null>(null)
const isDragging = ref(false)
const dragStart = ref({ x: 0, y: 0 })

watch(() => props.thumbnailBase64, async (b64) => {
  if (!b64) return
  // Wait for Vue to mount the canvas element (v-if just became true)
  await nextTick()
  if (!canvasRef.value) return
  const image = new Image()
  image.onload = () => {
    img.value = image
    fitAndDraw()
  }
  image.src = `data:image/jpeg;base64,${b64}`
})

function fitAndDraw() {
  const canvas = canvasRef.value
  const container = containerRef.value
  if (!canvas || !container || !img.value) return

  const cw = container.clientWidth
  const ch = container.clientHeight
  const scale = Math.min(cw / props.imageWidth, ch / props.imageHeight)
  displayScale.value = scale

  canvas.width = cw
  canvas.height = ch

  const ctx = canvas.getContext('2d')!
  ctx.clearRect(0, 0, cw, ch)

  const dw = props.imageWidth * scale
  const dh = props.imageHeight * scale
  const dx = (cw - dw) / 2
  const dy = (ch - dh) / 2

  ctx.drawImage(img.value, dx, dy, dw, dh)
  drawSelection(ctx, dx, dy, scale)
}

function drawSelection(ctx: CanvasRenderingContext2D, offX: number, offY: number, scale: number) {
  if (!selection.value) return
  const s = selection.value
  const x = offX + s.x * scale
  const y = offY + s.y * scale
  const w = s.width * scale
  const h = s.height * scale

  ctx.strokeStyle = '#7dd3fc'
  ctx.lineWidth = 2
  ctx.setLineDash([6, 4])
  ctx.strokeRect(x, y, w, h)
  ctx.setLineDash([])

  const hs = 6
  ctx.fillStyle = '#7dd3fc'
  for (const [cx, cy] of [[x, y], [x + w, y], [x, y + h], [x + w, y + h]] as [number, number][]) {
    ctx.fillRect(cx - hs / 2, cy - hs / 2, hs, hs)
  }
}

function canvasToImage(cx: number, cy: number): { x: number; y: number } {
  const container = containerRef.value!
  const cw = container.clientWidth
  const ch = container.clientHeight
  const scale = displayScale.value
  const dw = props.imageWidth * scale
  const dh = props.imageHeight * scale
  const dx = (cw - dw) / 2
  const dy = (ch - dh) / 2
  return {
    x: Math.round((cx - dx) / scale),
    y: Math.round((cy - dy) / scale),
  }
}

function onMouseDown(e: MouseEvent) {
  const rect = canvasRef.value!.getBoundingClientRect()
  const pos = canvasToImage(e.clientX - rect.left, e.clientY - rect.top)
  isDragging.value = true
  dragStart.value = pos
}

function onMouseMove(e: MouseEvent) {
  if (!isDragging.value) return
  const rect = canvasRef.value!.getBoundingClientRect()
  const pos = canvasToImage(e.clientX - rect.left, e.clientY - rect.top)
  const x = Math.max(0, Math.min(dragStart.value.x, pos.x))
  const y = Math.max(0, Math.min(dragStart.value.y, pos.y))
  const w = Math.min(Math.abs(pos.x - dragStart.value.x), props.imageWidth - x)
  const h = Math.min(Math.abs(pos.y - dragStart.value.y), props.imageHeight - y)
  selection.value = { x, y, width: w, height: h }
  fitAndDraw()
}

function onMouseUp() {
  isDragging.value = false
}

onMounted(() => {
  window.addEventListener('resize', fitAndDraw)
})

onUnmounted(() => {
  window.removeEventListener('resize', fitAndDraw)
})
</script>

<template>
  <div class="flex flex-col w-full h-full border-r border-zinc-800">
    <div class="px-3 py-1.5 bg-zinc-900 border-b border-zinc-800 text-xs text-zinc-500 uppercase tracking-wider">
      Source Image
    </div>
    <div ref="containerRef" class="flex-1 relative overflow-hidden bg-zinc-950">
      <template v-if="thumbnailBase64">
        <canvas
          ref="canvasRef"
          class="absolute inset-0 cursor-crosshair"
          @mousedown="onMouseDown"
          @mousemove="onMouseMove"
          @mouseup="onMouseUp"
          @mouseleave="onMouseUp"
        />
        <!-- Loading overlay — always in DOM, instant show, fade out -->
        <div
          class="absolute inset-0 z-30 flex flex-col items-center justify-center bg-zinc-950/80"
          :class="loading ? 'opacity-100' : 'opacity-0 pointer-events-none transition-opacity duration-300'"
        >
          <div class="w-8 h-8 border-2 border-purple-500 border-t-transparent rounded-full animate-spin mb-3" />
          <span class="text-sm text-purple-400">Tracing...</span>
        </div>
        <!-- Hint overlay -->
        <Transition
          enter-active-class="transition-opacity duration-200"
          leave-active-class="transition-opacity duration-500"
          enter-from-class="opacity-0"
          leave-to-class="opacity-0"
        >
          <div
            v-if="!selection && !isDragging && !loading"
            class="absolute inset-0 z-20 flex items-center justify-center pointer-events-none"
          >
            <div class="bg-zinc-900/80 backdrop-blur-sm rounded-lg px-4 py-3 text-center">
              <div class="text-sm text-zinc-300">Drag to select a region, or click Trace for full image</div>
            </div>
          </div>
        </Transition>
      </template>
      <div v-else class="flex items-center justify-center h-full text-zinc-600">
        Open an image to start
      </div>
    </div>
    <div class="px-3 py-1.5 bg-zinc-900 border-t border-zinc-800 text-xs text-zinc-600">
      <template v-if="thumbnailBase64">
        {{ imageWidth }}&times;{{ imageHeight }}
        <template v-if="selection">
          &mdash; Selection: {{ selection.width }}&times;{{ selection.height }}
        </template>
      </template>
      <template v-else>No image loaded</template>
    </div>
  </div>
</template>
