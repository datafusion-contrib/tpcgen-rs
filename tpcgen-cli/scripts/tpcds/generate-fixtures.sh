#!/usr/bin/env bash
#
# generate-fixtures.sh — Generate reference TPC-DS fixtures used by the
# conformance suite (compare-table.sh / compare-all-tables.sh).
#
# Please see print_usage() below for details.

set -euo pipefail

print_usage() {
    cat << 'EOF'
generate-fixtures.sh — Generate reference TPC-DS fixtures.

Two reference implementations are supported, selected by --compat:

    --compat trino  (default)
        Runs the Java / Trino TPC-DS implementation (set up by
        ./scripts/tpcds/bootstrap-trino.sh) and writes the resulting *.dat files
        into tests/fixtures/tpcds/scale-N-trino/. These are the "golden reference"
        the Rust port targets byte-for-byte.

    --compat c
        Downloads pre-generated C `dsdgen` reference data from
        https://github.com/alamb/tpcds-data (branch sfN; one branch per
        scale factor). The branch is cloned with --depth 1, re-assembled
        from split bzip2 tarballs, and extracted into
        tests/fixtures/tpcds/scale-N-c/. No local C toolchain needed.

Usage:
    generate-fixtures.sh [OPTIONS] [TABLES...]

Options:
    --compat trino|c    Reference implementation (default: trino).
    --scale N           Scale factor (default: 1).
    --quiet             Quiet mode (minimal output).
    --rebuild           --compat c only: re-download and re-extract even
                        if fixtures already exist.
    --verify            --compat c only: only check that fixtures look
                        sane; do not download.
    --help              Show this help message.

Arguments:
    TABLES              --compat trino only: space-separated list of table
                        names to generate. If omitted, generates all 25.
                        Not meaningful for --compat c (the published
                        archive includes all 25 tables together).

Environment variables:
    TPCDS_C_DATA_REPO   Override the C reference data repo URL.
                        Default: https://github.com/alamb/tpcds-data.git
    TPCDS_SCALE         Default scale factor (overridden by --scale).
    TPCDS_COMPAT        Default compat mode  (overridden by --compat).

Requirements:
    --compat trino: Java 11+, Maven (a built tpcds-*.jar; see bootstrap-trino.sh)
    --compat c    : git, tar, bzip2

Output:
    tests/fixtures/tpcds/scale-N-trino/<table>.dat — pipe-delimited, trailing |.
    tests/fixtures/tpcds/scale-N-c/<table>.dat    — same format, C dsdgen origin.
    Files are gitignored; regenerate as needed.

Examples:
    # Java reference, all 25 tables at scale 1 (default).
    ./scripts/tpcds/generate-fixtures.sh

    # Java reference, scale 10, two specific tables.
    ./scripts/tpcds/generate-fixtures.sh --scale 10 call_center warehouse

    # C dsdgen reference, scale 1.
    ./scripts/tpcds/generate-fixtures.sh --compat c

    # C dsdgen reference, scale 2, force re-download.
    ./scripts/tpcds/generate-fixtures.sh --compat c --scale 2 --rebuild

See scripts/README.md for the full conformance-testing workflow.
EOF
}

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TRINO_DIR="$PROJECT_ROOT/../tpcds"

# Configuration (overridable by flags / env vars)
SCALE_FACTOR=${TPCDS_SCALE:-1}
COMPAT=${TPCDS_COMPAT:-trino}
QUIET=0
FORCE_REBUILD=0
VERIFY_ONLY=0

# alamb/tpcds-data repository (for --compat c).
TPCDS_DATA_REPO="${TPCDS_C_DATA_REPO:-https://github.com/alamb/tpcds-data.git}"

# All TPC-DS tables (25 tables).
ALL_TABLES=(
    "call_center"
    "catalog_page"
    "catalog_returns"
    "catalog_sales"
    "customer"
    "customer_address"
    "customer_demographics"
    "date_dim"
    "dbgen_version"
    "household_demographics"
    "income_band"
    "inventory"
    "item"
    "promotion"
    "reason"
    "ship_mode"
    "store"
    "store_returns"
    "store_sales"
    "time_dim"
    "warehouse"
    "web_page"
    "web_returns"
    "web_sales"
    "web_site"
)

# Logging
log_info() {
    if [[ $QUIET -eq 0 ]]; then
        echo -e "${BLUE}[INFO]${NC} $*"
    fi
}

log_success() {
    if [[ $QUIET -eq 0 ]]; then
        echo -e "${GREEN}[SUCCESS]${NC} $*"
    fi
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" >&2
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

# -----------------------------------------------------------------------------
# --compat trino (Java) helpers
# -----------------------------------------------------------------------------

find_java_jar() {
    local jar_file
    jar_file=$(find "$TRINO_DIR/target" -name "tpcds-*-jar-with-dependencies.jar" 2>/dev/null | head -1)
    if [[ -z "$jar_file" ]]; then
        return 1
    fi
    echo "$jar_file"
}

ensure_java_build() {
    log_info "Checking Java implementation..."
    if ! find_java_jar >/dev/null 2>&1; then
        # bootstrap-trino.sh owns the whole Java setup (clone, patch,
        # mvn package); a bare mvn build here fails on fresh checkouts
        # where $TRINO_DIR does not exist yet.
        log_warn "Java JAR not found. Bootstrapping the Java implementation..."
        if ! "$SCRIPT_DIR/bootstrap-trino.sh"; then
            log_error "Failed to bootstrap the Java implementation"
            log_error "See $SCRIPT_DIR/bootstrap-trino.sh --help"
            exit 1
        fi
        if ! find_java_jar >/dev/null 2>&1; then
            log_error "Bootstrap completed but no tpcds-*-jar-with-dependencies.jar found in $TRINO_DIR/target"
            exit 1
        fi
        log_success "Java implementation built successfully"
    else
        log_info "Java JAR found: $(find_java_jar)"
    fi
}

generate_java_table() {
    local table=$1
    local fixture_dir=$2
    local jar_file
    jar_file=$(find_java_jar)

    log_info "Generating $table..."

    local temp_dir
    temp_dir=$(mktemp -d)

    if java -jar "$jar_file" \
        --table "$table" \
        --scale "$SCALE_FACTOR" \
        --overwrite \
        --directory "$temp_dir" \
        >/dev/null 2>&1; then

        local output_file="$temp_dir/${table}.dat"
        if [[ -f "$output_file" && -s "$output_file" ]]; then
            mv "$output_file" "$fixture_dir/"
            local file_size row_count
            file_size=$(du -h "$fixture_dir/${table}.dat" | cut -f1)
            row_count=$(wc -l < "$fixture_dir/${table}.dat" | tr -d ' ')
            log_success "$table generated: $row_count rows, $file_size"
            rm -rf "$temp_dir"
            return 0
        else
            # An empty file means the Java generator silently produced no
            # rows (seen with jars built from upstream HEAD on JDK 25) —
            # never let that become a fixture.
            log_error "Java generator produced no output for $table (missing or empty $output_file)"
            rm -rf "$temp_dir"
            return 1
        fi
    else
        log_error "Failed to generate $table"
        rm -rf "$temp_dir"
        return 1
    fi
}

generate_trino_fixtures() {
    local fixture_dir=$1
    shift
    local tables_to_generate=("$@")

    log_info "========================================="
    log_info "Java TPC-DS Fixture Generator"
    log_info "========================================="
    log_info "Scale Factor:      $SCALE_FACTOR"
    log_info "Tables to generate: ${#tables_to_generate[@]}"
    log_info "Fixture directory: $fixture_dir"
    log_info "========================================="

    ensure_java_build

    mkdir -p "$fixture_dir"
    log_info "Created fixture directory: $fixture_dir"

    local success_count=0 fail_count=0 start_time end_time
    start_time=$(date +%s)

    for table in "${tables_to_generate[@]}"; do
        if generate_java_table "$table" "$fixture_dir"; then
            success_count=$((success_count + 1))
        else
            fail_count=$((fail_count + 1))
        fi
    done

    end_time=$(date +%s)
    local duration=$((end_time - start_time))

    echo ""
    log_info "========================================="
    log_info "Generation Complete"
    log_info "========================================="
    log_success "Successfully generated: $success_count tables"
    if [[ $fail_count -gt 0 ]]; then
        log_error "Failed to generate: $fail_count tables"
    fi
    log_info "Total time: ${duration}s"
    log_info "Fixtures saved to: $fixture_dir"
    log_info "========================================="

    if [[ $fail_count -gt 0 ]]; then
        exit 1
    fi

    # MD5SUMS is what the conformance suite actually compares against and the
    # only part of the fixture directory checked into git, so refresh it
    # whenever a full set was generated. A partial run would truncate it, so
    # leave it alone in that case.
    if [[ ${#tables_to_generate[@]} -eq ${#ALL_TABLES[@]} ]]; then
        write_md5sums "$fixture_dir"
    else
        log_info "Partial run; leaving $fixture_dir/MD5SUMS untouched"
    fi
}

# Write $fixture_dir/MD5SUMS: one "<md5> <table>.dat" line per table.
#
# dbgen_version is skipped. It records the generation timestamp and the command
# line that produced it, so its hash changes on every run — which is also why
# the conformance suite never compares it. Hashing it would make every
# regeneration look like a data change.
#
# Lines are sorted the way the MD5SUMS already in the repo are, so
# regenerating an existing fixture directory shows only real hash changes.
# The hash itself is computed with whichever of md5sum (Linux) or md5 (macOS)
# is present.
write_md5sums() {
    local fixture_dir=$1
    local md5sums="$fixture_dir/MD5SUMS"

    local lines=() table file hash
    for table in "${ALL_TABLES[@]}"; do
        [[ "$table" == "dbgen_version" ]] && continue
        file="$fixture_dir/${table}.dat"
        if [[ ! -f "$file" ]]; then
            log_error "Cannot hash missing fixture: $file"
            exit 1
        fi
        if command -v md5sum >/dev/null 2>&1; then
            hash=$(md5sum "$file" | cut -d' ' -f1)
        else
            hash=$(md5 -q "$file")
        fi
        lines+=("$hash ${table}.dat")
    done

    printf '%s\n' "${lines[@]}" | LC_ALL=en_US.UTF-8 sort -k2 > "$md5sums"
    log_success "Wrote $md5sums"
}

# -----------------------------------------------------------------------------
# --compat c (C dsdgen) helpers
# -----------------------------------------------------------------------------

check_c_prerequisites() {
    local missing=()
    command -v git   >/dev/null 2>&1 || missing+=(git)
    command -v bzip2 >/dev/null 2>&1 || missing+=(bzip2)
    command -v tar   >/dev/null 2>&1 || missing+=(tar)
    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing required tool(s) for --compat c: ${missing[*]}"
        return 1
    fi
    return 0
}

# Verify that the extracted fixture set is complete.
verify_c_fixtures() {
    local fixture_dir=$1

    if [[ ! -d "$fixture_dir" ]]; then
        log_error "Fixture directory does not exist: $fixture_dir"
        return 1
    fi

    local table f
    for table in "${ALL_TABLES[@]}"; do
        f="${table}.dat"
        if [[ ! -s "$fixture_dir/$f" ]]; then
            log_error "Missing or empty fixture: $fixture_dir/$f"
            return 1
        fi
    done

    local count
    count=$(find "$fixture_dir" -maxdepth 1 -name "*.dat" -type f | wc -l | tr -d ' ')
    log_success "Found $count .dat fixtures in $fixture_dir"
    return 0
}

download_and_extract_c() {
    local branch=$1
    local fixture_dir=$2
    local clone_dir
    clone_dir=$(mktemp -d -t tpcds-data-XXXXXX)

    # Cleanup helper. Called both on the success and failure paths below
    # rather than via `trap RETURN`, which under `set -u` causes the trap
    # to fire from later functions (e.g. `main`) where `$clone_dir` is no
    # longer in scope.
    _cleanup_clone_dir() {
        if [[ -n "${clone_dir:-}" && -d "$clone_dir" ]]; then
            rm -rf "$clone_dir"
        fi
    }

    log_info "Cloning $TPCDS_DATA_REPO branch '$branch' (depth 1) ..."
    if ! git clone --depth 1 --single-branch --branch "$branch" \
            "$TPCDS_DATA_REPO" "$clone_dir/tpcds-data"; then
        log_error "Failed to clone $TPCDS_DATA_REPO branch '$branch'"
        log_error "Confirm the branch exists (sf1, sf2, ...)"
        _cleanup_clone_dir
        return 1
    fi

    if ! ls "$clone_dir/tpcds-data"/data.tar.bz2.* >/dev/null 2>&1; then
        log_error "No data.tar.bz2.* parts found in cloned branch '$branch'"
        _cleanup_clone_dir
        return 1
    fi

    log_info "Extracting reference data into $fixture_dir ..."
    mkdir -p "$fixture_dir"

    # The archive expands as data/<table>.dat. Extract into a temp dir,
    # then flatten one level so the result is fixture_dir/<table>.dat.
    local extract_dir="$clone_dir/extract"
    mkdir -p "$extract_dir"
    if ! cat "$clone_dir/tpcds-data"/data.tar.bz2.* | bzip2 -d | tar -x -C "$extract_dir"; then
        log_error "Failed to extract data.tar.bz2.* parts"
        _cleanup_clone_dir
        return 1
    fi

    if [[ ! -d "$extract_dir/data" ]]; then
        log_error "Unexpected archive layout: $extract_dir/data not found"
        _cleanup_clone_dir
        return 1
    fi

    mv "$extract_dir/data"/*.dat "$fixture_dir/"
    _cleanup_clone_dir
    return 0
}

generate_c_fixtures() {
    local fixture_dir=$1
    local branch="sf${SCALE_FACTOR}"

    log_info "========================================="
    log_info "C dsdgen Reference Data Bootstrap"
    log_info "========================================="
    log_info "Repository:        $TPCDS_DATA_REPO"
    log_info "Branch:            $branch"
    log_info "Fixture directory: $fixture_dir"
    log_info "========================================="

    if ! check_c_prerequisites; then
        exit 1
    fi

    if [[ $VERIFY_ONLY -eq 1 ]]; then
        if verify_c_fixtures "$fixture_dir"; then
            exit 0
        else
            exit 1
        fi
    fi

    # Skip download if fixtures already look complete.
    if [[ $FORCE_REBUILD -eq 0 ]] && verify_c_fixtures "$fixture_dir" >/dev/null 2>&1; then
        log_success "C reference fixtures already present at $fixture_dir"
        log_info "Use --rebuild to force re-download"
        exit 0
    fi

    if [[ $FORCE_REBUILD -eq 1 && -d "$fixture_dir" ]]; then
        log_info "Removing existing fixture directory: $fixture_dir"
        rm -rf "$fixture_dir"
    fi

    local start_time end_time
    start_time=$(date +%s)
    if ! download_and_extract_c "$branch" "$fixture_dir"; then
        exit 1
    fi
    end_time=$(date +%s)

    if ! verify_c_fixtures "$fixture_dir"; then
        log_error "Bootstrap completed but verification failed"
        exit 1
    fi

    echo ""
    log_info "========================================="
    log_success "C dsdgen reference data ready"
    log_info "Time: $((end_time - start_time))s"
    log_info "========================================="
}

# -----------------------------------------------------------------------------
# main
# -----------------------------------------------------------------------------

main() {
    local tables_to_generate=()

    while [[ $# -gt 0 ]]; do
        case $1 in
            --compat)
                COMPAT="$2"
                shift 2
                ;;
            --scale)
                SCALE_FACTOR="$2"
                shift 2
                ;;
            --quiet)
                QUIET=1
                shift
                ;;
            --rebuild)
                FORCE_REBUILD=1
                shift
                ;;
            --verify)
                VERIFY_ONLY=1
                shift
                ;;
            --help)
                print_usage
                exit 0
                ;;
            --*)
                log_error "Unknown flag: $1"
                print_usage
                exit 1
                ;;
            *)
                tables_to_generate+=("$1")
                shift
                ;;
        esac
    done

    case $COMPAT in
        trino|c) ;;
        *)
            log_error "Unknown --compat value: $COMPAT (expected: trino, c)"
            exit 1
            ;;
    esac

    if [[ "$COMPAT" == "c" && ${#tables_to_generate[@]} -gt 0 ]]; then
        log_error "Per-table selection is not supported with --compat c"
        log_error "The published archive bundles all 25 tables together."
        exit 1
    fi

    if [[ "$COMPAT" == "trino" && ( $FORCE_REBUILD -eq 1 || $VERIFY_ONLY -eq 1 ) ]]; then
        log_error "--rebuild and --verify are only valid with --compat c"
        exit 1
    fi

    case $COMPAT in
        trino)
            local fixture_dir="$PROJECT_ROOT/tests/fixtures/tpcds/scale-${SCALE_FACTOR}-trino"
            if [[ ${#tables_to_generate[@]} -eq 0 ]]; then
                tables_to_generate=("${ALL_TABLES[@]}")
            fi
            generate_trino_fixtures "$fixture_dir" "${tables_to_generate[@]}"
            ;;
        c)
            local fixture_dir="$PROJECT_ROOT/tests/fixtures/tpcds/scale-${SCALE_FACTOR}-c"
            generate_c_fixtures "$fixture_dir"
            ;;
    esac
}

main "$@"
