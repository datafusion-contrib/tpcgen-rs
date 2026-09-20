//! Generators for each TPC-H Tables
use crate::dates;
use crate::decimal::TPCHDecimal;
use crate::distribution::Distribution;
use crate::distribution::Distributions;
use crate::random::RandomPhoneNumber;
use crate::random::RowRandomInt;
use crate::random::{PhoneNumberInstance, RandomBoundedLong, StringSequenceInstance};
use crate::random::{RandomAlphaNumeric, RandomAlphaNumericInstance};
use crate::text::TextPool;
use core::fmt;
use std::fmt::Display;

use crate::dates::{GenerateUtils, TPCHDate};
use crate::random::{RandomBoundedInt, RandomString, RandomStringSequence, RandomText};

/// Generator for Nation table data
#[derive(Debug, Clone)]
pub struct NationGenerator<'a> {
    distributions: &'a Distributions,
    text_pool: &'a TextPool,
}

impl Default for NationGenerator<'_> {
    fn default() -> Self {
        // arguments are ignored
        Self::new(1.0, 1, 1)
    }
}

impl<'a> NationGenerator<'a> {
    /// Creates a new NationGenerator with default distributions and text pool
    ///
    /// Nations does not depend on the scale factor or the part number. The signature of
    /// this method is provided to be consistent with the other generators, but the
    /// parameters are ignored. You can use [`NationGenerator::default`] to create a
    /// default generator.
    ///
    /// The generator's lifetime is `&'static` because it references global
    /// [`Distribution]`s and thus can be shared safely between threads.
    pub fn new(_scale_factor: f64, _part: i32, _part_count: i32) -> NationGenerator<'static> {
        // Note: use explicit lifetime to ensure this remains `&'static`
        Self::new_with_distributions_and_text_pool(
            Distributions::static_default(),
            TextPool::get_or_init_default(),
        )
    }

    /// Creates a NationGenerator with the specified distributions and text pool
    pub fn new_with_distributions_and_text_pool<'b>(
        distributions: &'b Distributions,
        text_pool: &'b TextPool,
    ) -> NationGenerator<'b> {
        NationGenerator {
            distributions,
            text_pool,
        }
    }

    /// Returns an iterator over the nation rows
    pub fn iter(&self) -> NationGeneratorIterator<'a> {
        NationGeneratorIterator::new(self.distributions.nations(), self.text_pool)
    }
}

impl<'a> IntoIterator for NationGenerator<'a> {
    type Item = Nation<'a>;
    type IntoIter = NationGeneratorIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// The NATION table
///
/// The Display trait is implemented to format the line item data as a string
/// in the default TPC-H 'tbl' format.
///
/// ```text
/// 0|ALGERIA|0| haggle. carefully final deposits detect slyly agai|
/// 1|ARGENTINA|1|al foxes promise slyly according to the regular accounts. bold requests alon|
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nation<'a> {
    /// Primary key (0-24)
    pub n_nationkey: i64,
    /// Nation name
    pub n_name: &'a str,
    /// Foreign key to REGION
    pub n_regionkey: i64,
    /// Variable length comment
    pub n_comment: &'a str,
}

impl fmt::Display for Nation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|",
            self.n_nationkey, self.n_name, self.n_regionkey, self.n_comment
        )
    }
}

impl<'a> Nation<'a> {
    /// Create a new `nation` record with the specified values.
    pub fn new(n_nationkey: i64, n_name: &'a str, n_regionkey: i64, n_comment: &'a str) -> Self {
        Nation {
            n_nationkey,
            n_name,
            n_regionkey,
            n_comment,
        }
    }
}

/// Iterator that generates Nation rows
#[derive(Debug)]
pub struct NationGeneratorIterator<'a> {
    nations: &'a Distribution,
    comment_random: RandomText<'a>,
    index: usize,
}

impl<'a> NationGeneratorIterator<'a> {
    const COMMENT_AVERAGE_LENGTH: i32 = 72;

    fn new(nations: &'a Distribution, text_pool: &'a TextPool) -> Self {
        NationGeneratorIterator {
            nations,
            comment_random: RandomText::new(
                606179079,
                text_pool,
                Self::COMMENT_AVERAGE_LENGTH as f64,
            ),
            index: 0,
        }
    }
}

impl<'a> Iterator for NationGeneratorIterator<'a> {
    type Item = Nation<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.nations.size() {
            return None;
        }

        let nation = Nation {
            // n_nationkey
            n_nationkey: self.index as i64,
            // n_name
            n_name: self.nations.get_value(self.index),
            // n_regionkey
            n_regionkey: self.nations.get_weight(self.index) as i64,
            // n_comment
            n_comment: self.comment_random.next_value(),
        };

        self.comment_random.row_finished();
        self.index += 1;

        Some(nation)
    }
}

/// The REGION table
///
/// The Display trait is implemented to format the line item data as a string
/// in the default TPC-H 'tbl' format.
///
/// ```text
/// 0|AFRICA|lar deposits. blithely final packages cajole. regular waters are final requests. regular accounts are according to |
/// 1|AMERICA|hs use ironic, even requests. s|
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region<'a> {
    /// Primary key (0-4)
    pub r_regionkey: i64,
    /// Region name (AFRICA, AMERICA, ASIA, EUROPE, MIDDLE EAST)
    pub r_name: &'a str,
    /// Variable length comment
    pub r_comment: &'a str,
}

impl fmt::Display for Region<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}|{}|{}|",
            self.r_regionkey, self.r_name, self.r_comment
        )
    }
}

impl<'a> Region<'a> {
    /// Creates a new `region` record with the specified values.
    pub fn new(r_regionkey: i64, r_name: &'a str, r_comment: &'a str) -> Self {
        Region {
            r_regionkey,
            r_name,
            r_comment,
        }
    }
}

/// Generator for Region table data
#[derive(Debug, Clone)]
pub struct RegionGenerator<'a> {
    distributions: &'a Distributions,
    text_pool: &'a TextPool,
}

impl Default for RegionGenerator<'_> {
    fn default() -> Self {
        // arguments are ignored
        Self::new(1.0, 1, 1)
    }
}

impl<'a> RegionGenerator<'a> {
    /// Creates a new RegionGenerator with default distributions and text pool
    ///
    /// Regions does not depend on the scale factor or the part number. The signature of
    /// this method is provided to be consistent with the other generators, but the
    /// parameters are ignored. You can use [`RegionGenerator::default`] to create a
    /// default generator.
    ///
    /// Note the generator's lifetime is `&'static`. See [`NationGenerator`] for
    /// more details.
    pub fn new(_scale_factor: f64, _part: i32, _part_count: i32) -> RegionGenerator<'static> {
        // Note: use explicit lifetime to ensure this remains `&'static`
        Self::new_with_distributions_and_text_pool(
            Distributions::static_default(),
            TextPool::get_or_init_default(),
        )
    }

    /// Creates a RegionGenerator with the specified distributions and text pool
    pub fn new_with_distributions_and_text_pool<'b>(
        distributions: &'b Distributions,
        text_pool: &'b TextPool,
    ) -> RegionGenerator<'b> {
        RegionGenerator {
            distributions,
            text_pool,
        }
    }

    /// Returns an iterator over the region rows
    pub fn iter(&self) -> RegionGeneratorIterator<'a> {
        RegionGeneratorIterator::new(self.distributions.regions(), self.text_pool)
    }
}

impl<'a> IntoIterator for RegionGenerator<'a> {
    type Item = Region<'a>;
    type IntoIter = RegionGeneratorIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that generates Region rows
#[derive(Debug)]
pub struct RegionGeneratorIterator<'a> {
    regions: &'a Distribution,
    comment_random: RandomText<'a>,
    index: usize,
}

impl<'a> RegionGeneratorIterator<'a> {
    const COMMENT_AVERAGE_LENGTH: i32 = 72;

    fn new(regions: &'a Distribution, text_pool: &'a TextPool) -> Self {
        RegionGeneratorIterator {
            regions,
            comment_random: RandomText::new(
                1500869201,
                text_pool,
                Self::COMMENT_AVERAGE_LENGTH as f64,
            ),
            index: 0,
        }
    }
}

impl<'a> Iterator for RegionGeneratorIterator<'a> {
    type Item = Region<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.regions.size() {
            return None;
        }

        let region = Region {
            r_regionkey: self.index as i64,
            r_name: self.regions.get_value(self.index),
            r_comment: self.comment_random.next_value(),
        };

        self.comment_random.row_finished();
        self.index += 1;

        Some(region)
    }
}

/// A Part Manufacturer, formatted as `"Manufacturer#<n>"`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartManufacturerName(i32);

impl PartManufacturerName {
    pub fn new(value: i32) -> Self {
        PartManufacturerName(value)
    }
}

impl fmt::Display for PartManufacturerName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Manufacturer#{}", self.0)
    }
}

/// A Part brand name, formatted as `"Brand#<n>"`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartBrandName(i32);

impl PartBrandName {
    pub fn new(value: i32) -> Self {
        PartBrandName(value)
    }
}

impl fmt::Display for PartBrandName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Brand#{}", self.0)
    }
}

/// The PART table
///
/// The Display trait is implemented to format the line item data as a string
/// in the default TPC-H 'tbl' format.
///
/// ```text
/// 1|goldenrod lavender spring chocolate lace|Manufacturer#1|Brand#13|PROMO BURNISHED COPPER|7|JUMBO PKG|901.00|ly. slyly ironi|
/// 2|blush thistle blue yellow saddle|Manufacturer#1|Brand#13|LARGE BRUSHED BRASS|1|LG CASE|902.00|lar accounts amo|
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Part<'a> {
    /// Primary key
    pub p_partkey: i64,
    /// Part name
    pub p_name: StringSequenceInstance<'a>,
    /// Part manufacturer.
    pub p_mfgr: PartManufacturerName,
    /// Part brand.
    pub p_brand: PartBrandName,
    /// Part type
    pub p_type: &'a str,
    /// Part size
    pub p_size: i32,
    /// Part container
    pub p_container: &'a str,
    /// Part retail price
    pub p_retailprice: TPCHDecimal,
    /// Variable length comment
    pub p_comment: &'a str,
}

impl fmt::Display for Part<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.p_partkey,
            self.p_name,
            self.p_mfgr,
            self.p_brand,
            self.p_type,
            self.p_size,
            self.p_container,
            self.p_retailprice,
            self.p_comment
        )
    }
}

/// Generator for Part table data
#[derive(Debug, Clone)]
pub struct PartGenerator<'a> {
    scale_factor: f64,
    part: i32,
    part_count: i32,
    distributions: &'a Distributions,
    text_pool: &'a TextPool,
}

impl<'a> PartGenerator<'a> {
    /// Base scale for part generation
    const SCALE_BASE: i32 = 200_000;

    // Constants for part generation
    const NAME_WORDS: i32 = 5;
    const MANUFACTURER_MIN: i32 = 1;
    const MANUFACTURER_MAX: i32 = 5;
    const BRAND_MIN: i32 = 1;
    const BRAND_MAX: i32 = 5;
    const SIZE_MIN: i32 = 1;
    const SIZE_MAX: i32 = 50;
    const COMMENT_AVERAGE_LENGTH: i32 = 14;

    /// Creates a new PartGenerator with the given scale factor
    ///
    /// Note the generator's lifetime is `&'static`. See [`NationGenerator`] for
    /// more details.
    pub fn new(scale_factor: f64, part: i32, part_count: i32) -> PartGenerator<'static> {
        // Note: use explicit lifetime to ensure this remains `&'static`
        Self::new_with_distributions_and_text_pool(
            scale_factor,
            part,
            part_count,
            Distributions::static_default(),
            TextPool::get_or_init_default(),
        )
    }

    /// Creates a PartGenerator with specified distributions and text pool
    pub fn new_with_distributions_and_text_pool<'b>(
        scale_factor: f64,
        part: i32,
        part_count: i32,
        distributions: &'b Distributions,
        text_pool: &'b TextPool,
    ) -> PartGenerator<'b> {
        PartGenerator {
            scale_factor,
            part,
            part_count,
            distributions,
            text_pool,
        }
    }

    /// Return the row count for the given scale factor and generator part count
    pub fn calculate_row_count(scale_factor: f64, part: i32, part_count: i32) -> i64 {
        GenerateUtils::calculate_row_count(Self::SCALE_BASE, scale_factor, part, part_count)
    }

    /// Returns an iterator over the part rows
    pub fn iter(&self) -> PartGeneratorIterator<'a> {
        PartGeneratorIterator::new(
            self.distributions,
            self.text_pool,
            GenerateUtils::calculate_start_index(
                Self::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
            Self::calculate_row_count(self.scale_factor, self.part, self.part_count),
        )
    }
}

impl<'a> IntoIterator for PartGenerator<'a> {
    type Item = Part<'a>;
    type IntoIter = PartGeneratorIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that generates Part rows
#[derive(Debug)]
pub struct PartGeneratorIterator<'a> {
    name_random: RandomStringSequence<'a>,
    manufacturer_random: RandomBoundedInt,
    brand_random: RandomBoundedInt,
    type_random: RandomString<'a>,
    size_random: RandomBoundedInt,
    container_random: RandomString<'a>,
    comment_random: RandomText<'a>,

    start_index: i64,
    row_count: i64,
    index: i64,
}

impl<'a> PartGeneratorIterator<'a> {
    fn new(
        distributions: &'a Distributions,
        text_pool: &'a TextPool,
        start_index: i64,
        row_count: i64,
    ) -> Self {
        let mut name_random = RandomStringSequence::new(
            709314158,
            PartGenerator::NAME_WORDS,
            distributions.part_colors(),
        );
        let mut manufacturer_random = RandomBoundedInt::new(
            1,
            PartGenerator::MANUFACTURER_MIN,
            PartGenerator::MANUFACTURER_MAX,
        );
        let mut brand_random =
            RandomBoundedInt::new(46831694, PartGenerator::BRAND_MIN, PartGenerator::BRAND_MAX);
        let mut type_random = RandomString::new(1841581359, distributions.part_types());
        let mut size_random =
            RandomBoundedInt::new(1193163244, PartGenerator::SIZE_MIN, PartGenerator::SIZE_MAX);
        let mut container_random = RandomString::new(727633698, distributions.part_containers());
        let mut comment_random = RandomText::new(
            804159733,
            text_pool,
            PartGenerator::COMMENT_AVERAGE_LENGTH as f64,
        );

        // Advance all generators to the starting position
        name_random.advance_rows(start_index);
        manufacturer_random.advance_rows(start_index);
        brand_random.advance_rows(start_index);
        type_random.advance_rows(start_index);
        size_random.advance_rows(start_index);
        container_random.advance_rows(start_index);
        comment_random.advance_rows(start_index);

        PartGeneratorIterator {
            name_random,
            manufacturer_random,
            brand_random,
            type_random,
            size_random,
            container_random,
            comment_random,
            start_index,
            row_count,
            index: 0,
        }
    }

    /// Creates a part with the given key
    fn make_part(&mut self, part_key: i64) -> Part<'a> {
        let name = self.name_random.next_value();

        let manufacturer = self.manufacturer_random.next_value();
        let brand = manufacturer * 10 + self.brand_random.next_value();

        Part {
            p_partkey: part_key,
            p_name: name,
            p_mfgr: PartManufacturerName::new(manufacturer),
            p_brand: PartBrandName::new(brand),
            p_type: self.type_random.next_value(),
            p_size: self.size_random.next_value(),
            p_container: self.container_random.next_value(),
            p_retailprice: TPCHDecimal(Self::calculate_part_price(part_key)),
            p_comment: self.comment_random.next_value(),
        }
    }

    /// Calculates the price for a part
    pub fn calculate_part_price(part_key: i64) -> i64 {
        let mut price = 90000;

        // limit contribution to $200
        price += (part_key / 10) % 20001;
        price += (part_key % 1000) * 100;

        price
    }
}

impl<'a> Iterator for PartGeneratorIterator<'a> {
    type Item = Part<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.row_count {
            return None;
        }

        let part = self.make_part(self.start_index + self.index + 1);

        self.name_random.row_finished();
        self.manufacturer_random.row_finished();
        self.brand_random.row_finished();
        self.type_random.row_finished();
        self.size_random.row_finished();
        self.container_random.row_finished();
        self.comment_random.row_finished();

        self.index += 1;

        Some(part)
    }
}

/// A supplier name, formatted as `"Supplier#<n>"`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SupplierName(i64);

impl SupplierName {
    /// Creates a new SupplierName with the given value
    pub fn new(value: i64) -> Self {
        SupplierName(value)
    }
}

impl fmt::Display for SupplierName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Supplier#{:09}", self.0)
    }
}

/// Records for the SUPPLIER table.
///
/// The Display trait is implemented to format the line item data as a string
/// in the default TPC-H 'tbl' format.
///
/// ```text
/// 1|Supplier#000000001| N kD4on9OM Ipw3,gf0JBoQDd7tgrzrddZ|17|27-918-335-1736|5755.94|each slyly above the careful|
/// 2|Supplier#000000002|89eJ5ksX3ImxJQBvxObC,|5|15-679-861-2259|4032.68| slyly bold instructions. idle dependen|
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Supplier {
    /// Primary key
    pub s_suppkey: i64,
    /// Supplier name.
    pub s_name: SupplierName,
    /// Supplier address
    pub s_address: RandomAlphaNumericInstance,
    /// Foreign key to NATION
    pub s_nationkey: i64,
    /// Supplier phone number
    pub s_phone: PhoneNumberInstance,
    /// Supplier account balance
    pub s_acctbal: TPCHDecimal,
    /// Variable length comment
    pub s_comment: String,
}

impl fmt::Display for Supplier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|",
            self.s_suppkey,
            self.s_name,
            self.s_address,
            self.s_nationkey,
            self.s_phone,
            self.s_acctbal,
            self.s_comment
        )
    }
}

/// Generator for Supplier table data
#[derive(Debug, Clone)]
pub struct SupplierGenerator<'a> {
    scale_factor: f64,
    part: i32,
    part_count: i32,
    distributions: &'a Distributions,
    text_pool: &'a TextPool,
}

impl<'a> SupplierGenerator<'a> {
    /// Base scale for supplier generation
    const SCALE_BASE: i32 = 10_000;

    // Constants for supplier generation
    const ACCOUNT_BALANCE_MIN: i32 = -99999;
    const ACCOUNT_BALANCE_MAX: i32 = 999999;
    const ADDRESS_AVERAGE_LENGTH: i32 = 25;
    const COMMENT_AVERAGE_LENGTH: i32 = 63;

    // Better Business Bureau comment constants
    pub const BBB_BASE_TEXT: &'static str = "Customer ";
    pub const BBB_COMPLAINT_TEXT: &'static str = "Complaints";
    pub const BBB_RECOMMEND_TEXT: &'static str = "Recommends";
    pub const BBB_COMMENT_LENGTH: usize =
        Self::BBB_BASE_TEXT.len() + Self::BBB_COMPLAINT_TEXT.len();
    pub const BBB_COMMENTS_PER_SCALE_BASE: i32 = 10;
    pub const BBB_COMPLAINT_PERCENT: i32 = 50;

    /// Creates a new SupplierGenerator with the given scale factor
    ///
    /// Note the generator's lifetime is `&'static`. See [`NationGenerator`] for
    /// more details.
    pub fn new(scale_factor: f64, part: i32, part_count: i32) -> SupplierGenerator<'static> {
        // Note: use explicit lifetime to ensure this remains `&'static`
        Self::new_with_distributions_and_text_pool(
            scale_factor,
            part,
            part_count,
            Distributions::static_default(),
            TextPool::get_or_init_default(),
        )
    }

    /// Creates a SupplierGenerator with specified distributions and text pool
    pub fn new_with_distributions_and_text_pool<'b>(
        scale_factor: f64,
        part: i32,
        part_count: i32,
        distributions: &'b Distributions,
        text_pool: &'b TextPool,
    ) -> SupplierGenerator<'b> {
        SupplierGenerator {
            scale_factor,
            part,
            part_count,
            distributions,
            text_pool,
        }
    }

    /// Return the row count for the given scale factor and generator part count
    pub fn calculate_row_count(scale_factor: f64, part: i32, part_count: i32) -> i64 {
        GenerateUtils::calculate_row_count(Self::SCALE_BASE, scale_factor, part, part_count)
    }

    /// Returns an iterator over the supplier rows
    pub fn iter(&self) -> SupplierGeneratorIterator<'a> {
        SupplierGeneratorIterator::new(
            self.distributions,
            self.text_pool,
            GenerateUtils::calculate_start_index(
                Self::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
            Self::calculate_row_count(self.scale_factor, self.part, self.part_count),
        )
    }
}

impl<'a> IntoIterator for SupplierGenerator<'a> {
    type Item = Supplier;
    type IntoIter = SupplierGeneratorIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that generates Supplier rows
#[derive(Debug)]
pub struct SupplierGeneratorIterator<'a> {
    address_random: RandomAlphaNumeric,
    nation_key_random: RandomBoundedInt,
    phone_random: RandomPhoneNumber,
    account_balance_random: RandomBoundedInt,
    comment_random: RandomText<'a>,
    bbb_comment_random: RandomBoundedInt,
    bbb_junk_random: RowRandomInt,
    bbb_offset_random: RowRandomInt,
    bbb_type_random: RandomBoundedInt,

    start_index: i64,
    row_count: i64,
    index: i64,
}

impl<'a> SupplierGeneratorIterator<'a> {
    fn new(
        distributions: &Distributions,
        text_pool: &'a TextPool,
        start_index: i64,
        row_count: i64,
    ) -> Self {
        let mut address_random =
            RandomAlphaNumeric::new(706178559, SupplierGenerator::ADDRESS_AVERAGE_LENGTH);
        let mut nation_key_random =
            RandomBoundedInt::new(110356601, 0, (distributions.nations().size() - 1) as i32);
        let mut phone_random = RandomPhoneNumber::new(884434366);
        let mut account_balance_random = RandomBoundedInt::new(
            962338209,
            SupplierGenerator::ACCOUNT_BALANCE_MIN,
            SupplierGenerator::ACCOUNT_BALANCE_MAX,
        );
        let mut comment_random = RandomText::new(
            1341315363,
            text_pool,
            SupplierGenerator::COMMENT_AVERAGE_LENGTH as f64,
        );
        let mut bbb_comment_random =
            RandomBoundedInt::new(202794285, 1, SupplierGenerator::SCALE_BASE);
        let mut bbb_junk_random = RowRandomInt::new(263032577, 1);
        let mut bbb_offset_random = RowRandomInt::new(715851524, 1);
        let mut bbb_type_random = RandomBoundedInt::new(753643799, 0, 100);

        // Advance all generators to the starting position
        address_random.advance_rows(start_index);
        nation_key_random.advance_rows(start_index);
        phone_random.advance_rows(start_index);
        account_balance_random.advance_rows(start_index);
        comment_random.advance_rows(start_index);
        bbb_comment_random.advance_rows(start_index);
        bbb_junk_random.advance_rows(start_index);
        bbb_offset_random.advance_rows(start_index);
        bbb_type_random.advance_rows(start_index);

        SupplierGeneratorIterator {
            address_random,
            nation_key_random,
            phone_random,
            account_balance_random,
            comment_random,
            bbb_comment_random,
            bbb_junk_random,
            bbb_offset_random,
            bbb_type_random,
            start_index,
            row_count,
            index: 0,
        }
    }

    /// Creates a supplier with the given key
    fn make_supplier(&mut self, supplier_key: i64) -> Supplier {
        let mut comment = self.comment_random.next_value().to_string();

        // Add supplier complaints or commendation to the comment
        let bbb_comment_random_value = self.bbb_comment_random.next_value();
        if bbb_comment_random_value <= SupplierGenerator::BBB_COMMENTS_PER_SCALE_BASE {
            let _buffer = comment.clone();

            // select random place for BBB comment
            let noise = self.bbb_junk_random.next_int(
                0,
                (comment.len() - SupplierGenerator::BBB_COMMENT_LENGTH) as i32,
            ) as usize;
            let offset = self.bbb_offset_random.next_int(
                0,
                (comment.len() - (SupplierGenerator::BBB_COMMENT_LENGTH + noise)) as i32,
            ) as usize;

            // select complaint or recommendation
            let type_text =
                if self.bbb_type_random.next_value() < SupplierGenerator::BBB_COMPLAINT_PERCENT {
                    SupplierGenerator::BBB_COMPLAINT_TEXT
                } else {
                    SupplierGenerator::BBB_RECOMMEND_TEXT
                };

            // Create a mutable string that we can modify in chunks
            let mut modified_comment = String::with_capacity(comment.len());
            modified_comment.push_str(&comment[..offset]);
            modified_comment.push_str(SupplierGenerator::BBB_BASE_TEXT);
            modified_comment.push_str(
                &comment[offset + SupplierGenerator::BBB_BASE_TEXT.len()
                    ..offset + SupplierGenerator::BBB_BASE_TEXT.len() + noise],
            );
            modified_comment.push_str(type_text);
            modified_comment.push_str(
                &comment
                    [offset + SupplierGenerator::BBB_BASE_TEXT.len() + noise + type_text.len()..],
            );

            comment = modified_comment;
        }

        let nation_key = self.nation_key_random.next_value() as i64;

        Supplier {
            s_suppkey: supplier_key,
            s_name: SupplierName::new(supplier_key),
            s_address: self.address_random.next_value(),
            s_nationkey: nation_key,
            s_phone: self.phone_random.next_value(nation_key),
            s_acctbal: TPCHDecimal(self.account_balance_random.next_value() as i64),
            s_comment: comment,
        }
    }
}

impl Iterator for SupplierGeneratorIterator<'_> {
    type Item = Supplier;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.row_count {
            return None;
        }

        let supplier = self.make_supplier(self.start_index + self.index + 1);

        self.address_random.row_finished();
        self.nation_key_random.row_finished();
        self.phone_random.row_finished();
        self.account_balance_random.row_finished();
        self.comment_random.row_finished();
        self.bbb_comment_random.row_finished();
        self.bbb_junk_random.row_finished();
        self.bbb_offset_random.row_finished();
        self.bbb_type_random.row_finished();

        self.index += 1;

        Some(supplier)
    }
}

/// A Customer Name, formatted as `"Customer#<n>"`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CustomerName(i64);

impl CustomerName {
    /// Creates a new CustomerName with the given value
    pub fn new(value: i64) -> Self {
        CustomerName(value)
    }
}

impl fmt::Display for CustomerName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Customer#{:09}", self.0)
    }
}

/// The CUSTOMER table
///
/// The Display trait is implemented to format the line item data as a string
/// in the default TPC-H 'tbl' format.
///
/// ```text
/// 1|Customer#000000001|IVhzIApeRb ot,c,E|15|25-989-741-2988|711.56|BUILDING|to the even, regular platelets. regular, ironic epitaphs nag e|
/// 2|Customer#000000002|XSTf4,NCwDVaWNe6tEgvwfmRchLXak|13|23-768-687-3665|121.65|AUTOMOBILE|l accounts. blithely ironic theodolites integrate boldly: caref|
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Customer<'a> {
    /// Primary key
    pub c_custkey: i64,
    /// Customer name
    pub c_name: CustomerName,
    /// Customer address
    pub c_address: RandomAlphaNumericInstance,
    /// Foreign key to NATION
    pub c_nationkey: i64,
    /// Customer phone number
    pub c_phone: PhoneNumberInstance,
    /// Customer account balance
    pub c_acctbal: TPCHDecimal,
    /// Customer market segment
    pub c_mktsegment: &'a str,
    /// Variable length comment
    pub c_comment: &'a str,
}

impl fmt::Display for Customer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|",
            self.c_custkey,
            self.c_name,
            self.c_address,
            self.c_nationkey,
            self.c_phone,
            self.c_acctbal,
            self.c_mktsegment,
            self.c_comment
        )
    }
}

/// Generator for Customer table data
#[derive(Debug, Clone)]
pub struct CustomerGenerator<'a> {
    scale_factor: f64,
    part: i32,
    part_count: i32,
    distributions: &'a Distributions,
    text_pool: &'a TextPool,
}

impl<'a> CustomerGenerator<'a> {
    /// Base scale for customer generation
    const SCALE_BASE: i32 = 150_000;

    // Constants for customer generation
    const ACCOUNT_BALANCE_MIN: i32 = -99999;
    const ACCOUNT_BALANCE_MAX: i32 = 999999;
    const ADDRESS_AVERAGE_LENGTH: i32 = 25;
    const COMMENT_AVERAGE_LENGTH: i32 = 73;

    /// Creates a new CustomerGenerator with the given scale factor
    ///
    /// Note the generator's lifetime is `&'static`. See [`NationGenerator`] for
    /// more details.
    pub fn new(scale_factor: f64, part: i32, part_count: i32) -> CustomerGenerator<'static> {
        // Note: use explicit lifetime to ensure this remains `&'static`
        Self::new_with_distributions_and_text_pool(
            scale_factor,
            part,
            part_count,
            Distributions::static_default(),
            TextPool::get_or_init_default(),
        )
    }

    /// Creates a CustomerGenerator with specified distributions and text pool
    pub fn new_with_distributions_and_text_pool<'b>(
        scale_factor: f64,
        part: i32,
        part_count: i32,
        distributions: &'b Distributions,
        text_pool: &'b TextPool,
    ) -> CustomerGenerator<'b> {
        CustomerGenerator {
            scale_factor,
            part,
            part_count,
            distributions,
            text_pool,
        }
    }

    /// Return the row count for the given scale factor and generator part count
    pub fn calculate_row_count(scale_factor: f64, part: i32, part_count: i32) -> i64 {
        GenerateUtils::calculate_row_count(Self::SCALE_BASE, scale_factor, part, part_count)
    }

    /// Returns an iterator over the customer rows
    pub fn iter(&self) -> CustomerGeneratorIterator<'a> {
        CustomerGeneratorIterator::new(
            self.distributions,
            self.text_pool,
            GenerateUtils::calculate_start_index(
                Self::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
            Self::calculate_row_count(self.scale_factor, self.part, self.part_count),
        )
    }
}

impl<'a> IntoIterator for CustomerGenerator<'a> {
    type Item = Customer<'a>;
    type IntoIter = CustomerGeneratorIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that generates Customer rows
#[derive(Debug)]
pub struct CustomerGeneratorIterator<'a> {
    address_random: RandomAlphaNumeric,
    nation_key_random: RandomBoundedInt,
    phone_random: RandomPhoneNumber,
    account_balance_random: RandomBoundedInt,
    market_segment_random: RandomString<'a>,
    comment_random: RandomText<'a>,

    start_index: i64,
    row_count: i64,
    index: i64,
}

impl<'a> CustomerGeneratorIterator<'a> {
    fn new(
        distributions: &'a Distributions,
        text_pool: &'a TextPool,
        start_index: i64,
        row_count: i64,
    ) -> Self {
        let mut address_random =
            RandomAlphaNumeric::new(881155353, CustomerGenerator::ADDRESS_AVERAGE_LENGTH);
        let mut nation_key_random =
            RandomBoundedInt::new(1489529863, 0, (distributions.nations().size() - 1) as i32);
        let mut phone_random = RandomPhoneNumber::new(1521138112);
        let mut account_balance_random = RandomBoundedInt::new(
            298370230,
            CustomerGenerator::ACCOUNT_BALANCE_MIN,
            CustomerGenerator::ACCOUNT_BALANCE_MAX,
        );
        let mut market_segment_random =
            RandomString::new(1140279430, distributions.market_segments());
        let mut comment_random = RandomText::new(
            1335826707,
            text_pool,
            CustomerGenerator::COMMENT_AVERAGE_LENGTH as f64,
        );

        // Advance all generators to the starting position
        address_random.advance_rows(start_index);
        nation_key_random.advance_rows(start_index);
        phone_random.advance_rows(start_index);
        account_balance_random.advance_rows(start_index);
        market_segment_random.advance_rows(start_index);
        comment_random.advance_rows(start_index);

        CustomerGeneratorIterator {
            address_random,
            nation_key_random,
            phone_random,
            account_balance_random,
            market_segment_random,
            comment_random,
            start_index,
            row_count,
            index: 0,
        }
    }

    /// Creates a customer with the given key
    fn make_customer(&mut self, customer_key: i64) -> Customer<'a> {
        let nation_key = self.nation_key_random.next_value() as i64;

        Customer {
            c_custkey: customer_key,
            c_name: CustomerName::new(customer_key),
            c_address: self.address_random.next_value(),
            c_nationkey: nation_key,
            c_phone: self.phone_random.next_value(nation_key),
            c_acctbal: TPCHDecimal(self.account_balance_random.next_value() as i64),
            c_mktsegment: self.market_segment_random.next_value(),
            c_comment: self.comment_random.next_value(),
        }
    }
}

impl<'a> Iterator for CustomerGeneratorIterator<'a> {
    type Item = Customer<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.row_count {
            return None;
        }

        let customer = self.make_customer(self.start_index + self.index + 1);

        self.address_random.row_finished();
        self.nation_key_random.row_finished();
        self.phone_random.row_finished();
        self.account_balance_random.row_finished();
        self.market_segment_random.row_finished();
        self.comment_random.row_finished();

        self.index += 1;

        Some(customer)
    }
}

/// The PARTSUPP (part supplier) table
///
/// The Display trait is implemented to format the line item data as a string
/// in the default TPC-H 'tbl' format.
///
/// ```text
/// 1|2|3325|771.64|, even theodolites. regular, final theodolites eat after the carefully pending foxes. ...
/// 1|4|8076|993.49|ven ideas. quickly even packages print. pending multipliers must have to are fluff|
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PartSupp<'a> {
    /// Primary key, foreign key to PART
    pub ps_partkey: i64,
    /// Primary key, foreign key to SUPPLIER
    pub ps_suppkey: i64,
    /// Available quantity
    pub ps_availqty: i32,
    /// Supplier cost
    pub ps_supplycost: TPCHDecimal,
    /// Variable length comment
    pub ps_comment: &'a str,
}

impl fmt::Display for PartSupp<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|",
            self.ps_partkey, self.ps_suppkey, self.ps_availqty, self.ps_supplycost, self.ps_comment
        )
    }
}

/// Generator for PartSupp table data
#[derive(Debug, Clone)]
pub struct PartSuppGenerator<'a> {
    scale_factor: f64,
    part: i32,
    part_count: i32,
    text_pool: &'a TextPool,
}

impl<'a> PartSuppGenerator<'a> {
    /// Base scale for part-supplier generation
    const SUPPLIERS_PER_PART: i32 = 4;

    // Constants for part-supplier generation
    const AVAILABLE_QUANTITY_MIN: i32 = 1;
    const AVAILABLE_QUANTITY_MAX: i32 = 9999;
    const SUPPLY_COST_MIN: i32 = 100;
    const SUPPLY_COST_MAX: i32 = 100000;
    const COMMENT_AVERAGE_LENGTH: i32 = 124;

    /// Creates a new PartSuppGenerator with the given scale factor
    ///
    /// Note the generator's lifetime is `&'static`. See [`NationGenerator`] for
    /// more details.
    pub fn new(scale_factor: f64, part: i32, part_count: i32) -> PartSuppGenerator<'static> {
        // Note: use explicit lifetime to ensure this remains `&'static`
        Self::new_with_text_pool(
            scale_factor,
            part,
            part_count,
            TextPool::get_or_init_default(),
        )
    }

    /// Creates a PartSuppGenerator with specified text pool
    pub fn new_with_text_pool(
        scale_factor: f64,
        part: i32,
        part_count: i32,
        text_pool: &TextPool,
    ) -> PartSuppGenerator<'_> {
        PartSuppGenerator {
            scale_factor,
            part,
            part_count,
            text_pool,
        }
    }

    /// Return the row count for the given scale factor and generator part count
    pub fn calculate_row_count(scale_factor: f64, part: i32, part_count: i32) -> i64 {
        // Use the part generator's scale base for start/row calculation
        GenerateUtils::calculate_row_count(
            PartGenerator::SCALE_BASE,
            scale_factor,
            part,
            part_count,
        )
    }

    /// Returns an iterator over the part supplier rows
    pub fn iter(&self) -> PartSuppGeneratorIterator<'a> {
        let scale_base = PartGenerator::SCALE_BASE;

        PartSuppGeneratorIterator::new(
            self.text_pool,
            self.scale_factor,
            GenerateUtils::calculate_start_index(
                scale_base,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
            Self::calculate_row_count(self.scale_factor, self.part, self.part_count),
        )
    }
}

impl<'a> IntoIterator for PartSuppGenerator<'a> {
    type Item = PartSupp<'a>;
    type IntoIter = PartSuppGeneratorIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that generates PartSupp rows
#[derive(Debug)]
pub struct PartSuppGeneratorIterator<'a> {
    scale_factor: f64,
    start_index: i64,
    row_count: i64,

    available_quantity_random: RandomBoundedInt,
    supply_cost_random: RandomBoundedInt,
    comment_random: RandomText<'a>,

    index: i64,
    part_supplier_number: i32,
}

impl<'a> PartSuppGeneratorIterator<'a> {
    fn new(text_pool: &'a TextPool, scale_factor: f64, start_index: i64, row_count: i64) -> Self {
        let mut available_quantity_random = RandomBoundedInt::new_with_seeds_per_row(
            1671059989,
            PartSuppGenerator::AVAILABLE_QUANTITY_MIN,
            PartSuppGenerator::AVAILABLE_QUANTITY_MAX,
            PartSuppGenerator::SUPPLIERS_PER_PART,
        );
        let mut supply_cost_random = RandomBoundedInt::new_with_seeds_per_row(
            1051288424,
            PartSuppGenerator::SUPPLY_COST_MIN,
            PartSuppGenerator::SUPPLY_COST_MAX,
            PartSuppGenerator::SUPPLIERS_PER_PART,
        );
        let mut comment_random = RandomText::new_with_expected_row_count(
            1961692154,
            text_pool,
            PartSuppGenerator::COMMENT_AVERAGE_LENGTH as f64,
            PartSuppGenerator::SUPPLIERS_PER_PART,
        );

        // Advance all generators to the starting position
        available_quantity_random.advance_rows(start_index);
        supply_cost_random.advance_rows(start_index);
        comment_random.advance_rows(start_index);

        PartSuppGeneratorIterator {
            scale_factor,
            start_index,
            row_count,
            available_quantity_random,
            supply_cost_random,
            comment_random,
            index: 0,
            part_supplier_number: 0,
        }
    }

    /// Creates a part-supplier entry with the given part key
    fn make_part_supplier(&mut self, part_key: i64) -> PartSupp<'a> {
        let supplier_key = Self::select_part_supplier(
            part_key,
            self.part_supplier_number as i64,
            self.scale_factor,
        );

        let ps_availqty = self.available_quantity_random.next_value();
        let ps_supplycost = TPCHDecimal(self.supply_cost_random.next_value() as i64);
        let ps_comment = self.comment_random.next_value();

        PartSupp {
            ps_partkey: part_key,
            ps_suppkey: supplier_key,
            ps_availqty,
            ps_supplycost,
            ps_comment,
        }
    }

    /// Selects a supplier for a given part and supplier number
    pub fn select_part_supplier(part_key: i64, supplier_number: i64, scale_factor: f64) -> i64 {
        // Use supplier generator's scale base
        let supplier_count = (SupplierGenerator::SCALE_BASE as f64 * scale_factor) as i64;

        ((part_key
            + (supplier_number
                * ((supplier_count / PartSuppGenerator::SUPPLIERS_PER_PART as i64)
                    + ((part_key - 1) / supplier_count))))
            % supplier_count)
            + 1
    }
}

impl<'a> Iterator for PartSuppGeneratorIterator<'a> {
    type Item = PartSupp<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.row_count {
            return None;
        }

        let part_key = self.start_index + self.index + 1;
        let part_supplier = self.make_part_supplier(part_key);
        self.part_supplier_number += 1;

        // advance next row only when all suppliers for the part have been produced
        if self.part_supplier_number >= PartSuppGenerator::SUPPLIERS_PER_PART {
            self.available_quantity_random.row_finished();
            self.supply_cost_random.row_finished();
            self.comment_random.row_finished();

            self.index += 1;
            self.part_supplier_number = 0;
        }

        Some(part_supplier)
    }
}

/// A clerk name, formatted as `"Clerk#<n>"`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClerkName(i32);

impl ClerkName {
    /// Creates a new ClerkName with the given value
    pub fn new(value: i32) -> Self {
        ClerkName(value)
    }
}

impl fmt::Display for ClerkName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Clerk#{:09}", self.0)
    }
}

/// Order status (F=final, O=open, P=pending)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
pub enum OrderStatus {
    /// Fulfilled - all line items shipped
    Fulfilled,
    /// Open - no line items shipped
    Open,
    /// Partially fulfilled - some line items shipped
    Pending,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Fulfilled => "F",
            OrderStatus::Open => "O",
            OrderStatus::Pending => "P",
        }
    }
}

impl Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// The ORDERS table
///
/// The Display trait is implemented to format the line item data as a string
/// in the default TPC-H 'tbl' format.
///
/// ```text
/// 1|37|O|131251.81|1996-01-02|5-LOW|Clerk#000000951|0|nstructions sleep furiously among |
///  2|79|O|40183.29|1996-12-01|1-URGENT|Clerk#000000880|0| foxes. pending accounts at the pending, silent asymptot|
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Order<'a> {
    /// Primary key
    pub o_orderkey: i64,
    /// Foreign key to CUSTOMER
    pub o_custkey: i64,
    /// Order status (F=final, O=open, P=pending)
    pub o_orderstatus: OrderStatus,
    /// Order total price
    pub o_totalprice: TPCHDecimal,
    /// Order date
    pub o_orderdate: TPCHDate,
    /// Order priority
    pub o_orderpriority: &'a str,
    /// Clerk who processed the order.
    pub o_clerk: ClerkName,
    /// Order shipping priority
    pub o_shippriority: i32,
    /// Variable length comment
    pub o_comment: &'a str,
}

impl fmt::Display for Order<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.o_orderkey,
            self.o_custkey,
            self.o_orderstatus,
            self.o_totalprice,
            self.o_orderdate,
            self.o_orderpriority,
            self.o_clerk,
            self.o_shippriority,
            self.o_comment
        )
    }
}

/// Generator for Order table data
#[derive(Debug, Clone)]
pub struct OrderGenerator<'a> {
    scale_factor: f64,
    part: i32,
    part_count: i32,
    distributions: &'a Distributions,
    text_pool: &'a TextPool,
}

impl<'a> OrderGenerator<'a> {
    /// Base scale for order generation
    pub const SCALE_BASE: i32 = 1_500_000;

    // Constants for order generation
    const CUSTOMER_MORTALITY: i32 = 3; // portion with no orders
    const ORDER_DATE_MIN: i32 = dates::MIN_GENERATE_DATE;
    const ORDER_DATE_MAX: i32 =
        Self::ORDER_DATE_MIN + (dates::TOTAL_DATE_RANGE - LineItemGenerator::ITEM_SHIP_DAYS - 1);
    const CLERK_SCALE_BASE: i32 = 1000;

    const LINE_COUNT_MIN: i32 = 1;
    pub const LINE_COUNT_MAX: i32 = 7;

    const COMMENT_AVERAGE_LENGTH: i32 = 49;

    const ORDER_KEY_SPARSE_BITS: i32 = 2;
    const ORDER_KEY_SPARSE_KEEP: i32 = 3;
    /// Creates a new OrderGenerator with the given scale factor
    ///
    /// Note the generator's lifetime is `&'static`. See [`NationGenerator`] for
    /// more details.
    pub fn new(scale_factor: f64, part: i32, part_count: i32) -> OrderGenerator<'static> {
        // Note: use explicit lifetime to ensure this remains `&'static`
        Self::new_with_distributions_and_text_pool(
            scale_factor,
            part,
            part_count,
            Distributions::static_default(),
            TextPool::get_or_init_default(),
        )
    }

    /// Creates a OrderGenerator with specified distributions and text pool
    pub fn new_with_distributions_and_text_pool<'b>(
        scale_factor: f64,
        part: i32,
        part_count: i32,
        distributions: &'b Distributions,
        text_pool: &'b TextPool,
    ) -> OrderGenerator<'b> {
        OrderGenerator {
            scale_factor,
            part,
            part_count,
            distributions,
            text_pool,
        }
    }

    /// Return the row count for the given scale factor and generator part count
    pub fn calculate_row_count(scale_factor: f64, part: i32, part_count: i32) -> i64 {
        GenerateUtils::calculate_row_count(Self::SCALE_BASE, scale_factor, part, part_count)
    }

    /// Returns an iterator over the order rows
    pub fn iter(&self) -> OrderGeneratorIterator<'a> {
        OrderGeneratorIterator::new(
            self.distributions,
            self.text_pool,
            self.scale_factor,
            GenerateUtils::calculate_start_index(
                Self::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
            Self::calculate_row_count(self.scale_factor, self.part, self.part_count),
        )
    }

    /// Creates the order date random generator
    pub fn create_order_date_random() -> RandomBoundedInt {
        RandomBoundedInt::new(1066728069, Self::ORDER_DATE_MIN, Self::ORDER_DATE_MAX)
    }

    /// Creates the line count random generator
    pub fn create_line_count_random() -> RandomBoundedInt {
        RandomBoundedInt::new(1434868289, Self::LINE_COUNT_MIN, Self::LINE_COUNT_MAX)
    }

    /// Creates an order key from an index
    pub fn make_order_key(order_index: i64) -> i64 {
        let low_bits = order_index & ((1 << Self::ORDER_KEY_SPARSE_KEEP) - 1);

        let mut ok = order_index;
        ok >>= Self::ORDER_KEY_SPARSE_KEEP;
        ok <<= Self::ORDER_KEY_SPARSE_BITS;
        ok <<= Self::ORDER_KEY_SPARSE_KEEP;
        ok += low_bits;

        ok
    }
}

impl<'a> IntoIterator for OrderGenerator<'a> {
    type Item = Order<'a>;
    type IntoIter = OrderGeneratorIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that generates Order rows
#[derive(Debug)]
pub struct OrderGeneratorIterator<'a> {
    order_date_random: RandomBoundedInt,
    line_count_random: RandomBoundedInt,
    customer_key_random: RandomBoundedLong,
    order_priority_random: RandomString<'a>,
    clerk_random: RandomBoundedInt,
    comment_random: RandomText<'a>,

    // For line item simulation to determine order status
    line_quantity_random: RandomBoundedInt,
    line_discount_random: RandomBoundedInt,
    line_tax_random: RandomBoundedInt,
    line_part_key_random: RandomBoundedLong,
    line_ship_date_random: RandomBoundedInt,

    start_index: i64,
    row_count: i64,
    max_customer_key: i64,

    index: i64,
}
impl<'a> OrderGeneratorIterator<'a> {
    fn new(
        distributions: &'a Distributions,
        text_pool: &'a TextPool,
        scale_factor: f64,
        start_index: i64,
        row_count: i64,
    ) -> Self {
        let mut order_date_random = OrderGenerator::create_order_date_random();
        let mut line_count_random = OrderGenerator::create_line_count_random();

        let max_customer_key = (CustomerGenerator::SCALE_BASE as f64 * scale_factor) as i64;

        let mut customer_key_random =
            RandomBoundedLong::new(851767375, scale_factor >= 30000.0, 1, max_customer_key);

        let mut order_priority_random =
            RandomString::new(591449447, distributions.order_priority());

        let max_clerk = (scale_factor * OrderGenerator::CLERK_SCALE_BASE as f64)
            .max(OrderGenerator::CLERK_SCALE_BASE as f64) as i32;
        let mut clerk_random = RandomBoundedInt::new(1171034773, 1, max_clerk);

        let mut comment_random = RandomText::new(
            276090261,
            text_pool,
            OrderGenerator::COMMENT_AVERAGE_LENGTH as f64,
        );

        // For line item simulation
        let mut line_quantity_random = LineItemGenerator::create_quantity_random();
        let mut line_discount_random = LineItemGenerator::create_discount_random();
        let mut line_tax_random = LineItemGenerator::create_tax_random();
        let mut line_part_key_random = LineItemGenerator::create_part_key_random(scale_factor);
        let mut line_ship_date_random = LineItemGenerator::create_ship_date_random();

        // Advance all generators to the starting position
        order_date_random.advance_rows(start_index);
        line_count_random.advance_rows(start_index);
        customer_key_random.advance_rows(start_index);
        order_priority_random.advance_rows(start_index);
        clerk_random.advance_rows(start_index);
        comment_random.advance_rows(start_index);

        line_quantity_random.advance_rows(start_index);
        line_discount_random.advance_rows(start_index);
        line_tax_random.advance_rows(start_index);
        line_part_key_random.advance_rows(start_index);
        line_ship_date_random.advance_rows(start_index);

        OrderGeneratorIterator {
            order_date_random,
            line_count_random,
            customer_key_random,
            order_priority_random,
            clerk_random,
            comment_random,
            line_quantity_random,
            line_discount_random,
            line_tax_random,
            line_part_key_random,
            line_ship_date_random,
            start_index,
            row_count,
            max_customer_key,
            index: 0,
        }
    }

    /// Creates an order with the given index
    fn make_order(&mut self, index: i64) -> Order<'a> {
        let order_key = OrderGenerator::make_order_key(index);

        let order_date = self.order_date_random.next_value();

        // generate customer key, taking into account customer mortality rate
        let mut customer_key = self.customer_key_random.next_value();
        let mut delta = 1;
        while customer_key % OrderGenerator::CUSTOMER_MORTALITY as i64 == 0 {
            customer_key += delta;
            customer_key = customer_key.min(self.max_customer_key);
            delta *= -1;
        }

        let mut total_price = 0;
        let mut shipped_count = 0;

        let line_count = self.line_count_random.next_value();
        for _ in 0..line_count {
            let quantity = self.line_quantity_random.next_value();
            let discount = self.line_discount_random.next_value();
            let tax = self.line_tax_random.next_value();

            let part_key = self.line_part_key_random.next_value();

            let part_price = PartGeneratorIterator::calculate_part_price(part_key);
            let extended_price = part_price * quantity as i64;
            let discounted_price = extended_price * (100 - discount as i64);
            total_price += ((discounted_price / 100) * (100 + tax as i64)) / 100;

            let ship_date = self.line_ship_date_random.next_value() + order_date;
            if TPCHDate::is_in_past(ship_date) {
                shipped_count += 1;
            }
        }

        let order_status = if shipped_count == line_count {
            OrderStatus::Fulfilled
        } else if shipped_count > 0 {
            OrderStatus::Pending
        } else {
            OrderStatus::Open
        };

        let clerk_id = self.clerk_random.next_value();
        let clerk_name = ClerkName::new(clerk_id);

        Order {
            o_orderkey: order_key,
            o_custkey: customer_key,
            o_orderstatus: order_status,
            o_totalprice: TPCHDecimal(total_price),
            o_orderdate: TPCHDate::new(order_date),
            o_orderpriority: self.order_priority_random.next_value(),
            o_clerk: clerk_name,
            o_shippriority: 0, // Fixed value per TPC-H spec
            o_comment: self.comment_random.next_value(),
        }
    }
}

impl<'a> Iterator for OrderGeneratorIterator<'a> {
    type Item = Order<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.row_count {
            return None;
        }

        let order = self.make_order(self.start_index + self.index + 1);

        self.order_date_random.row_finished();
        self.line_count_random.row_finished();
        self.customer_key_random.row_finished();
        self.order_priority_random.row_finished();
        self.clerk_random.row_finished();
        self.comment_random.row_finished();

        self.line_quantity_random.row_finished();
        self.line_discount_random.row_finished();
        self.line_tax_random.row_finished();
        self.line_part_key_random.row_finished();
        self.line_ship_date_random.row_finished();

        self.index += 1;

        Some(order)
    }
}

/// The LINEITEM table
///
/// The Display trait is implemented to format the line item data as a string
/// in the default TPC-H 'tbl' format.
///
/// Example
/// ```text
/// 1|156|4|1|17|17954.55|0.04|0.02|N|O|1996-03-13|1996-02-12|1996-03-22|DELIVER IN PERSON|TRUCK|egular courts above the|
/// 1|68|9|2|36|34850.16|0.09|0.06|N|O|1996-04-12|1996-02-28|1996-04-20|TAKE BACK RETURN|MAIL|ly final dependencies: slyly bold |
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct LineItem<'a> {
    /// Foreign key to ORDERS
    pub l_orderkey: i64,
    /// Foreign key to PART
    pub l_partkey: i64,
    /// Foreign key to SUPPLIER
    pub l_suppkey: i64,
    /// Line item number within order
    pub l_linenumber: i32,
    /// Quantity ordered
    // TODO: Spec has this as decimal.
    pub l_quantity: i64,
    /// Extended price (l_quantity * p_retailprice)
    pub l_extendedprice: TPCHDecimal,
    /// Discount percentage
    pub l_discount: TPCHDecimal,
    /// Tax percentage
    pub l_tax: TPCHDecimal,
    /// Return flag (R=returned, A=accepted, null=pending)
    pub l_returnflag: &'a str,
    /// Line status (O=ordered, F=fulfilled)
    pub l_linestatus: &'static str,
    /// Date shipped
    pub l_shipdate: TPCHDate,
    /// Date committed to ship
    pub l_commitdate: TPCHDate,
    /// Date received
    pub l_receiptdate: TPCHDate,
    /// Shipping instructions
    pub l_shipinstruct: &'a str,
    /// Shipping mode
    pub l_shipmode: &'a str,
    /// Variable length comment
    pub l_comment: &'a str,
}

impl fmt::Display for LineItem<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.l_orderkey,
            self.l_partkey,
            self.l_suppkey,
            self.l_linenumber,
            self.l_quantity,
            self.l_extendedprice,
            self.l_discount,
            self.l_tax,
            self.l_returnflag,
            self.l_linestatus,
            self.l_shipdate,
            self.l_commitdate,
            self.l_receiptdate,
            self.l_shipinstruct,
            self.l_shipmode,
            self.l_comment
        )
    }
}

/// Generator for LineItem table data
#[derive(Debug, Clone)]
pub struct LineItemGenerator<'a> {
    scale_factor: f64,
    part: i32,
    part_count: i32,
    distributions: &'a Distributions,
    text_pool: &'a TextPool,
}

impl<'a> LineItemGenerator<'a> {
    // Constants for line item generation
    const QUANTITY_MIN: i32 = 1;
    const QUANTITY_MAX: i32 = 50;
    const TAX_MIN: TPCHDecimal = TPCHDecimal(0); // 0.00
    const TAX_MAX: TPCHDecimal = TPCHDecimal(8); // 0.08
    const DISCOUNT_MIN: TPCHDecimal = TPCHDecimal(0); // 0.00
    const DISCOUNT_MAX: TPCHDecimal = TPCHDecimal(10); // 0.10
    const PART_KEY_MIN: i32 = 1;

    const SHIP_DATE_MIN: i32 = 1;
    const SHIP_DATE_MAX: i32 = 121;
    const COMMIT_DATE_MIN: i32 = 30;
    const COMMIT_DATE_MAX: i32 = 90;
    const RECEIPT_DATE_MIN: i32 = 1;
    const RECEIPT_DATE_MAX: i32 = 30;

    pub const ITEM_SHIP_DAYS: i32 = Self::SHIP_DATE_MAX + Self::RECEIPT_DATE_MAX;

    const COMMENT_AVERAGE_LENGTH: i32 = 27;

    /// Creates a new LineItemGenerator with the given scale factor
    ///
    /// Note the generator's lifetime is `&'static`. See [`NationGenerator`] for
    /// more details.
    pub fn new(scale_factor: f64, part: i32, part_count: i32) -> LineItemGenerator<'static> {
        Self::new_with_distributions_and_text_pool(
            scale_factor,
            part,
            part_count,
            Distributions::static_default(),
            TextPool::get_or_init_default(),
        )
    }

    /// Creates a LineItemGenerator with specified distributions and text pool
    pub fn new_with_distributions_and_text_pool<'b>(
        scale_factor: f64,
        part: i32,
        part_count: i32,
        distributions: &'b Distributions,
        text_pool: &'b TextPool,
    ) -> LineItemGenerator<'b> {
        LineItemGenerator {
            scale_factor,
            part,
            part_count,
            distributions,
            text_pool,
        }
    }

    /// Returns an iterator over the line item rows
    pub fn iter(&self) -> LineItemGeneratorIterator<'a> {
        LineItemGeneratorIterator::new(
            self.distributions,
            self.text_pool,
            self.scale_factor,
            GenerateUtils::calculate_start_index(
                OrderGenerator::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
            GenerateUtils::calculate_row_count(
                OrderGenerator::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
        )
    }

    /// Creates a quantity random generator
    pub fn create_quantity_random() -> RandomBoundedInt {
        RandomBoundedInt::new_with_seeds_per_row(
            209208115,
            Self::QUANTITY_MIN,
            Self::QUANTITY_MAX,
            OrderGenerator::LINE_COUNT_MAX,
        )
    }

    /// Creates a discount random generator
    pub fn create_discount_random() -> RandomBoundedInt {
        RandomBoundedInt::new_with_seeds_per_row(
            554590007,
            Self::DISCOUNT_MIN.0 as i32,
            Self::DISCOUNT_MAX.0 as i32,
            OrderGenerator::LINE_COUNT_MAX,
        )
    }

    /// Creates a tax random generator
    pub fn create_tax_random() -> RandomBoundedInt {
        RandomBoundedInt::new_with_seeds_per_row(
            721958466,
            Self::TAX_MIN.0 as i32,
            Self::TAX_MAX.0 as i32,
            OrderGenerator::LINE_COUNT_MAX,
        )
    }

    /// Creates a part key random generator
    pub fn create_part_key_random(scale_factor: f64) -> RandomBoundedLong {
        // If scale_factor >= 30000, use long `RandomBoundedLong` otherwise
        // use `RandomBoundedInt` to avoid overflow.
        RandomBoundedLong::new_with_seeds_per_row(
            1808217256,
            scale_factor >= 30000.0,
            Self::PART_KEY_MIN as i64,
            (PartGenerator::SCALE_BASE as f64 * scale_factor) as i64,
            OrderGenerator::LINE_COUNT_MAX,
        )
    }

    /// Creates a ship date random generator
    pub fn create_ship_date_random() -> RandomBoundedInt {
        RandomBoundedInt::new_with_seeds_per_row(
            1769349045,
            Self::SHIP_DATE_MIN,
            Self::SHIP_DATE_MAX,
            OrderGenerator::LINE_COUNT_MAX,
        )
    }
}

impl<'a> IntoIterator for LineItemGenerator<'a> {
    type Item = LineItem<'a>;
    type IntoIter = LineItemGeneratorIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that generates LineItem rows
#[derive(Debug)]
pub struct LineItemGeneratorIterator<'a> {
    order_date_random: RandomBoundedInt,
    line_count_random: RandomBoundedInt,

    quantity_random: RandomBoundedInt,
    discount_random: RandomBoundedInt,
    tax_random: RandomBoundedInt,

    line_part_key_random: RandomBoundedLong,

    supplier_number_random: RandomBoundedInt,

    ship_date_random: RandomBoundedInt,
    commit_date_random: RandomBoundedInt,
    receipt_date_random: RandomBoundedInt,

    returned_flag_random: RandomString<'a>,
    ship_instructions_random: RandomString<'a>,
    ship_mode_random: RandomString<'a>,

    comment_random: RandomText<'a>,

    scale_factor: f64,
    start_index: i64,
    row_count: i64,

    index: i64,
    order_date: i32,
    line_count: i32,
    line_number: i32,
}

impl<'a> LineItemGeneratorIterator<'a> {
    fn new(
        distributions: &'a Distributions,
        text_pool: &'a TextPool,
        scale_factor: f64,
        start_index: i64,
        row_count: i64,
    ) -> Self {
        let mut order_date_random = OrderGenerator::create_order_date_random();
        let mut line_count_random = OrderGenerator::create_line_count_random();

        let mut quantity_random = LineItemGenerator::create_quantity_random();
        let mut discount_random = LineItemGenerator::create_discount_random();
        let mut tax_random = LineItemGenerator::create_tax_random();

        let mut line_part_key_random = LineItemGenerator::create_part_key_random(scale_factor);

        let mut supplier_number_random = RandomBoundedInt::new_with_seeds_per_row(
            2095021727,
            0,
            3,
            OrderGenerator::LINE_COUNT_MAX,
        );

        let mut ship_date_random = LineItemGenerator::create_ship_date_random();
        let mut commit_date_random = RandomBoundedInt::new_with_seeds_per_row(
            904914315,
            LineItemGenerator::COMMIT_DATE_MIN,
            LineItemGenerator::COMMIT_DATE_MAX,
            OrderGenerator::LINE_COUNT_MAX,
        );
        let mut receipt_date_random = RandomBoundedInt::new_with_seeds_per_row(
            373135028,
            LineItemGenerator::RECEIPT_DATE_MIN,
            LineItemGenerator::RECEIPT_DATE_MAX,
            OrderGenerator::LINE_COUNT_MAX,
        );

        let mut returned_flag_random = RandomString::new_with_expected_row_count(
            717419739,
            distributions.return_flags(),
            OrderGenerator::LINE_COUNT_MAX,
        );
        let mut ship_instructions_random = RandomString::new_with_expected_row_count(
            1371272478,
            distributions.ship_instructions(),
            OrderGenerator::LINE_COUNT_MAX,
        );
        let mut ship_mode_random = RandomString::new_with_expected_row_count(
            675466456,
            distributions.ship_modes(),
            OrderGenerator::LINE_COUNT_MAX,
        );
        let mut comment_random = RandomText::new_with_expected_row_count(
            1095462486,
            text_pool,
            LineItemGenerator::COMMENT_AVERAGE_LENGTH as f64,
            OrderGenerator::LINE_COUNT_MAX,
        );

        // Advance all generators to the starting position
        order_date_random.advance_rows(start_index);
        line_count_random.advance_rows(start_index);

        quantity_random.advance_rows(start_index);
        discount_random.advance_rows(start_index);
        tax_random.advance_rows(start_index);

        line_part_key_random.advance_rows(start_index);

        supplier_number_random.advance_rows(start_index);

        ship_date_random.advance_rows(start_index);
        commit_date_random.advance_rows(start_index);
        receipt_date_random.advance_rows(start_index);

        returned_flag_random.advance_rows(start_index);
        ship_instructions_random.advance_rows(start_index);
        ship_mode_random.advance_rows(start_index);

        comment_random.advance_rows(start_index);

        // generate information for initial order
        let order_date = order_date_random.next_value();
        let line_count = line_count_random.next_value() - 1;

        LineItemGeneratorIterator {
            order_date_random,
            line_count_random,
            quantity_random,
            discount_random,
            tax_random,
            line_part_key_random,
            supplier_number_random,
            ship_date_random,
            commit_date_random,
            receipt_date_random,
            returned_flag_random,
            ship_instructions_random,
            ship_mode_random,
            comment_random,
            scale_factor,
            start_index,
            row_count,
            index: 0,
            order_date,
            line_count,
            line_number: 0,
        }
    }

    /// Creates a line item with the given order index
    fn make_line_item(&mut self, order_index: i64) -> LineItem<'a> {
        let order_key = OrderGenerator::make_order_key(order_index);

        let quantity = self.quantity_random.next_value();
        let discount = self.discount_random.next_value();
        let tax = self.tax_random.next_value();

        let part_key = self.line_part_key_random.next_value();

        let supplier_number = self.supplier_number_random.next_value() as i64;
        let supplier_key = PartSuppGeneratorIterator::select_part_supplier(
            part_key,
            supplier_number,
            self.scale_factor,
        );

        let part_price = PartGeneratorIterator::calculate_part_price(part_key);
        let extended_price = part_price * quantity as i64;

        let mut ship_date = self.ship_date_random.next_value();
        ship_date += self.order_date;
        let mut commit_date = self.commit_date_random.next_value();
        commit_date += self.order_date;
        let mut receipt_date = self.receipt_date_random.next_value();
        receipt_date += ship_date;

        let returned_flag = if TPCHDate::is_in_past(receipt_date) {
            self.returned_flag_random.next_value()
        } else {
            "N"
        };

        let status = if TPCHDate::is_in_past(ship_date) {
            "F" // Fulfilled
        } else {
            "O" // Open
        };

        let ship_instructions = self.ship_instructions_random.next_value();
        let ship_mode = self.ship_mode_random.next_value();
        let comment = self.comment_random.next_value();

        LineItem {
            l_orderkey: order_key,
            l_partkey: part_key,
            l_suppkey: supplier_key,
            l_linenumber: (self.line_number + 1),
            l_quantity: quantity as i64,
            l_extendedprice: TPCHDecimal(extended_price),
            l_discount: TPCHDecimal(discount as i64),
            l_tax: TPCHDecimal(tax as i64),
            l_returnflag: returned_flag,
            l_linestatus: status,
            l_shipdate: TPCHDate::new(ship_date),
            l_commitdate: TPCHDate::new(commit_date),
            l_receiptdate: TPCHDate::new(receipt_date),
            l_shipinstruct: ship_instructions,
            l_shipmode: ship_mode,
            l_comment: comment,
        }
    }
}

impl<'a> Iterator for LineItemGeneratorIterator<'a> {
    type Item = LineItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.row_count {
            return None;
        }

        let line_item = self.make_line_item(self.start_index + self.index + 1);
        self.line_number += 1;

        // advance next row only when all lines for the order have been produced
        if self.line_number > self.line_count {
            self.order_date_random.row_finished();
            self.line_count_random.row_finished();

            self.quantity_random.row_finished();
            self.discount_random.row_finished();
            self.tax_random.row_finished();

            self.line_part_key_random.row_finished();
            self.supplier_number_random.row_finished();

            self.ship_date_random.row_finished();
            self.commit_date_random.row_finished();
            self.receipt_date_random.row_finished();

            self.returned_flag_random.row_finished();
            self.ship_instructions_random.row_finished();
            self.ship_mode_random.row_finished();

            self.comment_random.row_finished();

            self.index += 1;

            // generate information for next order
            self.line_count = self.line_count_random.next_value() - 1;
            self.order_date = self.order_date_random.next_value();
            self.line_number = 0;
        }

        Some(line_item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nation_generator() {
        let generator = NationGenerator::default();
        let nations: Vec<_> = generator.iter().collect();

        // TPC-H typically has 25 nations
        assert_eq!(nations.len(), 25);
    }

    #[test]
    fn test_region_generator() {
        let generator = RegionGenerator::default();
        let regions: Vec<_> = generator.iter().collect();

        // TPC-H typically has 5 regions
        assert_eq!(regions.len(), 5);
    }

    #[test]
    fn test_part_generation() {
        // Create a generator with a small scale factor
        let generator = PartGenerator::new(0.01, 1, 1);
        let parts: Vec<_> = generator.iter().collect();

        // Should have 0.01 * 200,000 = 2,000 parts
        assert_eq!(parts.len(), 2000);
    }

    #[test]
    fn test_calculate_part_price() {
        // Test with a few part keys
        assert_eq!(PartGeneratorIterator::calculate_part_price(1), 90100);
        assert_eq!(PartGeneratorIterator::calculate_part_price(10), 91001);
        assert_eq!(PartGeneratorIterator::calculate_part_price(100), 100010);
        assert_eq!(PartGeneratorIterator::calculate_part_price(1000), 90100);
    }

    #[test]
    fn test_supplier_generation() {
        // Create a generator with a small scale factor
        let generator = SupplierGenerator::new(0.01, 1, 1);
        let suppliers: Vec<_> = generator.iter().collect();

        // Should have 0.01 * 10,000 = 100 suppliers
        assert_eq!(suppliers.len(), 100);

        // Check first supplier
        let first = &suppliers[0];
        assert_eq!(first.s_suppkey, 1);
        assert_eq!(first.to_string(), "1|Supplier#000000001| N kD4on9OM Ipw3,gf0JBoQDd7tgrzrddZ|17|27-918-335-1736|5755.94|each slyly above the careful|")
    }

    #[test]
    fn test_customer_generation() {
        // Create a generator with a small scale factor
        let generator = CustomerGenerator::new(0.01, 1, 1);
        let customers: Vec<_> = generator.iter().collect();

        // Should have 0.01 * 150,000 = 1,500 customers
        assert_eq!(customers.len(), 1500);

        // Check first customer
        let first = &customers[0];
        assert_eq!(first.c_custkey, 1);
        // typos:off
        assert_eq!(first.to_string(), "1|Customer#000000001|IVhzIApeRb ot,c,E|15|25-989-741-2988|711.56|BUILDING|to the even, regular platelets. regular, ironic epitaphs nag e|");
        // typos:on

        // Check market segment distribution
        let market_segments: std::collections::HashSet<_> =
            customers.iter().map(|c| &c.c_mktsegment).collect();

        // Should have multiple different market segments
        assert!(market_segments.len() > 1);

        // Check nation key distribution
        let nation_keys: std::collections::HashSet<_> =
            customers.iter().map(|c| c.c_nationkey).collect();

        // Should have multiple different nation keys
        assert!(nation_keys.len() > 1);
    }

    #[test]
    fn test_part_supplier_generation() {
        // Create a generator with a small scale factor
        let generator = PartSuppGenerator::new(0.01, 1, 1);
        let part_suppliers: Vec<_> = generator.iter().collect();

        // Should have 0.01 * 200,000 * 4 = 8,000 part-supplier relationships
        assert_eq!(part_suppliers.len(), 8000);

        // Each part should have SUPPLIERS_PER_PART suppliers
        let part_keys: std::collections::HashSet<_> =
            part_suppliers.iter().map(|ps| ps.ps_partkey).collect();

        assert_eq!(part_keys.len(), 2000); // 8,000 / 4 = 2,000 parts

        // Check first part supplier
        let first = &part_suppliers[0];
        assert_eq!(first.ps_partkey, 1);
        assert_ne!(first.ps_suppkey, 0); // Should have a valid supplier key
        assert!(first.ps_availqty > 0);
        assert!(first.ps_supplycost > TPCHDecimal::ZERO);
        assert!(!first.ps_comment.is_empty());

        // Verify supplier distribution
        let suppliers_for_first_part: Vec<_> = part_suppliers
            .iter()
            .filter(|ps| ps.ps_partkey == 1)
            .map(|ps| ps.ps_suppkey)
            .collect();

        assert_eq!(
            suppliers_for_first_part.len(),
            PartSuppGenerator::SUPPLIERS_PER_PART as usize
        );

        // Supplier keys should be unique for each part
        let unique_suppliers: std::collections::HashSet<_> =
            suppliers_for_first_part.iter().collect();
        assert_eq!(
            unique_suppliers.len(),
            PartSuppGenerator::SUPPLIERS_PER_PART as usize
        );
    }

    #[test]
    fn test_select_part_supplier() {
        // Test the supplier selection logic for consistency
        let scale_factor = 1.0;

        // Same part with different supplier numbers should yield different suppliers
        let supplier1 = PartSuppGeneratorIterator::select_part_supplier(1, 0, scale_factor);
        let supplier2 = PartSuppGeneratorIterator::select_part_supplier(1, 1, scale_factor);
        let supplier3 = PartSuppGeneratorIterator::select_part_supplier(1, 2, scale_factor);
        let supplier4 = PartSuppGeneratorIterator::select_part_supplier(1, 3, scale_factor);

        // All suppliers should be different
        let suppliers = vec![supplier1, supplier2, supplier3, supplier4];
        let unique_suppliers: std::collections::HashSet<_> = suppliers.iter().collect();
        assert_eq!(
            unique_suppliers.len(),
            PartSuppGenerator::SUPPLIERS_PER_PART as usize
        );

        // All supplier keys should be within valid range (1 to supplier_count)
        let supplier_count = (SupplierGenerator::SCALE_BASE as f64 * scale_factor) as i64;
        for supplier in suppliers {
            assert!(supplier >= 1 && supplier <= supplier_count);
        }
    }

    #[test]
    fn test_order_generation() {
        // Create a generator with a small scale factor
        let generator = OrderGenerator::new(0.01, 1, 1);
        let orders: Vec<_> = generator.iter().collect();

        // Should have 0.01 * 1,500,000 = 15,000 orders
        assert_eq!(orders.len(), 15000);

        // Check first order
        let first = &orders[0];
        assert_eq!(first.o_orderkey, OrderGenerator::make_order_key(1));
        assert!(first.o_custkey > 0);
        assert!(first.o_totalprice > TPCHDecimal::ZERO);

        // Check order status distribution
        let status_counts =
            orders
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, order| {
                    *acc.entry(&order.o_orderstatus).or_insert(0) += 1;
                    acc
                });

        // Should have multiple order statuses
        assert!(status_counts.len() >= 2);

        // Check customer key distribution - no customer with mortality factor
        assert!(orders
            .iter()
            .all(|o| o.o_custkey % OrderGenerator::CUSTOMER_MORTALITY as i64 != 0));

        // Check order key sparsity
        for (i, order) in orders.iter().enumerate() {
            assert_eq!(
                order.o_orderkey,
                OrderGenerator::make_order_key(i as i64 + 1)
            );
        }
    }

    #[test]
    fn test_make_order_key() {
        // Test order key generation logic
        assert_eq!(OrderGenerator::make_order_key(1), 1); // Low values are preserved
        assert_eq!(OrderGenerator::make_order_key(8), 32); // 8 becomes 1000000
        assert_eq!(OrderGenerator::make_order_key(9), 32 + 1); // 9 becomes 1000001
        assert_eq!(OrderGenerator::make_order_key(10), 32 + 2); // 10 becomes 1000010
    }

    #[test]
    fn test_line_item_generation() {
        // Create a generator with a small scale factor
        let generator = LineItemGenerator::new(0.01, 1, 1);
        let line_items: Vec<_> = generator.iter().collect();

        // Check first line item
        let first = &line_items[0];
        assert_eq!(first.l_orderkey, OrderGenerator::make_order_key(1));
        assert_eq!(first.l_linenumber, 1);
        assert!(first.l_partkey > 0);
        assert!(first.l_suppkey > 0);

        assert!(first.l_quantity >= LineItemGenerator::QUANTITY_MIN as i64);
        assert!(first.l_quantity <= LineItemGenerator::QUANTITY_MAX as i64);

        assert!(first.l_discount >= LineItemGenerator::DISCOUNT_MIN);
        assert!(first.l_discount <= LineItemGenerator::DISCOUNT_MAX);

        assert!(first.l_tax >= LineItemGenerator::TAX_MIN);
        assert!(first.l_tax <= LineItemGenerator::TAX_MAX);

        // Verify line numbers are sequential per order
        let mut order_lines = std::collections::HashMap::new();
        for line in &line_items {
            order_lines
                .entry(line.l_orderkey)
                .or_insert_with(Vec::new)
                .push(line.l_linenumber);
        }

        // Check each order's line numbers
        for (_, lines) in order_lines {
            let mut sorted_lines = lines.clone();
            sorted_lines.sort();

            // Line numbers should start at 1 and be sequential
            for (i, line_num) in sorted_lines.iter().enumerate() {
                assert_eq!(*line_num, (i + 1) as i32);
            }
        }

        // Verify return flags and line status distributions
        let return_flags: std::collections::HashSet<_> =
            line_items.iter().map(|l| &l.l_returnflag).collect();

        assert!(return_flags.len() > 1);

        let line_statuses: std::collections::HashSet<_> =
            line_items.iter().map(|l| &l.l_linestatus).collect();

        assert!(!line_statuses.is_empty());
    }

    #[test]
    fn test_large_scale_partitioned_line_item_generation() {
        let line_item_count = LineItemGenerator::new(100_000.0, 1, 1_000_000_000)
            .iter()
            .count();

        assert_eq!(line_item_count, 586);
    }

    #[test]
    fn check_iter_static_lifetimes() {
        // Lifetimes of iterators should be independent of the generator that
        // created it. This test case won't compile if that's not the case.

        let _iter: NationGeneratorIterator<'static> = NationGenerator::default().iter();
        let _iter: RegionGeneratorIterator<'static> = RegionGenerator::default().iter();
        let _iter: PartGeneratorIterator<'static> = PartGenerator::new(0.1, 1, 1).iter();
        let _iter: SupplierGeneratorIterator<'static> = SupplierGenerator::new(0.1, 1, 1).iter();
        let _iter: CustomerGeneratorIterator<'static> = CustomerGenerator::new(0.1, 1, 1).iter();
        let _iter: PartSuppGeneratorIterator<'static> = PartSuppGenerator::new(0.1, 1, 1).iter();
        let _iter: OrderGeneratorIterator<'static> = OrderGenerator::new(0.1, 1, 1).iter();
        let _iter: LineItemGeneratorIterator<'static> = LineItemGenerator::new(0.1, 1, 1).iter();
    }
}
