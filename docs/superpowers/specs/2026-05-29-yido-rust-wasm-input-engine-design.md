# 이도 Rust WASM 입력기 엔진 설계

## 목표

이도의 첫 입력기 엔진 MVP를 Rust core, WASM 브리지, SolidJS 웹 테스트 화면으로 만든다. 엔진은 TOML로 정의한 자판 배열을 로드하고, 기본 두벌식 배열을 사용해 한글 음절을 올바르게 조합하며, 조합 상태를 고려한 백스페이스를 지원해야 한다.

## 범위

이 MVP는 TOML로 정의한 자판 배열이 브라우저에서 실제 한글 입력기 엔진을 구동할 수 있는지 검증한다. 범위에는 기본 두벌식 배열, 배열 파싱과 한글 조합을 검증하는 Rust 단위 테스트, WASM 래퍼, 입력 결과와 조합 중인 문자열을 확인하는 최소 웹 UI가 포함된다.

이번 범위에는 TOML 편집기, 배열 저장, 사용자 계정, 브라우저 입력기 패키징, 모바일 키보드 연동, UI에서 여러 배열을 선택하는 기능은 포함하지 않는다.

## 아키텍처

저장소는 Rust와 웹을 함께 다루는 workspace 구조로 확장한다.

Rust 쪽에는 플랫폼에 독립적인 `yido-core` crate와 얇은 `yido-wasm` crate를 둔다. `yido-core`는 배열 파싱, 배열 검증, 키 매핑, 한글 조합 상태, 백스페이스 동작을 담당한다. `yido-wasm`은 `wasm-bindgen` 기반 API만 노출하고, TOML과 키 이벤트를 받아 모든 동작을 `yido-core`에 위임한 뒤 JavaScript에서 쓰기 쉬운 엔진 상태를 반환한다.

기존 `yido-web` SolidJS/Vite 앱은 브라우저 테스트 화면으로 유지한다. 웹 앱은 생성된 WASM 패키지를 가져오고, 번들된 두벌식 TOML을 로드하고, 엔진 인스턴스를 만든 뒤 키 입력을 받아 결과 문자열을 렌더링한다.

## 예상 파일 구조

`Cargo.toml`은 Rust workspace를 정의한다.

`crates/yido-core`에는 core 라이브러리를 둔다. 이 crate는 배열 타입, TOML 파싱, 한글 테이블, 조합 규칙, 단위 테스트를 포함한다.

`crates/yido-core/layouts/ko-dubeolsik.toml`에는 기본 두벌식 배열을 둔다.

`crates/yido-wasm`에는 `wasm-bindgen` 래퍼와 TypeScript에서 사용할 결과 타입을 둔다.

`yido-web`은 SolidJS 앱을 유지하고, WASM 연동 코드와 MVP 입력 테스트 화면을 추가한다.

## TOML 배열 스키마

배열은 Rust 코드에 특정 자판의 키 매핑을 박아 넣지 않기 위해 TOML로 정의한다. 다만 한글 조합 규칙은 Rust 엔진이 소유한다.

```toml
[layout]
id = "ko-dubeolsik"
name = "Korean Dubeolsik"
engine = "hangul"

[keys.r]
normal = { jamo = "ㄱ", role = "auto" }
shift = { jamo = "ㄲ", role = "auto" }

[keys.s]
normal = { jamo = "ㄴ", role = "auto" }

[keys.k]
normal = { jamo = "ㅏ", role = "medial" }
```

각 키는 `normal`과 `shift`를 가질 수 있다. 각 매핑은 `jamo` 문자열과 `role`을 포함한다.

유효한 `role`은 다음 네 가지다.

- `auto`: composer가 문맥에 따라 자음을 초성 또는 종성으로 판단한다.
- `initial`: 이 매핑은 명시적인 초성이다.
- `medial`: 이 매핑은 중성이다.
- `final`: 이 매핑은 명시적인 종성이다.

두벌식은 자음을 `auto`, 모음을 `medial`로 정의한다. 나중에 세벌식을 추가할 때는 `initial`, `medial`, `final`을 사용해 초성·중성·종성 키 그룹을 표현하고, 같은 core 조합 엔진을 재사용한다.

## Core 모델

`Layout`은 파싱된 TOML 배열을 나타내며 메타데이터와 키 매핑을 저장한다.

`KeyMapping`은 일반 키 또는 shift 키의 출력 하나를 나타낸다.

`JamoRole`은 `auto`, `initial`, `medial`, `final`을 나타낸다.

`Composer`는 현재 한글 조합 상태와 확정된 출력 버퍼를 소유한다.

`InputState`는 입력과 백스페이스 후 반환되는 공개 결과 타입이다.

```rust
pub struct InputState {
    pub committed: String,
    pub composing: String,
    pub text: String,
}
```

`committed`는 확정된 문자열 버퍼다. `composing`은 현재 조합 중인 preedit 문자열이다. `text`는 화면에 표시할 `committed + composing` 문자열이다.

## 한글 조합

composer는 Unicode 한글 음절 조합 공식을 사용한다.

```text
syllable = 0xAC00 + ((initial_index * 21) + medial_index) * 28 + final_index
```

core는 현대 한글 초성, 중성, 종성 테이블을 정의한다. 또한 `ㅗ + ㅏ = ㅘ`, `ㄱ + ㅅ = ㄳ`처럼 복합 모음과 복합 종성을 만드는 조합 맵도 정의한다.

종성이 있는 음절 뒤에 모음이 입력되면 composer는 종성을 떼어내 다음 음절의 초성으로 사용할 수 있는지 판단한다. 가능하면 그 자음을 다음 음절의 초성으로 이동한다. 이는 `gksrmf`가 `한글`이 되는 두벌식 입력에 필요하다.

새 입력을 현재 조합 중인 음절에 병합할 수 없으면 composer는 현재 조합 문자열을 확정하고 새 조합을 시작한다.

## 백스페이스 동작

백스페이스는 확정 문자열을 지우기 전에 조합 상태에 먼저 적용한다.

현재 음절에 복합 종성이 있으면 백스페이스는 복합 종성을 첫 번째 구성 자음으로 분해한다. 단일 종성이 있으면 종성을 제거한다. 중성이 복합 모음이면 첫 번째 구성 모음으로 분해한다. 중성이 단일 모음이면 중성을 제거하고 초성만 조합 문자열로 남긴다. 초성만 남아 있으면 조합 문자열을 비운다.

조합 중인 문자열이 없으면 백스페이스는 확정 버퍼에서 마지막 Unicode scalar value를 제거한다.

예를 들어 `gks`를 입력하면 `한`이 표시된다. 백스페이스를 누르면 `하`가 표시되고, 다시 누르면 `ㅎ`이 표시되고, 다시 누르면 조합 문자열이 비워진다.

## WASM API

WASM 래퍼는 JavaScript에서 사용할 작은 엔진 API를 노출한다.

```ts
const engine = new YidoEngine(layoutToml)
engine.inputKey("g", false)
engine.inputKey("k", false)
engine.backspace()
engine.reset()
engine.state()
```

상태를 반환하는 메서드는 다음 형태를 반환한다.

```ts
type EngineState = {
  committed: string
  composing: string
  text: string
}
```

래퍼는 직렬화를 단순하게 유지하고, TypeScript 쪽에 조합 로직을 중복 구현하지 않는다.

## 웹 MVP

웹 앱은 번들된 두벌식 TOML을 로드하고 `YidoEngine`을 초기화한다. 첫 UI는 랜딩 페이지가 아니라 실제 입력을 검증하는 화면으로 만든다.

화면에는 포커스를 받을 수 있는 입력 영역, 표시 결과 문자열, 현재 조합 문자열, reset 버튼을 둔다. 테스트 영역에 포커스가 있을 때 인쇄 가능한 Latin 키와 백스페이스를 가로챈다. shift 상태는 엔진에 전달해 shift가 필요한 두벌식 자음도 검증할 수 있게 한다.

이번 MVP에는 TOML 편집 UI를 넣지 않는다. Rust API는 TOML 문자열을 받는 형태로 유지하므로, 이후 편집 UI를 추가해도 엔진 경계를 다시 설계할 필요가 없다.

## 검증

Rust 단위 테스트는 TOML 파싱, 잘못된 배열 오류, shift를 포함한 키 조회, 기본 음절 조합, 복합 모음, 복합 종성, 종성을 다음 음절 초성으로 넘기는 동작, 백스페이스 분해를 다룬다.

초기 두벌식 동작 테스트에는 다음 사례를 포함한다.

```text
gks -> 한
gksrmf -> 한글
hk -> ㅘ
ghk -> 화
rkrt -> 갃
gks + backspace -> 하
gks + backspace + backspace -> ㅎ
```

웹 앱 빌드는 TypeScript 연동과 WASM 패키징을 검증한다. 브라우저 테스트는 테스트 화면에서 키를 입력했을 때 표시 문자열이 갱신되고, reset이 상태를 비우는지 확인한다.

## 위험 요소와 결정 사항

중요한 설계 결정은 한글 조합 규칙을 Rust에 두고, TOML은 배열의 입력 의도만 표현하게 하는 것이다. 이 방식은 각 배열이 한국어 맞춤법 수준의 조합 규칙을 다시 정의하지 않아도 두벌식과 이후 세벌식을 함께 지원할 수 있게 한다.

가장 큰 구현 위험은 복합 종성 뒤에 모음이 이어지는 경우를 올바르게 처리하는 것이다. 첫 구현 계획에는 웹 UI를 넓히기 전에 이 전이를 좁게 검증하는 테스트를 포함해야 한다.

두 번째 위험은 기존 Vite 앱 안에서 WASM 패키징을 반복 가능하게 만드는 것이다. 구현 계획에서는 하나의 빌드 경로를 선택하고 개발과 CI에서 사용할 명령을 문서화해야 한다.
