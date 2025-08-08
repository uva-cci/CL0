# CL0 Project

A Rust project for the CL0 Engine.

---

## Project Overview

This repository is structured as a Cargo workspace and is designed for extensibility. It currently contains:

- **cl0_parser**: The core parsing library for the CL0 Engine.
- **cl0_node**: The core engine and evaluator of CL0.
(More to be added in the future.)

---

## Libraries

### cl0_parser
- **Location:** `crates/cl0_parser`
- **Description:**
  - Provides tokenization, parsing, and AST generation for the CL0 language.
  - Exposes a flexible API for embedding the parser in other Rust projects.
  - Can be used as a standalone binary for parsing CL0 source code and printing the resulting AST.
- **API:** See the Rust docs and the `src/` directory for details.

### cl0_node
- **Location:** `crates/cl0_node`
- **Description:**
  - The evaluator of the CL0 Engine.
  - Exposes a flexible API for embedding the engine in other Rust projects.
  - Can be used as a standalone binary for a REPL that will parse and execute the input.
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


### Testing the Parser

To run all the unit tests of the parser, run the following commands from the project root:

```bash
cd crates/cl0_parser
cargo test
```

You should then see the status of every unit test that was created.

---

### Limitations

None currently. Please open an issue if you think otherwise!

---

## Running the Engine REPL

After building, you can run the REPL binary directly or via the symlink:

```bash
./cl0_repl
```

- This should run an executable of the REPL that allows the user to input an initial policy and interact with the node system.

---

### Testing the Engine

To run all the unit tests of the node engine, run the following commands from the project root:

```bash
cd crates/cl0_node
cargo test
```

You should then see the status of every unit test that was created.

---

### Limitations
- Need more unit testing especially for more complex operations and parallel actions.
- Need to clean up the api and REPL code.

Please open an issue if you have more!

---
