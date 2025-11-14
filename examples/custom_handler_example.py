"""Example: Creating a custom statement handler for Gulf of Mexico.

This example demonstrates how to add a new 'repeat' statement to the language
that executes code multiple times.

Syntax:
    repeat 5 { print("Hello")! }
"""

from typing import Optional, Type
from gulfofmexico.handlers import StatementHandler
from gulfofmexico.processor.syntax_tree import CodeStatement
from gulfofmexico.builtin import GulfOfMexicoValue, db_to_number
from gulfofmexico.context import ExecutionContext


# Step 1: Define the statement structure (would go in syntax_tree.py)
from dataclasses import dataclass
from gulfofmexico.base import Token


@dataclass
class RepeatStatement(CodeStatement):
    """Represents: repeat <count> { <code> }"""

    keyword: Token  # "repeat"
    count_expression: list[Token]  # Expression for repeat count
    code: list[tuple[CodeStatement, ...]]  # Code to repeat


# Step 2: Create the handler
class RepeatStatementHandler(StatementHandler):
    """Handler for 'repeat' statements."""

    def can_handle(self, statement: CodeStatement) -> bool:
        """Check if this is a repeat statement.

        Args:
            statement: Statement to check

        Returns:
            True if this is a RepeatStatement
        """
        return isinstance(statement, RepeatStatement)

    def execute(
        self,
        statement: CodeStatement,
        context: ExecutionContext,
    ) -> Optional[GulfOfMexicoValue]:
        """Execute the repeat statement.

        Args:
            statement: The repeat statement to execute
            context: Execution context

        Returns:
            None (repeat doesn't return a value)
        """
        # Import here to avoid circular imports
        from gulfofmexico.interpreter import (
            evaluate_expression,
            interpret_code_statements,
        )

        # Cast to correct type for type checker
        repeat_stmt = statement
        assert isinstance(repeat_stmt, RepeatStatement)

        # Evaluate the count expression
        count_value = evaluate_expression(
            repeat_stmt.count_expression,
            context.namespaces,
            context.async_statements,
            context.when_watchers,
        )

        # Convert to number
        count_number = db_to_number(count_value)
        count = int(count_number.value)

        # Validate count
        if count < 0:
            from gulfofmexico.base import raise_error_at_token

            raise_error_at_token(
                context.filename,
                context.code,
                f"Repeat count must be non-negative, got {count}",
                repeat_stmt.keyword,
            )

        # Execute the code block 'count' times
        for i in range(count):
            # You could even add a loop variable to the namespace:
            # context.namespaces[-1]["_repeat_index"] = Name("_repeat_index",
            #                                                GulfOfMexicoNumber(i))

            result = interpret_code_statements(
                repeat_stmt.code,
                context.namespaces,
                context.async_statements,
                context.when_watchers,
                context.importable_names,
                context.exported_names,
            )

            # If code returns a value, stop repeating
            if result is not None:
                return result

        return None

    @property
    def statement_type(self) -> Type[CodeStatement]:
        """The statement type this handler processes.

        Returns:
            RepeatStatement class
        """
        return RepeatStatement


# Step 3: Register the handler
def setup_custom_handlers(registry):
    """Register custom statement handlers.

    Args:
        registry: The HandlerRegistry to register with
    """
    from gulfofmexico.handlers import HandlerRegistry

    # Register the repeat handler
    registry.register(RepeatStatementHandler())

    print("Custom handlers registered!")


# Example usage:
if __name__ == "__main__":
    # This demonstrates the concept - actual integration would be in the
    # main interpreter initialization code

    from gulfofmexico.handlers import HandlerRegistry
    from gulfofmexico.context import ExecutionContext

    # Create registry
    registry = HandlerRegistry()

    # Register custom handlers
    setup_custom_handlers(registry)

    # Create execution context
    context = ExecutionContext(
        filename="example.gom",
        code='repeat 3 { print("Hello")! }',
        namespaces=[{}],
        async_statements=[],
        when_watchers=[{}],
        importable_names={},
        exported_names=[],
    )

    # Create a mock repeat statement
    # (In reality, this would come from the parser)
    from gulfofmexico.processor.expression_tree import ValueNode
    from gulfofmexico.builtin import GulfOfMexicoNumber

    repeat_stmt = RepeatStatement(
        keyword=Token(None, None, None, None),  # type: ignore
        count_expression=ValueNode(GulfOfMexicoNumber(3)),  # type: ignore
        code=[],  # Empty code block for demo
    )

    # Execute using the registry
    try:
        result = registry.execute_statement(repeat_stmt, context)
        print(f"Execution complete! Result: {result}")
    except Exception as e:
        print(f"Error: {e}")


# Additional Examples:


class DebugPrintHandler(StatementHandler):
    """Handler that prints debug information for any statement."""

    def can_handle(self, statement: CodeStatement) -> bool:
        """Can handle any statement if debug mode is enabled.

        This is an example of a "decorator" handler that wraps
        execution of other statements.
        """
        return True  # Handles everything (should be registered last!)

    def execute(
        self,
        statement: CodeStatement,
        context: ExecutionContext,
    ) -> Optional[GulfOfMexicoValue]:
        """Print debug info before/after execution.

        This is a simplified example - in practice, you'd delegate
        to the actual handler for the statement type.
        """
        print(f"[DEBUG] Executing: {type(statement).__name__}")
        print(f"[DEBUG] Line: {context.current_line}")
        print(f"[DEBUG] Namespace depth: {len(context.namespaces)}")

        # In a real implementation, you'd call the actual handler here
        # For this example, we just return None
        return None

    @property
    def statement_type(self) -> Type[CodeStatement]:
        return CodeStatement


# Performance monitoring handler
class PerformanceMonitoringHandler(StatementHandler):
    """Handler that tracks execution time for statements."""

    def __init__(self):
        super().__init__()
        self.execution_times: dict[str, list[float]] = {}

    def can_handle(self, statement: CodeStatement) -> bool:
        return True

    def execute(
        self,
        statement: CodeStatement,
        context: ExecutionContext,
    ) -> Optional[GulfOfMexicoValue]:
        """Measure execution time."""
        import time

        stmt_type = type(statement).__name__

        start = time.perf_counter()
        # Delegate to actual handler (simplified for example)
        result = None
        end = time.perf_counter()

        elapsed = end - start
        if stmt_type not in self.execution_times:
            self.execution_times[stmt_type] = []
        self.execution_times[stmt_type].append(elapsed)

        return result

    @property
    def statement_type(self) -> Type[CodeStatement]:
        return CodeStatement

    def print_stats(self):
        """Print performance statistics."""
        print("\n=== Performance Statistics ===")
        for stmt_type, times in self.execution_times.items():
            avg_time = sum(times) / len(times)
            total_time = sum(times)
            print(f"{stmt_type}:")
            print(f"  Count: {len(times)}")
            print(f"  Avg: {avg_time*1000:.3f}ms")
            print(f"  Total: {total_time*1000:.3f}ms")
