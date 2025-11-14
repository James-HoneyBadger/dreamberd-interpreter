# Gulf of Mexico Interpreter - Actual Production Architecture

## Overview

The Gulf of Mexico interpreter is a **monolithic Python interpreter** (~2,900 lines in `gulfofmexico/interpreter.py`) that directly executes Gulf of Mexico code using a three-stage pipeline:

1. **Lexer** - Tokenizes source code
2. **Parser** - Builds abstract syntax tree (AST)
3. **Interpreter** - Executes AST via pattern matching

## What's Actually Used in Production

### Entry Points

**CLI/REPL:** `gulfofmexico/__init__.py`

- Calls `run_file()` for file execution
- Uses `tokenize()` → `generate_syntax_tree()` → `interpret_code_statements_main_wrapper()`

**Main Function:** `interpret_code_statements_main_wrapper()` in `interpreter.py`

- Wrapper around `interpret_code_statements()`
- This is what actually executes your Gulf of Mexico code

### Production Code Path

```
Source Code (.gom file)
    ↓
tokenize() [processor/lexer.py]
    ↓
generate_syntax_tree() [processor/syntax_tree.py]
    ↓
interpret_code_statements_main_wrapper() [interpreter.py]
    ↓
interpret_code_statements() [interpreter.py]
    ↓
Pattern matching on statement types
    ↓
Direct execution
```

### Core Execution: `interpreter.py`

This file contains ALL the actual execution logic:

```python
def interpret_code_statements(
    statements: list[tuple[CodeStatement, ...]],
    namespaces: list[Namespace],
    async_statements: AsyncStatements,
    when_statement_watchers: WhenStatementWatchers,
    importable_names: dict[str, dict[str, GulfOfMexicoValue]],
    exported_names: list[tuple[str, str, GulfOfMexicoValue]],
) -> Optional[GulfOfMexicoValue]:
    """Main interpreter loop - executes statements via pattern matching."""
    for statement_tuple in statements:
        statement = determine_statement_type(statement_tuple, namespaces)
        match statement:
            case VariableDeclaration():
                # Handle directly
            case VariableAssignment():
                # Handle directly
            case Conditional():
                # Handle directly
            # ... all other statement types
```

### Global State (Production)

The production interpreter uses **module-level globals** in `interpreter.py`:

```python
# Current file/code being executed
filename: str = ""
code: str = ""

# Current line for error reporting
current_line: int = 0

# Deleted values tracking
deleted_values: set[GulfOfMexicoValue] = set()

# Reactive programming support
name_watchers: NameWatchers = {}  # For 'when' statements
after_listeners: list = []  # For 'after' statements

# Flags
is_lifetime_temporal: bool = False
```

### Key Functions (All in `interpreter.py`)

**Statement Execution:**

- `interpret_code_statements()` - Main execution loop
- `determine_statement_type()` - Resolves conditional statements
- `execute_conditional()` - Handles if/else
- `register_when_statement()` - Sets up 'when' watchers
- `execute_after_statement()` - Handles 'after' events

**Expression Evaluation:**

- `evaluate_expression()` - Main expression evaluator
- `evaluate_expression_for_real()` - Actual evaluation logic
- Pattern matching on expression tree nodes

**Variable Management:**

- `declare_new_variable()` - Creates new variables
- `assign_variable()` - Updates variable values
- `get_name_from_namespaces()` - Namespace lookup
- `get_name_and_namespace_from_namespaces()` - Lookup with namespace

### File Structure (Production)

```
gulfofmexico/
├── __init__.py           ⭐ Entry point (run_file)
├── interpreter.py        ⭐ MAIN INTERPRETER (~2,900 lines)
├── builtin.py            ⭐ Type system and built-in functions
├── base.py               ⭐ Tokens, errors, utilities
├── serialize.py          ⭐ Persistent variable storage
│
├── processor/            ⭐ Lexer and parser
│   ├── lexer.py          - Tokenization
│   ├── syntax_tree.py    - AST generation and statement types
│   └── expression_tree.py - Expression parsing
│
├── engine/               🔬 EXPERIMENTAL (NOT USED)
│   ├── core.py           - Alternative engine
│   ├── evaluator.py      - Caching experiment
│   ├── namespace.py      - Caching experiment
│   └── handlers/         - Handler experiments
│
├── context.py            🔬 Used by experimental engine only
├── handlers.py           🔬 Experimental handler base classes
├── plugin_system.py      🔬 Experimental plugin system
├── constants.py          ⚙️ Shared configuration
└── utils.py              ⚙️ Shared utilities
```

## What's NOT Used in Production

### Experimental Engine Package (`engine/`)

**Status:** Proof-of-concept only, NOT integrated

The `gulfofmexico/engine/` package contains:

- `InterpretEngine` - Alternative handler-based execution
- `ExpressionEvaluator` - Caching experiment (1.77x speedup in benchmarks)
- `NamespaceManager` - Namespace caching experiment
- Handler modules - Modular statement handlers

**Why it exists:** Exploration of alternative architecture
**Why it's not used:** Production code still uses monolithic `interpreter.py`

### Handler System (`handlers.py`, `engine/handlers/`)

**Status:** Experimental architecture pattern

The handler system demonstrates how statements could be handled modularly:

- `StatementHandler` ABC
- `HandlerRegistry` for dispatch
- Individual handlers for each statement type

**Why it exists:** Proof-of-concept for future refactoring
**Why it's not used:** Production uses direct pattern matching

### Plugin System (`plugin_system.py`, `plugins/`)

**Status:** Prototype only

Demonstrates how third-party extensions could work:

- `Plugin` ABC
- `PluginManager` for registration
- Example plugins showing custom statements and functions

**Why it exists:** Shows potential extensibility
**Why it's not used:** No plugin support in production interpreter

### ExecutionContext (`context.py`)

**Status:** Used by experimental engine only

Encapsulates interpreter state as an object instead of globals:

- `ExecutionContext` dataclass
- `InterpreterConfig` for settings

**Why it exists:** Cleaner state management for experimental engine
**Why it's not used:** Production uses module-level globals

## How to Extend the Production Interpreter

Since the production interpreter is monolithic, extensions require modifying `interpreter.py` directly:

### Adding a New Statement Type

1. **Define statement class** in `processor/syntax_tree.py`:

```python
@dataclass
class MyNewStatement(CodeStatement):
    keyword: Token
    value: Token
```

2. **Add parsing** in `processor/syntax_tree.py`:

```python
# Add to generate_syntax_tree() or relevant parsing function
if keyword.value in ["mynew", "mynewkeyword"]:
    return MyNewStatement(keyword, value)
```

3. **Add execution** in `interpreter.py`:

```python
def interpret_code_statements(...):
    for statement_tuple in statements:
        statement = determine_statement_type(statement_tuple, namespaces)
        match statement:
            # ... existing cases ...
            case MyNewStatement():
                # Your execution logic here
                pass
```

### Adding a New Built-in Function

1. **Define function** in `interpreter.py`:

```python
def builtin_myfunction(args: list[GulfOfMexicoValue]) -> GulfOfMexicoValue:
    # Your logic here
    return GulfOfMexicoNumber(42)
```

2. **Register in BUILTIN_FUNCTIONS** in `interpreter.py`:

```python
BUILTIN_FUNCTIONS = {
    "print": builtin_print,
    "myfunction": builtin_myfunction,
    # ... other functions
}
```

## Performance Characteristics

### Production Interpreter

- **Startup:** ~200-500ms (cold start)
- **Execution:** Interpreted directly, no optimization
- **Memory:** Python garbage collection
- **Concurrency:** Single-threaded with event listeners for 'after'

### Experimental Engine

- **Namespace caching:** 1.77x faster (benchmarked in isolation)
- **Cache hit rate:** 99.99% in tests
- **NOT used in production:** Benchmarks don't reflect real-world performance

## Testing

### Production Tests

Tests that validate the actual interpreter:

- Example `.gom` files in `examples/`
- Manual testing via REPL and file execution

### Experimental Tests

Tests that validate experimental code (NOT production):

- `tests/test_variable_handlers.py`
- `tests/test_control_flow_handlers.py`
- `tests/test_function_handlers.py`
- `tests/test_special_handlers.py`
- `tests/test_integration.py`
- `benchmarks.py`

## Common Misconceptions

❌ "The interpreter uses a modular handler architecture"
✅ **Actually:** Monolithic pattern matching in `interpreter.py`

❌ "The interpreter has performance-optimized caching"
✅ **Actually:** No caching in production; experimental caching exists but unused

❌ "The interpreter supports plugins"
✅ **Actually:** No plugin system in production

❌ "ExecutionContext encapsulates all state"
✅ **Actually:** Production uses module-level globals

❌ "The engine/ package contains the main interpreter"
✅ **Actually:** `interpreter.py` is the main interpreter; `engine/` is experimental

## Summary

**Production Architecture:**

- ✅ Monolithic `interpreter.py` (~2,900 lines)
- ✅ Direct pattern matching on statement types
- ✅ Module-level globals for state
- ✅ Three-stage pipeline: Lexer → Parser → Interpreter

**Experimental Architecture:**

- 🔬 Modular `engine/` package
- 🔬 Handler-based statement execution
- 🔬 Caching infrastructure
- 🔬 Plugin system
- 🔬 **NOT integrated into production**

When working with Gulf of Mexico code, remember: **everything runs through `interpreter.py`**, not the experimental engine package.
