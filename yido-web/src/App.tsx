import { createSignal, onCleanup, onMount } from 'solid-js'
import init, { YidoEngine } from './wasm/yido_wasm/yido_wasm'
import dubeolsikLayout from '../../crates/yido-core/layouts/ko-dubeolsik.toml?raw'

type EngineState = {
  committed: string
  composing: string
  text: string
}

const emptyState: EngineState = {
  committed: '',
  composing: '',
  text: '',
}

function App() {
  const [engine, setEngine] = createSignal<YidoEngine>()
  const [state, setState] = createSignal<EngineState>(emptyState)
  const [error, setError] = createSignal('')
  let inputPanel: HTMLElement | undefined

  onMount(async () => {
    try {
      await init()
      const nextEngine = new YidoEngine(dubeolsikLayout)
      setEngine(nextEngine)
      setState(nextEngine.state() as EngineState)
      inputPanel?.focus()
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught))
    }
  })

  onCleanup(() => {
    engine()?.free()
  })

  const handleKeyDown = (event: KeyboardEvent) => {
    const currentEngine = engine()
    if (!currentEngine || event.metaKey || event.ctrlKey || event.altKey) {
      return
    }

    if (event.key === 'Backspace') {
      event.preventDefault()
      setState(currentEngine.backspace() as EngineState)
      return
    }

    if (event.key.length === 1) {
      event.preventDefault()
      const key = /^[a-zA-Z]$/.test(event.key) ? event.key.toLowerCase() : event.key
      setState(currentEngine.inputKey(key, event.shiftKey) as EngineState)
    }
  }

  const reset = () => {
    const currentEngine = engine()
    if (!currentEngine) {
      return
    }

    setState(currentEngine.reset() as EngineState)
  }

  return (
    <main class="app-shell">
      <section ref={inputPanel} class="input-panel" tabIndex={0} onKeyDown={handleKeyDown}>
        <div class="panel-header">
          <h1>이도 입력기</h1>
          <button type="button" onClick={reset}>초기화</button>
        </div>

        {error() ? <p class="error-message">{error()}</p> : null}

        <div class="result-box" aria-live="polite">
          {state().text || <span class="empty-output">입력 없음</span>}
        </div>

        <dl class="state-grid">
          <div>
            <dt>확정</dt>
            <dd>{state().committed || '없음'}</dd>
          </div>
          <div>
            <dt>조합 중</dt>
            <dd>{state().composing || '없음'}</dd>
          </div>
        </dl>
      </section>
    </main>
  )
}

export default App
