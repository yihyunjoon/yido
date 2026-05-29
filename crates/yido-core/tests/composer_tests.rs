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
