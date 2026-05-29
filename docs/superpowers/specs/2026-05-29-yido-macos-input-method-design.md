# 이도 macOS 입력기 설계

## 목표

이도의 첫 macOS 입력기 MVP를 만든다. 입력기는 기존 Rust core의 한글 조합 엔진을 사용하고, Swift 쪽은 macOS InputMethodKit 연결, 설정 창, 레이아웃 파일 관리를 담당한다. 사용자는 설정 창에서 TOML 레이아웃 파일을 가져오고 제거할 수 있으며, 기본 두벌식 레이아웃은 항상 제공되고 제거할 수 없다.

## 범위

이번 MVP는 후보창과 변환 기능이 없는 순수 한글 조합 입력기다. 레이아웃에 매핑된 인쇄 가능한 키를 Rust core로 보내 조합 문자열을 만들고, 경계 키가 입력되면 현재 조합을 확정한다. 숫자, 기호, 공백, Enter, Escape 같은 기본 입력은 현재 조합을 먼저 확정한 뒤 클라이언트 앱의 기본 동작으로 넘긴다. 백스페이스는 조합 중일 때만 입력기가 처리하고, 조합 중인 문자열이 없으면 클라이언트 앱이 처리하게 둔다.

이번 범위에는 후보창, 단어 변환, 레이아웃 편집기, 자동 업데이트, 앱 그룹 동기화, iCloud 동기화, Windows/Linux 입력기 지원을 포함하지 않는다.

## 기준 환경

macOS 입력기는 macOS 26.0 이상, Xcode 26 이상, Swift 6.2 이상을 기준으로 만든다. Swift 입력기 코드는 IMKSwift를 사용한다. IMKSwift는 Swift 6 strict concurrency에서 기존 `IMKInputController` 직접 상속이 만드는 actor isolation 문제를 피하기 위해 `IMKInputSessionController`를 제공하므로, 입력기 컨트롤러는 이 클래스를 상속한다.

Rust core와 FFI crate는 불필요하게 macOS 26 API에 묶지 않는다. 플랫폼 의존성은 Swift 입력기와 설정 UI에만 둔다.

## 핵심 결정

`yido-core`는 macOS 전용으로 바꾸지 않는다. 대신 실제 입력기 런타임에 맞는 플랫폼 독립 엔진으로 고친다. 현재 core의 `InputState.text`는 웹 데모에 표시할 전체 문자열이며, 입력기 core가 소유해야 할 모델이 아니다. 전체 문서 텍스트는 클라이언트 앱이 소유해야 하며, core는 현재 조합 상태와 이번 입력의 효과만 반환해야 한다.

새 core API는 누적 문서 문자열 대신 입력 효과를 반환한다.

```rust
pub struct InputEffect {
    pub committed: String,
    pub composing: String,
    pub handled: bool,
}
```

`committed`는 이번 입력으로 확정해야 하는 문자열이다. 누적 버퍼가 아니다. `composing`은 현재 조합 중인 preedit 문자열이다. `handled`는 입력기가 해당 키를 소비했는지 나타낸다. 이 모델은 macOS IMK뿐 아니라 이후 다른 플랫폼 입력기에도 재사용할 수 있다.

`Composer`는 더 이상 확정된 문서 문자열을 장기 보관하지 않는다. 내부에는 조합 중인 preedit만 둔다. 기존 `backspace_committed` 경로는 제거하고, 조합 중인 문자열이 없을 때 백스페이스는 `InputEffect { committed: "", composing: "", handled: false }`를 반환한다. 확정된 문서 텍스트 삭제는 클라이언트 앱의 책임이다.

Core는 조합을 확정하는 `flush`와 조합을 버리는 `cancel`을 구분한다. `flush`는 현재 preedit 문자열을 `committed`에 담아 반환하고 preedit를 비운다. 조합 중인 문자열이 없으면 `handled = false`를 반환한다. `cancel`은 현재 preedit를 버리고, 버릴 조합이 있었을 때만 `handled = true`를 반환한다.

```rust
impl Composer {
    pub fn input_key(&mut self, key: &str, shift: bool) -> InputEffect;
    pub fn backspace(&mut self) -> InputEffect;
    pub fn flush(&mut self) -> InputEffect;
    pub fn cancel(&mut self) -> InputEffect;
}
```

미매핑 인쇄 키의 라우팅 주체는 Rust core다. Swift는 단일 문자 키를 core에 전달하고, core가 레이아웃 조회 결과에 따라 처리 여부를 결정한다. 레이아웃에 매핑이 없는 키가 들어오면 core는 기존 preedit가 있을 경우 `flush`와 같은 효과를 반환하되 `handled = false`로 둔다. core는 미매핑 숫자, 기호, 공백을 직접 `committed`에 넣지 않는다. Swift는 `committed`를 먼저 적용한 뒤 `handled = false`를 보고 이벤트를 클라이언트 앱으로 넘긴다.

기존 웹 데모가 전체 표시 문자열을 필요로 하면 웹 앱이 자체 표시 버퍼를 들고 `InputEffect.committed`와 `InputEffect.composing`을 합쳐 렌더링한다. WASM의 `state()`는 누적 `committed`와 `text`를 반환하지 않는다. 초기 상태와 웹 UI의 초기화 화면은 웹 앱의 자체 버퍼와 마지막 `composing` 값으로 표현한다.

## 저장소 구조

새 macOS 작업은 `yido-swift/` 아래에 둔다. Swift 코드는 Swift Package 중심으로 구성하고, 실제 입력기 번들은 얇은 Xcode 프로젝트나 번들 타깃으로 둔다.

예상 구조는 다음과 같다.

```text
yido-swift/
  Package.swift
  Sources/
    YidoInputCore/
    YidoInputMethod/
    YidoSettings/
    YidoRustFFI/
  Tests/
    YidoInputCoreTests/
    YidoSettingsTests/
  YidoInputMethod.xcodeproj
```

`YidoRustFFI`는 generated C header를 통해 Rust FFI를 호출하는 Swift 래퍼다. `YidoInputCore`는 입력 세션, 키 처리 정책, IMK에 독립적인 입력 효과 적용 로직을 담는다. `YidoSettings`는 레이아웃 목록, 가져오기, 제거, 선택 상태 저장을 담당한다. `YidoInputMethod`는 IMKSwift와 AppKit/SwiftUI 설정 창 연결만 담당한다.

## Rust FFI

Swift와 Rust core의 연결은 새 Rust crate인 `yido-ffi`로 분리한다. `yido-ffi`는 `yido-core`에만 의존하고, `cdylib`와 `staticlib` 산출물을 만든다. C header는 `cbindgen`으로 생성한다.

FFI는 Rust 타입 내부를 Swift에 노출하지 않는다. Swift는 opaque handle만 들고, 모든 문자열은 UTF-8 C string으로 주고받는다.

예상 C ABI는 다음 형태다.

```c
typedef struct YidoEngine YidoEngine;

typedef struct YidoEngineCreateResult {
  YidoEngine *engine;
  char *error;
} YidoEngineCreateResult;

typedef struct YidoInputEffect {
  char *committed;
  char *composing;
  bool handled;
  char *error;
} YidoInputEffect;

YidoEngineCreateResult yido_engine_new(const char *layout_toml);
YidoInputEffect yido_engine_input_key(YidoEngine *engine, const char *key, bool shift);
YidoInputEffect yido_engine_backspace(YidoEngine *engine);
YidoInputEffect yido_engine_flush(YidoEngine *engine);
YidoInputEffect yido_engine_cancel(YidoEngine *engine);
void yido_engine_create_result_free(YidoEngineCreateResult result);
void yido_input_effect_free(YidoInputEffect effect);
void yido_engine_free(YidoEngine *engine);
```

Rust가 반환한 문자열의 소유권은 Rust에 있다. Swift는 값을 Swift `String`으로 복사한 뒤 반드시 대응하는 free 함수를 호출한다. 엔진 생성 실패와 레이아웃 검증 실패는 `YidoEngineCreateResult.error`로 전달한다. 입력 처리 중 오류가 발생하면 `YidoInputEffect.error`를 채우고 `handled = false`를 반환한다. Swift 래퍼는 raw result를 외부에 노출하지 않고 Swift `Result`로 변환한다.

FFI에는 `state()`를 노출하지 않는다. IMK 입력기는 polling으로 상태를 읽지 않고, 각 입력 이벤트의 `YidoInputEffect`만 적용한다. 디버그나 테스트에 상태 조회가 필요하면 Swift wrapper 내부 테스트 전용 API에서 마지막 `composing`만 보관한다.

## IMKSwift 연동

입력기 컨트롤러는 `IMKInputSessionController`를 상속한다. 컨트롤러는 입력 세션을 강하게 직접 소유하지 않는다. 클라이언트 객체를 키로 하는 weak-key cache를 두고, 입력기 전환이 잦을 때도 세션을 재사용한다. 이는 IMKSwift와 2026 macOS 입력기 가이드가 권장하는 구조와 맞다.

세션은 선택된 레이아웃 id를 읽고 해당 TOML로 Rust engine을 만든다. 레이아웃 선택이 변경되면 다음 활성화 시 새 engine을 구성한다. 조합 중에 레이아웃이 바뀌면 기존 조합을 먼저 확정하고 engine을 교체한다.

입력기 번들은 가능한 한 얇게 유지한다. 실제 판단은 테스트 가능한 Swift Package 라이브러리에 둔다. macOS 입력기는 브레이크포인트 디버깅이 위험하므로, mock text client를 이용한 단위 테스트를 중심으로 검증한다.

## 입력 처리

레이아웃에 매핑된 인쇄 가능한 키는 입력기가 소비한다. Swift 세션은 키 문자열과 shift 상태를 Rust engine에 전달하고, 반환된 `InputEffect`를 IMK text client에 적용한다.

`committed`가 비어 있지 않으면 클라이언트에 확정 문자열로 삽입한다. `composing`이 비어 있지 않으면 IMK composition으로 표시한다. `handled`가 `false`이면 입력기는 해당 이벤트를 소비하지 않는다.

경계 키는 현재 조합을 먼저 확정한다. 공백처럼 단일 문자로 표현되는 미매핑 키는 `yido_engine_input_key`가 preedit를 확정하고 `handled = false`를 반환한다. Enter처럼 문자 키가 아닌 경계 키는 Swift 세션이 `yido_engine_flush`를 호출한 뒤 이벤트를 클라이언트 기본 동작으로 넘긴다. Escape는 조합 중이면 `yido_engine_cancel`을 호출하고 소비한다. 조합 중이 아니면 소비하지 않는다. 백스페이스는 조합 중이면 Rust engine의 backspace를 호출하고 소비한다. 조합 중이 아니면 core가 `handled = false`를 반환하고 Swift도 이벤트를 소비하지 않는다.

키 문자열은 MVP에서 문자 기반으로 정의한다. Swift 세션은 `NSEvent.charactersIgnoringModifiers`에서 단일 Unicode scalar를 얻고, ASCII 알파벳은 소문자로 정규화해 `key`로 전달한다. Shift 상태는 별도 `shift` 인자로 전달한다. 예를 들어 `A`는 `key = "a", shift = true`이고, `!`는 `key = "1", shift = true`로 전달된다. 현재 설계는 물리 키코드 기반 배열을 지원하지 않으므로 Dvorak 같은 비-QWERTY 물리 배열 고정 동작은 MVP 범위 밖이다.

## 레이아웃 관리

기본 두벌식 레이아웃은 번들 리소스로 제공한다. 기본 레이아웃 id는 `ko-dubeolsik`이다. 설정 창에는 기본 두벌식이 항상 표시되고, 제거 버튼은 비활성화한다.

사용자 레이아웃은 TOML 파일 가져오기만 지원한다. 설정 창 안에서 키 매핑을 직접 편집하지 않는다. 가져온 TOML은 Rust core로 파싱하고 검증한다. `layout.id`가 기존 사용자 레이아웃 또는 `ko-dubeolsik`과 중복되면 가져오기를 거부한다. `layout.engine`은 미래의 다중 엔진 확장을 위한 필드지만, MVP에서는 `hangul`만 지원하고 다른 값은 가져오기를 거부한다. 가져오기에 성공하면 원본 TOML을 아래 위치에 저장한다.

```text
~/Library/Application Support/Yido/Layouts/
```

선택된 레이아웃 id와 설정 값은 입력기 번들 식별자 도메인의 `UserDefaults`에 저장한다. MVP에서는 앱 그룹을 사용하지 않는다.

## 설정 창

설정 창은 레이아웃 목록, 현재 선택 레이아웃, 가져오기 버튼, 제거 버튼을 제공한다. 제거 버튼은 사용자 레이아웃에서만 활성화한다. 가져오기 실패 시에는 실패 이유를 표시한다. 오류 표시는 core가 실제로 구분하는 단위에 맞춘다. MVP에서는 중복 id, TOML 또는 schema 파싱 실패, jamo 길이 오류, 지원하지 않는 engine 오류를 구분한다. 지원하지 않는 role은 serde 역직렬화 단계에서 TOML/schema 파싱 실패로 표시한다.

About 화면은 별도 창으로 만들지 않고 설정 창 안에 포함한다. macOS 26에서 NSWindow 수를 줄이는 것이 메모리 측면에서 유리하므로, MVP에서는 설정 창 하나만 둔다.

## 빌드와 산출물

Rust 쪽에는 `cbindgen.toml`과 FFI header 생성 태스크를 추가한다. Swift 빌드 전에 Rust static library와 generated header를 준비하는 스크립트를 둔다. `mise.toml`에는 macOS 입력기 빌드와 검증 태스크를 추가한다.

입력기 설치와 활성화는 개발 중 반복 가능해야 한다. 개발용 태스크는 빌드 산출물을 `~/Library/Input Methods/`로 복사하고 입력기 프로세스를 재시작한다. 배포용 서명과 notarization은 MVP 범위에서 제외한다.

## 검증

Rust core 테스트는 `InputEffect` 모델을 기준으로 다시 작성한다. 주요 검증은 두벌식 조합, 복합 모음, 복합 종성, 종성 이동, 조합 중 백스페이스, 조합 없음 백스페이스의 `handled = false`, `flush`의 조합 확정, `cancel`의 조합 폐기, 미매핑 키에서 조합 확정 후 `handled = false` 반환이다.

FFI 테스트는 엔진 생성, 키 입력, 백스페이스, flush, cancel, 문자열 free를 검증한다. Swift 테스트는 mock text client를 사용해 입력 효과가 commit과 composition으로 올바르게 변환되는지 확인한다. 설정 테스트는 기본 두벌식 존재, 두벌식 제거 불가, TOML 가져오기 성공, 중복 id 거부, 사용자 레이아웃 제거를 다룬다.

수동 검증은 TextEdit, Notes, Safari 주소창 같은 서로 다른 text client에서 수행한다. 입력기 활성화, `gksrmf` 입력 시 `한글` 조합, 공백으로 확정, 백스페이스 조합 분해, 레이아웃 가져오기와 선택 변경을 확인한다.

## 구현 순서

먼저 `yido-core`를 `InputEffect`와 `flush`/`cancel` 중심으로 리팩터링하고 테스트를 새 모델로 옮긴다. 다음으로 WASM과 웹 데모가 자체 표시 버퍼를 갖도록 수정한다. 그 뒤 `yido-ffi` crate와 cbindgen header 생성을 추가하고, 마지막으로 `yido-swift` Swift Package와 입력기 번들을 만든다.

## 위험 요소

가장 큰 위험은 core 모델 변경이 기존 WASM과 웹 데모에 영향을 주는 것이다. 이를 줄이기 위해 core 리팩터링은 macOS 입력기보다 먼저 수행하고, 웹 앱이 자체 표시 버퍼를 갖도록 함께 수정한다.

두 번째 위험은 Rust FFI 문자열 소유권이다. 모든 FFI 반환 문자열은 하나의 free 함수로 해제하게 하고, Swift 래퍼에서 raw pointer를 외부로 노출하지 않게 한다.

세 번째 위험은 IMK lifecycle이다. 컨트롤러가 세션을 강하게 들고 있으면 입력기 전환 시 ARC 비용과 지연이 생길 수 있다. 클라이언트별 weak-key cache를 사용해 세션 생명주기를 분리한다.

네 번째 위험은 macOS 입력기 디버깅이다. 브레이크포인트에 기대지 않고 Swift Package 단위 테스트와 mock client 테스트를 먼저 만든다. 입력기 번들 자체는 얇게 유지한다.

## 참고 자료

이 설계는 vChewing의 IMKSwift README와 Shiki Suen의 2026 macOS 입력기 개발 가이드를 참고했다. IMKSwift는 Swift 6 strict concurrency에서 `IMKInputSessionController`를 사용하도록 안내하며, 2026 가이드는 컨트롤러를 pass-through layer로 두고 business logic을 라이브러리와 테스트로 분리할 것을 권장한다. Rust와 Swift 연결에는 cbindgen을 사용해 C ABI header를 생성한다.
