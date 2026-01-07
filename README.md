# Stapel

**Stapel** is a minimalist, stack-based, concatenative programming language that compiles directly to x86-64 assembly (NASM) for Linux. It provides low-level control over memory and the stack while offering high-level abstractions like procedures, loops, inline macros, and string literals.

## TODO:

- [] Local memory in procedures
- [] Type and stack checker (web assembly style)
- [] Importing stapel files

## 🚀 Quick Start

### Prerequisites

* **Rust** (for compiling the compiler)
* **NASM** (for assembling the output)
* **ld** (GNU Linker)

### Installation & Usage

1. **Build the compiler:** `cargo build --release`
2. **Compile a Stapel program:** `./target/release/stapel build /hello.spl`
3. **Run the executable:** `./hello`

---

## Language Specification

### 1. The Stack

Stapel is a stack machine. All operations consume arguments from the top of the stack and push results back.

* **Integers:** 64-bit signed integers (`i64`).
* **Strings:** Pushed as `[length, address]` pairs.

### 2. Literals

| Literal | Example | Description |
| --- | --- | --- |
| **Integer** | `123`, `-45` | Pushes a 64-bit signed integer. |
| **String** | `"Hello"` | Pushes **Length** then **Address** (2 items). |
| **Character** | `'A'`, `'\n'` | Pushes the ASCII integer value (e.g., `'A'` -> 65). |

### 3. Stack Manipulation

| Keyword | Effect `( Before -- After )` | Description |
| --- | --- | --- |
| `dup` | `( a -- a a )` | Duplicates the top item. |
| `swap` | `( a b -- b a )` | Swaps the top two items. |
| `drop` | `( a -- )` | Discards the top item. |
| `over` | `( a b -- a b a )` | Copies the second item to the top. |
| `rot` | `( a b c -- b c a )` | Rotates the third item to the top. |
| `pick` | `( a b c 2 -- a b c a )` | copies nth item of stack to top. |

### 4. Arithmetic & Logic

Arithmetic operators consume operands from the stack and push the result.

| Operator | Description |
| --- | --- |
| `+`, `-`, `*`, `/`, `%` | Standard integer math. |
| `++` | Increment top of stack (Prefix operator). |
| `=`, `!=`, `<`, `>`, `<=`, `>=` | Comparison. Pushes `1` (true) or `0` (false). |

#### **Example: Stack Consumption**

In Stack languages, the operator appears *after* the numbers.

**Code:**

```forth
3 10 /
```

**Step-by-Step State:**

1. `3` is pushed. Stack: `[ 3 ]`
2. `10` is pushed. Stack: `[ 3, 10 ]` (Top is 10)
3. `%` runs. It **pops** 10, then **pops** 3.
4. It calculates .
5. It **pushes** 1. Stack: `[ 1 ]`

### 5. Memory Management

Stapel allows manual memory allocation and access. Note that **@** is used for **Store** and **!** is used for **Load**, which reverses the convention seen in some other stack languages.

**Allocation:**

```forth
memory buffer 1024 end  # Reserves 1024 bytes in the .bss section
```

**Access:**
| Op | Usage | Description |
| :--- | :--- | :--- |
| **`@`** | `addr val @size` | **Store**. Writes `val` into memory at `addr`. |
| **`!`** | `addr !size` | **Load**. Reads a value from memory at `addr`. |

* **Sizes:** `1` (byte), `2` (word), `4` (dword), `8` (qword).

**Example:**

```forth
memory buf 64 end

# Storing a character
buf 'A' @1   # Store 'A' (65) at the start of 'buf'

# Loading it back
buf !1       # Push buf address, Load 1 byte. Stack: [ 65 ]
```

### 6. Control Flow

**Conditionals:**

```forth
<condition> if
    "True" print
else 
    "False" print
end

```

**Loops:**

```forth
0 while <condition> do
    dup put  # Print current number
    1 +      # Increment
end
```

> The condition of the loop can contain a lot more than just the condition. However, once the program is a the `do` keyword what is at the to counts as the conditional value

### 7. Procedures & Inlining

* **`proc`**: Defines a reusable subroutine.
* **`inline`**: Defines a block of code injected directly where called (macros).

```forth
proc square do
    dup *
end

inline INC 1 + end  # Replaced at compile time

proc main do
   5 square put  # Output: 25
   10 INC put    # Output: 11
end
```

### 8. System Calls

Direct Linux syscalls are supported via `syscall<N>` where N is the argument count (0-6).
Arguments are popped from the stack in reverse order (`rdi`, `rsi`, `rdx`, `r10`, `r8`, `r9`).

```forth
# Example: write(stdout, "Hi", 2)
# Syscall ID for write is 1
proc print do
   1 1 syscall4  # Pops: ID (1), FD (1), Buffer, Length
end
```

Results (normally in `RAX`) of syscall is pushed to stack

---

## 🛠 Project Structure

* **`main.rs`**: CLI entry point and build pipeline.
* **`compiler.rs`**: Generates x86-64 NASM assembly. Handles string constants and BSS layout.
* **`lexer.rs`**: Tokenizes input
* **`parser.rs`**: Recursive descent parser that constructs the AST (Procedures, Loops, Ifs, Memory definitions).
* **`program.rs`**: Handles AST optimization and inlining passes.
* **`tokens.rs`**: Defines Token types and Span (source location) for error reporting.