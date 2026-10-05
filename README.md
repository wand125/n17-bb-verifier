# n17-bb-verifier

`n17bb-verify` is an exact verifier for the n = 17 sub-pattern branch-and-bound certificates of the [squares project](https://github.com/jlevy/squares) (schema `n17-subpattern-bb-certificate/v1`).
It decides the same question as the project’s standing verifier, `packing/devtools/verify_n17_bb_certificate.py`:

> Does the certificate prove that no k unit squares, square s centred in cell s at any angle, all inside the container, can have pairwise disjoint interiors?

It is a second implementation of the same checks, not a different proof.
It exists because the standing verifier is slow and memory-hungry on large trees.
On the 194,328-node certificate below it takes 173 seconds and 1.0 GB with four threads, against 98 minutes and 11.1 GB for the standing verifier in one process.

## What it checks

The checks are those of the standing verifier, under the same names. The certificate’s own `README.txt` lists them too.

| Check | What is verified | Where |
| --- | --- | --- |
| Header | the cell polygons equal the declared cover (exactly or by SHA-256), pairs, root angles and boxes, the π/2 multiples | `check_header` in `src/main.rs` |
| Trig | every recorded cos/sin enclosure contains the true value: 160-bit fixed-point Taylor sums with outward Lagrange remainder, retried at 2400 bits | `check_trig`, `cos_sin_bits` in `src/exact.rs` |
| T1–T3 | one root with the root angles and no windows; children partition their parent; every node is reached once; closed nodes are leaves; open nodes have a split and final boxes | `check_tree` |
| B1–B2 | round-0 boxes contain the wall contraction clipped to the cell; each next box and the final box contain the recomputed ones | `check_node_inner`, `contract` |
| P1–P4 | the gap lower bound; the angle pieces cover every normal modulo 2π inside the window; the three-plane and one-plane relaxations; pair splits leave no possible piece uncovered | `pair_planes`, `pieces_cover`, `piece_planes`, `check_open` |
| C1–C5 | disc and pair closures; each cut row is valid over the exact plane minimum; the Farkas combination is positive (LP closure); each bound tightening is valid in order; `obbt` emptiness | `check_closed_pair`, `row`, `combination`, `check_bounds` |

All arithmetic is exact: GMP integers and rationals through [`rug`](https://crates.io/crates/rug), with the trigonometric recurrence kept in integer form until the enclosure endpoints become rationals.
π is enclosed with Machin’s formula.
Critical multiples of π/2 are located by exact interval division.
Floating point is used only to report elapsed time.

The receipt has the standing verifier’s fields: status, node and leaf counts, maximum depth, closure reasons, the per-check counters, failures, and seconds.
The counters match the standing verifier’s exactly, including on certificates that fail.

## How it was written, and how independent it is

It was written from the standing verifier and the certificate format’s README, as a specification.
The generator, the native Rust producer, the hull kernel and the selector were not read or reused.
It shares no code with the standing verifier, and it depends on no code from the squares repository.

It is not independent of its author.
The same contributor, working with an AI coding agent, also wrote the native Rust producer that generates some of these certificates (squares PR #350).
Its value as a cross-check is that it shares no code with either side, and that its receipts agree with the standing verifier’s on every check counter.

## Results

The standing verifier ran unchanged in full mode, in one process, on a separate 8-vCPU Linux host (AMD EPYC 7B13).
Receipts for both are in [`results/`](results/), with local paths removed.

| Certificate | Nodes | Standing verifier | This verifier | Receipts agree |
| --- | --- | --- | --- | --- |
| Pattern A (6 interior cells) | 41,958 | PASS, 1,287 s | PASS, 91 s, 3 threads | every field and counter |
| side-S0, side-N0 + 5 interior cells (B2 rule) | 194,328 | PASS, 5,906 s, 11.1 GB | PASS, 173 s, 4 threads, 1.0 GB | every field and counter |
| side-S0, side-W0 + 5 interior cells (B2 rule) | 79,396 | PASS, 2,220 s, 4.3 GB | PASS, 68 s, 4 threads, 0.45 GB | every field and counter |

The A run of this verifier was on a shared Apple M4 host. The other two were on the Linux host.

Further certificates will be added here as they are checked.

## Build and run

Rust 1.98.0 is pinned in `rust-toolchain.toml`. Building needs GMP’s headers and library (`libgmp-dev` on Debian and Ubuntu, `brew install gmp` on macOS).

```bash
cargo build --locked --release
./target/release/n17bb-verify CERTIFICATE_DIR --threads 4 --output receipt.json
```

The exit status is 0 for PASS, 1 for FAIL and 2 for an invocation or output error.
The receipt goes to stdout, and also to `--output` when given.

- `--threads N` sets the worker threads; the default is 1.
- `--cells FILE --cells-sha256 DIGEST` checks against an external cell file. The default is the squares project’s capacity-one cover, embedded from `tests/cells.json`.
- `--manifest NAME` overrides the manifest named on the last line of the certificate’s `README.txt`.
- `--node-ids FILE` checks only the listed nodes, after checking the whole tree and all trigonometry. The receipt is then marked `mode: sample` and does not stand for full verification.

It reads the certificate in two streaming passes, one gzip chunk at a time.
The first pass keeps compact tree metadata; the second checks nodes in parallel.
Memory grows with the number of nodes, not with the size of their records.

## Tests

```bash
cargo test --locked
cargo clippy --locked --all-targets
cargo fmt --check
```

- Unit tests cover exact rational endpoints exported from the standing verifier at positive and negative angles, signed critical-angle boundaries, clipping, square-root bounds, and tree partitions.
- `tests/cli.rs` runs the binary on `tests/fixtures/small-certificate`, a complete 83-node certificate, and on six corrupted copies of it. Each copy must be rejected at its check:

  | Copy | Change | Rejected at |
  | --- | --- | --- |
  | `mutated-multiplier` | one Farkas multiplier made negative | C3/C4 |
  | `mutated-bound` | one tightened bound raised far above its value | C4 |
  | `mutated-cut` | one cut’s right side raised far above its value | C2 |
  | `mutated-split` | one angle split point moved outside its interval | T2 |
  | `mutated-drop-leaf` | one closed leaf removed | T2 |
  | `mutated-narrow-final` | one open node’s final box narrowed | B2 |

CI builds and tests on Ubuntu 24.04 with every push.

## Limits

- Only the non-Taylor schema `n17-subpattern-bb-certificate/v1` is supported. Certificates written with Taylor rows are rejected as outside scope.
- It verifies a certificate against a cover; it does not check that the cover’s cells have capacity one or that the cover is complete. Those are separate checks in the squares project.
- A PASS means the certificate proves its pattern infeasible. What that pattern excludes from the n = 17 census is decided elsewhere.
- The timings above are single runs on shared hosts.

## Data and licence

The code is under the MIT License (see [`LICENSE`](LICENSE)).

Three kinds of data come from the squares project, by Joshua Levy, under CC BY 4.0. None of them is code.

- `tests/cells.json` is the project’s capacity-one cover, exported from its tools.
- `tests/fixtures/small-certificate` was written by the project’s branch-and-bound pilot, including its `README.txt`. The `mutated-*` copies change one field each.
- The exact endpoints in the unit tests were exported from the standing verifier.
