const INITIALS: [char; 19] = [
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ',
    'ㅌ', 'ㅍ', 'ㅎ',
];

const MEDIALS: [char; 21] = [
    'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ',
    'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
];

const FINALS: [char; 27] = [
    'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ', 'ㅁ',
    'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
];

pub fn is_initial(jamo: char) -> bool {
    INITIALS.contains(&jamo)
}

pub fn is_medial(jamo: char) -> bool {
    MEDIALS.contains(&jamo)
}

pub fn is_final(jamo: char) -> bool {
    FINALS.contains(&jamo)
}

pub fn compose_syllable(
    initial: char,
    medial: char,
    final_consonant: Option<char>,
) -> Option<char> {
    let initial_index = INITIALS.iter().position(|jamo| *jamo == initial)?;
    let medial_index = MEDIALS.iter().position(|jamo| *jamo == medial)?;
    let final_index = match final_consonant {
        Some(final_consonant) => FINALS
            .iter()
            .position(|jamo| *jamo == final_consonant)
            .map(|index| index + 1)?,
        None => 0,
    };

    char::from_u32(0xAC00 + (((initial_index * 21) + medial_index) * 28 + final_index) as u32)
}
