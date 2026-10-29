//! Implementation of text pool and text generation.
//!
//! Most of this code has been ported from the Trino TPC-H generator
//! implementation. The original code can be found in the following link:
//!
//! <https://github.com/trinodb/tpch/blob/master/src/main/java/io/trino/tpch/TextPool.java>

use crate::distribution::{Distribution, Distributions};
use crate::random::RowRandomInt;
use std::sync::OnceLock;

/// Pool of random text that follows TPC-H grammar.
#[derive(Debug, Clone)]
pub struct TextPool {
    /// Bytes making up the text pool, exact size.
    text: Vec<u8>,
}

/// The default global text pool is lazily initialized once and shared across
/// all the table generators.
static DEFAULT_TEXT_POOL: OnceLock<TextPool> = OnceLock::new();

impl TextPool {
    /// Default text pool size.
    const DEFAULT_TEXT_POOL_SIZE: i32 = 300 * 1024 * 1024;
    /// Maximum length of a sentence in the text.
    const MAX_SENTENCE_LENGTH: i32 = 256;

    /// Returns the default text pool or initializes for the first time if
    /// that's not already the case.
    pub fn get_or_init_default() -> &'static Self {
        DEFAULT_TEXT_POOL.get_or_init(|| {
            Self::new(
                Self::DEFAULT_TEXT_POOL_SIZE,
                Distributions::static_default(),
            )
        })
    }

    /// Returns a new text pool with a predefined size and set of distributions.
    pub fn new(size: i32, distributions: &Distributions) -> Self {
        let size = size as usize;
        let mut rng = RowRandomInt::new(933588178, i32::MAX);
        let grammar = distributions.grammar().weighted_values();
        let verb_phrase = distributions.verb_phrase().weighted_values();
        let noun_phrase = distributions.noun_phrase().weighted_values();
        let words = Words {
            prepositions: WordTable::new(distributions.prepositions()),
            terminators: WordTable::new(distributions.terminators()),
            adverbs: WordTable::new(distributions.adverbs()),
            verbs: WordTable::new(distributions.verbs()),
            auxiliaries: WordTable::new(distributions.auxiliaries()),
            articles: WordTable::new(distributions.articles()),
            adjectives: WordTable::new(distributions.adjectives()),
            nouns: WordTable::new(distributions.nouns()),
        };

        // A sentence starts only while the text is shorter than `size`, and
        // is at most MAX_SENTENCE_LENGTH bytes. Each word store writes a whole
        // slot, so leave room for one more slot after that.
        let mut out = TextWriter {
            buf: vec![0; size + Self::MAX_SENTENCE_LENGTH as usize + WORD_SLOT],
            len: 0,
        };
        while out.len < size {
            let syntax = pick(grammar, &mut rng).as_bytes();
            for &c in syntax.iter().step_by(2) {
                match c {
                    b'V' => {
                        let syntax = pick(verb_phrase, &mut rng).as_bytes();
                        for &c in syntax.iter().step_by(2) {
                            let table = match c {
                                b'D' => &words.adverbs,
                                b'V' => &words.verbs,
                                b'X' => &words.auxiliaries,
                                c => panic!("Unknown token '{}'", c as char),
                            };
                            out.push_word_and_space(table, &mut rng);
                        }
                    }
                    b'N' => Self::generate_noun_phrase(noun_phrase, &words, &mut out, &mut rng),
                    b'P' => {
                        out.push_word(&words.prepositions, &mut rng);
                        out.push_bytes(b" the ");
                        Self::generate_noun_phrase(noun_phrase, &words, &mut out, &mut rng);
                    }
                    b'T' => {
                        out.pop();
                        out.push_word(&words.terminators, &mut rng);
                    }
                    c => panic!("Unknown token '{}'", c as char),
                };

                if out.last() != b' ' {
                    out.push_bytes(b" ");
                }
            }
        }

        let mut text = out.buf;
        text.truncate(size);
        Self { text }
    }

    /// Returns the text pool size.
    pub fn size(&self) -> i32 {
        // Cast is fine since we truncated the bytes to `size` in `new`, which
        // is an i32.
        self.text.len() as i32
    }

    /// Returns a chunk of text from the pool
    ///
    /// Returns the text from the pool between the given begin and end indices.
    pub fn text(&self, begin: i32, end: i32) -> &str {
        // get slice of bytes (note this also does bounds checks)
        let result: &[u8] = &self.text[begin as usize..end as usize];
        // Safety: text pool contains only ASCII
        unsafe { std::str::from_utf8_unchecked(result) }
    }

    fn generate_noun_phrase(
        noun_phrase: &[&str],
        words: &Words,
        out: &mut TextWriter,
        rng: &mut RowRandomInt,
    ) {
        let syntax = pick(noun_phrase, rng).as_bytes();
        for &c in syntax {
            let table = match c {
                b'A' => &words.articles,
                b'J' => &words.adjectives,
                b'D' => &words.adverbs,
                b'N' => &words.nouns,
                b',' => {
                    out.pop();
                    out.push_bytes(b", ");
                    continue;
                }
                b' ' => continue,
                c => panic!("Unknown token '{}'", c as char),
            };
            out.push_word_and_space(table, rng);
        }
    }
}

/// Size of a [`WordTable`] slot. All the words in the TPC-H grammar
/// distributions are shorter than this.
const WORD_SLOT: usize = 32;

/// The words of a [`Distribution`], each in a fixed-size slot padded with
/// spaces, repeated by weight like [`Distribution::weighted_values`].
///
/// Copying a whole slot is a single fixed-size store, which is much faster
/// than copying a word of variable length.
struct WordTable {
    slots: Vec<([u8; WORD_SLOT], usize)>,
}

impl WordTable {
    fn new(distribution: &Distribution) -> Self {
        let slots = distribution
            .weighted_values()
            .iter()
            .map(|word| {
                let word = word.as_bytes();
                assert!(word.len() < WORD_SLOT, "word too long: {word:?}");
                let mut slot = [b' '; WORD_SLOT];
                slot[..word.len()].copy_from_slice(word);
                (slot, word.len())
            })
            .collect();
        Self { slots }
    }

    /// Picks a random word, as [`Distribution::random_value`] does. Returns
    /// its slot and its length.
    fn random_word(&self, rng: &mut RowRandomInt) -> (&[u8; WORD_SLOT], usize) {
        let (slot, len) = &self.slots[rng.next_int(0, self.slots.len() as i32 - 1) as usize];
        (slot, *len)
    }
}

/// The word tables used by the TPC-H text grammar
struct Words {
    prepositions: WordTable,
    terminators: WordTable,
    adverbs: WordTable,
    verbs: WordTable,
    auxiliaries: WordTable,
    articles: WordTable,
    adjectives: WordTable,
    nouns: WordTable,
}

/// Picks a random value from the weighted values of a distribution, as
/// [`Distribution::random_value`] does.
fn pick<'a>(weighted_values: &[&'a str], rng: &mut RowRandomInt) -> &'a str {
    weighted_values[rng.next_int(0, weighted_values.len() as i32 - 1) as usize]
}

/// Writes text to a buffer that has room for a whole [`WordTable`] slot
/// after the text.
struct TextWriter {
    buf: Vec<u8>,
    len: usize,
}

impl TextWriter {
    /// Appends a random word from `table`.
    #[inline(always)]
    fn push_word(&mut self, table: &WordTable, rng: &mut RowRandomInt) {
        let (slot, len) = table.random_word(rng);
        self.buf[self.len..self.len + WORD_SLOT].copy_from_slice(slot);
        self.len += len;
    }

    /// Appends a random word from `table`, followed by a space.
    #[inline(always)]
    fn push_word_and_space(&mut self, table: &WordTable, rng: &mut RowRandomInt) {
        let (slot, len) = table.random_word(rng);
        // the slot is padded with spaces, so the space is already there
        self.buf[self.len..self.len + WORD_SLOT].copy_from_slice(slot);
        self.len += len + 1;
    }

    #[inline(always)]
    fn push_bytes(&mut self, bytes: &[u8]) {
        self.buf[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
    }

    #[inline(always)]
    fn pop(&mut self) {
        self.len -= 1;
    }

    #[inline(always)]
    fn last(&self) -> u8 {
        self.buf[self.len - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The original implementation of [`TextPool::new`], which follows the
    /// Trino implementation closely. The fast implementation must produce
    /// the same text.
    struct Reference;

    impl Reference {
        fn new_text(size: i32, distributions: &Distributions) -> Vec<u8> {
            let mut rng = RowRandomInt::new(933588178, i32::MAX);
            let mut text_bytes = Vec::with_capacity(size as usize + 256);
            while text_bytes.len() < size as usize {
                Self::generate_sentence(distributions, &mut text_bytes, &mut rng);
            }
            text_bytes.truncate(size as usize);
            text_bytes
        }

        fn generate_sentence(
            distributions: &Distributions,
            output: &mut Vec<u8>,
            random: &mut RowRandomInt,
        ) {
            let syntax = distributions.grammar().random_value(random);
            let max_length = syntax.len();

            for c in syntax.chars().take(max_length).step_by(2) {
                match c {
                    'V' => Self::generate_verb_phrase(distributions, output, random),
                    'N' => Self::generate_noun_phrase(distributions, output, random),
                    'P' => {
                        let preposition = distributions.prepositions().random_value(random);
                        output.extend_from_slice(preposition.as_bytes());
                        output.extend_from_slice(b" the ");
                        Self::generate_noun_phrase(distributions, output, random);
                    }
                    'T' => {
                        output.pop().expect("at least one byte");
                        let terminator = distributions.terminators().random_value(random);
                        output.extend_from_slice(terminator.as_bytes());
                    }
                    c => panic!("Unknown token '{}'", c),
                };

                let last = output.last().copied().expect("at least one byte");
                if last != b' ' {
                    output.push(b' ');
                }
            }
        }

        fn generate_verb_phrase(
            distributions: &Distributions,
            output: &mut Vec<u8>,
            random: &mut RowRandomInt,
        ) {
            let syntax = distributions.verb_phrase().random_value(random);
            let max_length = syntax.len();

            for c in syntax.chars().take(max_length).step_by(2) {
                let source = match c {
                    'D' => distributions.adverbs(),
                    'V' => distributions.verbs(),
                    'X' => distributions.auxiliaries(),
                    c => panic!("Unknown token '{}'", c),
                };

                // pick a random word
                let word = source.random_value(random);
                output.extend_from_slice(word.as_bytes());

                // add a space
                output.push(b' ');
            }
        }

        fn generate_noun_phrase(
            distributions: &Distributions,
            output: &mut Vec<u8>,
            random: &mut RowRandomInt,
        ) {
            let syntax = distributions.noun_phrase().random_value(random);
            let max_length = syntax.len();

            for c in syntax.chars().take(max_length) {
                let source = match c {
                    'A' => distributions.articles(),
                    'J' => distributions.adjectives(),
                    'D' => distributions.adverbs(),
                    'N' => distributions.nouns(),
                    ',' => {
                        output.pop().expect("at least one byte");
                        output.extend_from_slice(b", ");
                        continue;
                    }
                    ' ' => continue,
                    c => panic!("Unknown token '{}'", c),
                };

                // pick a random word
                let word = source.random_value(random);
                output.extend_from_slice(word.as_bytes());
                output.push(b' ');
            }
        }
    }

    #[test]
    fn text_pool_matches_reference() {
        let distributions = Distributions::static_default();
        for size in [0, 1, 255, 256, 257, 10_000, 4 * 1024 * 1024 + 17] {
            let pool = TextPool::new(size, distributions);
            assert_eq!(
                pool.text,
                Reference::new_text(size, distributions),
                "size {size}"
            );
        }
    }
}
