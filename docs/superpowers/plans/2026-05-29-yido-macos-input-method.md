# 이도 macOS 입력기 구현 계획

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rust core를 입력기 런타임에 맞는 effect 모델로 고치고, WASM/web, C FFI, Swift macOS 입력기 골격까지 연결한다.

**Architecture:** `yido-core`는 누적 문서 문자열을 소유하지 않고 `InputEffect`만 반환한다. `yido-wasm`과 `yido-web`은 새 effect API를 사용해 자체 표시 버퍼를 관리한다. `yido-ffi`는 cbindgen으로 C ABI를 노출하고, `yido-swift`는 Swift Package 중심으로 FFI 래퍼, 입력 세션, 설정 저장소, IMKSwift 컨트롤러 골격을 둔다.

**Tech Stack:** Rust 2024, Cargo workspace, wasm-bindgen, cbindgen, Swift 6.2+, macOS 26+, IMKSwift, Swift Package Manager.

---

## 파일 구조

`yido-core/src/composer.rs`는 `InputEffect`, `flush`, `cancel`, 미매핑 키 통과, 조합 중 백스페이스를 담당한다. `yido-core/src/layout.rs`는 `engine = "hangul"` 검증을 추가한다.

`yido-wasm/src/lib.rs`는 `InputEffect`를 JS 값으로 반환하고 `flush`, `cancel`을 노출한다. `state()`와 누적 `text` 의존은 제거한다.

`yido-web/src/App.tsx`는 웹 데모의 자체 표시 버퍼를 들고 effect를 적용한다.

`yido-ffi/`는 새 Rust crate다. C ABI handle, effect/result 구조체, 문자열 소유권/free 함수를 제공한다.

`yido-swift/`는 새 Swift Package다. `YidoRustFFI`, `YidoInputCore`, `YidoSettings`, `YidoInputMethod` 타깃과 단위 테스트를 둔다.

`mise.toml`, `Cargo.toml`, `cbindgen.toml`은 새 빌드 태스크와 workspace 멤버를 반영한다.

---

### Task 1: core effect 모델

**Files:**
- Modify: `yido-core/src/composer.rs`
- Modify: `yido-core/src/lib.rs`
- Modify: `yido-core/src/layout.rs`
- Modify: `yido-core/tests/composer_tests.rs`
- Modify: `yido-core/tests/layout_tests.rs`

- [ ] **Step 1: 실패 테스트 작성**

`yido-core/tests/composer_tests.rs`를 `InputEffect` 기준으로 바꾼다. 다음 동작을 검증한다.

```rust
#[test]
fn returns_delta_when_previous_syllable_commits() {
    let mut composer = composer();
    type_keys(&mut composer, "gks");

    let effect = composer.input_key("r", false);

    assert_eq!(effect.committed, "한");
    assert_eq!(effect.composing, "ㄱ");
    assert!(effect.handled);
}

#[test]
fn unmapped_key_flushes_preedit_without_handling_key() {
    let mut composer = composer();
    type_keys(&mut composer, "gks");

    let effect = composer.input_key("1", false);

    assert_eq!(effect.committed, "한");
    assert_eq!(effect.composing, "");
    assert!(!effect.handled);
}

#[test]
fn backspace_without_preedit_is_not_handled() {
    let mut composer = composer();

    let effect = composer.backspace();

    assert_eq!(effect.committed, "");
    assert_eq!(effect.composing, "");
    assert!(!effect.handled);
}

#[test]
fn flush_commits_preedit_and_cancel_discards_preedit() {
    let mut composer = composer();
    type_keys(&mut composer, "gks");

    let flushed = composer.flush();
    assert_eq!(flushed.committed, "한");
    assert_eq!(flushed.composing, "");
    assert!(flushed.handled);

    type_keys(&mut composer, "gks");
    let cancelled = composer.cancel();
    assert_eq!(cancelled.committed, "");
    assert_eq!(cancelled.composing, "");
    assert!(cancelled.handled);
}
```

`yido-core/tests/layout_tests.rs`에 지원하지 않는 engine 검증을 추가한다.

```rust
#[test]
fn unsupported_engine_returns_layout_error() {
    let toml = r#"
[layout]
id = "broken"
name = "Broken"
engine = "romaja"

[keys.r]
normal = { jamo = "ㄱ", role = "auto" }
"#;

    let error = Layout::from_toml(toml).expect_err("지원하지 않는 engine은 오류여야 한다");

    assert!(matches!(error, LayoutError::UnsupportedEngine { engine } if engine == "romaja"));
}
```

- [ ] **Step 2: 실패 확인**

Run: `cargo test -p yido-core`

Expected: `InputEffect`, `flush`, `cancel`, `UnsupportedEngine`가 없어서 컴파일 실패한다.

- [ ] **Step 3: 최소 구현**

`InputState`를 `InputEffect`로 대체하고 `Composer`에서 누적 `committed` 필드를 제거한다. `input_key`, `backspace`, `flush`, `cancel`은 항상 호출 단위 effect를 반환한다. 미매핑 키는 preedit를 flush하되 `handled = false`로 반환한다. `Layout::from_toml`은 `engine != "hangul"`이면 `LayoutError::UnsupportedEngine`을 반환한다.

- [ ] **Step 4: 통과 확인**

Run: `cargo test -p yido-core`

Expected: 모든 core 테스트가 통과한다.

- [ ] **Step 5: 커밋**

```bash
git add yido-core/src/composer.rs yido-core/src/lib.rs yido-core/src/layout.rs yido-core/tests/composer_tests.rs yido-core/tests/layout_tests.rs
git commit -m "core 입력 효과 모델 적용"
```

---

### Task 2: WASM과 웹 데모 이행

**Files:**
- Modify: `yido-wasm/src/lib.rs`
- Modify: `yido-web/src/App.tsx`

- [ ] **Step 1: 실패 테스트로 빌드 확인**

Run: `mise run web:build`

Expected: `Composer::state` 또는 기존 `EngineState.text` 의존 때문에 빌드 실패한다.

- [ ] **Step 2: WASM API 수정**

`YidoEngine`에서 `state()`를 제거하고 `flush()`와 `cancel()`을 노출한다. `inputKey`, `backspace`, `flush`, `cancel`은 `InputEffect`를 직렬화해 반환한다.

- [ ] **Step 3: 웹 effect 적용 로직 추가**

`App.tsx`에 웹 전용 표시 상태를 둔다.

```ts
type EngineEffect = {
  committed: string
  composing: string
  handled: boolean
}

type DisplayState = {
  committed: string
  composing: string
  text: string
}
```

effect 적용은 `committed` 버퍼에 `effect.committed`를 더하고, `handled === false`인 인쇄 가능 키는 웹 데모가 기본 입력을 흉내 내기 위해 원래 키를 추가한다. 백스페이스에서 `handled === false`이면 웹 표시 버퍼의 마지막 Unicode scalar를 제거한다.

- [ ] **Step 4: 빌드 확인**

Run: `mise run web:build`

Expected: WASM 패키지 생성과 웹 빌드가 통과한다.

- [ ] **Step 5: 커밋**

```bash
git add yido-wasm/src/lib.rs yido-web/src/App.tsx yido-web/src/wasm
git commit -m "웹 데모 입력 효과 모델 적용"
```

---

### Task 3: Rust FFI와 cbindgen

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Create: `yido-ffi/Cargo.toml`
- Create: `yido-ffi/src/lib.rs`
- Create: `yido-ffi/tests/ffi_tests.rs`
- Create: `cbindgen.toml`
- Modify: `mise.toml`

- [ ] **Step 1: 실패 테스트 작성**

`yido-ffi/tests/ffi_tests.rs`에 C ABI 함수를 직접 호출하는 Rust 테스트를 추가한다.

```rust
use std::ffi::{CStr, CString};

use yido_ffi::{
    yido_engine_backspace, yido_engine_cancel, yido_engine_create_result_free, yido_engine_flush,
    yido_engine_free, yido_engine_input_key, yido_engine_new, yido_input_effect_free,
};

const DUBEOLSIK: &str = include_str!("../../layouts/ko-dubeolsik.toml");

#[test]
fn creates_engine_and_returns_effects() {
    let layout = CString::new(DUBEOLSIK).unwrap();
    let result = yido_engine_new(layout.as_ptr());
    assert!(result.error.is_null());
    assert!(!result.engine.is_null());
    let engine = result.engine;
    yido_engine_create_result_free(result);

    let g = CString::new("g").unwrap();
    let effect = yido_engine_input_key(engine, g.as_ptr(), false);
    assert!(effect.error.is_null());
    assert_eq!(unsafe { CStr::from_ptr(effect.composing) }.to_str().unwrap(), "ㅎ");
    yido_input_effect_free(effect);

    yido_engine_free(engine);
}
```

- [ ] **Step 2: 실패 확인**

Run: `cargo test -p yido-ffi`

Expected: `yido-ffi` package가 없어서 실패한다.

- [ ] **Step 3: FFI 구현**

`yido-ffi` crate를 추가한다. `YidoEngineCreateResult`, `YidoInputEffect`, opaque `YidoEngine`, free 함수들을 `#[repr(C)]`와 `#[unsafe(no_mangle)] extern "C"`로 노출한다. null pointer, invalid UTF-8, panic boundary는 오류 문자열로 변환한다.

- [ ] **Step 4: cbindgen 설정과 태스크 추가**

`cbindgen.toml`은 C language, include guard `YIDO_FFI_H`, output path `yido-swift/Sources/CYidoFFI/include/yido_ffi.h`를 기준으로 둔다. `mise.toml`에 `ffi:build`, `ffi:header`, `ffi:test` 태스크를 추가한다.

- [ ] **Step 5: 통과 확인**

Run: `cargo test -p yido-ffi`

Expected: FFI 테스트가 통과한다.

Run: `mise run ffi:header`

Expected: `yido-swift/Sources/CYidoFFI/include/yido_ffi.h`가 생성된다.

- [ ] **Step 6: 커밋**

```bash
git add Cargo.toml Cargo.lock yido-ffi cbindgen.toml mise.toml yido-swift/Sources/CYidoFFI/include/yido_ffi.h
git commit -m "Rust FFI 계층 추가"
```

---

### Task 4: Swift Package 골격과 테스트

**Files:**
- Create: `yido-swift/Package.swift`
- Create: `yido-swift/Sources/CYidoFFI/module.modulemap`
- Create: `yido-swift/Sources/YidoRustFFI/RustEngine.swift`
- Create: `yido-swift/Sources/YidoInputCore/InputSession.swift`
- Create: `yido-swift/Sources/YidoSettings/LayoutStore.swift`
- Create: `yido-swift/Sources/YidoInputMethod/YidoInputController.swift`
- Create: `yido-swift/Tests/YidoInputCoreTests/InputSessionTests.swift`
- Create: `yido-swift/Tests/YidoSettingsTests/LayoutStoreTests.swift`

- [ ] **Step 1: Swift 테스트 작성**

`InputSessionTests`는 mock engine으로 `committed`와 `composing` 적용 순서를 검증한다.

```swift
@Test
func appliesCommittedBeforeMarkedText() throws {
    let engine = MockEngine(effects: [
        .init(committed: "한", composing: "ㄱ", handled: true)
    ])
    let client = MockTextClient()
    let session = InputSession(engine: engine)

    let handled = try session.handlePrintableKey("r", shift: false, client: client)

    #expect(handled)
    #expect(client.operations == [.insert("한"), .mark("ㄱ")])
}
```

`LayoutStoreTests`는 기본 두벌식 제거 불가와 중복 id 거부를 검증한다.

- [ ] **Step 2: 실패 확인**

Run: `swift test --package-path yido-swift`

Expected: package가 없어서 실패한다.

- [ ] **Step 3: Swift Package 구현**

`Package.swift`는 macOS 26을 platforms에 선언하고 IMKSwift dependency를 추가한다. `YidoInputCore`는 FFI와 분리된 protocol 기반 세션 로직을 제공한다. `YidoSettings`는 파일 시스템 주입 기반 store를 제공한다. `YidoInputMethod`는 IMKSwift 컨트롤러 골격을 제공하되 비즈니스 로직은 `YidoInputCore`에 위임한다.

- [ ] **Step 4: 통과 확인**

Run: `swift test --package-path yido-swift`

Expected: Swift Package 테스트가 통과한다.

- [ ] **Step 5: 커밋**

```bash
git add yido-swift
git commit -m "Swift 입력기 패키지 골격 추가"
```

---

### Task 5: 전체 검증

**Files:**
- Modify: `docs/superpowers/plans/2026-05-29-yido-macos-input-method.md`

- [ ] **Step 1: 계획 체크박스 갱신**

완료한 항목을 체크한다.

- [ ] **Step 2: 전체 검증 실행**

Run: `cargo test`

Expected: Rust workspace 테스트가 통과한다.

Run: `mise run web:build`

Expected: WASM과 웹 빌드가 통과한다.

Run: `cargo test -p yido-ffi`

Expected: FFI 테스트가 통과한다.

Run: `swift test --package-path yido-swift`

Expected: Swift Package 테스트가 통과한다.

- [ ] **Step 3: 최종 커밋**

```bash
git add docs/superpowers/plans/2026-05-29-yido-macos-input-method.md
git commit -m "macOS 입력기 구현 계획 갱신"
```
