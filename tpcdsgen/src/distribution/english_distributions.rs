use crate::distribution::string_values_distribution::StringValuesDistribution;
use crate::error::Result;
use crate::random::stream::RandomNumberStream;
use std::sync::OnceLock;

static ADJECTIVES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static ADVERBS_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static ARTICLES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static AUXILIARIES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static PREPOSITIONS_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static NOUNS_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static SENTENCES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static SYLLABLES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static TERMINATORS_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();
static VERBS_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();

pub fn pick_random_adjective(stream: &mut RandomNumberStream) -> Result<&'static str> {
    let dist = ADJECTIVES_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("adjectives.dst", 1, 1)
            .expect("Failed to load adjectives distribution")
    });

    dist.pick_random_value(0, 0, stream)
}

pub fn pick_random_adverb(stream: &mut RandomNumberStream) -> Result<&'static str> {
    let dist = ADVERBS_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("adverbs.dst", 1, 1)
            .expect("Failed to load adverbs distribution")
    });

    dist.pick_random_value(0, 0, stream)
}

pub fn pick_random_article(stream: &mut RandomNumberStream) -> Result<&'static str> {
    let dist = ARTICLES_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("articles.dst", 1, 1)
            .expect("Failed to load articles distribution")
    });

    dist.pick_random_value(0, 0, stream)
}

pub fn pick_random_auxiliary(stream: &mut RandomNumberStream) -> Result<&'static str> {
    let dist = AUXILIARIES_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("auxiliaries.dst", 1, 1)
            .expect("Failed to load auxiliaries distribution")
    });

    dist.pick_random_value(0, 0, stream)
}

pub fn pick_random_preposition(stream: &mut RandomNumberStream) -> Result<&'static str> {
    let dist = PREPOSITIONS_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("prepositions.dst", 1, 1)
            .expect("Failed to load prepositions distribution")
    });

    dist.pick_random_value(0, 0, stream)
}

pub fn pick_random_noun(stream: &mut RandomNumberStream) -> Result<&'static str> {
    let dist = NOUNS_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("nouns.dst", 1, 1)
            .expect("Failed to load nouns distribution")
    });

    dist.pick_random_value(0, 0, stream)
}

pub fn pick_random_sentence(stream: &mut RandomNumberStream) -> Result<&'static str> {
    let dist = SENTENCES_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("sentences.dst", 1, 1)
            .expect("Failed to load sentences distribution")
    });

    dist.pick_random_value(0, 0, stream)
}

pub fn pick_random_terminator(stream: &mut RandomNumberStream) -> Result<&'static str> {
    let dist = TERMINATORS_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("terminators.dst", 1, 1)
            .expect("Failed to load terminators distribution")
    });

    dist.pick_random_value(0, 0, stream)
}

pub fn pick_random_verb(stream: &mut RandomNumberStream) -> Result<&'static str> {
    let dist = VERBS_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("verbs.dst", 1, 1)
            .expect("Failed to load verbs distribution")
    });

    dist.pick_random_value(0, 0, stream)
}

pub fn get_syllables_distribution() -> &'static StringValuesDistribution {
    SYLLABLES_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("syllables.dst", 1, 1)
            .expect("Failed to load syllables distribution")
    })
}
