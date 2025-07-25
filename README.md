# CL0 Project

A Rust project for the CL0 Engine.

---

## Project Overview

This repository is structured as a Cargo workspace and is designed for extensibility. It currently contains:

- **cl0_parser**: The core parsing library for the CL0 Engine. (More to be added in the future.)

---

## Libraries

### cl0_parser
- **Location:** `crates/cl0_parser`
- **Description:**
  - Provides tokenization, parsing, and AST generation for the CL0 language.
  - Exposes a flexible API for embedding the parser in other Rust projects.
  - Can be used as a standalone binary for parsing CL0 source code and printing the resulting AST.
- **API:** See the Rust docs and the `src/` directory for details.

---

## Building the Project

1. **Clone the repository:**
   ```bash
   git clone <repo-url>
   cd CL0
   ```

2. **Build the workspace:**
   ```bash
   cargo build [ --release ] (If a release build is required)
   ```

---

## Running the Parser

After building, you can run the parser binary directly or via the symlink:

```bash
./cl0_parser "<input>"
```

- Replace `<input>` with your CL0 source code (quotes required if it contains spaces).
- The program will print the parsed AST to stdout.

**Example:**
```bash
./cl0_parser "#event : condition => #action."
```

---


## Testing the Parser

To run all the unit tests of the parser, run the following commands from the project root:

```bash
cd crates/cl0_parser
cargo test
```

You should then see the status of every unit test that was created.

## Limitations

Currently there is an issue with referring to a compound as an atomic conditional type. Because of this, the compound statement functionally is disabled.

---