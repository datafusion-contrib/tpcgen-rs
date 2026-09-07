# Contributing to tpcgen-rs

We want to make contributing to this project as easy and transparent as
possible, if you have suggestions to improve the contributions guide feel
free to open an issue.

## Our Development Process

We follow a very standard development process, start by picking an existing issue or opening a new
one to address a problem you've found. Letting us know which issue you are working on allows us to
better track progress.

## Pull Requests

We actively welcome your pull requests.

1. Fork the repo and create your branch from `main`.
2. If you've added code that should be tested, add tests.
3. If you've changed APIs, update the documentation.
4. Ensure the standard tests and conformance tests are passing.
5. Make sure your code follows Rust best practices, address any linting issues clippy might find.
6. Run [typos](https://github.com/crate-ci/typos) locally to catch any spelling mistakes in your changes. For intentional exceptions in rust source files, use `// typos:off` and `// typos:on` comments.
7. Open your pull request and wait for a review and approval.

### Continuous integration

Pull requests and merge-queue commits run Rust linting, workspace tests, doc tests,
documentation builds, and native Windows/macOS compilation checks.
Packaging coverage grows as changes move toward release:

| Run | Wheel targets |
| --- | --- |
| Pull request | Linux x86_64 manylinux |
| Merge queue | Linux x86_64 manylinux and musllinux, Windows x64, macOS ARM64 |
| Main, manual, release | All 13 targets |

Both CLIs build source distributions in every case. Packaging failures specific
to architectures outside the merge-queue subset may be detected after merging.

The reusable packaging workflow accepts `wheel-coverage: smoke`,
`wheel-coverage: major-platforms`, or `wheel-coverage: all` (the default).
Wheel coverage is independent of `package` and `upload-artifacts`.

The `Rust checks` status requires all of these jobs to succeed. TPC-H and both
TPC-DS compatibility modes also remain required conformance checks.

## Issues

When opening a new issue try and follow the issue template, there are no specific
requirements with regards to the overall format but we do like to have as much
details as possible and even better reproducible examples.

## Coding Style

Prefer following standard Rust guidelines with regards to formatting as for coding
style, [Effective Rust](https://www.lurklurk.org/effective-rust/title-page.html) is
a good resource for idiomatic code.

## Benchmarking
Benchmarking is an important part of this project. We typically evaluate performance
using [hyperfine](https://github.com/sharkdp/hyperfine).

For example to benchmark the performance of generating TPC-H data using
`tpcgen-cli` we use a command such as

```shell
hyperfine --runs 5 \
--prepare "rm -rf /tmp/output" \
"target/release/tpcgen-cli tpch parquet --scale-factor=100 --tables=lineitem --parts=10 --output-dir /tmp/output" \
```

To benchmark the performance of generating TPC-DS data using `tpcgen` we use a command such as
```shell
hyperfine --runs 5 \
--prepare "rm -rf /tmp/output" \
"target/release/tpcgen-cli tpcds parquet --scale-factor=100 --tables=store_sales --output-dir /tmp/output"
```

## License

By contributing to tpcgen-rs, you agree that your contributions will be licensed
under the LICENSE file in the root directory of this source tree.
