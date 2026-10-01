use crate::{check_argument, error::Result, TpcdsError};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Address {
    suite_number: String,
    street_number: i32,
    street_name1: String,
    street_name2: String,
    street_type: String,
    city: String,
    county: Option<String>,
    state: String,
    country: String,
    zip: i32,
    gmt_offset: i32,
}

#[allow(clippy::too_many_arguments)]
impl Address {
    pub fn new(
        suite_number: String,
        street_number: i32,
        street_name1: String,
        street_name2: String,
        street_type: String,
        city: String,
        county: Option<String>,
        state: String,
        country: String,
        zip: i32,
        gmt_offset: i32,
    ) -> Result<Self> {
        check_argument!(
            (1..=1000).contains(&street_number),
            "streetNumber is not between 1 and 1000"
        );
        check_argument!((0..=99999).contains(&zip), "zip is not between 0 and 99999");

        Ok(Address {
            suite_number,
            street_number,
            street_name1,
            street_name2,
            street_type,
            city,
            county,
            state,
            country,
            zip,
            gmt_offset,
        })
    }

    pub fn get_street_number(&self) -> i32 {
        self.street_number
    }

    pub fn get_street_name(&self) -> String {
        format!("{} {}", self.street_name1, self.street_name2)
    }

    pub fn get_suite_number(&self) -> &str {
        &self.suite_number
    }

    pub fn get_street_type(&self) -> &str {
        &self.street_type
    }

    pub fn get_city(&self) -> &str {
        &self.city
    }

    pub fn get_county(&self) -> Option<&str> {
        self.county.as_deref()
    }

    pub fn get_state(&self) -> &str {
        &self.state
    }

    pub fn get_zip(&self) -> i32 {
        self.zip
    }

    pub fn get_country(&self) -> &str {
        &self.country
    }

    pub fn get_gmt_offset(&self) -> i32 {
        self.gmt_offset
    }

    pub fn get_street_name1(&self) -> &str {
        &self.street_name1
    }

    pub fn get_street_name2(&self) -> &str {
        &self.street_name2
    }

    /// Create a new AddressBuilder
    pub fn builder() -> AddressBuilder {
        AddressBuilder::new()
    }

    // Static method to compute city hash (implementation exactly)
    pub fn compute_city_hash(name: &str) -> i32 {
        let mut hash_value = 0i32;
        let mut result = 0i32;

        for ch in name.chars() {
            hash_value = hash_value.wrapping_mul(26);
            hash_value = hash_value.wrapping_add((ch as i32) - ('A' as i32));
            if hash_value > 1000000 {
                hash_value %= 10000;
                result = result.wrapping_add(hash_value);
                hash_value = 0;
            }
        }

        hash_value %= 1000;
        result = result.wrapping_add(hash_value);
        result % 10000 // looking for a 4 digit result
    }

    // Static method to create address for a specific table column (implementation exactly)
    pub fn make_address_for_column(
        table: crate::table::Table,
        stream: &mut crate::random::stream::RandomNumberStream,
        scaling: &crate::config::Scaling,
    ) -> Result<Self> {
        use crate::distribution::{
            get_city_at_index, pick_random_city, pick_random_street_name, pick_random_street_type,
            CitiesWeights, FipsCountyDistribution, FipsWeights, StreetNamesWeights,
        };
        use crate::pseudo_table_scaling_infos::PseudoTableScalingInfos;
        use crate::random::RandomValueGenerator;

        let street_number = RandomValueGenerator::generate_uniform_random_int(1, 1000, stream);
        let street_name1 = pick_random_street_name(StreetNamesWeights::Default, stream)
            .unwrap_or("Main")
            .to_string();
        let street_name2 = pick_random_street_name(StreetNamesWeights::HalfEmpty, stream)
            .unwrap_or("")
            .to_string();
        let street_type = pick_random_street_type(stream)
            .unwrap_or("Street")
            .to_string();

        let random_int = RandomValueGenerator::generate_uniform_random_int(1, 100, stream);
        let suite_number = if random_int % 2 == 1 {
            // if i is odd, suiteNumber is a number
            format!("Suite {}", (random_int / 2) * 10)
        } else {
            // if i is even, suiteNumber is a letter
            format!(
                "Suite {}",
                ((random_int / 2) % 25 + ('A' as i32)) as u8 as char
            )
        };

        let config_table = match table {
            crate::table::Table::CallCenter => crate::config::table::Table::CallCenter,
            crate::table::Table::WebSite => crate::config::table::Table::WebSite,
            crate::table::Table::Warehouse => crate::config::table::Table::Warehouse,
            crate::table::Table::CustomerAddress => crate::config::table::Table::CustomerAddress,
            crate::table::Table::Store => crate::config::table::Table::Store,
            _ => panic!(
                "Table {:?} not yet supported in Address::make_address_for_column",
                table
            ),
        };
        let row_count = scaling.get_row_count(config_table) as i32;
        let city = if table.is_small() {
            let max_cities =
                PseudoTableScalingInfos::get_active_cities_row_count_for_scale(scaling.get_scale())
                    as i32;
            let random_int = RandomValueGenerator::generate_uniform_random_int(
                0,
                if max_cities > row_count {
                    row_count - 1
                } else {
                    max_cities - 1
                },
                stream,
            );
            get_city_at_index(random_int as usize)
                .unwrap_or("Midway")
                .to_string()
        } else {
            pick_random_city(CitiesWeights::UnifiedStepFunction, stream)
                .unwrap_or("Midway")
                .to_string()
        };

        // county is picked from a distribution, based on population and keys the rest
        let region_number = if table.is_small() {
            let max_counties = PseudoTableScalingInfos::get_active_counties_row_count_for_scale(
                scaling.get_scale(),
            ) as i32;
            RandomValueGenerator::generate_uniform_random_int(
                0,
                if max_counties > row_count {
                    row_count - 1
                } else {
                    max_counties - 1
                },
                stream,
            ) as usize
        } else {
            FipsCountyDistribution::pick_random_index(FipsWeights::Uniform, stream).unwrap_or(0)
        };

        let county = FipsCountyDistribution::get_county_at_index(region_number)
            .unwrap_or("Williamson County");
        // let county = if table.is_small() {
        //     FipsCountyDistribution::get_county_at_index(region_number)
        //         .unwrap_or("Williamson County")
        // } else {
        //     // stubbed.
        //     FipsCountyDistribution::get_county_at_index(region_number)
        //         .unwrap_or("Williamson County")
        // };

        // match state with the selected region/county
        let state =
            FipsCountyDistribution::get_state_abbreviation_at_index(region_number).unwrap_or("TN");

        // match the zip prefix with the selected region/county
        let mut zip = Self::compute_city_hash(&city);

        // 00000 - 00600 are unused. Avoid them
        let zip_prefix =
            FipsCountyDistribution::get_zip_prefix_at_index(region_number).unwrap_or(0);
        if zip_prefix == 0 && zip < 9400 {
            zip += 600;
        }
        zip += zip_prefix * 10000;

        let gmt_offset =
            FipsCountyDistribution::get_gmt_offset_at_index(region_number).unwrap_or(-5);
        let country = "United States";

        Address::new(
            suite_number,
            street_number,
            street_name1,
            street_name2,
            street_type,
            city,
            Some(county.to_string()),
            state.to_string(),
            country.to_string(),
            zip,
            gmt_offset,
        )
    }
}

/// Builder for Address
#[derive(Debug, Default)]
pub struct AddressBuilder {
    suite_number: Option<String>,
    street_number: Option<i32>,
    street_name1: Option<String>,
    street_name2: Option<String>,
    street_type: Option<String>,
    city: Option<String>,
    county: Option<String>,
    state: Option<String>,
    country: Option<String>,
    zip: Option<i32>,
    gmt_offset: Option<i32>,
}

impl AddressBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn suite_number(mut self, value: String) -> Self {
        self.suite_number = Some(value);
        self
    }

    pub fn street_number(mut self, value: i32) -> Self {
        self.street_number = Some(value);
        self
    }

    pub fn street_name(mut self, value: String) -> Self {
        self.street_name1 = Some(value);
        self
    }

    pub fn street_name1(mut self, value: String) -> Self {
        self.street_name1 = Some(value);
        self
    }

    pub fn street_name2(mut self, value: String) -> Self {
        self.street_name2 = Some(value);
        self
    }

    pub fn street_type(mut self, value: String) -> Self {
        self.street_type = Some(value);
        self
    }

    pub fn city(mut self, value: String) -> Self {
        self.city = Some(value);
        self
    }

    pub fn county(mut self, value: String) -> Self {
        self.county = Some(value);
        self
    }

    pub fn state(mut self, value: String) -> Self {
        self.state = Some(value);
        self
    }

    pub fn country(mut self, value: String) -> Self {
        self.country = Some(value);
        self
    }

    pub fn zip(mut self, value: i32) -> Self {
        self.zip = Some(value);
        self
    }

    pub fn gmt_offset(mut self, value: i32) -> Self {
        self.gmt_offset = Some(value);
        self
    }

    pub fn build(self) -> Address {
        Address {
            suite_number: self.suite_number.unwrap_or_default(),
            street_number: self.street_number.unwrap_or(1),
            street_name1: self.street_name1.unwrap_or_default(),
            street_name2: self.street_name2.unwrap_or_default(),
            street_type: self.street_type.unwrap_or_default(),
            city: self.city.unwrap_or_default(),
            county: self.county,
            state: self.state.unwrap_or_default(),
            country: self.country.unwrap_or_default(),
            zip: self.zip.unwrap_or(0),
            gmt_offset: self.gmt_offset.unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_creation() {
        let address = Address::new(
            "Suite 100".to_string(),
            123,
            "Main".to_string(),
            "Street".to_string(),
            "St".to_string(),
            "Anytown".to_string(),
            Some("AnyCounty".to_string()),
            "CA".to_string(),
            "United States".to_string(),
            12345,
            -8,
        )
        .unwrap();

        assert_eq!(address.get_street_number(), 123);
        assert_eq!(address.get_street_name(), "Main Street");
        assert_eq!(address.get_city(), "Anytown");
        assert_eq!(address.get_zip(), 12345);
    }

    #[test]
    fn test_city_hash() {
        let hash = Address::compute_city_hash("TESTCITY");
        assert!((0..10000).contains(&hash));
    }

    #[test]
    fn test_address_builder() {
        let address = Address::builder()
            .street_number(456)
            .city("TestCity".to_string())
            .zip(54321)
            .build();

        assert_eq!(address.get_street_number(), 456);
        assert_eq!(address.get_city(), "TestCity");
        assert_eq!(address.get_zip(), 54321);
    }
}
