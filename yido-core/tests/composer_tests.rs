use yido_core::{Composer, InputEffect, Layout};

const DUBEOLSIK: &str = include_str!("../../layouts/ko-dubeolsik.toml");

fn composer() -> Composer {
    let layout = Layout::from_toml(DUBEOLSIK).expect("두벌식 배열을 파싱해야 한다");
    Composer::new(layout)
}

fn composer_from_toml(source: &str) -> Composer {
    let layout = Layout::from_toml(source).expect("테스트 배열을 파싱해야 한다");
    Composer::new(layout)
}

fn type_keys(composer: &mut Composer, keys: &str) -> DisplayState {
    let mut state = DisplayState::default();

    for key in keys.chars() {
        let effect = composer.input_key(&key.to_string(), false);
        state.apply(effect);
    }

    state
}

#[derive(Default)]
struct DisplayState {
    committed: String,
    composing: String,
}

impl DisplayState {
    fn apply(&mut self, effect: InputEffect) {
        self.committed.push_str(&effect.committed);
        self.composing = effect.composing;
    }

    fn text(&self) -> String {
        format!("{}{}", self.committed, self.composing)
    }
}

#[test]
fn composes_single_dubeolsik_syllable() {
    let mut composer = composer();

    let state = type_keys(&mut composer, "gks");

    assert_eq!(state.committed, "");
    assert_eq!(state.composing, "한");
    assert_eq!(state.text(), "한");
}

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
fn composes_standalone_compound_vowel() {
    let mut composer = composer();

    let state = type_keys(&mut composer, "hk");

    assert_eq!(state.committed, "");
    assert_eq!(state.composing, "ㅘ");
    assert_eq!(state.text(), "ㅘ");
}

#[test]
fn composes_compound_vowel_inside_syllable() {
    let mut composer = composer();

    let state = type_keys(&mut composer, "ghk");

    assert_eq!(state.committed, "");
    assert_eq!(state.composing, "화");
    assert_eq!(state.text(), "화");
}

#[test]
fn composes_compound_final_inside_syllable() {
    let mut composer = composer();

    let state = type_keys(&mut composer, "rkrt");

    assert_eq!(state.committed, "");
    assert_eq!(state.composing, "갃");
    assert_eq!(state.text(), "갃");
}

#[test]
fn composes_multiple_dubeolsik_syllables() {
    let mut composer = composer();

    let state = type_keys(&mut composer, "gksrmf");

    assert_eq!(state.committed, "한");
    assert_eq!(state.composing, "글");
    assert_eq!(state.text(), "한글");
}

#[test]
fn moves_simple_final_to_next_initial_before_vowel() {
    let mut composer = composer();

    let state = type_keys(&mut composer, "gksk");

    assert_eq!(state.committed, "하");
    assert_eq!(state.composing, "나");
    assert_eq!(state.text(), "하나");
}

#[test]
fn splits_compound_final_before_vowel() {
    let mut composer = composer();

    let state = type_keys(&mut composer, "rkrtk");

    assert_eq!(state.committed, "각");
    assert_eq!(state.composing, "사");
    assert_eq!(state.text(), "각사");
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
fn unmapped_symbol_flushes_preedit_without_inserting_symbol() {
    let mut composer = composer();
    type_keys(&mut composer, "gks");

    let effect = composer.input_key("1", true);

    assert_eq!(effect.committed, "한");
    assert_eq!(effect.composing, "");
    assert!(!effect.handled);
}

#[test]
fn unmapped_space_flushes_preedit_without_inserting_space() {
    let mut composer = composer();
    type_keys(&mut composer, "gks");

    let effect = composer.input_key(" ", false);

    assert_eq!(effect.committed, "한");
    assert_eq!(effect.composing, "");
    assert!(!effect.handled);
}

#[test]
fn backspace_decomposes_final_then_medial_then_initial() {
    let mut composer = composer();
    type_keys(&mut composer, "gks");

    let first = composer.backspace();
    assert_eq!(first.committed, "");
    assert_eq!(first.composing, "하");
    assert!(first.handled);

    let second = composer.backspace();
    assert_eq!(second.committed, "");
    assert_eq!(second.composing, "ㅎ");
    assert!(second.handled);

    let third = composer.backspace();
    assert_eq!(third.committed, "");
    assert_eq!(third.composing, "");
    assert!(third.handled);
}

#[test]
fn backspace_splits_compound_vowel() {
    let mut composer = composer();
    type_keys(&mut composer, "ghk");

    let effect = composer.backspace();

    assert_eq!(effect.committed, "");
    assert_eq!(effect.composing, "호");
    assert!(effect.handled);
}

#[test]
fn backspace_splits_compound_final() {
    let mut composer = composer();
    type_keys(&mut composer, "rkrt");

    let effect = composer.backspace();

    assert_eq!(effect.committed, "");
    assert_eq!(effect.composing, "각");
    assert!(effect.handled);
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

#[test]
fn flush_and_cancel_without_preedit_are_not_handled() {
    let mut composer = composer();

    let flushed = composer.flush();
    assert_eq!(flushed.committed, "");
    assert_eq!(flushed.composing, "");
    assert!(!flushed.handled);

    let cancelled = composer.cancel();
    assert_eq!(cancelled.committed, "");
    assert_eq!(cancelled.composing, "");
    assert!(!cancelled.handled);
}

#[test]
fn explicit_initial_role_starts_next_syllable() {
    let mut composer = composer_from_toml(
        r#"
[layout]
id = "test-initial"
name = "Test Initial"
engine = "hangul"

[keys.g]
normal = { jamo = "ㄱ", role = "initial" }

[keys.k]
normal = { jamo = "ㅏ", role = "medial" }

[keys.s]
normal = { jamo = "ㄴ", role = "initial" }
"#,
    );

    let state = type_keys(&mut composer, "gks");

    assert_eq!(state.committed, "가");
    assert_eq!(state.composing, "ㄴ");
    assert_eq!(state.text(), "가ㄴ");
}

#[test]
fn explicit_final_role_attaches_as_syllable_final() {
    let mut composer = composer_from_toml(
        r#"
[layout]
id = "test-final"
name = "Test Final"
engine = "hangul"

[keys.g]
normal = { jamo = "ㄱ", role = "initial" }

[keys.k]
normal = { jamo = "ㅏ", role = "medial" }

[keys.s]
normal = { jamo = "ㄴ", role = "final" }
"#,
    );

    let state = type_keys(&mut composer, "gks");

    assert_eq!(state.committed, "");
    assert_eq!(state.composing, "간");
    assert_eq!(state.text(), "간");
}
