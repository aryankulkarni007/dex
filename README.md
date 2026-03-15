# Dex

Dex is a statically-typed, expression-oriented programming language implemented in Rust. It was built as a learning project to explore language design and Rust's type system by implementing a full compilation pipeline from scratch — lexer, parser, AST, and tree-walking interpreter.

The core design philosophy is **expressiveness with transparency**: concise on the surface, but never magical underneath. Every language feature should be predictable and knowable to the developer.

---

## Installation

```bash
cargo install --path .
```

Then run any `.dex` file:

```bash
dex program.dex
```

---

## Language Reference

### Types

| Type     | Description                          |
| -------- | ------------------------------------ |
| `int`    | 64-bit signed integer                |
| `flt`    | 64-bit float                         |
| `str`    | UTF-8 string                         |
| `bool`   | `true` or `false`                    |
| `abyss`  | void — functions that return nothing |
| `[T]`    | list of type T                       |
| `{K: V}` | map from K to V                      |

### Variables

Variables are immutable by default. Use `mut` for mutable bindings.

```
x = 10
y: int = 5
mut z = 15
```

### Functions

```
@(x: int, y: int) add -> int {
    x + y
}
```

Functions return the value of their last expression. No explicit `return` keyword yet.

### Structs

```
struct Point {
    x: flt, y: flt

    @(self, other) distance -> flt {
        dx = self.x - other.x
        dy = self.y - other.y
        (dx * dx + dy * dy) ^ 0.5
    }
}

p1 = Point(0.0, 0.0)
p2 = Point(3.0, 4.0)
d = p1:distance(p2)
```

### Operators

| Operator                    | Description                          |
| --------------------------- | ------------------------------------ |
| `+` `-` `*` `/` `%`         | Arithmetic                           |
| `^`                         | Exponentiation                       |
| `==` `!=` `<` `>` `<=` `>=` | Comparison                           |
| `&&` `\|\|`                 | Logical                              |
| `!`                         | Logical not                          |
| `-`                         | Unary negation                       |
| `band` `bor` `bxor` `bnot`  | Bitwise (lexed, not yet interpreted) |

No implicit type promotion — `1 + 2.0` is a type error. Cast explicitly.

### Control Flow

```
if x > 5 {
    print("big")
} else {
    print("small")
}
```

### Loops

```
I (n in list) -> {
    print(n)
}
```

### Pipelines

```
list >> filter(n -> n % 2 == 0) >> map(n -> n * 2)
```

Pipelines pass the left-hand value as the first argument to the right-hand function call.

### Lambdas

```
n -> n * 2
```

Single-parameter, single-expression functions.

### String Interpolation

```
name = "Aryan"
print("Hello {name}!")
```

### List Indexing

```
list = [1, 2, 3]
print(list[0])
```

### Built-in Functions

| Function           | Description                      |
| ------------------ | -------------------------------- |
| `print(...)`       | Print values to stdout           |
| `filter(list, fn)` | Filter a list by a predicate     |
| `map(list, fn)`    | Transform each element of a list |

---

## Architecture

### Pipeline

```
source text → Lexer → Vec<Token> → Parser → Vec<SpannedDecl> → Interpreter → Value
```

### Lexer (`lexer.rs`)

A hand-written character-by-character lexer. It produces a flat `Vec<Token>` from source text.

Each `Token` carries a `kind`, `value`, `line`, and `column`. The lexer handles multi-character tokens via maximal munch — it peeks ahead before deciding what token to emit. String literals are consumed in one pass with unterminated string detection. Comments (single-line `#` and multi-line `#- -#`) are discarded.

### Parser (`parser.rs`)

A hand-written recursive descent parser. It consumes the token stream and produces a `Vec<SpannedDecl>` — a list of top-level declarations, each wrapped with source location info.

**Precedence** (lowest to highest):

```
pipeline >>
or ||
and &&
comparison == != < > <= >=
addition + -
multiplication * / %
exponentiation ^
unary ! -
postfix . : () [] ?
primary
```

Every AST node is wrapped in `Spanned<T>` which carries `line` and `column`. This allows precise error reporting throughout the pipeline.

The parser uses one token of lookahead (`peek_offset`) to disambiguate ambiguous constructs — for example, `x: int = 5` (typed binding) vs `x:method()` (method call).

Error recovery uses synchronization — on a parse error, the parser advances until it finds a safe restart point (`@`, `struct`, or EOF) and continues parsing to collect multiple errors.

### AST (`ast.rs`)

The AST is a tree of Rust enums. Key types:

- `Decl` — top-level declarations (functions, structs, bindings)
- `Stmt` — statements inside blocks (bindings or expression statements)
- `Expr` — expressions that produce values
- `Spanned<T>` — any node T with source location attached
- `Type` — type annotations

### Interpreter (`interpreter.rs`)

A tree-walking interpreter. It walks the AST recursively and produces `Value`s.

**Environment model:** scopes are a `Vec<HashMap<String, Value>>` — a stack of hashmaps. Entering a scope pushes a new hashmap, leaving pops it. Variable lookup searches from the top of the stack downward, giving correct shadowing behaviour.

**Struct instances** are heap-allocated in a `Vec<HashMap<String, Value>>` on the interpreter. `Value::Struct` holds a type name and a `usize` index into this heap. This gives reference semantics — mutations to struct fields inside methods are visible to the caller.

**Built-ins** are pre-loaded into the global scope as `Value::Builtin(String)` and dispatched by name in the `Call` evaluator.

---

## Known Limitations and Conscious Deferrals

These are intentional tradeoffs made during development to keep scope manageable.

### Language

- No `return` keyword — functions always return their last expression
- No type checker — type errors are caught at runtime only
- No implicit type promotion — `int` and `flt` cannot be mixed in arithmetic
- No error handling — `T | error` union types and `?` operator are parsed but not interpreted
- No module system — all code must be in one file
- Mutability is not enforced — `mut` is tracked in the AST but not checked at runtime (deferred to type checker)
- Lambdas are single-parameter only
- No `return` from inside loops

### Interpreter

- No recursion depth limit — deep recursion will stack overflow Rust's call stack
- No garbage collection for the struct heap — struct instances are never freed
- Float equality uses `==` directly — subject to floating point precision issues
- Negative exponents silently truncate to `u32` in `i64::pow`
- `display()` cannot show struct field values — shows `<StructName at index>` instead

### Parser

- Error recovery only works at top-level declaration boundaries
- No maximum recursion depth for deeply nested expressions

### Pipelines

- Right-hand side must be a function call — `list >> someValue` is an error
- Only `filter` and `map` are built-in pipeline functions

---

## Example Program

```
struct Vector {
    x: flt, y: flt

    @(self, other) add -> Vector {
        Vector(self.x + other.x, self.y + other.y)
    }

    @(self) magnitude -> flt {
        (self.x * self.x + self.y * self.y) ^ 0.5
    }
}

@(numbers: [int]) process -> [int] {
    numbers >> filter(n -> n % 2 == 0) >> map(n -> n * n)
}

@() main -> int {
    v1 = Vector(3.0, 4.0)
    v2 = Vector(1.0, 2.0)
    v3 = v1:add(v2)
    print("magnitude of v1: {v1:magnitude()}")

    numbers = [1, 2, 3, 4, 5, 6, 7, 8]
    result = process(numbers)
    print("even squares: {result}")

    greeting = "Dex"
    print("Hello from {greeting}!")
}
```

---

## What's Next

- Type checker
- Garbage collection
- `return` keyword
- Error handling with `T | error` and `?`
- REPL
- Standard library expansion
- Neovim integration — syntax highlighting and LSP
