#!/usr/bin/env python3
"""
GulfOfMexico GUI IDE
A full-featured graphical IDE for Gulf of Mexico development
"""

import tkinter as tk
from tkinter import ttk, filedialog, messagebox, scrolledtext
import os
from pathlib import Path
import json
import re


class DreamberdGUIIDE:
    def __init__(self, root):
        self.root = root
        self.root.title("GulfOfMexico IDE")
        self.root.geometry("1200x800")

        # Initialize variables
        self.current_file = None
        self.code = "print 'Hello, GulfOfMexico!'!"
        self.output_text = "Welcome to Gulf of Mexico GUI IDE!\n"
        self.debug_info = ""
        self.breakpoints = set()
        self.current_line = 1
        self.is_debugging = False

        # Theme and font settings
        self.themes = {
            "light": {
                "bg": "#ffffff",
                "fg": "#000000",
                "editor_bg": "#f8f8f8",
                "editor_fg": "#000000",
                "output_bg": "#f0f0f0",
                "output_fg": "#000000",
                "highlight": "#e6f3ff",
                "keyword": "#0000ff",
                "string": "#008000",
                "comment": "#808080",
                "error": "#ff0000",
            },
            "dark": {
                "bg": "#2b2b2b",
                "fg": "#ffffff",
                "editor_bg": "#1e1e1e",
                "editor_fg": "#ffffff",
                "output_bg": "#1a1a1a",
                "output_fg": "#ffffff",
                "highlight": "#264f78",
                "keyword": "#569cd6",
                "string": "#ce9178",
                "comment": "#6a9955",
                "error": "#f44747",
            },
            "custom": {
                "bg": "#1a1a2e",
                "fg": "#e94560",
                "editor_bg": "#16213e",
                "editor_fg": "#e94560",
                "output_bg": "#0f3460",
                "output_fg": "#e94560",
                "highlight": "#533483",
                "keyword": "#f39c12",
                "string": "#27ae60",
                "comment": "#95a5a6",
                "error": "#e74c3c",
            },
        }

        self.current_theme = "dark"
        self.font_family = "Consolas"
        self.font_size = 11

        # Load settings
        self.load_settings()

        # Syntax highlighting patterns
        self.syntax_patterns = {
            "keyword": r"\b(const|var|function|if|else|for|while|return|"
            r"print|class|try|catch|finally|true|false|null)\b",
            "string": r'(["\'])(?:(?=(\\?))\2.)*?\1',
            "comment": r"//.*$|/\*.*?\*/",
            "number": r"\b\d+(\.\d+)?\b",
        }

        # Create GUI components
        self.create_menu()
        self.create_toolbar()
        self.create_main_layout()
        self.create_status_bar()

        # Apply theme
        self.apply_theme()

        # Bind events
        self.bind_events()

    def create_menu(self):
        menubar = tk.Menu(self.root)
        self.root.config(menu=menubar)

        # File menu
        file_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="File", menu=file_menu)
        file_menu.add_command(label="New", command=self.new_file, accelerator="Ctrl+N")
        file_menu.add_command(
            label="Open", command=self.open_file, accelerator="Ctrl+O"
        )
        file_menu.add_command(
            label="Save", command=self.save_file, accelerator="Ctrl+S"
        )
        file_menu.add_command(
            label="Save As", command=self.save_file_as, accelerator="Ctrl+Shift+S"
        )
        file_menu.add_separator()
        file_menu.add_command(
            label="Exit", command=self.on_closing, accelerator="Ctrl+Q"
        )

        # Edit menu
        edit_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="Edit", menu=edit_menu)
        edit_menu.add_command(label="Undo", command=self.undo, accelerator="Ctrl+Z")
        edit_menu.add_command(label="Redo", command=self.redo, accelerator="Ctrl+Y")
        edit_menu.add_separator()
        edit_menu.add_command(label="Cut", command=self.cut, accelerator="Ctrl+X")
        edit_menu.add_command(label="Copy", command=self.copy, accelerator="Ctrl+C")
        edit_menu.add_command(label="Paste", command=self.paste, accelerator="Ctrl+V")
        edit_menu.add_separator()
        edit_menu.add_command(
            label="Find", command=self.find_text, accelerator="Ctrl+F"
        )
        edit_menu.add_command(
            label="Replace", command=self.replace_text, accelerator="Ctrl+H"
        )

        # View menu
        view_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="View", menu=view_menu)
        view_menu.add_command(
            label="Zoom In", command=self.zoom_in, accelerator="Ctrl+="
        )
        view_menu.add_command(
            label="Zoom Out", command=self.zoom_out, accelerator="Ctrl+-"
        )
        view_menu.add_separator()
        view_menu.add_command(
            label="Toggle Line Numbers", command=self.toggle_line_numbers
        )

        # Run menu
        run_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="Run", menu=run_menu)
        run_menu.add_command(label="Run Code", command=self.run_code, accelerator="F5")
        run_menu.add_command(
            label="Debug Code", command=self.debug_code, accelerator="F10"
        )
        run_menu.add_separator()
        run_menu.add_command(label="Clear Output", command=self.clear_output)

        # Tools menu
        tools_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="Tools", menu=tools_menu)
        tools_menu.add_command(label="Settings", command=self.show_settings)
        tools_menu.add_command(label="Theme", command=self.change_theme)
        tools_menu.add_command(label="Font", command=self.change_font)

        # Help menu
        help_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="Help", menu=help_menu)
        help_menu.add_command(label="About", command=self.show_about)
        help_menu.add_command(label="GulfOfMexico Docs", command=self.show_docs)

    def create_toolbar(self):
        toolbar = ttk.Frame(self.root)
        toolbar.pack(side=tk.TOP, fill=tk.X, padx=2, pady=2)

        # File operations
        ttk.Button(toolbar, text="New", command=self.new_file).pack(
            side=tk.LEFT, padx=2
        )
        ttk.Button(toolbar, text="Open", command=self.open_file).pack(
            side=tk.LEFT, padx=2
        )
        ttk.Button(toolbar, text="Save", command=self.save_file).pack(
            side=tk.LEFT, padx=2
        )

        ttk.Separator(toolbar, orient=tk.VERTICAL).pack(side=tk.LEFT, fill=tk.Y, padx=5)

        # Run operations
        ttk.Button(toolbar, text="Run", command=self.run_code).pack(
            side=tk.LEFT, padx=2
        )
        ttk.Button(toolbar, text="Debug", command=self.debug_code).pack(
            side=tk.LEFT, padx=2
        )
        ttk.Button(toolbar, text="Clear", command=self.clear_output).pack(
            side=tk.LEFT, padx=2
        )

        ttk.Separator(toolbar, orient=tk.VERTICAL).pack(side=tk.LEFT, fill=tk.Y, padx=5)

        # Theme selector
        ttk.Label(toolbar, text="Theme:").pack(side=tk.LEFT, padx=2)
        self.theme_var = tk.StringVar(value=self.current_theme)
        theme_combo = ttk.Combobox(
            toolbar,
            textvariable=self.theme_var,
            values=list(self.themes.keys()),
            width=8,
        )
        theme_combo.pack(side=tk.LEFT, padx=2)
        theme_combo.bind("<<ComboboxSelected>>", lambda e: self.apply_theme())

    def create_main_layout(self):
        # Create paned window for resizable panels
        main_paned = ttk.PanedWindow(self.root, orient=tk.HORIZONTAL)
        main_paned.pack(fill=tk.BOTH, expand=True, padx=5, pady=5)

        # Left panel - Editor
        left_frame = ttk.Frame(main_paned)
        main_paned.add(left_frame, weight=3)

        # Editor with line numbers
        editor_frame = ttk.Frame(left_frame)
        editor_frame.pack(fill=tk.BOTH, expand=True)

        # Line numbers
        self.line_numbers = tk.Text(
            editor_frame,
            width=4,
            padx=3,
            takefocus=0,
            border=0,
            background=self.themes[self.current_theme]["editor_bg"],
            foreground=self.themes[self.current_theme]["comment"],
            font=(self.font_family, self.font_size),
        )
        self.line_numbers.pack(side=tk.LEFT, fill=tk.Y)

        # Code editor
        self.code_editor = scrolledtext.ScrolledText(
            editor_frame,
            wrap=tk.WORD,
            font=(self.font_family, self.font_size),
            undo=True,
        )
        self.code_editor.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        self.code_editor.insert(tk.END, self.code)

        # Right panel - Output and Debug
        right_frame = ttk.Frame(main_paned)
        main_paned.add(right_frame, weight=2)

        # Notebook for tabs
        self.notebook = ttk.Notebook(right_frame)
        self.notebook.pack(fill=tk.BOTH, expand=True)

        # Output tab
        output_frame = ttk.Frame(self.notebook)
        self.notebook.add(output_frame, text="Output")

        self.output_text_widget = scrolledtext.ScrolledText(
            output_frame, wrap=tk.WORD, font=(self.font_family, self.font_size - 1)
        )
        self.output_text_widget.pack(fill=tk.BOTH, expand=True)
        self.output_text_widget.insert(tk.END, self.output_text)

        # Debug tab
        debug_frame = ttk.Frame(self.notebook)
        self.notebook.add(debug_frame, text="Debug")

        # Debug controls
        debug_controls = ttk.Frame(debug_frame)
        debug_controls.pack(fill=tk.X, pady=5)

        ttk.Button(debug_controls, text="Step Over", command=self.debug_step_over).pack(
            side=tk.LEFT, padx=2
        )
        ttk.Button(debug_controls, text="Step Into", command=self.debug_step_into).pack(
            side=tk.LEFT, padx=2
        )
        ttk.Button(debug_controls, text="Continue", command=self.debug_continue).pack(
            side=tk.LEFT, padx=2
        )
        ttk.Button(debug_controls, text="Stop", command=self.debug_stop).pack(
            side=tk.LEFT, padx=2
        )

        # Debug output
        self.debug_text = scrolledtext.ScrolledText(
            debug_frame, wrap=tk.WORD, font=(self.font_family, self.font_size - 1)
        )
        self.debug_text.pack(fill=tk.BOTH, expand=True)

        # Variables panel
        vars_frame = ttk.Frame(debug_frame)
        vars_frame.pack(fill=tk.X, pady=5)

        ttk.Label(vars_frame, text="Variables:").pack(side=tk.LEFT)
        self.vars_text = scrolledtext.ScrolledText(
            vars_frame,
            height=8,
            wrap=tk.WORD,
            font=(self.font_family, self.font_size - 2),
        )
        self.vars_text.pack(fill=tk.BOTH, expand=True)

    def create_status_bar(self):
        self.status_bar = ttk.Frame(self.root)
        self.status_bar.pack(side=tk.BOTTOM, fill=tk.X)

        self.status_label = ttk.Label(self.status_bar, text="Ready")
        self.status_label.pack(side=tk.LEFT, padx=5)

        self.line_col_label = ttk.Label(self.status_bar, text="Ln 1, Col 1")
        self.line_col_label.pack(side=tk.RIGHT, padx=5)

    def bind_events(self):
        # Keyboard shortcuts
        self.root.bind("<Control-n>", lambda e: self.new_file())
        self.root.bind("<Control-o>", lambda e: self.open_file())
        self.root.bind("<Control-s>", lambda e: self.save_file())
        self.root.bind("<Control-Shift-S>", lambda e: self.save_file_as())
        self.root.bind("<Control-q>", lambda e: self.on_closing())
        self.root.bind("<Control-z>", lambda e: self.undo())
        self.root.bind("<Control-y>", lambda e: self.redo())
        self.root.bind("<Control-x>", lambda e: self.cut())
        self.root.bind("<Control-c>", lambda e: self.copy())
        self.root.bind("<Control-v>", lambda e: self.paste())
        self.root.bind("<Control-f>", lambda e: self.find_text())
        self.root.bind("<Control-h>", lambda e: self.replace_text())
        self.root.bind("<F5>", lambda e: self.run_code())
        self.root.bind("<F10>", lambda e: self.debug_code())
        self.root.bind("<Control-plus>", lambda e: self.zoom_in())
        self.root.bind("<Control-minus>", lambda e: self.zoom_out())

        # Editor events
        self.code_editor.bind("<KeyRelease>", self.on_code_change)
        self.code_editor.bind("<Button-1>", self.update_cursor_position)
        self.code_editor.bind("<Key>", self.update_cursor_position)

        # Line numbers update
        self.code_editor.bind("<KeyRelease>", self.update_line_numbers)
        self.code_editor.bind("<MouseWheel>", self.update_line_numbers)

        # Window close event
        self.root.protocol("WM_DELETE_WINDOW", self.on_closing)

    def apply_theme(self):
        theme = self.themes[self.current_theme]

        # Update root
        self.root.configure(bg=theme["bg"])

        # Update style
        style = ttk.Style()
        style.configure("TFrame", background=theme["bg"])
        style.configure("TLabel", background=theme["bg"], foreground=theme["fg"])
        style.configure("TButton", background=theme["bg"])
        style.configure("TNotebook", background=theme["bg"])
        style.configure("TNotebook.Tab", background=theme["bg"], foreground=theme["fg"])

        # Update text widgets
        self.code_editor.configure(
            bg=theme["editor_bg"], fg=theme["editor_fg"], insertbackground=theme["fg"]
        )
        self.line_numbers.configure(bg=theme["editor_bg"], fg=theme["comment"])
        self.output_text_widget.configure(bg=theme["output_bg"], fg=theme["output_fg"])
        self.debug_text.configure(bg=theme["output_bg"], fg=theme["output_fg"])
        self.vars_text.configure(bg=theme["output_bg"], fg=theme["output_fg"])

        # Apply syntax highlighting
        self.highlight_syntax()

        # Save theme preference
        self.save_settings()

    def highlight_syntax(self):
        # Remove existing tags
        for tag in self.code_editor.tag_names():
            self.code_editor.tag_delete(tag)

        # Configure tags
        theme = self.themes[self.current_theme]
        self.code_editor.tag_configure("keyword", foreground=theme["keyword"])
        self.code_editor.tag_configure("string", foreground=theme["string"])
        self.code_editor.tag_configure("comment", foreground=theme["comment"])
        self.code_editor.tag_configure("number", foreground=theme["keyword"])

        # Get code content
        code = self.code_editor.get("1.0", tk.END)

        # Apply highlighting
        for pattern_name, pattern in self.syntax_patterns.items():
            for match in re.finditer(pattern, code, re.MULTILINE):
                start = f"1.0+{match.start()}c"
                end = f"1.0+{match.end()}c"
                self.code_editor.tag_add(pattern_name, start, end)

    def update_line_numbers(self, event=None):
        # Get number of lines
        lines = int(self.code_editor.index("end-1c").split(".")[0])

        # Update line numbers
        line_numbers_text = "\n".join(str(i) for i in range(1, lines + 1))
        self.line_numbers.delete("1.0", tk.END)
        self.line_numbers.insert("1.0", line_numbers_text)

        # Sync scrolling
        self.line_numbers.yview_moveto(self.code_editor.yview()[0])

    def update_cursor_position(self, event=None):
        cursor_pos = self.code_editor.index(tk.INSERT)
        line, col = cursor_pos.split(".")
        self.line_col_label.config(text=f"Ln {line}, Col {int(col) + 1}")

    def on_code_change(self, event=None):
        self.highlight_syntax()
        self.update_line_numbers()
        self.update_cursor_position()

    def new_file(self):
        if self.code_editor.get("1.0", tk.END).strip() and self.current_file:
            if messagebox.askyesno(
                "Save", "Save current file before creating new one?"
            ):
                self.save_file()

        self.code_editor.delete("1.0", tk.END)
        self.code_editor.insert("1.0", "print 'Hello, GulfOfMexico!'!")
        self.current_file = None
        self.root.title("GulfOfMexico IDE - Untitled")
        self.status_label.config(text="New file created")

    def open_file(self):
        filename = filedialog.askopenfilename(
            title="Open Gulf of Mexico File",
            filetypes=[("GulfOfMexico files", "*.gom"), ("All files", "*.*")],
        )

        if filename:
            try:
                with open(filename, "r", encoding="utf-8") as f:
                    content = f.read()

                self.code_editor.delete("1.0", tk.END)
                self.code_editor.insert("1.0", content)
                self.current_file = Path(filename)
                self.root.title(f"GulfOfMexico IDE - {filename}")
                self.status_label.config(text=f"Opened {filename}")
                self.on_code_change()

            except Exception as e:
                messagebox.showerror("Error", f"Failed to open file: {e}")

    def save_file(self):
        if self.current_file:
            self._save_to_file(self.current_file)
        else:
            self.save_file_as()

    def save_file_as(self):
        filename = filedialog.asksaveasfilename(
            title="Save Gulf of Mexico File",
            defaultextension=".gom",
            filetypes=[("GulfOfMexico files", "*.gom"), ("All files", "*.*")],
        )

        if filename:
            self.current_file = Path(filename)
            self._save_to_file(self.current_file)
            self.root.title(f"GulfOfMexico IDE - {filename}")

    def _save_to_file(self, filepath):
        try:
            content = self.code_editor.get("1.0", tk.END)
            with open(filepath, "w", encoding="utf-8") as f:
                f.write(content.rstrip() + "\n")

            self.status_label.config(text=f"Saved {filepath}")

        except Exception as e:
            messagebox.showerror("Error", f"Failed to save file: {e}")

    def run_code(self):
        code = self.code_editor.get("1.0", tk.END).strip()
        if not code:
            self.output_text_widget.insert(tk.END, "No code to run!\n")
            return

        self.output_text_widget.insert(tk.END, "Running Gulf of Mexico code...\n")
        self.status_label.config(text="Running...")

        # Execute synchronously - Gulf of Mexico execution is fast
        self._execute_code(code)

    def _execute_code(self, code):
        try:
            # Save code to temp file
            import tempfile
            import io
            from contextlib import redirect_stdout

            with tempfile.NamedTemporaryFile(mode="w", suffix=".gom", delete=False) as f:
                f.write(code)
                temp_file = f.name

            try:
                # Import required functions
                from dreamberd.processor.lexer import tokenize
                from dreamberd.processor.syntax_tree import generate_syntax_tree
                from dreamberd.interpreter import (
                    load_globals,
                    load_global_dreamberd_variables,
                    load_public_global_variables,
                    interpret_code_statements_main_wrapper,
                )
                from dreamberd.builtin import KEYWORDS

                # Capture stdout
                f = io.StringIO()
                with redirect_stdout(f):
                    # Execute code synchronously (modified version of run_file without waiting)
                    filename = temp_file
                    tokens = tokenize(filename, code)
                    statements = generate_syntax_tree(filename, tokens, code)

                    # Load variables and run the code
                    namespaces = [KEYWORDS.copy()]  # type: ignore
                    exported_names = []
                    importable_names = {}
                    load_globals(
                        filename,
                        code,
                        {},
                        set(),
                        exported_names,
                        importable_names.get(filename, {}),
                    )
                    load_global_dreamberd_variables(namespaces)  # type: ignore
                    load_public_global_variables(namespaces)  # type: ignore
                    interpret_code_statements_main_wrapper(
                        statements, namespaces, [], [{}]  # type: ignore
                    )

                output = f.getvalue()

                # Filter out the warning message if present (including ANSI color codes)
                import re

                output_lines = output.split("\n")
                filtered_lines = []
                for line in output_lines:
                    # Remove ANSI escape sequences and check for warning
                    clean_line = re.sub(r"\x1b\[[0-9;]*m", "", line)
                    if not clean_line.startswith(
                        "Warning: Could not load public global variables"
                    ):
                        filtered_lines.append(line)
                output = "\n".join(filtered_lines).strip()

                self._update_output_success(output)

            except Exception as e:
                error_msg = str(e)
                self._update_output_error(error_msg)

        except Exception as e:
            error_msg = str(e)
            self._update_output_error(error_msg)
        finally:
            # Clean up temp file
            if "temp_file" in locals():
                os.unlink(temp_file)

    def _update_output(self, result):
        if result.returncode == 0:
            self.output_text_widget.insert(
                tk.END, "✓ Execution completed successfully!\n"
            )
            if result.stdout:
                self.output_text_widget.insert(tk.END, f"Output:\n{result.stdout}")
        else:
            self.output_text_widget.insert(
                tk.END, f"✗ Execution failed (code {result.returncode})\n"
            )
            if result.stderr:
                self.output_text_widget.insert(tk.END, f"Errors:\n{result.stderr}")

        self.output_text_widget.see(tk.END)
        self.status_label.config(text="Ready")

    def _update_output_error(self, error):
        # Strip ANSI color codes from error messages
        import re

        clean_error = re.sub(r"\x1b\[[0-9;]*m", "", str(error))
        self.output_text_widget.insert(tk.END, f"Error: {clean_error}\n")
        self.output_text_widget.see(tk.END)
        self.status_label.config(text="Error")

    def _update_output_success(self, output):
        self.output_text_widget.insert(tk.END, "✓ Execution completed successfully!\n")
        if output:
            self.output_text_widget.insert(tk.END, f"Output:\n{output}")
        self.output_text_widget.see(tk.END)
        self.status_label.config(text="Ready")

    def debug_code(self):
        self.is_debugging = True
        self.notebook.select(1)  # Switch to debug tab
        self.debug_text.insert(tk.END, "Debug mode started...\n")
        # TODO: Implement actual debugging logic
        self.debug_text.insert(tk.END, "Debugging not yet implemented\n")

    def debug_step_over(self):
        if self.is_debugging:
            self.debug_text.insert(tk.END, "Step over...\n")

    def debug_step_into(self):
        if self.is_debugging:
            self.debug_text.insert(tk.END, "Step into...\n")

    def debug_continue(self):
        if self.is_debugging:
            self.debug_text.insert(tk.END, "Continue...\n")

    def debug_stop(self):
        self.is_debugging = False
        self.debug_text.insert(tk.END, "Debug stopped\n")

    def clear_output(self):
        self.output_text_widget.delete("1.0", tk.END)
        self.debug_text.delete("1.0", tk.END)
        self.vars_text.delete("1.0", tk.END)

    def change_theme(self):
        theme_window = tk.Toplevel(self.root)
        theme_window.title("Select Theme")
        theme_window.geometry("300x200")

        ttk.Label(theme_window, text="Choose a theme:").pack(pady=10)

        for theme_name in self.themes.keys():
            ttk.Button(
                theme_window,
                text=theme_name.capitalize(),
                command=lambda t=theme_name: self._set_theme(t, theme_window),
            ).pack(pady=5)

    def _set_theme(self, theme_name, window):
        self.current_theme = theme_name
        self.theme_var.set(theme_name)
        self.apply_theme()
        window.destroy()

    def change_font(self):
        font_window = tk.Toplevel(self.root)
        font_window.title("Font Settings")
        font_window.geometry("400x200")

        # Font family
        ttk.Label(font_window, text="Font Family:").grid(
            row=0, column=0, padx=5, pady=5
        )
        font_families = [
            "Consolas",
            "Courier New",
            "Monaco",
            "Menlo",
            "DejaVu Sans Mono",
            "Liberation Mono",
        ]
        font_var = tk.StringVar(value=self.font_family)
        font_combo = ttk.Combobox(
            font_window, textvariable=font_var, values=font_families
        )
        font_combo.grid(row=0, column=1, padx=5, pady=5)

        # Font size
        ttk.Label(font_window, text="Font Size:").grid(row=1, column=0, padx=5, pady=5)
        size_var = tk.StringVar(value=str(self.font_size))
        size_combo = ttk.Combobox(
            font_window, textvariable=size_var, values=[str(i) for i in range(8, 25)]
        )
        size_combo.grid(row=1, column=1, padx=5, pady=5)

        def apply_font():
            self.font_family = font_var.get()
            self.font_size = int(size_var.get())
            self._update_font()
            font_window.destroy()

        ttk.Button(font_window, text="Apply", command=apply_font).grid(
            row=2, column=0, columnspan=2, pady=10
        )

    def _update_font(self):
        font_config = (self.font_family, self.font_size)
        small_font = (self.font_family, self.font_size - 1)
        tiny_font = (self.font_family, self.font_size - 2)

        self.code_editor.configure(font=font_config)
        self.line_numbers.configure(font=font_config)
        self.output_text_widget.configure(font=small_font)
        self.debug_text.configure(font=small_font)
        self.vars_text.configure(font=tiny_font)

        self.save_settings()

    def zoom_in(self):
        if self.font_size < 24:
            self.font_size += 1
            self._update_font()

    def zoom_out(self):
        if self.font_size > 8:
            self.font_size -= 1
            self._update_font()

    def toggle_line_numbers(self):
        if self.line_numbers.winfo_ismapped():
            self.line_numbers.pack_forget()
        else:
            self.line_numbers.pack(side=tk.LEFT, fill=tk.Y)

    def undo(self):
        try:
            self.code_editor.edit_undo()
        except tk.TclError:
            pass

    def redo(self):
        try:
            self.code_editor.edit_redo()
        except tk.TclError:
            pass

    def cut(self):
        self.code_editor.event_generate("<<Cut>>")

    def copy(self):
        self.code_editor.event_generate("<<Copy>>")

    def paste(self):
        self.code_editor.event_generate("<<Paste>>")

    def find_text(self):
        # Simple find dialog
        find_window = tk.Toplevel(self.root)
        find_window.title("Find")
        find_window.geometry("300x100")

        ttk.Label(find_window, text="Find:").grid(row=0, column=0, padx=5, pady=5)
        find_var = tk.StringVar()
        find_entry = ttk.Entry(find_window, textvariable=find_var)
        find_entry.grid(row=0, column=1, padx=5, pady=5)

        def do_find():
            text = find_var.get()
            if text:
                start = self.code_editor.search(text, tk.INSERT, tk.END)
                if start:
                    end = f"{start}+{len(text)}c"
                    self.code_editor.tag_remove("sel", "1.0", tk.END)
                    self.code_editor.tag_add("sel", start, end)
                    self.code_editor.mark_set(tk.INSERT, end)
                    self.code_editor.see(start)

        ttk.Button(find_window, text="Find", command=do_find).grid(
            row=1, column=0, columnspan=2, pady=5
        )
        find_entry.bind("<Return>", lambda e: do_find())

    def replace_text(self):
        replace_window = tk.Toplevel(self.root)
        replace_window.title("Replace")
        replace_window.geometry("400x120")

        ttk.Label(replace_window, text="Find:").grid(row=0, column=0, padx=5, pady=5)
        find_var = tk.StringVar()
        find_entry = ttk.Entry(replace_window, textvariable=find_var)
        find_entry.grid(row=0, column=1, padx=5, pady=5)

        ttk.Label(replace_window, text="Replace:").grid(row=1, column=0, padx=5, pady=5)
        replace_var = tk.StringVar()
        replace_entry = ttk.Entry(replace_window, textvariable=replace_var)
        replace_entry.grid(row=1, column=1, padx=5, pady=5)

        def do_replace():
            find_text = find_var.get()
            replace_text = replace_var.get()
            if find_text:
                content = self.code_editor.get("1.0", tk.END)
                new_content = content.replace(find_text, replace_text)
                self.code_editor.delete("1.0", tk.END)
                self.code_editor.insert("1.0", new_content)
                self.on_code_change()

        ttk.Button(replace_window, text="Replace All", command=do_replace).grid(
            row=2, column=0, columnspan=2, pady=5
        )

    def show_settings(self):
        settings_window = tk.Toplevel(self.root)
        settings_window.title("Settings")
        settings_window.geometry("400x300")

        # Settings content would go here
        ttk.Label(settings_window, text="Settings panel - Coming soon!").pack(pady=20)

    def show_about(self):
        messagebox.showinfo(
            "About Gulf of Mexico IDE",
            "GulfOfMexico GUI IDE v1.0\n\n"
            "A full-featured integrated development environment\n"
            "for the Gulf of Mexico programming language.\n\n"
            "Features:\n"
            "• Code editor with syntax highlighting\n"
            "• Output window\n"
            "• Debug support\n"
            "• Multiple themes\n"
            "• Font customization",
        )

    def show_docs(self):
        messagebox.showinfo(
            "GulfOfMexico Documentation",
            "GulfOfMexico Language Features:\n\n"
            "• Dynamic typing\n"
            "• Object-oriented programming\n"
            "• Functional programming\n"
            "• Built-in print statements\n"
            "• Variable declarations\n\n"
            "Example:\n"
            "const message = 'Hello, World!';\nprint message;",
        )

    def load_settings(self):
        try:
            settings_file = Path.home() / ".dreamberd_ide_settings.json"
            if settings_file.exists():
                with open(settings_file, "r") as f:
                    settings = json.load(f)
                    self.current_theme = settings.get("theme", "dark")
                    self.font_family = settings.get("font_family", "Consolas")
                    self.font_size = settings.get("font_size", 11)
        except:
            pass

    def save_settings(self):
        try:
            settings_file = Path.home() / ".dreamberd_ide_settings.json"
            settings = {
                "theme": self.current_theme,
                "font_family": self.font_family,
                "font_size": self.font_size,
            }
            with open(settings_file, "w") as f:
                json.dump(settings, f)
        except:
            pass

    def on_closing(self):
        if self.code_editor.get("1.0", tk.END).strip() and not self.current_file:
            if messagebox.askyesno("Save", "Save current work before exiting?"):
                self.save_file()

        self.save_settings()
        self.root.destroy()


def main():
    root = tk.Tk()
    app = DreamberdGUIIDE(root)
    root.mainloop()


if __name__ == "__main__":
    main()
