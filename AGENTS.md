# Agent Instructions

This repository implements TPC-H and TPC-DS data generation in Rust,
including Arrow integration and CLI tools.

Follow [CONTRIBUTING.md](CONTRIBUTING.md) for development conventions and
contribution requirements. Keep changes focused on the requested task.

## Documentation Guidelines

Each Markdown file has a specific purpose and audience. Keep documentation
focused on that purpose and at the appropriate level of detail.

Update documentation when a change affects information relevant to that
document, not simply because the implementation has changed.

### README.md

README files are the landing pages for the repository and its individual
packages. Package READMEs may also be displayed on registries such as
crates.io and PyPI.

The root README should provide:
- A project overview and its main capabilities.
- Installation and getting-started instructions.
- Common usage examples.
- Links to packages and more detailed documentation.

Package READMEs should focus on their respective package's purpose,
installation, public interfaces, and common usage.

Keep README files concise and user-focused. Avoid internal implementation
details, minor optimizations, and descriptions of individual bug fixes.

Not every new feature or configuration option needs a README entry.
Use CLI help and API documentation for exhaustive reference information.

### TESTING.md

TESTING.md explains how the project is tested and how contributors can
run and understand the tests.

It should cover:
- The overall testing strategy and infrastructure.
- The purpose and coverage of major test suites.
- How to run tests and conformance checks.
- Important testing guarantees, prerequisites, and limitations.

Describe testing infrastructure at a high level. Avoid documenting
individual test implementations, internal helpers, or incidental CI
implementation details.

Changes to testing infrastructure do not necessarily require documentation
updates. Update TESTING.md when the testing strategy, coverage, guarantees,
or instructions for running tests change.

Link to relevant scripts or component-specific documentation for details.

### ARCHITECTURE.md

Describe the overall architecture, crate organization, component
responsibilities, dependencies, and important design decisions.

Avoid function-level implementation details and ordinary refactoring
that does not change the overall architecture.

### CONTRIBUTING.md

Document contributor workflows, development practices, and expectations
that apply across contributions.

Avoid information specific to an individual PR or implementation.

### benchmarks/BENCHMARKS.md

Document benchmarking methodology, reproducible commands, and performance
results. Keep results and claims supported by measurements.

Avoid incidental implementation details unless necessary to explain
the benchmark results.

### Component-specific documentation

Keep specialized information close to the component it describes.

For example, detailed instructions for TPC-DS conformance scripts belong
in tpcgen-cli/scripts/tpcds/README.md rather than the root TESTING.md.

Use rustdoc for public APIs and source comments for important implementation
details and invariants.

### Documentation scope and review

Prefer a single authoritative location for detailed information.
High-level documents should summarize and link to more specific sources
rather than duplicate their contents.

Before submitting changes:
1. Review all modified Markdown files.
2. Verify that each addition belongs in that document and is appropriate
   for its intended audience.
3. Remove unnecessary implementation details, duplication, and unrelated
   documentation changes.
4. Ensure documentation remains accurate when public behavior, APIs, or
   contributor workflows change.

Use the PR description for change-specific rationale, implementation
decisions, and verification results.
