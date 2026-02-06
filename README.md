# hashcrack

A multi-algorithm, multi-threaded hash cracking tool. Written in Rust.

Started as a simple SHA-1 dictionary cracker — a single file, ~40 lines, one algorithm. Then it kept growing. Added more algorithms, parallelism, mutation rules, salt support, resume capability, output formats. Piece by piece, each module doing exactly one thing.

It breaks hashes. That's it.

## Getting Started

```sh
git clone git@github.com:TheDarkArtist/sha1_cracker.git
cd sha1_cracker
cargo build --release
```

## Usage

```sh
# basic — algorithm auto-detected from hash length
hashcrack <wordlist> <hash>

# pick your algorithm
hashcrack wordlist.txt --algo sha256 <hash>

# multiple targets from a file
hashcrack wordlist.txt --hashes-file targets.txt

# pipe from stdin
cat wordlist.txt | hashcrack - <hash>

# benchmark hashing speed
hashcrack wordlist.txt --benchmark
```

### Examples

```sh
# crack a SHA-1 hash
hashcrack wordlist.txt 5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8

# crack MD5 with mutation rules
hashcrack wordlist.txt -r append_digits,leet 5f4dcc3b5aa765d61d8327deb882cf99

# multiple hashes, JSON output, 8 threads
hashcrack wordlist.txt --hashes-file hashes.txt --format json -j 8

# salted hash (hash:salt format)
hashcrack wordlist.txt "e99a18c428cb38d5f260853678922e03:mysalt" --salt-position suffix
```

## Options

```
-a, --algo <ALGO>              md5 | sha1 | sha256 | sha512 (auto-detected if omitted)
-j, --threads <NUM>            number of threads (defaults to CPU count)
-r, --rules <RULES>            comma-separated mutation rules
    --hashes-file <FILE>       file with hashes to crack (one per line, or hash:salt)
    --format <FORMAT>          normal | quiet | verbose | json
    --ignore-case              case-insensitive hash comparison
    --salt-position <POS>      prefix | suffix (default: prefix)
    --resume <FILE>            resume from checkpoint
    --checkpoint-interval <N>  save checkpoint every N candidates (default: 1000000)
    --benchmark                test hashing speed per algorithm
```

## Mutation Rules

Rules generate candidate passwords from each word in the wordlist. Stack them with commas.

| Rule | What it does | `password` becomes |
|------|-------------|-------------------|
| `append_digits` | Appends 0-9 | password0, password1, ... password9 |
| `toggle_case` | UPPER + Capitalized | PASSWORD, Password |
| `leet` | a→4, e→3, i→1, o→0, s→5, t→7 | p455w0rd |
| `common_suffixes` | Appends !, 123, @, #, etc. | password!, password123 |
| `capitalize` | First letter uppercase | Password |
| `reverse` | Reverses the word | drowssap |

```sh
hashcrack wordlist.txt -r append_digits,leet,capitalize <hash>
```

## Output Formats

**normal** — `hash:password` pairs, summary stats

```
5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8:password
1/1 hashes cracked in 0.00s (5434 candidates/sec)
```

**verbose** — tagged output with detailed summary

```
[FOUND] 5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8 => password
[MISS]  deadbeefdeadbeefdeadbeefdeadbeefdeadbeef

--- Summary ---
Targets:    2
Found:      1
Candidates: 4
Duration:   0.00s
Speed:      5235 candidates/sec
```

**json** — one JSON object per line, for piping and scripting

```json
{"type":"found","hash":"5baa61...","password":"password"}
{"type":"summary","found":1,"targets":1,"candidates":4,"duration_secs":0.001,"speed":5434.0}
```

**quiet** — just the passwords, nothing else

## How It's Built

Started with the question: what's the simplest thing that could crack a hash? A wordlist, a hash function, a loop. That was v0.1 — a single `main.rs`.

Then it evolved, piece by piece:

**Phase 1 — Foundation.** Replaced manual arg parsing with `clap`. Added proper exit codes (0=found, 1=not found, 2=error). Custom error types with `thiserror`. The boring stuff that makes everything else possible.

**Phase 2 — Algorithms.** Defined a `HashAlgorithm` trait. Each algorithm — MD5, SHA-1, SHA-256, SHA-512 — implements it in its own file. A registry handles lookup by name and auto-detection by hash length. Adding a new algorithm means writing one struct, implementing one trait.

**Phase 3 — Speed.** `rayon` for parallel wordlist processing across all cores. `indicatif` for a progress bar that shows speed and ETA. The wordlist gets loaded into memory so rayon can chunk it freely.

**Phase 4 — Features.** Mutation rules that generate candidate passwords from each word — leet speak, digit appending, case toggling. Salt support with `hash:salt` format. Multi-target cracking from a file with `HashSet` for O(1) lookup. Four output formats. stdin support.

**Phase 5 — Resilience.** Checkpoint system for saving and resuming long-running sessions. JSON format so you can inspect it.

### Architecture

```
src/
├── main.rs          # thin orchestrator — parse, wire, run, exit
├── cli.rs           # argument definitions
├── error.rs         # error types + exit codes
├── hash/
│   ├── mod.rs       # HashAlgorithm trait + Registry
│   ├── md5.rs       # 32 hex chars
│   ├── sha1.rs      # 40 hex chars
│   ├── sha256.rs    # 64 hex chars
│   └── sha512.rs    # 128 hex chars
├── cracker.rs       # parallel cracking core
├── wordlist.rs      # file + stdin loading
├── rules.rs         # mutation rules engine
├── progress.rs      # progress bar wrapper
├── output.rs        # output formatters
└── checkpoint.rs    # save/resume state
```

Every module does one thing. The cracker's core function is pure — all dependencies passed as parameters, no hidden state. Algorithms are trait objects so they're selected at runtime. Rules compose through a chain. Output follows the strategy pattern.

12 files. 27 tests. Each piece testable in isolation.

## Tests

```sh
cargo test
```

All algorithms verified against known test vectors. Rule mutations, hex validation, checkpoint serialization, wordlist loading, target parsing — all covered.

## Benchmark

```sh
hashcrack wordlist.txt --benchmark
```

```
md5      2.48s (403455 hashes/sec)
sha1     2.47s (405125 hashes/sec)
sha256   4.10s (243729 hashes/sec)
sha512   12.89s (77606 hashes/sec)
```

1M iterations per algorithm. Debug build. Release build goes faster.

## License

MIT
