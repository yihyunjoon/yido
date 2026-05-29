import { createSignal, onCleanup, onMount } from 'solid-js'
import init, { YidoEngine } from './wasm/yido_wasm/yido_wasm'
import dubeolsikLayout from '../../layouts/ko-dubeolsik.toml?raw'

type EngineState = {
  committed: string
  composing: string
  text: string
}

type EngineEffect = {
  committed: string
  composing: string
  handled: boolean
}

const emptyState: EngineState = {
  committed: '',
  composing: '',
  text: '',
}

const shiftedKeys = new Map<string, string>([
  ['~', '`'],
  ['!', '1'],
  ['@', '2'],
  ['#', '3'],
  ['$', '4'],
  ['%', '5'],
  ['^', '6'],
  ['&', '7'],
  ['*', '8'],
  ['(', '9'],
  [')', '0'],
  ['_', '-'],
  ['+', '='],
  ['{', '['],
  ['}', ']'],
  ['|', '\\'],
  [':', ';'],
  ['"', "'"],
  ['<', ','],
  ['>', '.'],
  ['?', '/'],
])

function makeState(committed: string, composing: string): EngineState {
  return {
    committed,
    composing,
    text: `${committed}${composing}`,
  }
}

function removeLastScalar(text: string): string {
  return Array.from(text).slice(0, -1).join('')
}

function keyForEngine(event: KeyboardEvent): string {
  if (/^[a-zA-Z]$/.test(event.key)) {
    return event.key.toLowerCase()
  }

  return shiftedKeys.get(event.key) ?? event.key
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
      setState(emptyState)
      inputPanel?.focus()
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught))
    }
  })

  onCleanup(() => {
    engine()?.free()
  })

  const applyEffect = (effect: EngineEffect, passthrough = '') => {
    setState((current) => {
      const committed = `${current.committed}${effect.committed}${effect.handled ? '' : passthrough}`
      return makeState(committed, effect.composing)
    })
  }

  const handleKeyDown = (event: KeyboardEvent) => {
    const currentEngine = engine()
    if (!currentEngine || event.metaKey || event.ctrlKey || event.altKey) {
      return
    }

    if (event.key === 'Backspace') {
      event.preventDefault()
      const effect = currentEngine.backspace() as EngineEffect
      if (effect.handled) {
        applyEffect(effect)
      } else {
        setState((current) => makeState(removeLastScalar(current.committed), ''))
      }
      return
    }

    if (event.key === 'Escape') {
      const effect = currentEngine.cancel() as EngineEffect
      if (effect.handled) {
        event.preventDefault()
        applyEffect(effect)
      }
      return
    }

    if (event.key === 'Enter') {
      event.preventDefault()
      const effect = currentEngine.flush() as EngineEffect
      applyEffect(effect, '\n')
      return
    }

    if (event.key.length === 1) {
      event.preventDefault()
      const effect = currentEngine.inputKey(keyForEngine(event), event.shiftKey) as EngineEffect
      applyEffect(effect, event.key)
    }
  }

  const reset = () => {
    const currentEngine = engine()
    if (!currentEngine) {
      return
    }

    currentEngine.cancel()
    setState(emptyState)
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
