export function useZoomPan() {
  const zoom = ref(1)
  const panX = ref(0)
  const panY = ref(0)
  const isPanning = ref(false)
  const lastMouse = ref({ x: 0, y: 0 })
  const containerRef = ref<HTMLElement | null>(null)

  const MIN_ZOOM = 0.1
  const MAX_ZOOM = 20

  const zoomPercent = computed(() => Math.round(zoom.value * 100))
  const isViewModified = computed(() => zoom.value !== 1 || panX.value !== 0 || panY.value !== 0)

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

  return {
    zoom, panX, panY, isPanning, containerRef,
    zoomPercent, isViewModified,
    onWheel, onMouseDown, onMouseMove, onMouseUp, resetView,
  }
}
