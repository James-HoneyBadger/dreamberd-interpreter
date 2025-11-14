# Quick Reference: Experimental Engine Package

## IMPORTANT NOTICE

**This describes the EXPERIMENTAL `engine/` package that is NOT used in production.**

### Using the ACTUAL Production Interpreter

The real Gulf of Mexico interpreter is used via:

```python
from gulfofmexico import run_file

# Run a Gulf of Mexico file
run_file("my_script.gom")
```

Or programmatically:

```python
from gulfofmexico.interpreter import interpret_code_statements_main_wrapper
from gulfofmexico.processor.lexer import tokenize
from gulfofmexico.processor.syntax_tree import generate_syntax_tree

code = "var x = 42!"
tokens = tokenize("test.gom", code)
statements = generate_syntax_tree("test.gom", tokens, code)

# Execute using the actual interpreter
result = interpret_code_statements_main_wrapper(
    statements,
    namespaces=[{}],
    async_statements=[],
    when_statement_watchers=[{}],
    importable_names={},
    exported_names=[]
)
```

## Using the Experimental Engine (NOT in Production)

### Basic Usage

```python
from gulfofmexico.engine.core import InterpretEngine
from gulfofmexico.context import ExecutionContext, InterpreterConfig
from gulfofmexico.processor.syntax_tree import parse_code

# Create engine with default configuration
engine = InterpretEngine()

# Or with custom configuration
config = InterpreterConfig(
    enable_expression_cache=True,
    enable_namespace_cache=True,
)
engine = InterpretEngine(config)

# Create execution context
context = ExecutionContext(
    filename="test.gom",
    code="var x = 42!",
)

# Parse and execute code
statements = parse_code(context.code)
result = engine.execute_statements(statements, context)

# Get performance statistics
stats = engine.get_stats()
print(f"Expression cache: {stats['expression_cache']}")
print(f"Registered handlers: {stats['registered_handlers']}")
```

---

## Working with Handlers

### Creating a Custom Handler

```python
from typing import Optional, Type
from gulfofmexico.handlers import StatementHandler
from gulfofmexico.context import ExecutionContext
from gulfofmexico.processor.syntax_tree import CodeStatement
from gulfofmexico.builtin import GulfOfMexicoValue

class MyCustomHandler(StatementHandler):
    """Handler for custom statement type."""
    
    def can_handle(self, statement: CodeStatement) -> bool:
        """Check if this handler can process the statement."""
        return isinstance(statement, MyCustomStatement)
    
    def execute(
        self,
        statement: CodeStatement,
        context: ExecutionContext,
    ) -> Optional[GulfOfMexicoValue]:
        """Execute the statement."""
        assert isinstance(statement, MyCustomStatement)
        
        # Your execution logic here
        # Access context.namespaces, context.async_statements, etc.
        
        return None  # or return a value
    
    @property
    def statement_type(self) -> Type[CodeStatement]:
        """The statement type this handler processes."""
        return MyCustomStatement
```

### Registering a Handler

```python
# Register with existing engine
engine = InterpretEngine()
engine.registry.register(MyCustomHandler())

# Or create plugin (recommended)
from gulfofmexico.plugin_system import Plugin

class MyPlugin(Plugin):
    @property
    def name(self) -> str:
        return "my-plugin"
    
    def get_handlers(self) -> list[StatementHandler]:
        return [MyCustomHandler()]
```

---

## Working with Plugins

### Creating a Plugin

```python
from typing import Callable
from gulfofmexico.plugin_system import Plugin
from gulfofmexico.handlers import StatementHandler
from gulfofmexico.builtin import GulfOfMexicoValue, GulfOfMexicoNumber

class MyPlugin(Plugin):
    @property
    def name(self) -> str:
        return "my-awesome-plugin"
    
    @property
    def version(self) -> str:
        return "1.0.0"
    
    @property
    def description(self) -> str:
        return "Does awesome things"
    
    def get_handlers(self) -> list[StatementHandler]:
        """Provide custom statement handlers."""
        return [MyCustomHandler()]
    
    def get_builtin_functions(self) -> dict[str, Callable]:
        """Provide custom built-in functions."""
        return {
            "my_function": self._my_function,
        }
    
    def _my_function(self, args: list[GulfOfMexicoValue]) -> GulfOfMexicoValue:
        """Custom built-in function."""
        # Your function logic
        return GulfOfMexicoNumber(42)
    
    def on_load(self) -> None:
        """Called when plugin is loaded."""
        print(f"[{self.name}] Plugin loaded!")
    
    def on_unload(self) -> None:
        """Called when plugin is unloaded."""
        print(f"[{self.name}] Plugin unloaded!")
```

### Using Plugins

```python
from gulfofmexico.plugin_system import PluginManager
from gulfofmexico.engine.core import InterpretEngine

# Create plugin manager
manager = PluginManager()

# Register plugin
plugin = MyPlugin()
manager.register_plugin(plugin)

# Get handlers and functions from all plugins
all_handlers = manager.collect_handlers()
all_functions = manager.collect_builtin_functions()

# Integrate with engine
engine = InterpretEngine()
for handler in all_handlers:
    engine.registry.register(handler)

# Use plugin functions in your interpreter
# (integrate with builtin function lookup)
```

---

## Using Caching

### Namespace Caching

```python
from gulfofmexico.engine.namespace import NamespaceManager

# Create namespace manager
namespace_mgr = NamespaceManager()

# Lookup with caching
value = namespace_mgr.lookup(
    name="my_variable",
    namespaces=context.namespaces,
)

# Invalidate cache when namespace changes
namespace_mgr.invalidate(len(context.namespaces))

# Get cache statistics
stats = namespace_mgr.get_stats()
print(f"Cache hits: {stats['cache_hits']}")
print(f"Cache misses: {stats['cache_misses']}")
print(f"Hit rate: {stats['hit_rate']:.2f}%")
```

### Expression Caching

```python
from gulfofmexico.engine.evaluator import ExpressionEvaluator

# Create evaluator with caching enabled
evaluator = ExpressionEvaluator(enable_cache=True)

# Evaluate expression (automatically cached if pure)
result = evaluator.evaluate(expression, context)

# Clear cache if needed
evaluator.clear_cache()

# Get statistics
stats = evaluator.get_stats()
print(f"Expression cache size: {stats['cache_size']}")
print(f"Hit rate: {stats['hit_rate']:.2f}%")
```

---

## ExecutionContext

### Creating a Context

```python
from gulfofmexico.context import ExecutionContext

# Minimal context
context = ExecutionContext(
    filename="script.gom",
    code="var x = 42!",
)

# Full context with all options
context = ExecutionContext(
    filename="script.gom",
    code="var x = 42!",
    namespaces=[{"builtin": "value"}],  # Initial namespace
    async_statements=[],                  # Async statements
    when_watchers=[],                     # When watchers
    importable_names={},                  # Importable names
    exported_names=[],                    # Exported names
)
```

### Using ExecutionContext

```python
# Update current line (for error reporting)
context.update_line(42)

# Mark value as deleted
context.mark_deleted(some_value)

# Check if value is deleted
if context.is_deleted(some_value):
    print("Value was deleted!")

# Clear caches (when namespace changes)
context.clear_caches()

# Invalidate namespace cache at specific depth
context.invalidate_namespace_cache(depth=2)
```

---

## Error Handling

### Raising Errors with Context

```python
from gulfofmexico.base import raise_error_at_token

# Raise error at specific token location
raise_error_at_token(
    context.filename,
    context.code,
    "Variable not found",
    some_token,
)
```

---

## Performance Benchmarking

### Running Benchmarks

```python
import sys
sys.path.insert(0, '/home/james/GOM')

from benchmarks import (
    benchmark_namespace_lookup,
    benchmark_handler_dispatch,
)

# Benchmark namespace caching
namespace_results = benchmark_namespace_lookup()
print(f"Speedup: {namespace_results['speedup']:.2f}x")

# Benchmark handler dispatch
handler_results = benchmark_handler_dispatch()
print(f"Mean time: {handler_results['mean_time_ms']:.4f}ms")
```

### Custom Benchmarks

```python
import time
import statistics

def benchmark_my_feature(iterations=10000):
    """Benchmark custom feature."""
    times = []
    
    for _ in range(iterations):
        start = time.perf_counter()
        
        # Your code to benchmark
        my_feature()
        
        end = time.perf_counter()
        times.append((end - start) * 1000)  # Convert to ms
    
    return {
        "mean_time_ms": statistics.mean(times),
        "median_time_ms": statistics.median(times),
        "stdev_ms": statistics.stdev(times),
    }
```

---

## Testing

### Unit Testing Handlers

```python
import unittest
from gulfofmexico.engine.handlers.variables import VariableDeclarationHandler
from gulfofmexico.processor.syntax_tree import VariableDeclaration

class TestMyHandler(unittest.TestCase):
    def setUp(self):
        self.handler = VariableDeclarationHandler()
    
    def test_can_handle(self):
        stmt = VariableDeclaration(...)
        self.assertTrue(self.handler.can_handle(stmt))
    
    def test_statement_type(self):
        self.assertEqual(
            self.handler.statement_type,
            VariableDeclaration
        )

if __name__ == "__main__":
    unittest.main()
```

### Integration Testing

```python
import unittest
from gulfofmexico.engine.core import InterpretEngine
from gulfofmexico.context import ExecutionContext

class TestIntegration(unittest.TestCase):
    def test_variable_declaration(self):
        engine = InterpretEngine()
        context = ExecutionContext(
            filename="test.gom",
            code="var x = 42!",
        )
        
        # Parse and execute
        from gulfofmexico.processor.syntax_tree import parse_code
        statements = parse_code(context.code)
        result = engine.execute_statements(statements, context)
        
        # Verify result
        self.assertIsNotNone(result)
```

---

## File Structure Reference

```
gulfofmexico/
├── engine/                  # New modular engine
│   ├── core.py             # InterpretEngine
│   ├── evaluator.py        # Expression evaluation + caching
│   ├── namespace.py        # Namespace management + caching
│   └── handlers/           # Statement handlers
│       ├── __init__.py
│       ├── variables.py
│       ├── control_flow.py
│       ├── functions.py
│       └── special.py
│
├── plugins/                 # Plugin system
│   ├── __init__.py
│   ├── example_custom_statement.py
│   └── example_math_utils.py
│
├── processor/               # Lexer, parser, syntax tree
│   ├── lexer.py
│   ├── syntax_tree.py
│   └── expression_tree.py
│
├── interpreter.py           # Legacy interpreter (still used)
├── context.py              # ExecutionContext
├── handlers.py             # Handler base classes
├── plugin_system.py        # Plugin architecture
├── constants.py            # Configuration
├── utils.py                # Utilities
├── builtin.py              # Built-in types
├── base.py                 # Base utilities
└── serialize.py            # Serialization

tests/                       # Test suite
├── test_variable_handlers.py
├── test_control_flow_handlers.py
├── test_function_handlers.py
├── test_special_handlers.py
└── test_integration.py

benchmarks.py               # Performance benchmarks
```

---

## Configuration Options

### InterpreterConfig

```python
from gulfofmexico.context import InterpreterConfig

config = InterpreterConfig(
    enable_expression_cache=True,   # Enable expression caching
    enable_namespace_cache=True,    # Enable namespace caching
    debug_mode=False,                # Enable debug output
    max_recursion_depth=1000,        # Max recursion depth
)
```

### Constants

```python
from gulfofmexico import constants

# Cache sizes
constants.NAMESPACE_CACHE_SIZE  # Default: 1000
constants.EXPRESSION_CACHE_SIZE # Default: 500

# Confidence levels
constants.MAX_CONFIDENCE        # Default: 100
constants.MIN_CONFIDENCE        # Default: 0
```

---

## Migration from Legacy Code

### Before (Legacy)

```python
from gulfofmexico.interpreter import interpret_code_statements

result = interpret_code_statements(
    statements,
    namespaces,
    async_statements,
    when_watchers,
    importable_names,
    exported_names,
)
```

### After (New Architecture)

```python
from gulfofmexico.engine.core import InterpretEngine
from gulfofmexico.context import ExecutionContext

engine = InterpretEngine()
context = ExecutionContext(
    filename="script.gom",
    code=code_string,
    namespaces=namespaces,
    async_statements=async_statements,
    when_watchers=when_watchers,
    importable_names=importable_names,
    exported_names=exported_names,
)

result = engine.execute_statements(statements, context)
```

**Note**: Legacy interpreter still works! The new architecture includes a compatibility shim.

---

## Common Patterns

### Pattern 1: Adding a New Built-in Function

```python
# 1. Define function
def my_builtin(args: list[GulfOfMexicoValue]) -> GulfOfMexicoValue:
    # Validate args
    if len(args) != 1:
        raise ValueError("my_builtin() takes 1 argument")
    
    # Process and return
    return GulfOfMexicoNumber(42)

# 2. Create plugin
class MyPlugin(Plugin):
    def get_builtin_functions(self):
        return {"my_builtin": my_builtin}

# 3. Register plugin
manager.register_plugin(MyPlugin())
```

### Pattern 2: Adding a New Statement Type

```python
# 1. Define statement in syntax_tree.py
class MyStatement(CodeStatement):
    def __init__(self, token, value):
        self.token = token
        self.value = value

# 2. Create handler
class MyStatementHandler(StatementHandler):
    def can_handle(self, stmt):
        return isinstance(stmt, MyStatement)
    
    def execute(self, stmt, context):
        # Execute logic
        return None
    
    @property
    def statement_type(self):
        return MyStatement

# 3. Register handler
engine.registry.register(MyStatementHandler())
```

### Pattern 3: Caching Custom Data

```python
# Use ExecutionContext caches dictionary
def my_expensive_operation(key, context):
    cache_key = f"my_cache_{key}"
    
    if cache_key in context.caches:
        return context.caches[cache_key]
    
    result = expensive_computation(key)
    context.caches[cache_key] = result
    return result
```

---

## Tips & Best Practices

### Performance

- ✅ Enable caching for production use
- ✅ Clear caches when namespaces change significantly
- ✅ Use plugins for modular features
- ✅ Profile before optimizing

### Code Organization

- ✅ Group related handlers in same file
- ✅ Use plugins for optional features
- ✅ Keep handlers focused (single responsibility)
- ✅ Document handler behavior

### Testing

- ✅ Write unit tests for each handler
- ✅ Test edge cases and error conditions
- ✅ Use integration tests for complex scenarios
- ✅ Benchmark performance-critical code

### Debugging

- ✅ Use `context.current_line` for error locations
- ✅ Check cache statistics to diagnose performance
- ✅ Enable debug mode in config during development
- ✅ Use plugin lifecycle hooks for debugging

---

## Getting Help

### Documentation

- `ARCHITECTURE_GUIDE.md` - Architecture overview
- `HANDLER_MIGRATION_STATUS.md` - Handler details
- `IMPLEMENTATION_SUMMARY.md` - Implementation details
- `REFACTORING_COMPLETE.md` - Quick start guide

### Examples

- `examples/` - Gulf of Mexico code examples
- `gulfofmexico/plugins/` - Example plugins
- `tests/` - Unit and integration test examples
- `benchmarks.py` - Performance benchmark examples

---

## Summary

The new Gulf of Mexico architecture provides:

- ✅ **Modular handlers** for easy maintenance
- ✅ **Plugin system** for extensibility
- ✅ **Performance caching** for speed
- ✅ **Comprehensive testing** for reliability
- ✅ **Clear documentation** for ease of use

Happy coding! 🎉
