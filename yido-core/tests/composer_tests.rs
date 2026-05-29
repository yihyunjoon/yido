use yido_core::{Composer, Layout};

const DUBEOLSIK: &str = include_str!("../../layouts/ko-dubeolsik.toml");

fn composer() -> Composer {
    let layout = Layout::from_toml(DUBEOLSIK).expect("두벌식 배열을 파싱해야 한다");
    Composer::new(layout)
}

fn composer_from_toml(source: &str) -> Composer {
    let layout = Layout::from_toml(source).expect("테스트 배열을 파싱해야 한다");
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
fn composes_multiple_dubeolsik_syllables() {
    let mut composer = composer();

    type_keys(&mut composer, "gksrmf");

    let state = composer.state();
    assert_eq!(state.committed, "한");
    assert_eq!(state.composing, "글");
    assert_eq!(state.text, "한글");
}

#[test]
fn moves_simple_final_to_next_initial_before_vowel() {
    let mut composer = composer();

    type_keys(&mut composer, "gksk");

    let state = composer.state();
    assert_eq!(state.committed, "하");
    assert_eq!(state.composing, "나");
    assert_eq!(state.text, "하나");
}

#[test]
fn splits_compound_final_before_vowel() {
    let mut composer = composer();

    type_keys(&mut composer, "rkrtk");

    let state = composer.state();
    assert_eq!(state.committed, "각");
    assert_eq!(state.composing, "사");
    assert_eq!(state.text, "각사");
}

#[test]
fn commits_preedit_before_literal_number() {
    let mut composer = composer();
    type_keys(&mut composer, "gks");

    let state = composer.input_key("1", false);

    assert_eq!(state.committed, "한1");
    assert_eq!(state.composing, "");
    assert_eq!(state.text, "한1");
}

#[test]
fn commits_preedit_before_literal_symbol() {
    let mut composer = composer();
    type_keys(&mut composer, "gks");

    let state = composer.input_key("!", true);

    assert_eq!(state.committed, "한!");
    assert_eq!(state.composing, "");
    assert_eq!(state.text, "한!");
}

#[test]
fn commits_preedit_before_literal_space() {
    let mut composer = composer();
    type_keys(&mut composer, "gks");

    let state = composer.input_key(" ", false);

    assert_eq!(state.committed, "한 ");
    assert_eq!(state.composing, "");
    assert_eq!(state.text, "한 ");
}

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

    assert_eq!(state.committed, "");
    assert_eq!(state.composing, "");
    assert_eq!(state.text, "");
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

    type_keys(&mut composer, "gks");

    let state = composer.state();
    assert_eq!(state.committed, "가");
    assert_eq!(state.composing, "ㄴ");
    assert_eq!(state.text, "가ㄴ");
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

    type_keys(&mut composer, "gks");

    let state = composer.state();
    assert_eq!(state.committed, "");
    assert_eq!(state.composing, "간");
    assert_eq!(state.text, "간");
}
