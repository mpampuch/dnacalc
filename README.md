# A Calculator for DNA

`dnacalc` is a lightweight command-line tool that converts DNA (and RNA) base-pair counts into human-readable units.

## Installation

Install from [crates.io](https://crates.io/crates/dnacalc) with Cargo:

```bash
cargo install dnacalc
```

Alternatively, build from source:

```bash
git clone https://github.com/mpampuch/dnacalc
cd dnacalc
cargo install --path .
```

## Usage

```text
dnacalc [OPTIONS] <BASE_PAIRS>
```

Convert a base-pair count using automatically selected units:

```bash
dnacalc 11200
# 11.2 Kbp

dnacalc 11200000
# 11.2 Mbp
```

Force a specific unit with one of the unit flags:

```bash
dnacalc -k 11200
# 11.2 Kbp

dnacalc -m 11200
# 0.0112 Mbp

dnacalc -g 11200000
# 0.0112 Gbp
```

Remove the space between the number and unit:

```bash
dnacalc -n 11200
# 11.2Kbp

dnacalc --no-space 11200000
# 11.2Mbp
```

Grouping separators are supported:

```bash
dnacalc 11,200
dnacalc 11.200
dnacalc 11_200
```

Use `-h` or `--help` to display the available options:

```bash
dnacalc --help
```

## Units

| Flag        | Unit  | Meaning        |
| ----------- | ----- | -------------- |
| *(default)* | `bp`  | Base pairs     |
| `-k`        | `Kbp` | Kilobase pairs |
| `-m`        | `Mbp` | Megabase pairs |
| `-g`        | `Gbp` | Gigabase pairs |
| `-t`        | `Tbp` | Terabase pairs |
| `-p`        | `Pbp` | Petabase pairs |




