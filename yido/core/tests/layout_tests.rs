use yido_core::{JamoRole, Layout, LayoutError};

const DUBEOLSIK: &str = include_str!("../../../layouts/ko-dubeolsik.toml");

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

    let normal = layout
        .lookup("r", false)
        .expect("r 키의 일반 매핑이 있어야 한다");
    assert_eq!(normal.jamo(), 'ㄱ');
    assert_eq!(normal.role(), JamoRole::Auto);

    let shift = layout
        .lookup("r", true)
        .expect("r 키의 shift 매핑이 있어야 한다");
    assert_eq!(shift.jamo(), 'ㄲ');
    assert_eq!(shift.role(), JamoRole::Auto);

    let fallback = layout
        .lookup("s", true)
        .expect("shift 매핑이 없으면 normal 매핑을 사용한다");
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

    assert!(
        matches!(error, LayoutError::InvalidJamo { key, value } if key == "r.normal" && value == "ㄱㄴ")
    );
}

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
