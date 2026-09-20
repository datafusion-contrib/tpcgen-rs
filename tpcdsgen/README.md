# TPC-DS Data Generator Crate

This crate provides the core data generator logic for TPC-DS.

## Usage

```bash
# Build the unified generator CLI
cargo build --locked --release -p tpcgen-cli

# Generate all tables at scale factor 1 (default)
./target/release/tpcgen-cli tpcds dat

# Generate all tables at scale factor 10
./target/release/tpcgen-cli tpcds dat --scale-factor 10

# Generate specific table
./target/release/tpcgen-cli tpcds dat --tables store_sales --scale-factor 10

# Generate to a specific directory
./target/release/tpcgen-cli tpcds dat --scale-factor 10 --output-dir /path/to/output
```

## Generating Fixtures

Fixtures are pre-generated TPC-DS data files used for conformance testing.

### Directory Structure

```
tpcgen-cli/tests/fixtures/tpcds/
├── scale-1-trino/    # Java reference fixtures (`--compat trino`)
├── scale-1-c/       # C dsdgen reference fixtures (`--compat c`)
└── scale-10-trino/   # higher scale factors as needed
```

### Conformance Testing

`tpcdsgen` ships with two conformance suites, both implemented as shell
scripts that do byte-for-byte (MD5) comparison of `.dat` output. See
[scripts/README.md](../tpcgen-cli/scripts/tpcds/README.md) for full details.

**vs. Java / Trino reference (default, `--compat trino`):**

```bash
# Default (MD5-only): no Java setup required.
./tpcgen-cli/scripts/tpcds/compare-all-tables.sh --scale 1

# Byte-for-byte (--full): one-time Java setup + fixture generation.
./tpcgen-cli/scripts/tpcds/bootstrap-trino.sh
./tpcgen-cli/scripts/tpcds/generate-fixtures.sh
./tpcgen-cli/scripts/tpcds/compare-all-tables.sh --scale 1 --full
```

**vs. C dsdgen reference (`--compat c`):**

```bash
# Default (MD5-only): no download needed.
./tpcgen-cli/scripts/tpcds/compare-all-tables.sh --compat c --scale 1

# Byte-for-byte (--full): one-time data download from
# https://github.com/alamb/tpcds-data into tpcgen-cli/tests/fixtures/tpcds/scale-N-c/.
./tpcgen-cli/scripts/tpcds/generate-fixtures.sh --compat c --scale 1
./tpcgen-cli/scripts/tpcds/compare-all-tables.sh --compat c --scale 1 --full
```

Both suites also support comparing a single table:

```bash
./tpcgen-cli/scripts/tpcds/compare-table.sh reason                       # MD5-only, vs. Java
./tpcgen-cli/scripts/tpcds/compare-table.sh reason --compat c            # MD5-only, vs. C dsdgen
./tpcgen-cli/scripts/tpcds/compare-table.sh reason --full                # byte-for-byte, vs. Java
```

### Verifying Fixtures with MD5SUMS

Each fixture directory contains an `MD5SUMS` file for verification.

**On Linux:**
```bash
cd tpcgen-cli/tests/fixtures/tpcds/scale-1-trino
md5sum -c MD5SUMS
```

**On macOS:**
```bash
cd tpcgen-cli/tests/fixtures/tpcds/scale-1-trino
while read hash file; do
  [[ $(md5 -q "$file") == "$hash" ]] && echo "$file: OK" || echo "$file: FAILED"
done < MD5SUMS
```

## Known Bugs

The TPC-DS reference implementation contains several bugs that must be replicated for benchmark compliance.
These bugs originated in the C implementation and were faithfully reproduced in the Java port. Our Rust implementation
also replicates these bugs to ensure byte-for-byte compatibility with the reference implementation.

See [BUGS.md](BUGS.md) for a detailed list of documented bugs, more will be added.


## TPC-DS Reference MD5 Hashes

These are the canonical MD5 hashes the Rust implementation is verified
against. They are committed alongside the test fixtures so the
conformance scripts can do an MD5-only check without downloading or
re-generating the full reference data.

### Java / Trino reference (`--compat trino`)

Generated locally from `java -jar tpcds-*.jar --scale N`:

- Scale 1:  [`tpcgen-cli/tests/fixtures/tpcds/scale-1-trino/MD5SUMS`](tpcgen-cli/tests/fixtures/tpcds/scale-1-trino/MD5SUMS)
- Scale 10: [`tpcgen-cli/tests/fixtures/tpcds/scale-10-trino/MD5SUMS`](tpcgen-cli/tests/fixtures/tpcds/scale-10-trino/MD5SUMS)

### C `dsdgen` reference (`--compat c`)

Sourced from [alamb/tpcds-data](https://github.com/alamb/tpcds-data#md5-checksums):

- Scale 1:  [`tpcgen-cli/tests/fixtures/tpcds/scale-1-c/MD5SUMS`](tpcgen-cli/tests/fixtures/tpcds/scale-1-c/MD5SUMS)
- Scale 2:  [`tpcgen-cli/tests/fixtures/tpcds/scale-2-c/MD5SUMS`](tpcgen-cli/tests/fixtures/tpcds/scale-2-c/MD5SUMS)
- Scale 5:  [`tpcgen-cli/tests/fixtures/tpcds/scale-5-c/MD5SUMS`](tpcgen-cli/tests/fixtures/tpcds/scale-5-c/MD5SUMS)
- Scale 10: [`tpcgen-cli/tests/fixtures/tpcds/scale-10-c/MD5SUMS`](tpcgen-cli/tests/fixtures/tpcds/scale-10-c/MD5SUMS)

`dbgen_version.dat` contains a generation timestamp and will differ
between runs; the conformance suite excludes it from comparison.

## Verification

To verify the Rust implementation matches:

```bash
# Verify at scale 1
./tpcgen-cli/scripts/tpcds/compare-all-tables.sh --scale 1

# Verify at scale 10
./tpcgen-cli/scripts/tpcds/compare-all-tables.sh --scale 10
```
