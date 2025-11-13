#!/usr/bin/env python3
"""
GulfOfMexico Terminal IDE
A simple terminal-based IDE for Gulf of Mexico development
"""

import os
import sys
import subprocess
import tempfile
from pathlib import Path


class DreamberdTerminalIDE:
    def __init__(self):
        self.current_file = None
        self.code = "print 'Hello, GulfOfMexico!'!"
        self.output = "Welcome to Gulf of Mexico Terminal IDE!\n"
        self.font_size = 12
        self.theme = "dark"

    def display_menu(self):
        print("\n" + "=" * 50)
        print("GulfOfMexico Terminal IDE")
        print("=" * 50)
        print("1. Edit Code")
        print("2. Run Code")
        print("3. Clear Output")
        print("4. New File")
        print("5. Open File")
        print("6. Save File")
        print("7. Change Theme")
        print("8. Change Font Size")
        print("9. Show Help")
        print("0. Exit")
        print("=" * 50)

    def edit_code(self):
        # Create a temporary file for editing
        with tempfile.NamedTemporaryFile(mode="w", suffix=".gom", delete=False) as f:
            f.write(self.code)
            temp_file = f.name

        try:
            # Try to use nano, vim, or emacs
            editors = ["nano", "vim", "emacs", "vi"]
            editor = None
            for ed in editors:
                if subprocess.run(["which", ed], capture_output=True).returncode == 0:
                    editor = ed
                    break

            if editor:
                print(f"Opening {editor} for editing...")
                subprocess.run([editor, temp_file])

                # Read back the edited code
                with open(temp_file, "r") as f:
                    self.code = f.read()
                print("Code updated!")
            else:
                print("No text editor found. Please install nano, vim, or emacs.")
        finally:
            # Clean up temp file
            os.unlink(temp_file)

    def run_code(self):
        if not self.code.strip():
            self.output += "No code to run!\n"
            return

        self.output += "Running Gulf of Mexico code...\n"

        # Save code to temp file
        with tempfile.NamedTemporaryFile(mode="w", suffix=".gom", delete=False) as f:
            f.write(self.code)
            temp_file = f.name

        try:
            # Run the Gulf of Mexico interpreter
            result = subprocess.run(
                ["cargo", "run", "--", temp_file],
                cwd="/home/james/dreamberd-interpreter",
                capture_output=True,
                text=True,
                timeout=30,
            )

            if result.returncode == 0:
                self.output += "Execution completed successfully!\n"
                if result.stdout:
                    self.output += "Output:\n" + result.stdout
            else:
                self.output += f"Execution failed with code {result.returncode}\n"
                if result.stderr:
                    self.output += "Errors:\n" + result.stderr

        except subprocess.TimeoutExpired:
            self.output += "Execution timed out after 30 seconds\n"
        except FileNotFoundError:
            self.output += "GulfOfMexico interpreter not found. Please build it first.\n"
        finally:
            os.unlink(temp_file)

    def clear_output(self):
        self.output = ""

    def new_file(self):
        self.code = ""
        self.current_file = None
        self.output = "New file created\n"

    def open_file(self):
        filename = input("Enter filename to open: ").strip()
        if not filename:
            return

        try:
            with open(filename, "r") as f:
                self.code = f.read()
            self.current_file = Path(filename)
            self.output = f"Opened file: {filename}\n"
        except FileNotFoundError:
            self.output = f"File not found: {filename}\n"
        except Exception as e:
            self.output = f"Error opening file: {e}\n"

    def save_file(self):
        if self.current_file:
            try:
                with open(self.current_file, "w") as f:
                    f.write(self.code)
                self.output = f"Saved to {self.current_file}\n"
            except Exception as e:
                self.output = f"Error saving file: {e}\n"
        else:
            filename = input("Enter filename to save as: ").strip()
            if filename:
                try:
                    with open(filename, "w") as f:
                        f.write(self.code)
                    self.current_file = Path(filename)
                    self.output = f"Saved to {filename}\n"
                except Exception as e:
                    self.output = f"Error saving file: {e}\n"

    def change_theme(self):
        print("Available themes: light, dark, custom")
        theme = input("Enter theme: ").strip().lower()
        if theme in ["light", "dark", "custom"]:
            self.theme = theme
            print(f"Theme changed to {theme}")
        else:
            print("Invalid theme")

    def change_font_size(self):
        try:
            size = int(input("Enter font size (8-24): ").strip())
            if 8 <= size <= 24:
                self.font_size = size
                print(f"Font size changed to {size}")
            else:
                print("Font size must be between 8 and 24")
        except ValueError:
            print("Invalid font size")

    def show_help(self):
        print(
            """
GulfOfMexico Terminal IDE Help:

This is a terminal-based IDE for Gulf of Mexico programming language.

Features:
- Code editing with your preferred text editor
- Code execution with the Gulf of Mexico interpreter
- File operations (new, open, save)
- Theme and font size customization
- Output display and error reporting

GulfOfMexico Language Features:
- Dynamic typing
- Object-oriented programming
- Functional programming constructs
- Built-in print statements
- Variable declarations and assignments

Example Code:
```
const message = 'Hello, World!';
print message;
```

Use the menu to navigate and edit your code!
        """
        )

    def display_status(self):
        print(f"\nCurrent file: {self.current_file or 'None'}")
        print(f"Theme: {self.theme}")
        print(f"Font size: {self.font_size}")
        print(f"Code length: {len(self.code)} characters")

    def run(self):
        print("Welcome to Gulf of Mexico Terminal IDE!")

        while True:
            self.display_status()
            print("\nCurrent Code:")
            print("-" * 30)
            print(self.code[:200] + ("..." if len(self.code) > 200 else ""))
            print("-" * 30)

            print("\nOutput:")
            print("-" * 30)
            print(self.output[-500:] if len(self.output) > 500 else self.output)
            print("-" * 30)

            self.display_menu()
            choice = input("Enter your choice: ").strip()

            if choice == "0":
                print("Goodbye!")
                break
            elif choice == "1":
                self.edit_code()
            elif choice == "2":
                self.run_code()
            elif choice == "3":
                self.clear_output()
            elif choice == "4":
                self.new_file()
            elif choice == "5":
                self.open_file()
            elif choice == "6":
                self.save_file()
            elif choice == "7":
                self.change_theme()
            elif choice == "8":
                self.change_font_size()
            elif choice == "9":
                self.show_help()
            else:
                print("Invalid choice. Please try again.")


if __name__ == "__main__":
    ide = DreamberdTerminalIDE()
    ide.run()
