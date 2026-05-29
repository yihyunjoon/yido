use serde::Serialize;

use crate::{JamoRole, Layout, hangul};

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

impl Composer {
    pub fn new(layout: Layout) -> Self {
        Self {
            layout,
            committed: String::new(),
            preedit: Preedit::Empty,
        }
    }

    pub fn input_key(&mut self, key: &str, shift: bool) -> InputState {
        let Some(mapping) = self.layout.lookup(key, shift) else {
            return self.state();
        };
        let jamo = mapping.jamo();
        let role = mapping.role();

        match role {
            JamoRole::Auto => {
                if hangul::is_medial(jamo) {
                    self.input_vowel(jamo);
                } else {
                    self.input_consonant(jamo);
                }
            }
            JamoRole::Initial | JamoRole::Final => self.input_consonant(jamo),
            JamoRole::Medial => self.input_vowel(jamo),
        }

        self.state()
    }

    pub fn state(&self) -> InputState {
        let composing = self.composing_text();
        let text = format!("{}{}", self.committed, composing);

        InputState {
            committed: self.committed.clone(),
            composing,
            text,
        }
    }

    pub fn reset(&mut self) -> InputState {
        self.committed.clear();
        self.preedit = Preedit::Empty;
        self.state()
    }

    pub fn backspace(&mut self) -> InputState {
        match self.preedit {
            Preedit::Empty => {
                self.backspace_committed();
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

    fn backspace_committed(&mut self) {
        let Some(last) = self.committed.pop() else {
            return;
        };

        if let Some(reduced) = Self::reduce_committed_char(last) {
            self.committed.push(reduced);
        }
    }

    fn input_consonant(&mut self, consonant: char) {
        match self.preedit {
            Preedit::Empty => {
                self.preedit = Preedit::Consonant(consonant);
            }
            Preedit::Consonant(_) | Preedit::Vowel(_) => {
                self.commit_preedit();
                self.preedit = Preedit::Consonant(consonant);
            }
            Preedit::Syllable {
                initial,
                medial,
                final_consonant: None,
            } if hangul::is_final(consonant) => {
                self.preedit = Preedit::Syllable {
                    initial,
                    medial,
                    final_consonant: Some(consonant),
                };
            }
            Preedit::Syllable {
                initial,
                medial,
                final_consonant: Some(final_consonant),
            } => {
                if let Some(combined_final) = hangul::combine_final(final_consonant, consonant) {
                    self.preedit = Preedit::Syllable {
                        initial,
                        medial,
                        final_consonant: Some(combined_final),
                    };
                } else {
                    self.commit_preedit();
                    self.preedit = Preedit::Consonant(consonant);
                }
            }
            Preedit::Syllable { .. } => {
                self.commit_preedit();
                self.preedit = Preedit::Consonant(consonant);
            }
        }
    }

    fn input_vowel(&mut self, vowel: char) {
        match self.preedit {
            Preedit::Empty => {
                self.preedit = Preedit::Vowel(vowel);
            }
            Preedit::Vowel(left) => {
                if let Some(combined_medial) = hangul::combine_medial(left, vowel) {
                    self.preedit = Preedit::Vowel(combined_medial);
                } else {
                    self.commit_preedit();
                    self.preedit = Preedit::Vowel(vowel);
                }
            }
            Preedit::Consonant(initial) if hangul::is_initial(initial) => {
                self.preedit = Preedit::Syllable {
                    initial,
                    medial: vowel,
                    final_consonant: None,
                };
            }
            Preedit::Syllable {
                initial,
                medial,
                final_consonant: None,
            } => {
                if let Some(combined_medial) = hangul::combine_medial(medial, vowel) {
                    self.preedit = Preedit::Syllable {
                        initial,
                        medial: combined_medial,
                        final_consonant: None,
                    };
                } else {
                    self.commit_preedit();
                    self.preedit = Preedit::Vowel(vowel);
                }
            }
            Preedit::Syllable {
                initial,
                medial,
                final_consonant: Some(final_consonant),
            } => {
                if let Some((left_final, right_initial)) = hangul::split_final(final_consonant) {
                    self.commit_syllable(initial, medial, Some(left_final));
                    self.preedit = Preedit::Syllable {
                        initial: right_initial,
                        medial: vowel,
                        final_consonant: None,
                    };
                } else if hangul::is_initial(final_consonant) {
                    self.commit_syllable(initial, medial, None);
                    self.preedit = Preedit::Syllable {
                        initial: final_consonant,
                        medial: vowel,
                        final_consonant: None,
                    };
                } else {
                    self.commit_preedit();
                    self.preedit = Preedit::Vowel(vowel);
                }
            }
            Preedit::Consonant(_) => {
                self.commit_preedit();
                self.preedit = Preedit::Vowel(vowel);
            }
        }
    }

    fn commit_preedit(&mut self) {
        self.committed.push_str(&self.composing_text());
        self.preedit = Preedit::Empty;
    }

    fn commit_syllable(&mut self, initial: char, medial: char, final_consonant: Option<char>) {
        self.committed
            .push_str(&Self::syllable_text(initial, medial, final_consonant));
    }

    fn composing_text(&self) -> String {
        match self.preedit {
            Preedit::Empty => String::new(),
            Preedit::Consonant(consonant) => consonant.to_string(),
            Preedit::Vowel(vowel) => vowel.to_string(),
            Preedit::Syllable {
                initial,
                medial,
                final_consonant,
            } => Self::syllable_text(initial, medial, final_consonant),
        }
    }

    fn syllable_text(initial: char, medial: char, final_consonant: Option<char>) -> String {
        hangul::compose_syllable(initial, medial, final_consonant)
            .map(|syllable| syllable.to_string())
            .unwrap_or_else(|| {
                let mut text = format!("{initial}{medial}");
                if let Some(final_consonant) = final_consonant {
                    text.push(final_consonant);
                }
                text
            })
    }

    fn reduce_committed_char(character: char) -> Option<char> {
        if let Some((initial, medial, final_consonant)) = Self::decompose_syllable(character) {
            if let Some(final_consonant) = final_consonant {
                return hangul::compose_syllable(
                    initial,
                    medial,
                    hangul::split_final(final_consonant).map(|(left, _)| left),
                );
            }

            if let Some((left, _)) = hangul::split_medial(medial) {
                return hangul::compose_syllable(initial, left, None).or(Some(initial));
            }

            return Some(initial);
        }

        if hangul::is_medial(character) {
            return hangul::split_medial(character).map(|(left, _)| left);
        }

        None
    }

    fn decompose_syllable(syllable: char) -> Option<(char, char, Option<char>)> {
        const BASE: u32 = 0xAC00;
        const END: u32 = 0xD7A3;
        const FINAL_COUNT: u32 = 28;
        const MEDIAL_COUNT: u32 = 21;
        const INITIALS: [char; 19] = [
            'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ',
            'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
        ];
        const MEDIALS: [char; 21] = [
            'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ',
            'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
        ];
        const FINALS: [char; 27] = [
            'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ',
            'ㅀ', 'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
        ];

        let code = u32::from(syllable);
        if !(BASE..=END).contains(&code) {
            return None;
        }

        let syllable_index = code - BASE;
        let initial_index = syllable_index / (MEDIAL_COUNT * FINAL_COUNT);
        let medial_index = (syllable_index % (MEDIAL_COUNT * FINAL_COUNT)) / FINAL_COUNT;
        let final_index = syllable_index % FINAL_COUNT;

        let initial = INITIALS.get(initial_index as usize).copied()?;
        let medial = MEDIALS.get(medial_index as usize).copied()?;
        let final_consonant = if final_index == 0 {
            None
        } else {
            Some(FINALS.get((final_index - 1) as usize).copied()?)
        };

        Some((initial, medial, final_consonant))
    }
}
