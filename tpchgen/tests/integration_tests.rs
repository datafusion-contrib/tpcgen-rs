//! Consistence and conformance test suite that runs against Trino's TPCH
//! Java implementation.
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tpchgen::generators::{
    CustomerGenerator, LineItemGenerator, NationGenerator, OrderGenerator, PartGenerator,
    PartSuppGenerator, RegionGenerator, SupplierGenerator,
};

fn read_tbl_gz<P: AsRef<Path>>(path: P) -> Vec<String> {
    let file = File::open(path).expect("Failed to open file");
    let gz = GzDecoder::new(file);
    let reader = BufReader::new(gz);
    reader
        .lines()
        .collect::<Result<_, _>>()
        .expect("Failed to read lines")
}

fn test_generator<T, I>(iter: I, reference_path: &str, transform_fn: impl Fn(T) -> String)
where
    I: Iterator<Item = T>,
{
    let mut dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    dir.push(reference_path);

    // Read reference data.
    let reference_data = read_tbl_gz(dir);

    // Generate data using our own generators.
    let generated_data: Vec<String> = iter.map(transform_fn).collect();

    // Compare that we have the same number of records.
    assert_eq!(
        reference_data.len(),
        generated_data.len(),
        "Number of records doesn't match for {}. Reference: {}, Generated: {}",
        reference_path,
        reference_data.len(),
        generated_data.len()
    );

    for (i, (reference, generated)) in reference_data.iter().zip(generated_data.iter()).enumerate()
    {
        assert_eq!(
            reference, generated,
            "Record {} doesn't match for {}.\nReference: {}\nGenerated: {}",
            i, reference_path, reference, generated
        );
    }
}

#[test]
fn test_nation_sf_0_001() {
    let _sf = 0.001;
    let generator = NationGenerator::default();
    test_generator(generator.iter(), "data/sf-0.001/nation.tbl.gz", |nation| {
        nation.to_string()
    });
}

#[test]
fn test_region_sf_0_001() {
    let _sf = 0.001;
    let generator = RegionGenerator::default();
    test_generator(generator.iter(), "data/sf-0.001/region.tbl.gz", |region| {
        region.to_string()
    });
}

#[test]
fn test_part_sf_0_001() {
    let sf = 0.001;
    let generator = PartGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.001/part.tbl.gz", |part| {
        part.to_string()
    });
}

#[test]
fn test_supplier_sf_0_001() {
    let sf = 0.001;
    let generator = SupplierGenerator::new(sf, 1, 1);
    test_generator(
        generator.iter(),
        "data/sf-0.001/supplier.tbl.gz",
        |supplier| supplier.to_string(),
    );
}

#[test]
fn test_partsupp_sf_0_001() {
    let sf = 0.001;
    let generator = PartSuppGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.001/partsupp.tbl.gz", |ps| {
        ps.to_string()
    });
}

#[test]
fn test_customer_sf_0_001() {
    let sf = 0.001;
    let generator = CustomerGenerator::new(sf, 1, 1);
    test_generator(
        generator.iter(),
        "data/sf-0.001/customer.tbl.gz",
        |customer| customer.to_string(),
    );
}

#[test]
fn test_orders_sf_0_001() {
    let sf = 0.001;
    let generator = OrderGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.001/orders.tbl.gz", |order| {
        order.to_string()
    });
}

#[test]
fn test_lineitem_sf_0_001() {
    let sf = 0.001;
    let generator = LineItemGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.001/lineitem.tbl.gz", |item| {
        item.to_string()
    });
}

#[test]
fn test_nation_sf_0_01() {
    let sf = 0.01;
    let generator = NationGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.01/nation.tbl.gz", |nation| {
        nation.to_string()
    });
}

#[test]
fn test_region_sf_0_01() {
    let sf = 0.01;
    let generator = RegionGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.01/region.tbl.gz", |region| {
        region.to_string()
    });
}

#[test]
fn test_part_sf_0_01() {
    let sf = 0.01;
    let generator = PartGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.01/part.tbl.gz", |part| {
        part.to_string()
    });
}

#[test]
fn test_supplier_sf_0_01() {
    let sf = 0.01;
    let generator = SupplierGenerator::new(sf, 1, 1);
    test_generator(
        generator.iter(),
        "data/sf-0.01/supplier.tbl.gz",
        |supplier| supplier.to_string(),
    );
}

#[test]
fn test_partsupp_sf_0_01() {
    let sf = 0.01;
    let generator = PartSuppGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.01/partsupp.tbl.gz", |ps| {
        ps.to_string()
    });
}

#[test]
fn test_customer_sf_0_01() {
    let sf = 0.01;
    let generator = CustomerGenerator::new(sf, 1, 1);
    test_generator(
        generator.iter(),
        "data/sf-0.01/customer.tbl.gz",
        |customer| customer.to_string(),
    );
}

#[test]
fn test_orders_sf_0_01() {
    let sf = 0.01;
    let generator = OrderGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.01/orders.tbl.gz", |order| {
        order.to_string()
    })
}

#[test]
fn test_lineitem_sf_0_01() {
    let sf = 0.01;
    let generator = LineItemGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.01/lineitem.tbl.gz", |item| {
        item.to_string()
    })
}

struct TestIntoIterator<G>
where
    G: IntoIterator,
    G::Item: std::fmt::Display,
{
    generator: Option<G>,
}

impl<G> TestIntoIterator<G>
where
    G: IntoIterator,
    G::Item: std::fmt::Display,
{
    fn new(generator: G) -> Self {
        Self {
            generator: Some(generator),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_string_vec(&mut self, take_num: i32) -> Vec<String> {
        if let Some(generator) = self.generator.take() {
            generator
                .into_iter()
                .take(take_num as usize)
                .map(|item| item.to_string())
                .collect()
        } else {
            vec![]
        }
    }
}

#[test]
fn test_nation_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(NationGenerator::default())
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = NationGenerator::default();
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}

#[test]
fn test_region_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(RegionGenerator::default())
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = RegionGenerator::default();
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}

#[test]
fn test_part_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(PartGenerator::new(0.01, 1, 1))
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = PartGenerator::new(0.01, 1, 1);
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}

#[test]
fn test_supplier_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(SupplierGenerator::new(0.01, 1, 1))
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = SupplierGenerator::new(0.01, 1, 1);
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}

#[test]
fn test_partsupp_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(PartSuppGenerator::new(0.01, 1, 1))
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = PartSuppGenerator::new(0.01, 1, 1);
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}

#[test]
fn test_customer_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(CustomerGenerator::new(0.01, 1, 1))
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = CustomerGenerator::new(0.01, 1, 1);
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}

#[test]
fn test_orders_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(OrderGenerator::new(0.01, 1, 1))
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = OrderGenerator::new(0.01, 1, 1);
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}

#[test]
fn test_lineitem_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(LineItemGenerator::new(0.01, 1, 1))
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = LineItemGenerator::new(0.01, 1, 1);
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}

/// Formats a row with its `write_tbl` method, without the trailing newline.
fn write_tbl_line(write_tbl: impl FnOnce(&mut Vec<u8>)) -> String {
    let mut buf = Vec::new();
    write_tbl(&mut buf);
    assert_eq!(
        buf.pop(),
        Some(b'\n'),
        "write_tbl must end the row with a newline"
    );
    String::from_utf8(buf).expect("write_tbl must write UTF-8")
}

/// Checks that `write_tbl` writes the reference data for every table at the
/// given scale factor.
macro_rules! test_write_tbl {
    ($name:ident, $sf:expr, $dir:literal) => {
        #[test]
        fn $name() {
            let sf = $sf;
            let path = |table: &str| format!("data/{}/{table}.tbl.gz", $dir);
            test_generator(
                NationGenerator::new(sf, 1, 1).iter(),
                &path("nation"),
                |r| write_tbl_line(|buf| r.write_tbl(buf)),
            );
            test_generator(
                RegionGenerator::new(sf, 1, 1).iter(),
                &path("region"),
                |r| write_tbl_line(|buf| r.write_tbl(buf)),
            );
            test_generator(PartGenerator::new(sf, 1, 1).iter(), &path("part"), |r| {
                write_tbl_line(|buf| r.write_tbl(buf))
            });
            test_generator(
                SupplierGenerator::new(sf, 1, 1).iter(),
                &path("supplier"),
                |r| write_tbl_line(|buf| r.write_tbl(buf)),
            );
            test_generator(
                PartSuppGenerator::new(sf, 1, 1).iter(),
                &path("partsupp"),
                |r| write_tbl_line(|buf| r.write_tbl(buf)),
            );
            test_generator(
                CustomerGenerator::new(sf, 1, 1).iter(),
                &path("customer"),
                |r| write_tbl_line(|buf| r.write_tbl(buf)),
            );
            test_generator(OrderGenerator::new(sf, 1, 1).iter(), &path("orders"), |r| {
                write_tbl_line(|buf| r.write_tbl(buf))
            });
            test_generator(
                LineItemGenerator::new(sf, 1, 1).iter(),
                &path("lineitem"),
                |r| write_tbl_line(|buf| r.write_tbl(buf)),
            );
        }
    };
}

test_write_tbl!(test_write_tbl_sf_0_001, 0.001, "sf-0.001");
test_write_tbl!(test_write_tbl_sf_0_01, 0.01, "sf-0.01");
