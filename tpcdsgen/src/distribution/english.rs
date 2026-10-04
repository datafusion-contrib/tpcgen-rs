use crate::distribution::{Distribution, StringValuesDistribution};
use crate::error::Result;
use crate::random::RandomNumberStream;
use std::sync::OnceLock;

/// English language distributions for text generation (EnglishDistributions)
pub struct EnglishDistributions;

impl EnglishDistributions {
    /// Get adjectives distribution (lazy initialized)
    fn adjectives_distribution() -> &'static StringValuesDistribution {
        static ADJECTIVES: OnceLock<StringValuesDistribution> = OnceLock::new();
        ADJECTIVES.get_or_init(|| {
            // Sample adjectives from the Java .dst file with approximate weights
            let data = &[
                ("good", 1200),
                ("new", 1100),
                ("first", 900),
                ("last", 800),
                ("long", 600),
                ("great", 550),
                ("little", 500),
                ("own", 450),
                ("other", 400),
                ("old", 380),
                ("right", 350),
                ("big", 320),
                ("high", 300),
                ("different", 280),
                ("small", 260),
                ("large", 240),
                ("next", 220),
                ("early", 200),
                ("young", 180),
                ("important", 160),
                ("few", 140),
                ("public", 120),
                ("bad", 100),
                ("same", 90),
                ("able", 80),
            ];

            StringValuesDistribution::from_embedded_data(data)
                .expect("Failed to create adjectives distribution")
        })
    }

    /// Get adverbs distribution (lazy initialized)
    fn adverbs_distribution() -> &'static StringValuesDistribution {
        static ADVERBS: OnceLock<StringValuesDistribution> = OnceLock::new();
        ADVERBS.get_or_init(|| {
            // Sample adverbs from the Java .dst file with approximate weights
            let data = &[
                ("then", 619),
                ("more", 615),
                ("also", 592),
                ("so", 540),
                ("now", 538),
                ("only", 524),
                ("as", 436),
                ("very", 431),
                ("just", 426),
                ("even", 329),
                ("still", 318),
                ("too", 316),
                ("however", 280),
                ("well", 275),
                ("here", 270),
                ("again", 250),
                ("never", 240),
                ("always", 230),
                ("often", 220),
                ("sometimes", 200),
                ("rather", 180),
                ("quite", 160),
                ("almost", 140),
                ("perhaps", 120),
                ("certainly", 100),
            ];

            StringValuesDistribution::from_embedded_data(data)
                .expect("Failed to create adverbs distribution")
        })
    }

    /// Get articles distribution (lazy initialized)
    fn articles_distribution() -> &'static StringValuesDistribution {
        static ARTICLES: OnceLock<StringValuesDistribution> = OnceLock::new();
        ARTICLES.get_or_init(|| {
            let data = &[("the", 2000), ("a", 800), ("an", 200)];

            StringValuesDistribution::from_embedded_data(data)
                .expect("Failed to create articles distribution")
        })
    }

    /// Get auxiliary verbs distribution (lazy initialized)
    fn auxiliaries_distribution() -> &'static StringValuesDistribution {
        static AUXILIARIES: OnceLock<StringValuesDistribution> = OnceLock::new();
        AUXILIARIES.get_or_init(|| {
            let data = &[
                ("is", 500),
                ("was", 400),
                ("are", 350),
                ("were", 300),
                ("be", 250),
                ("been", 200),
                ("being", 150),
                ("have", 400),
                ("has", 350),
                ("had", 300),
                ("will", 250),
                ("would", 200),
                ("can", 180),
                ("could", 160),
                ("should", 140),
                ("may", 120),
                ("might", 100),
                ("must", 80),
                ("do", 300),
                ("does", 250),
                ("did", 200),
            ];

            StringValuesDistribution::from_embedded_data(data)
                .expect("Failed to create auxiliaries distribution")
        })
    }

    /// Get nouns distribution (lazy initialized)
    fn nouns_distribution() -> &'static StringValuesDistribution {
        static NOUNS: OnceLock<StringValuesDistribution> = OnceLock::new();
        NOUNS.get_or_init(|| {
            // Sample nouns with business/commerce focus for TPC-DS
            let data = &[
                ("time", 900),
                ("person", 800),
                ("year", 750),
                ("way", 700),
                ("day", 650),
                ("thing", 600),
                ("man", 550),
                ("world", 500),
                ("life", 450),
                ("hand", 400),
                ("part", 380),
                ("child", 360),
                ("eye", 340),
                ("woman", 320),
                ("place", 300),
                ("work", 280),
                ("week", 260),
                ("case", 240),
                ("point", 220),
                ("government", 200),
                ("company", 190),
                ("number", 180),
                ("group", 170),
                ("problem", 160),
                ("fact", 150),
                ("business", 140),
                ("service", 130),
                ("product", 120),
                ("customer", 110),
                ("order", 100),
                ("price", 90),
                ("sale", 80),
                ("market", 70),
                ("store", 60),
                ("item", 50),
            ];

            StringValuesDistribution::from_embedded_data(data)
                .expect("Failed to create nouns distribution")
        })
    }

    /// Get prepositions distribution (lazy initialized)
    fn prepositions_distribution() -> &'static StringValuesDistribution {
        static PREPOSITIONS: OnceLock<StringValuesDistribution> = OnceLock::new();
        PREPOSITIONS.get_or_init(|| {
            let data = &[
                ("of", 1500),
                ("to", 1200),
                ("in", 1000),
                ("for", 800),
                ("with", 600),
                ("on", 550),
                ("by", 500),
                ("from", 450),
                ("about", 400),
                ("at", 380),
                ("through", 350),
                ("during", 320),
                ("before", 300),
                ("after", 280),
                ("above", 260),
                ("below", 240),
                ("between", 220),
                ("among", 200),
                ("against", 180),
                ("without", 160),
                ("within", 140),
                ("throughout", 120),
                ("upon", 100),
                ("beneath", 80),
                ("beside", 60),
            ];

            StringValuesDistribution::from_embedded_data(data)
                .expect("Failed to create prepositions distribution")
        })
    }

    /// Get verbs distribution (lazy initialized)
    fn verbs_distribution() -> &'static StringValuesDistribution {
        static VERBS: OnceLock<StringValuesDistribution> = OnceLock::new();
        VERBS.get_or_init(|| {
            let data = &[
                ("be", 1000),
                ("have", 800),
                ("do", 600),
                ("say", 500),
                ("get", 450),
                ("make", 400),
                ("go", 380),
                ("know", 360),
                ("take", 340),
                ("see", 320),
                ("come", 300),
                ("think", 280),
                ("look", 260),
                ("want", 240),
                ("give", 220),
                ("use", 200),
                ("find", 180),
                ("tell", 160),
                ("ask", 140),
                ("work", 130),
                ("seem", 120),
                ("feel", 110),
                ("try", 100),
                ("leave", 90),
                ("call", 80),
                ("buy", 70),
                ("sell", 60),
                ("order", 50),
                ("ship", 40),
                ("return", 30),
            ];

            StringValuesDistribution::from_embedded_data(data)
                .expect("Failed to create verbs distribution")
        })
    }

    /// Get sentence terminators distribution (lazy initialized)
    fn terminators_distribution() -> &'static StringValuesDistribution {
        static TERMINATORS: OnceLock<StringValuesDistribution> = OnceLock::new();
        TERMINATORS.get_or_init(|| {
            let data = &[(".", 70), ("!", 20), ("?", 10)];

            StringValuesDistribution::from_embedded_data(data)
                .expect("Failed to create terminators distribution")
        })
    }

    /// Get sentences distribution for complete phrases
    fn sentences_distribution() -> &'static StringValuesDistribution {
        static SENTENCES: OnceLock<StringValuesDistribution> = OnceLock::new();
        SENTENCES.get_or_init(|| {
            // Pre-built sentences for variety
            let data = &[
                ("Great product quality", 100),
                ("Excellent customer service", 95),
                ("Fast shipping and delivery", 90),
                ("Good value for money", 85),
                ("Highly recommended item", 80),
                ("Perfect for everyday use", 75),
                ("Outstanding performance", 70),
                ("Superior build quality", 65),
                ("Exceptional customer experience", 60),
                ("Reliable and durable", 55),
                ("Easy to use interface", 50),
                ("Professional grade equipment", 45),
                ("Innovative design features", 40),
                ("Competitive pricing available", 35),
                ("Premium quality materials", 30),
            ];

            StringValuesDistribution::from_embedded_data(data)
                .expect("Failed to create sentences distribution")
        })
    }

    // Public API methods (matching Java interface)

    pub fn pick_random_adjective(stream: &mut RandomNumberStream) -> Result<String> {
        Self::adjectives_distribution().pick_random_value(0, 0, stream)
    }

    pub fn pick_random_adverb(stream: &mut RandomNumberStream) -> Result<String> {
        Self::adverbs_distribution().pick_random_value(0, 0, stream)
    }

    pub fn pick_random_article(stream: &mut RandomNumberStream) -> Result<String> {
        Self::articles_distribution().pick_random_value(0, 0, stream)
    }

    pub fn pick_random_auxiliary(stream: &mut RandomNumberStream) -> Result<String> {
        Self::auxiliaries_distribution().pick_random_value(0, 0, stream)
    }

    pub fn pick_random_noun(stream: &mut RandomNumberStream) -> Result<String> {
        Self::nouns_distribution().pick_random_value(0, 0, stream)
    }

    pub fn pick_random_preposition(stream: &mut RandomNumberStream) -> Result<String> {
        Self::prepositions_distribution().pick_random_value(0, 0, stream)
    }

    pub fn pick_random_verb(stream: &mut RandomNumberStream) -> Result<String> {
        Self::verbs_distribution().pick_random_value(0, 0, stream)
    }

    pub fn pick_random_terminator(stream: &mut RandomNumberStream) -> Result<String> {
        Self::terminators_distribution().pick_random_value(0, 0, stream)
    }

    pub fn pick_random_sentence(stream: &mut RandomNumberStream) -> Result<String> {
        Self::sentences_distribution().pick_random_value(0, 0, stream)
    }

    /// Generate a random phrase by combining words
    pub fn generate_random_phrase(
        stream: &mut RandomNumberStream,
        word_count: usize,
    ) -> Result<String> {
        if word_count == 0 {
            return Ok(String::new());
        }

        let mut words = Vec::new();

        for i in 0..word_count {
            let word = match i % 4 {
                0 => Self::pick_random_article(stream)?,
                1 => Self::pick_random_adjective(stream)?,
                2 => Self::pick_random_noun(stream)?,
                3 => Self::pick_random_verb(stream)?,
                _ => Self::pick_random_noun(stream)?,
            };
            words.push(word);
        }

        Ok(words.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStream;

    #[test]
    fn test_pick_random_adjective() {
        let mut stream = RandomNumberStream::new(1);
        let adjective = EnglishDistributions::pick_random_adjective(&mut stream).unwrap();
        assert!(!adjective.is_empty());
        println!("Random adjective: {}", adjective);
    }

    #[test]
    fn test_pick_random_adverb() {
        let mut stream = RandomNumberStream::new(1);
        let adverb = EnglishDistributions::pick_random_adverb(&mut stream).unwrap();
        assert!(!adverb.is_empty());
        println!("Random adverb: {}", adverb);
    }

    #[test]
    fn test_pick_random_article() {
        let mut stream = RandomNumberStream::new(1);
        let article = EnglishDistributions::pick_random_article(&mut stream).unwrap();
        assert!(article == "the" || article == "a" || article == "an");
        println!("Random article: {}", article);
    }

    #[test]
    fn test_pick_random_noun() {
        let mut stream = RandomNumberStream::new(1);
        let noun = EnglishDistributions::pick_random_noun(&mut stream).unwrap();
        assert!(!noun.is_empty());
        println!("Random noun: {}", noun);
    }

    #[test]
    fn test_pick_random_verb() {
        let mut stream = RandomNumberStream::new(1);
        let verb = EnglishDistributions::pick_random_verb(&mut stream).unwrap();
        assert!(!verb.is_empty());
        println!("Random verb: {}", verb);
    }

    #[test]
    fn test_pick_random_sentence() {
        let mut stream = RandomNumberStream::new(1);
        let sentence = EnglishDistributions::pick_random_sentence(&mut stream).unwrap();
        assert!(!sentence.is_empty());
        println!("Random sentence: {}", sentence);
    }

    #[test]
    fn test_generate_random_phrase() {
        let mut stream = RandomNumberStream::new(1);

        let phrase = EnglishDistributions::generate_random_phrase(&mut stream, 4).unwrap();
        assert!(!phrase.is_empty());
        assert!(phrase.contains(' ')); // Should have spaces between words
        println!("Random phrase: {}", phrase);

        // Test empty phrase
        let empty_phrase = EnglishDistributions::generate_random_phrase(&mut stream, 0).unwrap();
        assert!(empty_phrase.is_empty());
    }

    #[test]
    fn test_deterministic_selection() {
        // Same seed should produce same results
        let mut stream1 = RandomNumberStream::new_with_column(42, 1);
        let mut stream2 = RandomNumberStream::new_with_column(42, 1);

        let word1 = EnglishDistributions::pick_random_noun(&mut stream1).unwrap();
        let word2 = EnglishDistributions::pick_random_noun(&mut stream2).unwrap();

        assert_eq!(word1, word2);
    }

    #[test]
    fn test_all_distributions_work() {
        let mut stream = RandomNumberStream::new(1);

        // Test that all distribution methods work without panicking
        assert!(EnglishDistributions::pick_random_adjective(&mut stream).is_ok());
        assert!(EnglishDistributions::pick_random_adverb(&mut stream).is_ok());
        assert!(EnglishDistributions::pick_random_article(&mut stream).is_ok());
        assert!(EnglishDistributions::pick_random_auxiliary(&mut stream).is_ok());
        assert!(EnglishDistributions::pick_random_noun(&mut stream).is_ok());
        assert!(EnglishDistributions::pick_random_preposition(&mut stream).is_ok());
        assert!(EnglishDistributions::pick_random_verb(&mut stream).is_ok());
        assert!(EnglishDistributions::pick_random_terminator(&mut stream).is_ok());
        assert!(EnglishDistributions::pick_random_sentence(&mut stream).is_ok());
    }

    #[test]
    fn test_weighted_distribution_variety() {
        let mut stream = RandomNumberStream::new(1);

        // Generate multiple words and ensure we get variety
        let mut words = std::collections::HashSet::new();
        for _ in 0..20 {
            let word = EnglishDistributions::pick_random_noun(&mut stream).unwrap();
            words.insert(word);
        }

        // Should have some variety (not just one word)
        assert!(words.len() > 1);
    }
}
