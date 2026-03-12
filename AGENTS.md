# AGENTS.md — Development Guide for `comtrade`

This file provides guidance for AI agents and developers working on the `comtrade` Rust library — a pure-Rust parser for [COMTRADE](https://en.wikipedia.org/wiki/Comtrade) files (Common format for Transient Data Exchange for power systems).

---

## Project Overview

The library parses COMTRADE file sets used to record power system disturbance data (oscillography and status). A typical COMTRADE recording consists of:

- **`.cfg`** — Configuration file describing channels, sample rates, timestamps, and encoding. Human-readable (ASCII or UTF-8).
- **`.dat`** — Data file containing the actual samples, in ASCII, Binary16, Binary32, or Float32 format.
- **`.cff`** — Combined File Format (introduced in IEEE C37.111-2013), which bundles `.cfg` and `.dat` content into one file with section delimiters.

### Standards versions supported

| Version | Status |
|---------|--------|
| IEEE C37.111-1991 | Supported |
| IEEE C37.111-1999 | Supported |
| IEEE C37.111-2013 | Supported (including `.cff`) |

---

## Repository Structure

```
src/
  lib.rs          # Public API surface; re-exports key types
  error.rs        # Error and warning types
src/parser/
  cff.rs          # .cff combined format splitter/loader
  mod.rs          # Public parser API; dispatches to .cfg and .dat parsers
  time.rs         # Time parsing utilities (timestamps, time multipliers, etc.)
src/parser/cfg
  analog_channel.rs  # Analog channel definition parsing
  date_time.rs    # Date/time parsing utilities
  id_line.rs      # Station/device ID line parser
  mod.rs          # .cfg file parser (ASCII/UTF-8)
  revision.rs     # Revision year parsing utilities
  sample_rate.rs  # Sample rate section parsing
  status_channel.rs  # Digital channel definition parsing
src/parser/dat
  mod.rs          # .dat file parser (ASCII, Binary16, Binary32, Float32)
  formats.rs      # Binary format parsing utilities
tests/
  ...             # Integration tests using sample files
```

---

## Build & Test Commands

```bash
# Build the library
cargo build

# Run all tests
cargo test

# Run tests with output shown (useful for debugging parsers)
cargo test -- --nocapture

# Check for warnings and lint issues
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check documentation builds without errors
cargo doc --no-deps
```

All of the above must pass cleanly before a change is considered complete.

---

## Implementation Status

| Feature | Status |
|---------|--------|
| Parse `.cfg` (ASCII, 1991/1999/2013) | ✅ Done |
| Parse `.cfg` (UTF-8) | ✅ Done |
| Parse `.cfg` (other encodings, e.g. latin1) | ⚠️ Not tested |
| Parse ASCII `.dat` files | ✅ Done |
| Parse Binary16 `.dat` files | ✅ Done |
| Parse Binary32 `.dat` files | ✅ Done (not tested) |
| Parse Float32 `.dat` files | ✅ Done (not tested) |
| Load `.cff` combined format | ✅ Done |
| Analog value retrieval (adders & multipliers) | ✅ Done |
| Analog value retrieval (primary vs. secondary) | ❌ Todo |
| Real timestamp calculation (time multiplier) | ✅ Done |
| Channel-specific timestamp skews | ❌ Todo |

---

## Priority Work Items

When implementing new features or fixing bugs, address items in this order:

### High priority
1. **Primary vs. secondary scaling** — Analog channels in COMTRADE have a `ps` flag (`P` or `S`) indicating whether values are in primary or secondary units. The conversion factor needs to be applied based on this flag. See the `.cfg` channel definition rows.
2. **Error message quality** — Errors should consistently include the filename and line number where the problem occurred.
3. **Test coverage for Binary32 and Float32** — These parsers exist but have no test files. Obtain or generate sample files and add regression tests.

### Medium priority
4. **Channel-specific timestamp skews** — Some COMTRADE files specify per-channel time offsets. This is not yet implemented.
5. **Non-critical missing data warnings** — Add non-fatal warnings when optional fields are absent, rather than silently ignoring them.
6. **Critical missing data errors** — Raise clear errors when required fields are missing instead of producing incorrect output silently.
7. **Unexpected value warnings** — E.g. warn (don't error) when channel numbers don't add up sequentially.

### Lower priority
8. **Continuously variable sample rate** — When `nrates=0`, timestamps come from the data records themselves (critical timestamps). Verify correctness of this path.
9. **Multiple sample rate sections** — Some recordings change sample rate mid-capture. Ensure this is handled correctly.
10. **Missing non-critical data test files** — Add test cases where optional `.cfg` fields are omitted.

---

## COMTRADE Format Notes

These notes help avoid common mistakes when working on the parsers.

### `.cfg` file structure (line-by-line)
1. Station name, recording device ID, revision year
2. Total channel count, analog channel count (`nA`), digital channel count (`nD`)
3. One line per analog channel: index, name, phase, circuit, unit, multiplier (`a`), adder (`b`), skew, min, max, primary, secondary, `ps`
4. One line per digital channel: index, name, phase, circuit, normal state
5. Line frequency
6. Number of sample rate sections (`nrates`); if 0, timestamps are in data
7. One line per sample rate section: rate (Hz), last sample number at this rate
8. Start datetime, trigger datetime
9. Data file type: `ASCII`, `BINARY`, `BINARY32`, `FLOAT32`
10. Time multiplier (1999+)
11. Time quality and leapcount fields (2013+)

### Analog value formula
```
actual_value = (raw_value * multiplier) + adder
```
Primary/secondary correction is an additional step not yet implemented.

### `.cff` combined format
The `.cff` file contains sections delimited by lines like:
```
--- file type, <SECTION_TYPE>, <encoding> ---
```
Sections are: `CFG`, `DAT`, `HDR`, `INF`. The parser must split these before passing to the `.cfg` and `.dat` parsers.

---

## Coding Conventions

- Follow standard Rust idioms. Run `cargo fmt` and `cargo clippy` before every commit.
- Prefer `thiserror` for error types (check `Cargo.toml` for current error handling crate).
- Avoid `unwrap()` and `expect()` in library code — propagate errors with `?`.
- Public API items must have doc comments (`///`).
- Keep parser logic separated by file type (`.cfg`, `.dat`, `.cff`) in their own modules.
- When adding a new feature, add at least one integration test with a real or synthetic sample file.

---

## Adding Test Files

COMTRADE sample files are needed to test parsers. When adding test files:

- Place them under `tests/comtrade_files/` in a descriptive subdirectory (e.g. `tests/comtrade_files/binary32/`, `tests/comtrade_files/cff/`).
- Include the minimal set of files needed (e.g. `.cfg` + `.dat`, or `.cff`).
- Add a comment in the test explaining what scenario the file exercises.
- Synthetic files (hand-crafted to test edge cases) are fine and preferred for unusual scenarios.

---

## Out of Scope

Do not implement the following without explicit discussion:

- Writing/generating COMTRADE files (this is a read-only parser library).
- Support for proprietary vendor extensions to the COMTRADE format.
- Async I/O — the library is synchronous by design.
