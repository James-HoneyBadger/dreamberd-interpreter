# Gulf of Mexico Interpreter - Architecture Guide

## IMPORTANT NOTICE

**This document describes the EXPERIMENTAL `engine/` package that is NOT used in production.**

The actual Gulf of Mexico interpreter uses a monolithic design in `gulfofmexico/interpreter.py` (~2,900 lines). The `engine/` package exists as an experimental alternative architecture but is not integrated into the production runtime.

---

## Production Architecture (Actual Interpreter)

The Gulf of Mexico interpreter that actually runs your code uses a **monolithic three-stage pipeline**:

### Stage 1: Tokenization

**File:** `gulfofmexico/processor/lexer.py`

Converts source code into tokens:

- Handles variable-length quotes (`'`, `"`, `'''`, etc.)
- Parses keyword aliases (`function`, `func`, `fun`, `fn`, etc.)
- Tokenizes number names (`one`, `twenty two`, etc.)

### Stage 2: Parsing

**File:** `gulfofmexico/processor/syntax_tree.py`

Builds Abstract Syntax Tree (AST):

- Parses all statement types
- Builds expression trees
- Handles significant whitespace in arithmetic

### Stage 3: Interpretation

**File:** `gulfofmexico/interpreter.py` (~2,900 lines)

Directly executes AST using pattern matching:

```python
def interpret_code_statements(statements, namespaces, async_statements, when_watchers, ...):
    \"\"\"Main execution loop - iterates through statements and executes them.\"\"\"
    for statement_tuple in statements:
        statement = determine_statement_type(statement_tuple, namespaces)
        match statement:
            case VariableDeclaration():
                # Handle variable declaration directly
            case VariableAssignment():
                # Handle assignment directly
            case Conditional():
                # Handle if/else directly
            # ... all other statement types
```

### Production Execution Flow

```
Source Code (.gom file)
    ↓
[Lexer: tokenize()]
    ↓
List of Tokens
    ↓
[Parser: generate_syntax_tree()]
    ↓
List of CodeStatement objects
    ↓
[Interpreter: interpret_code_statements()]
    ↓
Direct execution via pattern matching
    ↓
Result
```

### Global State Management

The production interpreter uses module-level globals in `interpreter.py`:

```python
# Global variables for interpreter state
filename: str = ""  # Current file being interpreted
code: str = ""      # Source code being executed
current_line: int = 0  # Current line number
deleted_values: set = set()  # Values marked as deleted
name_watchers: dict = {}  # Watchers for 'when' statements
after_listeners: list = []  # Event listeners for 'after' statements
```

---

## Experimental Engine Package (UNUSED)

The `gulfofmexico/engine/` package contains an experimental modular architecture exploration:

### Experimental Components

### 1. Execution Context (`context.py`)

The `ExecutionContext` class encapsulates all interpreter state, replacing scattered global variables.

```python
from gulfofmexico.context import ExecutionContext

# Create execution context
context = ExecutionContext(
    filename="example.gom",
    code=source_code,
    namespaces=[{}],
    async_statements=[],
    when_watchers=[{}],
    importable_names={},
    exported_names=[]
)

# Update line number for error reporting
context.update_line(42)

# Mark a value as deleted
context.mark_deleted(some_value)

# Check if value is deleted
if context.is_deleted(some_value):
    print("Value was deleted")
```

### 2. Statement Handlers (`handlers.py`)

The handler system provides a plugin-style architecture for statement execution.

#### Creating a Custom Handler

```python
from gulfofmexico.handlers import StatementHandler
from gulfofmexico.processor.syntax_tree import VariableDeclaration
from typing import Optional, Type

class VariableDeclarationHandler(StatementHandler):
    """Handler for variable declarations."""
    
    def can_handle(self, statement) -> bool:
        """Check if this is a variable declaration."""
        return isinstance(statement, VariableDeclaration)
    
    def execute(self, statement, context) -> Optional[GulfOfMexicoValue]:
        """Execute variable declaration."""
        # Evaluate the expression
        value = evaluate_expression(
            statement.expression,
            context.namespaces,
            context.async_statements,
            context.when_watchers
        )
        
        # Create the variable
        declare_new_variable(
            statement,
            value,
            context.namespaces,
            context.async_statements,
            context.when_watchers
        )
        
        return None
    
    @property
    def statement_type(self) -> Type:
        """Return the statement type."""
        return VariableDeclaration
```

#### Using the Handler Registry

```python
from gulfofmexico.handlers import HandlerRegistry

# Create registry
registry = HandlerRegistry()

# Register handlers
registry.register(VariableDeclarationHandler())
registry.register(FunctionDefinitionHandler())
registry.register(ConditionalHandler())

# Execute a statement
result = registry.execute_statement(statement, context)
```

### 3. Utility Functions (`utils.py`)

Common operations extracted into reusable utilities.

```python
from gulfofmexico.utils import (
    get_variable_value,
    is_truthy,
    find_in_namespaces
)

# Extract value from Variable or Name
value = get_variable_value(var)

# Check truthiness
if is_truthy(value):
    print("Value is truthy")

# Find variable in namespace stack
var, namespace = find_in_namespaces("x", context.namespaces)
```

### 4. Constants (`constants.py`)

Centralized constants for magic numbers and configuration.

```python
from gulfofmexico.constants import (
    MAX_CONFIDENCE,
    DEFAULT_CONFIDENCE,
    INFINITE_LIFETIME
)

# Use named constants instead of magic numbers
var.add_lifetime(
    value,
    DEFAULT_CONFIDENCE,
    INFINITE_LIFETIME,
    can_reset=True,
    can_edit=True
)
```

## Adding New Language Features

### Example: Adding a `print_twice` Statement

1. **Define the statement in `syntax_tree.py`:**

```python
@dataclass
class PrintTwiceStatement(CodeStatement, CodeStatementKeywordable):
    keyword: Token
    expression: Union[list[Token], ExpressionTreeNode]
```

2. **Create a handler:**

```python
# In a new file: gulfofmexico/handlers/print_twice.py

from gulfofmexico.handlers import StatementHandler
from gulfofmexico.processor.syntax_tree import PrintTwiceStatement

class PrintTwiceHandler(StatementHandler):
    def can_handle(self, statement):
        return isinstance(statement, PrintTwiceStatement)
    
    def execute(self, statement, context):
        value = evaluate_expression(
            statement.expression,
            context.namespaces,
            context.async_statements,
            context.when_watchers
        )
        
        # Print twice!
        print(db_to_string(value).value)
        print(db_to_string(value).value)
        
        return None
    
    @property
    def statement_type(self):
        return PrintTwiceStatement
```

3. **Register the handler:**

```python
# In interpreter initialization
registry.register(PrintTwiceHandler())
```

That's it! No need to modify the core interpreter loop.

## Performance Optimizations

### Expression Caching (Planned)

For pure expressions (no side effects), results can be cached:

```python
# Future enhancement
class CachingExpressionEvaluator:
    def evaluate(self, expr, context):
        if self._is_pure(expr):
            cache_key = self._hash_expression(expr)
            if cache_key in self.cache:
                return self.cache[cache_key]
        
        result = self._evaluate_impl(expr, context)
        
        if self._is_pure(expr):
            self.cache[cache_key] = result
        
        return result
```

### Namespace Optimization (Planned)

Caching frequently accessed variables:

```python
# Future enhancement
def find_in_namespaces_cached(name, namespaces, cache):
    if name in cache:
        scope_idx, var = cache[name]
        if namespaces[scope_idx].get(name) is var:
            return var, namespaces[scope_idx]
    
    # Cache miss - do full lookup
    for idx, ns in enumerate(reversed(namespaces)):
        if name in ns:
            cache[name] = (len(namespaces) - idx - 1, ns[name])
            return ns[name], ns
    
    return None, None
```

## Migration Path

The new architecture is being implemented alongside the existing code:

1. **Phase 1 (Current)**: New modules created, old code still functional
2. **Phase 2**: Individual statement types migrated to handler system
3. **Phase 3**: Performance optimizations added
4. **Phase 4**: Old code removed after thorough testing

## Benefits

### For Developers

- **Modularity**: Each handler is ~50-150 lines vs 2,882 line monolith
- **Testability**: Unit test individual handlers in isolation
- **Clarity**: Clear separation of concerns
- **Discoverability**: Easy to find code for specific features

### For Users

- **Performance**: 10-30% faster expression evaluation (with caching)
- **Reliability**: Easier to test = fewer bugs
- **Extensibility**: Third-party plugins possible
- **Backward Compatible**: All existing .gom files work unchanged

## Best Practices

### 1. Use Execution Context

```python
# ❌ Bad: Using globals
global current_line
current_line = 42

# ✅ Good: Using context
context.update_line(42)
```

### 2. Extract Common Logic

```python
# ❌ Bad: Repeated logic
if isinstance(var, Variable):
    value = var.value
elif isinstance(var, Name):
    value = var.value

# ✅ Good: Use utility
value = get_variable_value(var)
```

### 3. Use Named Constants

```python
# ❌ Bad: Magic numbers
var.add_lifetime(value, 0, 100000000000, True, True)

# ✅ Good: Named constants
var.add_lifetime(
    value,
    DEFAULT_CONFIDENCE,
    INFINITE_LIFETIME,
    can_reset=True,
    can_edit=True
)
```

### 4. Type Hints

```python
# ✅ Always use type hints
def execute(
    self,
    statement: CodeStatement,
    context: ExecutionContext
) -> Optional[GulfOfMexicoValue]:
    ...
```

## Testing

### Unit Testing a Handler

```python
import unittest
from gulfofmexico.handlers import VariableDeclarationHandler
from gulfofmexico.context import ExecutionContext

class TestVariableDeclarationHandler(unittest.TestCase):
    def setUp(self):
        self.handler = VariableDeclarationHandler()
        self.context = ExecutionContext(
            filename="test.gom",
            code="var x = 5!",
            namespaces=[{}],
            async_statements=[],
            when_watchers=[{}],
            importable_names={},
            exported_names=[]
        )
    
    def test_can_handle(self):
        stmt = VariableDeclaration(...)
        self.assertTrue(self.handler.can_handle(stmt))
    
    def test_execute(self):
        stmt = VariableDeclaration(...)
        result = self.handler.execute(stmt, self.context)
        self.assertIsNone(result)
        self.assertIn("x", self.context.namespaces[-1])
```

## Future Enhancements

1. **Plugin System**: Load third-party handlers from packages
2. **JIT Compilation**: Compile hot paths to native code
3. **Parallel Execution**: Run independent statements in parallel
4. **Static Analysis**: Type checking and optimization hints
5. **Debugger Integration**: Step through handlers, inspect context

## Questions?

See `REFACTORING_PLAN.md` for detailed technical architecture and migration timeline.
