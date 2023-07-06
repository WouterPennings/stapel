# Stapel

Stapel (meaning stack is Dutch) is a stack based language which compiles to X86 assembly Linux.

# Language reference

## Types

- **64-bit integer**
- **String**

**Note:** `0` means false, everything that is **not** 0 is true

## Operators

- `+`: plus, pops first two stack values and pushes result to stack `5 3` -> `8`
- `-`: minus, pops first two stack values and pushes result to stack `5 3` -> `2`
- `*`: multiply, pops first two stack values and pushes result to stack `3 5` -> `15`
- `/`: divide, pops first two stack values and pushes result to stack `10 2` -> `5`
- `=`: equals, pops first two stack values and pushes result to stack `3 5` -> `0`
- `!=`: notequals, pops first two stack values and pushes result to stack `3 5` -> `1`
- `<` : lesserthan, pops first two stack values and pushes result to stack `3 5` -> `1`
- `>`: greaterthan, pops first two stack values and pushes result to stack `3 5` -> `0`
- `<=`: lesserorequals, pops first two stack values and pushes result to stack `3 5` -> `1`
- `>=`: greaterorequals, pops first two stack values and pushes result to stack `3 3` -> `1`
- `%`: modulo, divides first two stack values, divides them and pushes remainder to stack `10 7` -> `3`

## Instructions

- `push`: Pushes a string or integer to the stack.
- `pop`: Pops top value from the stack `A B` -> `B`.
- `swap`: Swaps top two value `A B` -> `B A`.
- `dup`: Duplicates the top value on the stack `A B` -> `A A B`.
- `put`: Pops first value from stack and writes to stdout .
- `load`: Pops stack, this value is memory adress. Memory at address is read and value is pushed to the stack
- `store`: Pops stack twice, first value is value to store, second value is memory address.
- `mem`: Pushes start pointer of .bss memory to stack.
- `size`: Pushes size of stack to top of stack in bytes (new value is not included).
- `syscallX`: syscallX performs linux syscall, the X is the amount of parameters for the syscall (1 to 8). The sequence of registers to stack is first RAX than RDI, RSI, RDX, R10, etc.

### Control Flow

- `while`:
- `if`:
- `do`:
- `end`:
