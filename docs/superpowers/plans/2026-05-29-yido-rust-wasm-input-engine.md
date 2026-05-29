# 이도 Rust WASM 입력기 엔진 구현 계획

> **에이전트 작업자용:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** TOML로 정의한 두벌식 배열을 Rust core에서 파싱하고 한글을 조합한 뒤, WASM을 통해 SolidJS 웹 앱에서 입력 결과를 확인할 수 있게 만든다.

**Architecture:** 저장소 루트에 Rust workspace를 추가하고 `crates/yido-core`와 `crates/yido-wasm`을 둔다. `yido-core`는 배열 파싱과 한글 조합 상태 머신을 담당하고, `yido-wasm`은 `wasm-bindgen` API만 제공한다. `yido-web`은 생성된 WASM 패키지와 번들된 두벌식 TOML을 로드해 MVP 입력 테스트 화면을 렌더링한다.

**Tech Stack:** Rust 1.95, Cargo workspace, serde, toml, thiserror, wasm-bindgen, wasm-pack 0.14, SolidJS 1.9, Vite 8, TypeScript 6, pnpm 10.

---

## 파일 구조

`Cargo.toml`은 Rust workspace를 정의한다.

`crates/yido-core/Cargo.toml`은 core crate 의존성을 정의한다.

`crates/yido-core/src/lib.rs`는 public API를 다시 export한다.

`crates/yido-core/src/layout.rs`는 TOML 배열 파싱, 타입, 조회, 오류를 담당한다.

`crates/yido-core/src/hangul.rs`는 초성·중성·종성 테이블, Unicode 조합 공식, 복합 자모 조합과 분해를 담당한다.

`crates/yido-core/src/composer.rs`는 입력 상태 머신, 확정 문자열, 조합 문자열, 백스페이스를 담당한다.

`crates/yido-core/layouts/ko-dubeolsik.toml`은 기본 두벌식 배열이다.

`crates/yido-core/tests/layout_tests.rs`는 배열 파싱과 조회를 검증한다.

`crates/yido-core/tests/composer_tests.rs`는 한글 조합과 백스페이스를 검증한다.

`crates/yido-wasm/Cargo.toml`은 WASM crate 의존성을 정의한다.

`crates/yido-wasm/src/lib.rs`는 JavaScript에서 사용할 `YidoEngine` API를 제공한다.

`yido-web/package.json`은 WASM 빌드 스크립트를 추가한다.

`yido-web/vite.config.ts`는 웹 앱에서 저장소 루트의 TOML을 raw import할 수 있게 허용한다.

`yido-web/src/App.tsx`는 입력 테스트 화면과 키 이벤트 연결을 담당한다.

`yido-web/src/App.css`는 MVP 테스트 화면 스타일을 담당한다.

`yido-web/src/index.tsx`는 앱 entrypoint이고 CSS import만 추가한다.

`yido-web/src/wasm/yido_wasm/*`는 `wasm-pack`이 생성한 웹용 WASM 패키지다.

---

### Task 1: Rust Workspace Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `crates/yido-core/Cargo.toml`
- Create: `crates/yido-core/src/lib.rs`
- Create: `crates/yido-wasm/Cargo.toml`
- Create: `crates/yido-wasm/src/lib.rs`

- [ ] **Step 1: Create the workspace manifest**

Create `Cargo.toml`:

```toml
[workspace]
members = [
    "crates/yido-core",
    "crates/yido-wasm",
]
resolver = "2"

[workspace.package]
edition = "2024"
version = "0.1.0"
```

- [ ] **Step 2: Create the core crate manifest and minimal library**

Create `crates/yido-core/Cargo.toml`:

```toml
[package]
name = "yido-core"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { version = "1", features = ["derive"] }
thiserror = "2"
toml = "0.9"
```

Create `crates/yido-core/src/lib.rs`:

```rust
pub fn crate_name() -> &'static str {
    "yido-core"
}
```

- [ ] **Step 3: Create the WASM crate manifest and minimal library**

Create `crates/yido-wasm/Cargo.toml`:

```toml
[package]
name = "yido-wasm"
version.workspace = true
edition.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
serde = { version = "1", features = ["derive"] }
serde-wasm-bindgen = "0.6"
wasm-bindgen = "0.2"
yido-core = { path = "../yido-core" }
```

Create `crates/yido-wasm/src/lib.rs`:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn crate_name() -> String {
    yido_core::crate_name().to_owned()
}
```

- [ ] **Step 4: Verify the scaffold builds**

Run:

```bash
cargo test
```

Expected: Cargo compiles both crates and reports `test result: ok`.

- [ ] **Step 5: Format and commit**

Run:

```bash
cargo fmt
git status --short
git add Cargo.toml crates/yido-core/Cargo.toml crates/yido-core/src/lib.rs crates/yido-wasm/Cargo.toml crates/yido-wasm/src/lib.rs Cargo.lock
git commit -m "Rust workspace 구성"
```

Expected: commit succeeds with the workspace scaffold.

---

### Task 2: TOML Layout Parsing And Bundled Dubeolsik

**Files:**
- Modify: `crates/yido-core/src/lib.rs`
- Create: `crates/yido-core/src/layout.rs`
- Create: `crates/yido-core/layouts/ko-dubeolsik.toml`
- Create: `crates/yido-core/tests/layout_tests.rs`

- [ ] **Step 1: Write failing layout tests**

Create `crates/yido-core/tests/layout_tests.rs`:

```rust
use yido_core::{JamoRole, Layout, LayoutError};

const DUBEOLSIK: &str = include_str!("../layouts/ko-dubeolsik.toml");

#[test]
fn bundled_dubeolsik_layout_loads_metadata() {
    let layout = Layout::from_toml(DUBEOLSIK).expect("두벌식 배열을 파싱해야 한다");

    assert_eq!(layout.id(), "ko-dubeolsik");
    assert_eq!(layout.name(), "Korean Dubeolsik");
    assert_eq!(layout.engine(), "hangul");
}

#[test]
fn bundled_dubeolsik_layout_looks_up_normal_and_shift_keys() {
    let layout = Layout::from_toml(DUBEOLSIK).expect("두벌식 배열을 파싱해야 한다");

    let normal = layout.lookup("r", false).expect("r 키의 일반 매핑이 있어야 한다");
    assert_eq!(normal.jamo(), 'ㄱ');
    assert_eq!(normal.role(), JamoRole::Auto);

    let shift = layout.lookup("r", true).expect("r 키의 shift 매핑이 있어야 한다");
    assert_eq!(shift.jamo(), 'ㄲ');
    assert_eq!(shift.role(), JamoRole::Auto);

    let fallback = layout.lookup("s", true).expect("shift 매핑이 없으면 normal 매핑을 사용한다");
    assert_eq!(fallback.jamo(), 'ㄴ');
    assert_eq!(fallback.role(), JamoRole::Auto);
}

#[test]
fn invalid_role_returns_layout_error() {
    let toml = r#"
[layout]
id = "broken"
name = "Broken"
engine = "hangul"

[keys.r]
normal = { jamo = "ㄱ", role = "unknown" }
"#;

    let error = Layout::from_toml(toml).expect_err("알 수 없는 role은 오류여야 한다");

    assert!(matches!(error, LayoutError::Toml(_)));
}

#[test]
fn multi_character_jamo_returns_layout_error() {
    let toml = r#"
[layout]
id = "broken"
name = "Broken"
engine = "hangul"

[keys.r]
normal = { jamo = "ㄱㄴ", role = "auto" }
"#;

    let error = Layout::from_toml(toml).expect_err("두 글자 jamo는 오류여야 한다");

    assert!(matches!(error, LayoutError::InvalidJamo { key, value } if key == "r.normal" && value == "ㄱㄴ"));
}
```

- [ ] **Step 2: Run the test and confirm it fails**

Run:

```bash
cargo test -p yido-core --test layout_tests
```

Expected: FAIL because `Layout`, `JamoRole`, and `LayoutError` are not exported yet.

- [ ] **Step 3: Implement layout parsing**

Replace `crates/yido-core/src/lib.rs`:

```rust
mod layout;

pub use layout::{JamoRole, KeyEntry, KeyMapping, Layout, LayoutError};
```

Create `crates/yido-core/src/layout.rs` with these public items and behavior:

```rust
use std::collections::HashMap;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("배열 TOML을 파싱하지 못했습니다: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("{key}의 jamo는 정확히 한 글자여야 합니다: {value}")]
    InvalidJamo { key: String, value: String },
}

#[derive(Debug, Clone)]
pub struct Layout {
    id: String,
    name: String,
    engine: String,
    keys: HashMap<String, KeyEntry>,
}

#[derive(Debug, Clone)]
pub struct KeyEntry {
    normal: Option<KeyMapping>,
    shift: Option<KeyMapping>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyMapping {
    jamo: char,
    role: JamoRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JamoRole {
    Auto,
    Initial,
    Medial,
    Final,
}

impl Layout {
    pub fn from_toml(source: &str) -> Result<Self, LayoutError> {
        let raw: RawLayoutFile = toml::from_str(source)?;
        let mut keys = HashMap::new();

        for (key, entry) in raw.keys {
            keys.insert(
                key.clone(),
                KeyEntry {
                    normal: entry
                        .normal
                        .map(|mapping| mapping.into_mapping(format!("{key}.normal")))
                        .transpose()?,
                    shift: entry
                        .shift
                        .map(|mapping| mapping.into_mapping(format!("{key}.shift")))
                        .transpose()?,
                },
            );
        }

        Ok(Self {
            id: raw.layout.id,
            name: raw.layout.name,
            engine: raw.layout.engine,
            keys,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn engine(&self) -> &str {
        &self.engine
    }

    pub fn lookup(&self, key: &str, shift: bool) -> Option<&KeyMapping> {
        let entry = self.keys.get(key)?;
        if shift {
            entry.shift.as_ref().or(entry.normal.as_ref())
        } else {
            entry.normal.as_ref()
        }
    }
}

impl KeyMapping {
    pub fn jamo(&self) -> char {
        self.jamo
    }

    pub fn role(&self) -> JamoRole {
        self.role
    }
}

#[derive(Debug, Deserialize)]
struct RawLayoutFile {
    layout: RawLayoutMetadata,
    keys: HashMap<String, RawKeyEntry>,
}

#[derive(Debug, Deserialize)]
struct RawLayoutMetadata {
    id: String,
    name: String,
    engine: String,
}

#[derive(Debug, Deserialize)]
struct RawKeyEntry {
    normal: Option<RawKeyMapping>,
    shift: Option<RawKeyMapping>,
}

#[derive(Debug, Deserialize)]
struct RawKeyMapping {
    jamo: String,
    role: JamoRole,
}

impl RawKeyMapping {
    fn into_mapping(self, key: String) -> Result<KeyMapping, LayoutError> {
        let mut chars = self.jamo.chars();
        let Some(jamo) = chars.next() else {
            return Err(LayoutError::InvalidJamo {
                key,
                value: self.jamo,
            });
        };

        if chars.next().is_some() {
            return Err(LayoutError::InvalidJamo {
                key,
                value: self.jamo,
            });
        }

        Ok(KeyMapping {
            jamo,
            role: self.role,
        })
    }
}
```

- [ ] **Step 4: Add bundled Dubeolsik TOML**

Create `crates/yido-core/layouts/ko-dubeolsik.toml`:

```toml
[layout]
id = "ko-dubeolsik"
name = "Korean Dubeolsik"
engine = "hangul"

[keys.q]
normal = { jamo = "ㅂ", role = "auto" }
shift = { jamo = "ㅃ", role = "auto" }

[keys.w]
normal = { jamo = "ㅈ", role = "auto" }
shift = { jamo = "ㅉ", role = "auto" }

[keys.e]
normal = { jamo = "ㄷ", role = "auto" }
shift = { jamo = "ㄸ", role = "auto" }

[keys.r]
normal = { jamo = "ㄱ", role = "auto" }
shift = { jamo = "ㄲ", role = "auto" }

[keys.t]
normal = { jamo = "ㅅ", role = "auto" }
shift = { jamo = "ㅆ", role = "auto" }

[keys.y]
normal = { jamo = "ㅛ", role = "medial" }

[keys.u]
normal = { jamo = "ㅕ", role = "medial" }

[keys.i]
normal = { jamo = "ㅑ", role = "medial" }

[keys.o]
normal = { jamo = "ㅐ", role = "medial" }
shift = { jamo = "ㅒ", role = "medial" }

[keys.p]
normal = { jamo = "ㅔ", role = "medial" }
shift = { jamo = "ㅖ", role = "medial" }

[keys.a]
normal = { jamo = "ㅁ", role = "auto" }

[keys.s]
normal = { jamo = "ㄴ", role = "auto" }

[keys.d]
normal = { jamo = "ㅇ", role = "auto" }

[keys.f]
normal = { jamo = "ㄹ", role = "auto" }

[keys.g]
normal = { jamo = "ㅎ", role = "auto" }

[keys.h]
normal = { jamo = "ㅗ", role = "medial" }

[keys.j]
normal = { jamo = "ㅓ", role = "medial" }

[keys.k]
normal = { jamo = "ㅏ", role = "medial" }

[keys.l]
normal = { jamo = "ㅣ", role = "medial" }

[keys.z]
normal = { jamo = "ㅋ", role = "auto" }

[keys.x]
normal = { jamo = "ㅌ", role = "auto" }

[keys.c]
normal = { jamo = "ㅊ", role = "auto" }

[keys.v]
normal = { jamo = "ㅍ", role = "auto" }

[keys.b]
normal = { jamo = "ㅠ", role = "medial" }

[keys.n]
normal = { jamo = "ㅜ", role = "medial" }

[keys.m]
normal = { jamo = "ㅡ", role = "medial" }
```

- [ ] **Step 5: Verify layout tests pass**

Run:

```bash
cargo test -p yido-core --test layout_tests
```

Expected: all 4 layout tests pass.

- [ ] **Step 6: Format and commit**

Run:

```bash
cargo fmt
cargo test -p yido-core --test layout_tests
git status --short
git add crates/yido-core/src/lib.rs crates/yido-core/src/layout.rs crates/yido-core/layouts/ko-dubeolsik.toml crates/yido-core/tests/layout_tests.rs Cargo.lock
git commit -m "두벌식 배열 파싱 추가"
```

Expected: commit succeeds with parsing tests passing.

---

### Task 3: Basic Hangul Composition

**Files:**
- Modify: `crates/yido-core/src/lib.rs`
- Create: `crates/yido-core/src/hangul.rs`
- Create: `crates/yido-core/src/composer.rs`
- Create: `crates/yido-core/tests/composer_tests.rs`

- [ ] **Step 1: Write failing basic composition tests**

Create `crates/yido-core/tests/composer_tests.rs`:

```rust
use yido_core::{Composer, Layout};

const DUBEOLSIK: &str = include_str!("../layouts/ko-dubeolsik.toml");

fn composer() -> Composer {
    let layout = Layout::from_toml(DUBEOLSIK).expect("두벌식 배열을 파싱해야 한다");
    Composer::new(layout)
}

fn type_keys(composer: &mut Composer, keys: &str) {
    for key in keys.chars() {
        composer.input_key(&key.to_string(), false);
    }
}

#[test]
fn composes_single_dubeolsik_syllable() {
    let mut composer = composer();

    type_keys(&mut composer, "gks");

    let state = composer.state();
    assert_eq!(state.committed, "");
    assert_eq!(state.composing, "한");
    assert_eq!(state.text, "한");
}

#[test]
fn commits_previous_syllable_when_new_initial_starts() {
    let mut composer = composer();

    type_keys(&mut composer, "gksr");

    let state = composer.state();
    assert_eq!(state.committed, "한");
    assert_eq!(state.composing, "ㄱ");
    assert_eq!(state.text, "한ㄱ");
}

#[test]
fn reset_clears_committed_and_composing_text() {
    let mut composer = composer();
    type_keys(&mut composer, "gks");

    let state = composer.reset();

    assert_eq!(state.committed, "");
    assert_eq!(state.composing, "");
    assert_eq!(state.text, "");
}
```

- [ ] **Step 2: Run the test and confirm it fails**

Run:

```bash
cargo test -p yido-core --test composer_tests composes_single_dubeolsik_syllable
```

Expected: FAIL because `Composer` is not exported yet.

- [ ] **Step 3: Add Hangul tables and basic composer API**

Modify `crates/yido-core/src/lib.rs`:

```rust
mod composer;
mod hangul;
mod layout;

pub use composer::{Composer, InputState};
pub use layout::{JamoRole, KeyEntry, KeyMapping, Layout, LayoutError};
```

Create `crates/yido-core/src/hangul.rs` with these functions:

```rust
pub fn is_initial(jamo: char) -> bool;
pub fn is_medial(jamo: char) -> bool;
pub fn is_final(jamo: char) -> bool;
pub fn compose_syllable(initial: char, medial: char, final_consonant: Option<char>) -> Option<char>;
```

The implementation must use the modern Hangul tables from the design document. `compose_syllable('ㅎ', 'ㅏ', Some('ㄴ'))` must return `Some('한')`, and unknown jamo combinations must return `None`.

Create `crates/yido-core/src/composer.rs` with these public items:

```rust
use serde::Serialize;

use crate::{hangul, JamoRole, Layout};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InputState {
    pub committed: String,
    pub composing: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Composer {
    layout: Layout,
    committed: String,
    preedit: Preedit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Preedit {
    Empty,
    Consonant(char),
    Vowel(char),
    Syllable {
        initial: char,
        medial: char,
        final_consonant: Option<char>,
    },
}
```

Implement `Composer::new`, `Composer::input_key`, `Composer::state`, and `Composer::reset`. `input_key` must look up the key in `Layout`, dispatch `JamoRole::Medial` to vowel input, and dispatch `JamoRole::Auto` consonants to context-sensitive consonant input. The basic rules in this task are consonant start, vowel after consonant, final consonant after initial+medial, and committing the current syllable when another consonant starts.

- [ ] **Step 4: Verify basic composition tests pass**

Run:

```bash
cargo test -p yido-core --test composer_tests composes_single_dubeolsik_syllable
cargo test -p yido-core --test composer_tests commits_previous_syllable_when_new_initial_starts
cargo test -p yido-core --test composer_tests reset_clears_committed_and_composing_text
```

Expected: all 3 selected tests pass.

- [ ] **Step 5: Format and commit**

Run:

```bash
cargo fmt
cargo test -p yido-core --test composer_tests
cargo test -p yido-core --test layout_tests
git status --short
git add crates/yido-core/src/lib.rs crates/yido-core/src/hangul.rs crates/yido-core/src/composer.rs crates/yido-core/tests/composer_tests.rs
git commit -m "기본 한글 조합 추가"
```

Expected: commit succeeds with all core tests passing.

---

### Task 4: Compound Jamo And Final-To-Initial Split

**Files:**
- Modify: `crates/yido-core/src/hangul.rs`
- Modify: `crates/yido-core/src/composer.rs`
- Modify: `crates/yido-core/tests/composer_tests.rs`

- [ ] **Step 1: Add failing composition transition tests**

Append to `crates/yido-core/tests/composer_tests.rs`:

```rust
#[test]
fn composes_standalone_compound_vowel() {
    let mut composer = composer();

    type_keys(&mut composer, "hk");

    let state = composer.state();
    assert_eq!(state.committed, "");
    assert_eq!(state.composing, "ㅘ");
    assert_eq!(state.text, "ㅘ");
}

#[test]
fn composes_compound_vowel_inside_syllable() {
    let mut composer = composer();

    type_keys(&mut composer, "ghk");

    let state = composer.state();
    assert_eq!(state.committed, "");
    assert_eq!(state.composing, "화");
    assert_eq!(state.text, "화");
}

#[test]
fn composes_compound_final_inside_syllable() {
    let mut composer = composer();

    type_keys(&mut composer, "rkrt");

    let state = composer.state();
    assert_eq!(state.committed, "");
    assert_eq!(state.composing, "갃");
    assert_eq!(state.text, "갃");
}

#[test]
fn moves_final_consonant_to_next_initial_before_vowel() {
    let mut composer = composer();

    type_keys(&mut composer, "gksrmf");

    let state = composer.state();
    assert_eq!(state.committed, "한");
    assert_eq!(state.composing, "글");
    assert_eq!(state.text, "한글");
}
```

- [ ] **Step 2: Run the tests and confirm they fail**

Run:

```bash
cargo test -p yido-core --test composer_tests composes_standalone_compound_vowel
cargo test -p yido-core --test composer_tests composes_compound_vowel_inside_syllable
cargo test -p yido-core --test composer_tests composes_compound_final_inside_syllable
cargo test -p yido-core --test composer_tests moves_final_consonant_to_next_initial_before_vowel
```

Expected: FAIL because compound vowel, compound final, and final-to-initial transitions are not complete.

- [ ] **Step 3: Implement compound jamo helpers**

Extend `crates/yido-core/src/hangul.rs` with these public functions:

```rust
pub fn combine_medial(left: char, right: char) -> Option<char>;
pub fn split_medial(jamo: char) -> Option<(char, char)>;
pub fn combine_final(left: char, right: char) -> Option<char>;
pub fn split_final(jamo: char) -> Option<(char, char)>;
```

The required mappings for this MVP are:

```rust
// 중성 조합
('ㅗ', 'ㅏ') => 'ㅘ'
('ㅗ', 'ㅐ') => 'ㅙ'
('ㅗ', 'ㅣ') => 'ㅚ'
('ㅜ', 'ㅓ') => 'ㅝ'
('ㅜ', 'ㅔ') => 'ㅞ'
('ㅜ', 'ㅣ') => 'ㅟ'
('ㅡ', 'ㅣ') => 'ㅢ'

// 종성 조합
('ㄱ', 'ㅅ') => 'ㄳ'
('ㄴ', 'ㅈ') => 'ㄵ'
('ㄴ', 'ㅎ') => 'ㄶ'
('ㄹ', 'ㄱ') => 'ㄺ'
('ㄹ', 'ㅁ') => 'ㄻ'
('ㄹ', 'ㅂ') => 'ㄼ'
('ㄹ', 'ㅅ') => 'ㄽ'
('ㄹ', 'ㅌ') => 'ㄾ'
('ㄹ', 'ㅍ') => 'ㄿ'
('ㄹ', 'ㅎ') => 'ㅀ'
('ㅂ', 'ㅅ') => 'ㅄ'
```

- [ ] **Step 4: Update composer transitions**

Update `crates/yido-core/src/composer.rs` so vowel input tries `hangul::combine_medial` when the current state is `Preedit::Vowel` or a syllable without final. Update consonant input so a syllable with a final consonant tries `hangul::combine_final` before committing. Update vowel input for a syllable with final consonant so the final moves to the next syllable initial when `hangul::is_initial(final_consonant)` is true.

For compound final followed by vowel, split the final into `(left, right)`, keep `left` on the previous syllable, commit that syllable, and start the next syllable with `right` plus the new vowel.

- [ ] **Step 5: Verify transition tests pass**

Run:

```bash
cargo test -p yido-core --test composer_tests composes_standalone_compound_vowel
cargo test -p yido-core --test composer_tests composes_compound_vowel_inside_syllable
cargo test -p yido-core --test composer_tests composes_compound_final_inside_syllable
cargo test -p yido-core --test composer_tests moves_final_consonant_to_next_initial_before_vowel
```

Expected: all 4 selected tests pass.

- [ ] **Step 6: Format and commit**

Run:

```bash
cargo fmt
cargo test -p yido-core
git status --short
git add crates/yido-core/src/hangul.rs crates/yido-core/src/composer.rs crates/yido-core/tests/composer_tests.rs
git commit -m "복합 자모 조합 추가"
```

Expected: commit succeeds with all `yido-core` tests passing.

---

### Task 5: Composition-Aware Backspace

**Files:**
- Modify: `crates/yido-core/src/composer.rs`
- Modify: `crates/yido-core/tests/composer_tests.rs`

- [ ] **Step 1: Add failing backspace tests**

Append to `crates/yido-core/tests/composer_tests.rs`:

```rust
#[test]
fn backspace_decomposes_final_then_medial_then_initial() {
    let mut composer = composer();
    type_keys(&mut composer, "gks");

    let first = composer.backspace();
    assert_eq!(first.committed, "");
    assert_eq!(first.composing, "하");
    assert_eq!(first.text, "하");

    let second = composer.backspace();
    assert_eq!(second.committed, "");
    assert_eq!(second.composing, "ㅎ");
    assert_eq!(second.text, "ㅎ");

    let third = composer.backspace();
    assert_eq!(third.committed, "");
    assert_eq!(third.composing, "");
    assert_eq!(third.text, "");
}

#[test]
fn backspace_splits_compound_vowel() {
    let mut composer = composer();
    type_keys(&mut composer, "ghk");

    let state = composer.backspace();

    assert_eq!(state.committed, "");
    assert_eq!(state.composing, "호");
    assert_eq!(state.text, "호");
}

#[test]
fn backspace_splits_compound_final() {
    let mut composer = composer();
    type_keys(&mut composer, "rkrt");

    let state = composer.backspace();

    assert_eq!(state.committed, "");
    assert_eq!(state.composing, "각");
    assert_eq!(state.text, "각");
}

#[test]
fn backspace_removes_committed_text_when_not_composing() {
    let mut composer = composer();
    type_keys(&mut composer, "gksr");
    composer.backspace();

    let state = composer.backspace();

    assert_eq!(state.committed, "하");
    assert_eq!(state.composing, "");
    assert_eq!(state.text, "하");
}
```

- [ ] **Step 2: Run the tests and confirm they fail**

Run:

```bash
cargo test -p yido-core --test composer_tests backspace_decomposes_final_then_medial_then_initial
cargo test -p yido-core --test composer_tests backspace_splits_compound_vowel
cargo test -p yido-core --test composer_tests backspace_splits_compound_final
cargo test -p yido-core --test composer_tests backspace_removes_committed_text_when_not_composing
```

Expected: FAIL because `Composer::backspace` is not implemented or not exported.

- [ ] **Step 3: Implement backspace**

Add this public method to `Composer` in `crates/yido-core/src/composer.rs`:

```rust
pub fn backspace(&mut self) -> InputState {
    match self.preedit {
        Preedit::Empty => {
            self.committed.pop();
        }
        Preedit::Consonant(_) => {
            self.preedit = Preedit::Empty;
        }
        Preedit::Vowel(medial) => {
            self.preedit = hangul::split_medial(medial)
                .map(|(left, _)| Preedit::Vowel(left))
                .unwrap_or(Preedit::Empty);
        }
        Preedit::Syllable {
            initial,
            medial,
            final_consonant: Some(final_consonant),
        } => {
            self.preedit = Preedit::Syllable {
                initial,
                medial,
                final_consonant: hangul::split_final(final_consonant).map(|(left, _)| left),
            };
        }
        Preedit::Syllable {
            initial,
            medial,
            final_consonant: None,
        } => {
            self.preedit = hangul::split_medial(medial)
                .map(|(left, _)| Preedit::Syllable {
                    initial,
                    medial: left,
                    final_consonant: None,
                })
                .unwrap_or(Preedit::Consonant(initial));
        }
    }

    self.state()
}
```

- [ ] **Step 4: Verify backspace tests pass**

Run:

```bash
cargo test -p yido-core --test composer_tests backspace_decomposes_final_then_medial_then_initial
cargo test -p yido-core --test composer_tests backspace_splits_compound_vowel
cargo test -p yido-core --test composer_tests backspace_splits_compound_final
cargo test -p yido-core --test composer_tests backspace_removes_committed_text_when_not_composing
```

Expected: all 4 selected tests pass.

- [ ] **Step 5: Format and commit**

Run:

```bash
cargo fmt
cargo test -p yido-core
git status --short
git add crates/yido-core/src/composer.rs crates/yido-core/tests/composer_tests.rs
git commit -m "조합 백스페이스 추가"
```

Expected: commit succeeds with all `yido-core` tests passing.

---

### Task 6: WASM Wrapper

**Files:**
- Modify: `crates/yido-wasm/src/lib.rs`
- Modify: `yido-web/package.json`
- Generate: `yido-web/src/wasm/yido_wasm/*`

- [ ] **Step 1: Replace the WASM wrapper**

Replace `crates/yido-wasm/src/lib.rs`:

```rust
use wasm_bindgen::prelude::*;
use yido_core::{Composer, Layout};

#[wasm_bindgen]
pub struct YidoEngine {
    composer: Composer,
}

#[wasm_bindgen]
impl YidoEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(layout_toml: &str) -> Result<YidoEngine, JsValue> {
        let layout = Layout::from_toml(layout_toml)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;

        Ok(Self {
            composer: Composer::new(layout),
        })
    }

    #[wasm_bindgen(js_name = inputKey)]
    pub fn input_key(&mut self, key: &str, shift: bool) -> Result<JsValue, JsValue> {
        to_js_value(&self.composer.input_key(key, shift))
    }

    pub fn backspace(&mut self) -> Result<JsValue, JsValue> {
        to_js_value(&self.composer.backspace())
    }

    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        to_js_value(&self.composer.reset())
    }

    pub fn state(&self) -> Result<JsValue, JsValue> {
        to_js_value(&self.composer.state())
    }
}

fn to_js_value<T: serde::Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|error| JsValue::from_str(&error.to_string()))
}
```

- [ ] **Step 2: Add web scripts for WASM generation**

Modify `yido-web/package.json` scripts:

```json
{
  "scripts": {
    "dev": "vite",
    "build:wasm": "wasm-pack build ../crates/yido-wasm --target web --out-dir ../../yido-web/src/wasm/yido_wasm --no-pack",
    "build": "pnpm run build:wasm && tsc -b && vite build",
    "preview": "vite preview"
  }
}
```

Keep the existing dependencies and devDependencies unchanged.

- [ ] **Step 3: Verify WASM target and generate package**

Run:

```bash
rustup target add wasm32-unknown-unknown
```

Expected: command succeeds and either installs the target or reports it is already up to date.

Run:

```bash
cd yido-web
pnpm run build:wasm
```

Expected: `wasm-pack` writes files under `yido-web/src/wasm/yido_wasm`.

- [ ] **Step 4: Verify Rust and WASM builds**

Run:

```bash
cargo test
cd yido-web
pnpm run build:wasm
```

Expected: Rust tests pass and WASM generation succeeds.

- [ ] **Step 5: Format and commit**

Run:

```bash
cargo fmt
git status --short
git add crates/yido-wasm/src/lib.rs yido-web/package.json yido-web/src/wasm/yido_wasm Cargo.lock
git commit -m "WASM 입력기 래퍼 추가"
```

Expected: commit succeeds with wrapper and generated WASM package.

---

### Task 7: SolidJS Web Test Surface

**Files:**
- Modify: `yido-web/vite.config.ts`
- Modify: `yido-web/src/index.tsx`
- Modify: `yido-web/src/App.tsx`
- Create: `yido-web/src/App.css`

- [ ] **Step 1: Configure Vite to allow raw TOML import from the Rust crate**

Replace `yido-web/vite.config.ts`:

```ts
import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'

export default defineConfig({
  plugins: [solid()],
  server: {
    fs: {
      allow: [fileURLToPath(new URL('..', import.meta.url))],
    },
  },
})
```

- [ ] **Step 2: Replace the app with the WASM-backed test screen**

Replace `yido-web/src/App.tsx`:

```tsx
import { createSignal, onMount } from 'solid-js'
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

  onMount(async () => {
    try {
      await init()
      const nextEngine = new YidoEngine(dubeolsikLayout)
      setEngine(nextEngine)
      setState(nextEngine.state() as EngineState)
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught))
    }
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

    if (/^[a-zA-Z]$/.test(event.key)) {
      event.preventDefault()
      setState(currentEngine.inputKey(event.key.toLowerCase(), event.shiftKey) as EngineState)
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
      <section class="input-panel" tabIndex={0} onKeyDown={handleKeyDown}>
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
```

- [ ] **Step 3: Add app styles**

Create `yido-web/src/App.css`:

```css
:root {
  color: #1e2329;
  background: #f6f2ea;
  font-family:
    Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

* {
  box-sizing: border-box;
}

body {
  min-width: 320px;
  min-height: 100vh;
  margin: 0;
}

button {
  border: 1px solid #2e5f6e;
  border-radius: 6px;
  background: #2e5f6e;
  color: #ffffff;
  cursor: pointer;
  font: inherit;
  font-weight: 700;
  padding: 0.65rem 0.9rem;
}

button:hover {
  background: #244d59;
}

.app-shell {
  display: grid;
  min-height: 100vh;
  padding: clamp(1rem, 4vw, 3rem);
  place-items: center;
}

.input-panel {
  width: min(760px, 100%);
  border: 1px solid #d7c7ad;
  border-radius: 8px;
  background: #fffdf8;
  box-shadow: 0 20px 60px rgb(46 95 110 / 14%);
  outline: none;
  padding: clamp(1rem, 4vw, 2rem);
}

.input-panel:focus {
  border-color: #2e5f6e;
  box-shadow:
    0 0 0 4px rgb(46 95 110 / 16%),
    0 20px 60px rgb(46 95 110 / 14%);
}

.panel-header {
  align-items: center;
  display: flex;
  gap: 1rem;
  justify-content: space-between;
  margin-bottom: 1.5rem;
}

.panel-header h1 {
  font-size: clamp(1.5rem, 4vw, 2.25rem);
  line-height: 1.1;
  margin: 0;
}

.result-box {
  align-items: center;
  border: 1px solid #e2d5c0;
  border-radius: 8px;
  display: flex;
  min-height: 9rem;
  overflow-wrap: anywhere;
  padding: 1.25rem;
  white-space: pre-wrap;
  word-break: keep-all;
  font-size: clamp(2rem, 7vw, 4rem);
}

.empty-output {
  color: #8b8172;
  font-size: 1.25rem;
}

.state-grid {
  display: grid;
  gap: 0.75rem;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin: 1rem 0 0;
}

.state-grid div {
  border: 1px solid #e2d5c0;
  border-radius: 8px;
  padding: 0.85rem;
}

.state-grid dt {
  color: #665d52;
  font-size: 0.85rem;
  font-weight: 700;
  margin-bottom: 0.35rem;
}

.state-grid dd {
  margin: 0;
  min-height: 1.5rem;
  overflow-wrap: anywhere;
}

.error-message {
  border: 1px solid #d98b7c;
  border-radius: 8px;
  color: #8f2f20;
  margin: 0 0 1rem;
  padding: 0.75rem;
}

@media (max-width: 560px) {
  .panel-header {
    align-items: stretch;
    flex-direction: column;
  }

  .state-grid {
    grid-template-columns: 1fr;
  }
}
```

- [ ] **Step 4: Import styles in the entrypoint**

Replace `yido-web/src/index.tsx`:

```tsx
/* @refresh reload */
import { render } from 'solid-js/web'
import App from './App.tsx'
import './App.css'

const root = document.getElementById('root')

render(() => <App />, root!)
```

- [ ] **Step 5: Verify the web build**

Run:

```bash
cd yido-web
pnpm run build
```

Expected: `build:wasm`, `tsc -b`, and `vite build` all succeed.

- [ ] **Step 6: Commit**

Run:

```bash
git status --short
git add yido-web/package.json yido-web/vite.config.ts yido-web/src/index.tsx yido-web/src/App.tsx yido-web/src/App.css yido-web/src/wasm/yido_wasm
git commit -m "웹 입력 테스트 화면 연결"
```

Expected: commit succeeds with the SolidJS WASM integration.

---

### Task 8: Final Verification

**Files:**
- No planned source edits.

- [ ] **Step 1: Run full Rust verification**

Run:

```bash
cargo test
```

Expected: all Rust unit and integration tests pass.

- [ ] **Step 2: Run full web verification**

Run:

```bash
cd yido-web
pnpm run build
```

Expected: WASM generation, TypeScript build, and Vite build all pass.

- [ ] **Step 3: Run browser smoke test**

Run:

```bash
cd yido-web
pnpm run dev -- --host 127.0.0.1
```

Expected: Vite prints a local URL. Open that URL in the in-app browser, focus the input panel, type `gksrmf`, confirm the displayed text is `한글`, press Backspace twice, and confirm the display becomes `한`.

- [ ] **Step 4: Confirm the working tree is clean**

Run:

```bash
git status --short
```

Expected: no output. If tracked files changed during browser verification, stop and inspect the diff before deciding whether a new implementation task is needed.

---

## 자체 검토

설계 문서의 요구사항은 이 계획에 모두 연결되어 있다. Rust workspace와 `yido-core`/`yido-wasm` 분리는 Task 1과 Task 6에서 다룬다. TOML 두벌식 배열과 `role` 기반 스키마는 Task 2에서 다룬다. 한글 조합, 복합 자모, 종성 이동, 백스페이스는 Task 3, Task 4, Task 5에서 다룬다. WASM API는 Task 6에서 다룬다. SolidJS 웹 테스트 화면과 빌드 검증은 Task 7과 Task 8에서 다룬다.

계획에는 금지된 플레이스홀더나 구현하지 않은 빈 섹션이 없다. 타입 이름은 `Layout`, `KeyMapping`, `JamoRole`, `Composer`, `InputState`, `YidoEngine`으로 전 단계와 후속 단계가 일치한다.
