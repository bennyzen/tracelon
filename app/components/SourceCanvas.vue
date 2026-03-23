<script setup lang="ts">
import type { Rect } from '~/composables/useTracer'

const props = defineProps<{
  previewUrl: string | null
  imageWidth: number
  imageHeight: number
  loading: boolean
}>()

const selection = defineModel<Rect | null>('selection', { default: null })

const canvasRef = ref<HTMLCanvasElement | null>(null)
const containerRef = ref<HTMLDivElement | null>(null)
const displayScale = ref(1)
const img = ref<HTMLImageElement | null>(null)

type DragMode = 'none' | 'create' | 'move'
  | 'resize-n' | 'resize-s' | 'resize-e' | 'resize-w'
  | 'resize-nw' | 'resize-ne' | 'resize-sw' | 'resize-se'

const dragMode = ref<DragMode>('none')
const dragStart = ref({ x: 0, y: 0 })
const dragStartSelection = ref<Rect | null>(null)
const cursorStyle = ref('crosshair')
const isDragging = computed(() => dragMode.value !== 'none')

const cursorMap: Record<DragMode, string> = {
  'none': 'crosshair',
  'create': 'crosshair',
  'move': 'move',
  'resize-n': 'ns-resize',
  'resize-s': 'ns-resize',
  'resize-e': 'ew-resize',
  'resize-w': 'ew-resize',
  'resize-nw': 'nwse-resize',
  'resize-se': 'nwse-resize',
  'resize-ne': 'nesw-resize',
  'resize-sw': 'nesw-resize',
}

watch(() => props.previewUrl, async (url) => {
  if (!url) return
  await nextTick()
  if (!canvasRef.value) return
  const image = new Image()
  image.onload = () => {
    img.value = image
    fitAndDraw()
  }
  image.src = url
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
  const handles: [number, number][] = [
    [x, y], [x + w, y], [x, y + h], [x + w, y + h],
    [x + w / 2, y], [x + w / 2, y + h],
    [x, y + h / 2], [x + w, y + h / 2],
  ]
  for (const [cx, cy] of handles) {
    ctx.fillRect(cx - hs / 2, cy - hs / 2, hs, hs)
  }
}

function canvasToImage(cx: number, cy: number): { x: number; y: number } {
  const container = containerRef.value
  if (!container) return { x: 0, y: 0 }
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

function selectionScreenRect() {
  if (!selection.value || !containerRef.value) return null
  const container = containerRef.value
  const cw = container.clientWidth
  const ch = container.clientHeight
  const scale = displayScale.value
  const dw = props.imageWidth * scale
  const dh = props.imageHeight * scale
  const offX = (cw - dw) / 2
  const offY = (ch - dh) / 2
  const s = selection.value
  return {
    x: offX + s.x * scale,
    y: offY + s.y * scale,
    width: s.width * scale,
    height: s.height * scale,
  }
}

function hitTest(canvasX: number, canvasY: number): DragMode {
  const sr = selectionScreenRect()
  if (!sr || sr.width < 2 || sr.height < 2) return 'create'

  const t = 8
  const { x, y, width: w, height: h } = sr

  const nearL = Math.abs(canvasX - x) < t
  const nearR = Math.abs(canvasX - (x + w)) < t
  const nearT = Math.abs(canvasY - y) < t
  const nearB = Math.abs(canvasY - (y + h)) < t
  const inH = canvasX > x - t && canvasX < x + w + t
  const inV = canvasY > y - t && canvasY < y + h + t

  if (nearT && nearL) return 'resize-nw'
  if (nearT && nearR) return 'resize-ne'
  if (nearB && nearL) return 'resize-sw'
  if (nearB && nearR) return 'resize-se'
  if (nearT && inH) return 'resize-n'
  if (nearB && inH) return 'resize-s'
  if (nearL && inV) return 'resize-w'
  if (nearR && inV) return 'resize-e'
  if (canvasX > x && canvasX < x + w && canvasY > y && canvasY < y + h) return 'move'

  return 'create'
}

function clampRect(r: Rect): Rect {
  let { x, y, width, height } = r
  x = Math.max(0, x)
  y = Math.max(0, y)
  width = Math.min(width, props.imageWidth - x)
  height = Math.min(height, props.imageHeight - y)
  return { x, y, width, height }
}

function onMouseDown(e: MouseEvent) {
  if (!canvasRef.value) return
  const rect = canvasRef.value.getBoundingClientRect()
  const canvasX = e.clientX - rect.left
  const canvasY = e.clientY - rect.top
  const pos = canvasToImage(canvasX, canvasY)

  const mode = hitTest(canvasX, canvasY)
  dragMode.value = mode
  dragStart.value = pos

  if (mode === 'create') {
    selection.value = { x: pos.x, y: pos.y, width: 0, height: 0 }
    dragStartSelection.value = null
  } else {
    dragStartSelection.value = selection.value ? { ...selection.value } : null
  }
}

function onMouseMove(e: MouseEvent) {
  if (!canvasRef.value) return
  const rect = canvasRef.value.getBoundingClientRect()
  const canvasX = e.clientX - rect.left
  const canvasY = e.clientY - rect.top

  if (dragMode.value === 'none') {
    cursorStyle.value = cursorMap[hitTest(canvasX, canvasY)]
    return
  }

  const pos = canvasToImage(canvasX, canvasY)
  const ctrl = e.ctrlKey || e.metaKey
  const mode = dragMode.value

  if (mode === 'create') {
    let w = pos.x - dragStart.value.x
    let h = pos.y - dragStart.value.y

    if (ctrl) {
      const size = Math.max(Math.abs(w), Math.abs(h))
      w = w >= 0 ? size : -size
      h = h >= 0 ? size : -size
    }

    const x = w >= 0 ? dragStart.value.x : dragStart.value.x + w
    const y = h >= 0 ? dragStart.value.y : dragStart.value.y + h
    selection.value = clampRect({ x, y, width: Math.abs(w), height: Math.abs(h) })
  } else if (mode === 'move') {
    const orig = dragStartSelection.value!
    const dx = pos.x - dragStart.value.x
    const dy = pos.y - dragStart.value.y
    selection.value = {
      x: Math.max(0, Math.min(orig.x + dx, props.imageWidth - orig.width)),
      y: Math.max(0, Math.min(orig.y + dy, props.imageHeight - orig.height)),
      width: orig.width,
      height: orig.height,
    }
  } else {
    const orig = dragStartSelection.value!
    const dx = pos.x - dragStart.value.x
    const dy = pos.y - dragStart.value.y

    let left = orig.x
    let top = orig.y
    let right = orig.x + orig.width
    let bottom = orig.y + orig.height

    if (mode.includes('w')) left += dx
    if (mode.includes('e')) right += dx
    if (mode.includes('n')) top += dy
    if (mode.includes('s')) bottom += dy

    const isCorner = mode === 'resize-nw' || mode === 'resize-ne'
      || mode === 'resize-sw' || mode === 'resize-se'
    if (ctrl && isCorner) {
      const size = Math.max(Math.abs(right - left), Math.abs(bottom - top))
      if (mode === 'resize-nw') { left = right - size; top = bottom - size }
      else if (mode === 'resize-ne') { right = left + size; top = bottom - size }
      else if (mode === 'resize-sw') { left = right - size; bottom = top + size }
      else { right = left + size; bottom = top + size }
    }

    if (left > right) [left, right] = [right, left]
    if (top > bottom) [top, bottom] = [bottom, top]

    left = Math.max(0, left)
    top = Math.max(0, top)
    right = Math.min(props.imageWidth, right)
    bottom = Math.min(props.imageHeight, bottom)

    selection.value = { x: left, y: top, width: right - left, height: bottom - top }
  }

  fitAndDraw()
}

function onMouseUp() {
  if (selection.value && selection.value.width < 3 && selection.value.height < 3) {
    selection.value = null
  }
  dragMode.value = 'none'
  fitAndDraw()
}

let resizeTimer: ReturnType<typeof setTimeout> | null = null
function debouncedFitAndDraw() {
  if (resizeTimer) clearTimeout(resizeTimer)
  resizeTimer = setTimeout(fitAndDraw, 16)
}

onMounted(() => {
  window.addEventListener('resize', debouncedFitAndDraw)
})

onUnmounted(() => {
  window.removeEventListener('resize', debouncedFitAndDraw)
})
</script>

<template>
  <div class="flex flex-col w-full h-full border-r border-zinc-800">
    <div class="px-3 py-1.5 bg-zinc-900 border-b border-zinc-800 text-xs text-zinc-500 uppercase tracking-wider">
      Source Image
    </div>
    <div ref="containerRef" class="flex-1 relative overflow-hidden bg-zinc-950">
      <template v-if="previewUrl">
        <canvas
          ref="canvasRef"
          class="absolute inset-0"
          :style="{ cursor: cursorStyle }"
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
    <div class="px-3 h-8 flex items-center bg-zinc-900 border-t border-zinc-800 text-xs text-zinc-600">
      <template v-if="previewUrl">
        {{ imageWidth }}&times;{{ imageHeight }}
        <template v-if="selection">
          &mdash; Selection: {{ selection.width }}&times;{{ selection.height }}
        </template>
      </template>
      <template v-else>No image loaded</template>
    </div>
  </div>
</template>
