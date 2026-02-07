# SHA-1 Cracker

## Description

This Rust program attempts to crack a given SHA-1 hash by comparing it against hashes of words from a provided wordlist. It reads a wordlist file, computes the SHA-1 hash for each word, and compares it to the target hash. If a match is found, it prints the corresponding word (password).

## Usage

To use this program, you need to have Rust and Cargo installed on your system. You can get Rust and Cargo from [here](https://www.rust-lang.org/tools/install).

## Installation

1. Clone the repository or download the source code.
2. Navigate to the project directory.
3. Build the project using Cargo.

```sh
git clone git@github.com:IAmKushagraSharma/sha1_cracker.git
cd sha1_cracker
cargo build --release
```

## Running the Program

To run the program, use the following command:

```sh
cargo run -- <wordlist.txt> <sha1_hash>
```

- `<wordlist.txt>`: Path to the wordlist file containing possible passwords (one per line).
- `<sha1_hash>`: The SHA-1 hash you want to crack.

Example:

```sh
cargo run -- wordlist.txt 5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8
```

In this example, `5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8` is the SHA-1 hash of the password "password".

## Example Wordlist

Create a `wordlist.txt` file with the following content:

```
password
123456
qwerty
abc123
```

## Output

If the program finds a matching password, it will output:

```
 Password Found: <password>
```

If no match is found, the program will simply exit without any output.

## Error Handling

The program performs basic error handling:

- Checks if the correct number of arguments is provided.
- Validates the length of the SHA-1 hash.
- Handles file I/O errors when reading the wordlist file.
