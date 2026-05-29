use serde::Serialize;

use crate::{JamoRole, Layout, hangul};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InputEffect {
    pub committed: String,
    pub composing: String,
    pub handled: bool,
}

#[derive(Debug, Clone)]
pub struct Composer {
    layout: Layout,
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
            preedit: Preedit::Empty,
        }
    }

    pub fn input_key(&mut self, key: &str, shift: bool) -> InputEffect {
        let Some(mapping) = self.layout.lookup(key, shift) else {
            return self.flush_with_handled(false);
        };
        let jamo = mapping.jamo();
        let role = mapping.role();
        let mut committed = String::new();

        match role {
            JamoRole::Auto => {
                if hangul::is_medial(jamo) {
                    self.input_vowel(jamo, &mut committed);
                } else {
                    self.input_consonant(jamo, &mut committed);
                }
            }
            JamoRole::Initial => self.input_initial_consonant(jamo, &mut committed),
            JamoRole::Final => self.input_final_consonant(jamo, &mut committed),
            JamoRole::Medial => self.input_vowel(jamo, &mut committed),
        }

        self.effect(committed, true)
    }

    pub fn cancel(&mut self) -> InputEffect {
        if self.preedit == Preedit::Empty {
            return Self::empty_effect(false);
        }
        self.preedit = Preedit::Empty;
        Self::empty_effect(true)
    }

    pub fn flush(&mut self) -> InputEffect {
        self.flush_with_handled(true)
    }

    pub fn backspace(&mut self) -> InputEffect {
        match self.preedit {
            Preedit::Empty => {
                return Self::empty_effect(false);
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

        self.effect(String::new(), true)
    }

    fn flush_with_handled(&mut self, handled: bool) -> InputEffect {
        if self.preedit == Preedit::Empty {
            return Self::empty_effect(false);
        }

        let committed = self.composing_text();
        self.preedit = Preedit::Empty;
        self.effect(committed, handled)
    }

    fn effect(&self, committed: String, handled: bool) -> InputEffect {
        InputEffect {
            committed,
            composing: self.composing_text(),
            handled,
        }
    }

    fn empty_effect(handled: bool) -> InputEffect {
        InputEffect {
            committed: String::new(),
            composing: String::new(),
            handled,
        }
    }

    fn input_consonant(&mut self, consonant: char, committed: &mut String) {
        match self.preedit {
            Preedit::Empty => {
                self.preedit = Preedit::Consonant(consonant);
            }
            Preedit::Consonant(_) | Preedit::Vowel(_) => {
                self.commit_preedit(committed);
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
                    self.commit_preedit(committed);
                    self.preedit = Preedit::Consonant(consonant);
                }
            }
            Preedit::Syllable { .. } => {
                self.commit_preedit(committed);
                self.preedit = Preedit::Consonant(consonant);
            }
        }
    }

    fn input_initial_consonant(&mut self, consonant: char, committed: &mut String) {
        if self.preedit != Preedit::Empty {
            self.commit_preedit(committed);
        }

        self.preedit = Preedit::Consonant(consonant);
    }

    fn input_final_consonant(&mut self, consonant: char, committed: &mut String) {
        match self.preedit {
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
                    self.commit_preedit(committed);
                    self.preedit = Preedit::Consonant(consonant);
                }
            }
            _ => {
                if self.preedit != Preedit::Empty {
                    self.commit_preedit(committed);
                }

                self.preedit = Preedit::Consonant(consonant);
            }
        }
    }

    fn input_vowel(&mut self, vowel: char, committed: &mut String) {
        match self.preedit {
            Preedit::Empty => {
                self.preedit = Preedit::Vowel(vowel);
            }
            Preedit::Vowel(left) => {
                if let Some(combined_medial) = hangul::combine_medial(left, vowel) {
                    self.preedit = Preedit::Vowel(combined_medial);
                } else {
                    self.commit_preedit(committed);
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
                    self.commit_preedit(committed);
                    self.preedit = Preedit::Vowel(vowel);
                }
            }
            Preedit::Syllable {
                initial,
                medial,
                final_consonant: Some(final_consonant),
            } => {
                if let Some((left_final, right_initial)) = hangul::split_final(final_consonant) {
                    self.commit_syllable(initial, medial, Some(left_final), committed);
                    self.preedit = Preedit::Syllable {
                        initial: right_initial,
                        medial: vowel,
                        final_consonant: None,
                    };
                } else if hangul::is_initial(final_consonant) {
                    self.commit_syllable(initial, medial, None, committed);
                    self.preedit = Preedit::Syllable {
                        initial: final_consonant,
                        medial: vowel,
                        final_consonant: None,
                    };
                } else {
                    self.commit_preedit(committed);
                    self.preedit = Preedit::Vowel(vowel);
                }
            }
            Preedit::Consonant(_) => {
                self.commit_preedit(committed);
                self.preedit = Preedit::Vowel(vowel);
            }
        }
    }

    fn commit_preedit(&mut self, committed: &mut String) {
        committed.push_str(&self.composing_text());
        self.preedit = Preedit::Empty;
    }

    fn commit_syllable(
        &mut self,
        initial: char,
        medial: char,
        final_consonant: Option<char>,
        committed: &mut String,
    ) {
        committed.push_str(&Self::syllable_text(initial, medial, final_consonant));
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
}
