use crate::random::RowRandomInt;
use std::{
    io::{self},
    sync::LazyLock,
};

/// TPC-H distributions seed file.
pub(crate) const DISTS_SEED: &str = include_str!("dists.dss");

/// Distribution represents a weighted collection of string values from the TPC-H specification.
/// It provides methods to access values by index or randomly based on their weights.
#[derive(Debug, Clone, Default)]
pub struct Distribution {
    name: &'static str,
    values: Vec<&'static str>,
    weights: Vec<i32>,
    /// Weighted text values generated from the distribution.
    ///
    /// If the table this distribution is for isn't actually a valid
    /// distribution (e.g. 'nation'), then this vec will be empty, and
    /// `max_weight` set to -1.
    distribution: Vec<&'static str>,
    max_weight: i32,
}

impl Distribution {
    /// Creates a new Distribution with the given name and weighted values.
    pub fn new(name: &'static str, distribution: Vec<(&'static str, i32)>) -> Self {
        let mut weights = vec![0; distribution.len()];

        let mut running_weight = 0;
        let mut is_valid_distribution = true;

        // Process each value and its weight
        for (index, (_, weight)) in distribution.iter().enumerate() {
            running_weight += weight;
            weights[index] = running_weight;

            // A valid distribution requires all weights to be positive
            is_valid_distribution &= *weight > 0;
        }

        // Only create the full distribution array for valid distributions
        // "nations" is a special case that's not a valid distribution
        let (distribution_array, max_weight) = if is_valid_distribution {
            let max = weights[weights.len() - 1];
            let mut dist = vec![""; max as usize];

            let mut index = 0;
            for (value, weight) in &distribution {
                for _ in 0..*weight {
                    dist[index] = value;
                    index += 1;
                }
            }

            (dist, max)
        } else {
            (Vec::new(), -1)
        };

        let values = distribution.into_iter().map(|(tok, _)| tok).collect();

        Distribution {
            name,
            values,
            weights,
            distribution: distribution_array,
            max_weight,
        }
    }

    /// Returns the distribution name.
    pub fn name(&self) -> &str {
        self.name
    }

    /// Gets a value at the specified index.
    pub fn get_value(&self, index: usize) -> &str {
        self.values[index]
    }

    /// Gets all values in this distribution.
    pub fn get_values(&self) -> &[&'static str] {
        &self.values
    }

    /// Gets the cumulative weight at the specified index.
    pub fn get_weight(&self, index: usize) -> i32 {
        self.weights[index]
    }

    /// Gets the number of distinct values in this distribution.
    pub fn size(&self) -> usize {
        self.values.len()
    }

    /// Gets all values repeated by their weight, in the order
    /// [`Self::random_value`] picks from: `random_value` returns
    /// `weighted_values()[random.next_int(0, weighted_values().len() - 1)]`.
    pub(crate) fn weighted_values(&self) -> &[&'static str] {
        &self.distribution
    }

    /// Gets a random value from this distribution using the provided random number.
    pub fn random_value(&self, random: &mut RowRandomInt) -> &str {
        debug_assert!(
            !self.distribution.is_empty(),
            "Not a valid distribution, cannot get a random value"
        );
        let random_value = random.next_int(0, self.max_weight - 1);
        self.distribution[random_value as usize]
    }

    /// Loads a single distribution until its END marker.
    fn load_distribution<I>(
        lines: &mut std::iter::Peekable<I>,
        name: &'static str,
    ) -> io::Result<Self>
    where
        I: Iterator<Item = &'static str>,
    {
        // (Token, Weight) pairs within a distribution.
        let mut members: Vec<(&'static str, i32)> = Vec::new();
        let mut _count = -1;

        for line in lines.by_ref() {
            if Self::is_end(line) {
                let distribution = Distribution::new(name, members);
                return Ok(distribution);
            }

            let parts: Vec<&str> = line.split("|").collect::<Vec<_>>();
            if parts.len() < 2 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid distribution line format: {}", line),
                ));
            }

            let value = parts[0];
            let weight = match parts[1].trim().parse::<i32>() {
                Ok(w) => w,
                Err(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Invalid distribution {}: invalid weight on line {}",
                            name, line
                        ),
                    ));
                }
            };

            if value.eq_ignore_ascii_case("count") {
                _count = weight;
            } else {
                members.push((value, weight));
            }
        }

        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid distribution {}: no end statement", name),
        ))
    }

    /// Checks if a line is an END marker.
    fn is_end(line: &str) -> bool {
        let parts: Vec<&str> = line.split_whitespace().collect();
        parts.first().is_some_and(|p| p.eq_ignore_ascii_case("END"))
    }
}

/// Static global instance of the default distributions.
///
/// Initialized once on first access.
static DEFAULT_DISTRIBUTIONS: LazyLock<Distributions> =
    LazyLock::new(|| Distributions::try_load_default().unwrap());

/// Distributions wraps all TPC-H distributions and provides methods to access them.
#[derive(Debug, Clone, Default)]
pub struct Distributions {
    articles: Distribution,
    adjectives: Distribution,
    adverbs: Distribution,
    auxiliaries: Distribution,
    grammar: Distribution,
    category: Distribution,
    market_segments: Distribution,
    nations: Distribution,
    noun_phrase: Distribution,
    nouns: Distribution,
    order_priority: Distribution,
    part_colors: Distribution,
    part_containers: Distribution,
    part_types: Distribution,
    prepositions: Distribution,
    regions: Distribution,
    return_flags: Distribution,
    ship_instructions: Distribution,
    ship_modes: Distribution,
    terminators: Distribution,
    verb_phrase: Distribution,
    verbs: Distribution,
}

impl Distributions {
    pub fn try_load_default() -> io::Result<Self> {
        let lines = DISTS_SEED.split('\n');

        let mut new_self = Self::default();
        for (name, distribution) in Self::load_distributions(lines)? {
            match name {
                "articles" => new_self.articles = distribution,
                "adjectives" => new_self.adjectives = distribution,
                "adverbs" => new_self.adverbs = distribution,
                // P.S: The correct spelling is `auxiliaries` which is what we use.
                "auxillaries" => new_self.auxiliaries = distribution,
                "grammar" => new_self.grammar = distribution,
                "category" => new_self.category = distribution,
                "msegmnt" => new_self.market_segments = distribution,
                "nations" => new_self.nations = distribution,
                "np" => new_self.noun_phrase = distribution,
                "nouns" => new_self.nouns = distribution,
                "o_oprio" => new_self.order_priority = distribution,
                "colors" => new_self.part_colors = distribution,
                "p_cntr" => new_self.part_containers = distribution,
                "p_types" => new_self.part_types = distribution,
                "prepositions" => new_self.prepositions = distribution,
                "regions" => new_self.regions = distribution,
                "rflag" => new_self.return_flags = distribution,
                "instruct" => new_self.ship_instructions = distribution,
                "smode" => new_self.ship_modes = distribution,
                "terminators" => new_self.terminators = distribution,
                "vp" => new_self.verb_phrase = distribution,
                "verbs" => new_self.verbs = distribution,

                // currently unused distributions
                "nations2" | "Q13a" | "Q13b" | "p_names" => {}
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Internal Error: Unknown distribution: {name}"),
                    ));
                }
            }
        }

        Ok(new_self)
    }

    /// Loads distributions from a stream of lines.
    ///
    /// The format is expected to follow the TPC-H specification format where:
    /// - Lines starting with `"#"` are comments
    /// - Distributions start with `"BEGIN <name>"`
    /// - Distribution entries are formatted as `"value|weight"`
    /// - Distributions end with `"END"`
    fn load_distributions<I>(lines: I) -> io::Result<Vec<(&'static str, Distribution)>>
    where
        I: Iterator<Item = &'static str>,
    {
        let mut filtered_lines = lines
            .filter_map(|line| {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    Some(trimmed)
                } else {
                    None
                }
            })
            .peekable();

        let mut distributions = Vec::new();

        while let Some(line) = filtered_lines.next() {
            // This checks if the line has exactly two parts and the first part is "BEGIN"
            let mut part_iter = line.split_whitespace();
            let Some(part0) = part_iter.next() else {
                continue;
            };
            let Some(part1) = part_iter.next() else {
                continue;
            };
            // Check if len != 2
            if part_iter.next().is_some() {
                continue;
            }
            if part0.eq_ignore_ascii_case("BEGIN") {
                let name = part1;
                let distribution = Distribution::load_distribution(&mut filtered_lines, name)?;
                distributions.push((name, distribution));
            }
        }

        Ok(distributions)
    }

    /// Returns a static reference to the default distributions.
    pub fn static_default() -> &'static Distributions {
        &DEFAULT_DISTRIBUTIONS
    }

    /// Returns the `adjectives` distribution.
    pub fn adjectives(&self) -> &Distribution {
        &self.adjectives
    }

    /// Returns the `adverbs` distribution.
    pub fn adverbs(&self) -> &Distribution {
        &self.adverbs
    }

    /// Returns the `articles` distribution.
    pub fn articles(&self) -> &Distribution {
        &self.articles
    }

    /// Returns the `auxillaries` distribution.
    ///
    /// P.S: The correct spelling is `auxiliaries` which is what we use.
    pub fn auxiliaries(&self) -> &Distribution {
        &self.auxiliaries
    }

    /// Returns the `grammar` distribution.
    pub fn grammar(&self) -> &Distribution {
        &self.grammar
    }

    /// Returns the `category` distribution.
    pub fn category(&self) -> &Distribution {
        &self.category
    }

    /// Returns the `msegmnt` distribution.
    pub fn market_segments(&self) -> &Distribution {
        &self.market_segments
    }

    /// Returns the `nations` distribution.
    pub fn nations(&self) -> &Distribution {
        &self.nations
    }

    /// Returns the `noun_phrases` distribution.
    pub fn noun_phrase(&self) -> &Distribution {
        &self.noun_phrase
    }

    /// Returns the `nouns` distribution.
    pub fn nouns(&self) -> &Distribution {
        &self.nouns
    }

    /// Returns the `orders_priority` distribution.
    pub fn order_priority(&self) -> &Distribution {
        &self.order_priority
    }

    /// Returns the `part_colors` distribution.
    pub fn part_colors(&self) -> &Distribution {
        &self.part_colors
    }

    /// Returns the `part_containers` distribution.
    pub fn part_containers(&self) -> &Distribution {
        &self.part_containers
    }

    /// Returns the `part_types` distribution.
    pub fn part_types(&self) -> &Distribution {
        &self.part_types
    }

    /// Returns the `prepositions` distribution.
    pub fn prepositions(&self) -> &Distribution {
        &self.prepositions
    }

    /// Returns the `regions` distribution.
    pub fn regions(&self) -> &Distribution {
        &self.regions
    }

    /// Returns the `return_flags` distribution.
    pub fn return_flags(&self) -> &Distribution {
        &self.return_flags
    }

    /// Returns the `ship_instructions` distribution.
    pub fn ship_instructions(&self) -> &Distribution {
        &self.ship_instructions
    }

    /// Returns the `ship_modes` distribution.
    pub fn ship_modes(&self) -> &Distribution {
        &self.ship_modes
    }

    /// Returns the `terminators` distribution.
    pub fn terminators(&self) -> &Distribution {
        &self.terminators
    }

    // Returns the `verb_phrases` distribution.
    pub fn verb_phrase(&self) -> &Distribution {
        &self.verb_phrase
    }

    /// Returns the `verbs` distribution.
    pub fn verbs(&self) -> &Distribution {
        &self.verbs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_load_empty() {
        let input = "";
        let lines = input.split('\n');
        let distributions = Distributions::load_distributions(lines).unwrap();
        assert!(distributions.is_empty());
    }

    #[test]
    fn test_load_simple_distribution() {
        let input = "
            # Comment line
            BEGIN test
            value1|10
            value2|20
            END
        ";
        let lines = input.split('\n');
        let distributions: HashMap<_, _> = Distributions::load_distributions(lines)
            .unwrap()
            .into_iter()
            .collect();

        assert_eq!(distributions.len(), 1);
        assert!(distributions.contains_key("test"));

        let test_dist = distributions.get("test").unwrap();
        assert_eq!(test_dist.size(), 2);
        assert_eq!(test_dist.get_value(0), "value1");
        assert_eq!(test_dist.get_value(1), "value2");
        assert_eq!(test_dist.get_weight(0), 10);
        assert_eq!(test_dist.get_weight(1), 30); // Cumulative weight
    }

    #[test]
    fn test_load_multiple_distributions() {
        let input = "
            BEGIN first
            a|5
            b|10
            END

            BEGIN second
            x|2
            y|3
            z|4
            END
        ";
        let lines = input.split('\n');
        let distributions: HashMap<_, _> = Distributions::load_distributions(lines)
            .unwrap()
            .into_iter()
            .collect();

        assert_eq!(distributions.len(), 2);
        assert!(distributions.contains_key("first"));
        assert!(distributions.contains_key("second"));

        let first_dist = distributions.get("first").unwrap();
        assert_eq!(first_dist.size(), 2);

        let second_dist = distributions.get("second").unwrap();
        assert_eq!(second_dist.size(), 3);
    }

    #[test]
    fn test_error_on_invalid_weight() {
        let input = "
            BEGIN test
            value|invalid
            END
        ";
        let lines = input.split('\n');
        let result = Distributions::load_distributions(lines);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_on_missing_end() {
        let input = "
            BEGIN test
            value|10
        ";
        let lines = input.split('\n');
        let result = Distributions::load_distributions(lines);
        assert!(result.is_err());
    }

    #[test]
    fn test_with_default_seeds_file() {
        let expected_distributions = vec![
            "category",
            "p_cntr",
            "instruct",
            "msegmnt",
            "p_names",
            "nations",
            "nations2",
            "regions",
            "o_oprio",
            "regions",
            "rflag",
            "smode",
            "p_types",
            "colors",
            "articles",
            "nouns",
            "verbs",
            "adjectives",
            "adverbs",
            // P.S: The correct spelling is `auxiliaries` which is what we use.
            "auxillaries",
            "prepositions",
            "terminators",
            "grammar",
            "np",
            "vp",
            "Q13a",
            "Q13b",
        ];

        let lines = DISTS_SEED.split('\n');
        let distributions: HashMap<_, _> = Distributions::load_distributions(lines)
            .unwrap()
            .into_iter()
            .collect();
        assert_eq!(distributions.len(), 26);

        for name in expected_distributions {
            assert!(
                distributions.contains_key(name),
                "missing distribution: {}",
                name
            );
        }
    }
}
