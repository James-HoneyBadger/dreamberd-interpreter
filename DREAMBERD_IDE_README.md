# Gulf of Mexico GUI IDE

A full-featured graphical integrated development environment for Gulf of Mexico programming.

## Features

### Code Editor

- **Syntax Highlighting**: Automatic highlighting of Gulf of Mexico keywords, strings, comments, and numbers
- **Line Numbers**: Toggle-able line numbers for easy navigation
- **Multi-line Editing**: Full text editing with undo/redo support
- **Find & Replace**: Search and replace functionality
- **Font Customization**: Adjustable font family and size

### Output Window

- **Execution Results**: Displays program output and execution status
- **Error Reporting**: Shows compilation and runtime errors
- **Clear Function**: Easy output clearing

### Debug Screen

- **Debug Controls**: Step over, step into, continue, and stop debugging
- **Variable Inspection**: View variable values during debugging
- **Breakpoint Support**: Set and manage breakpoints (framework ready)

### Themes

- **Light Theme**: Clean white background with dark text
- **Dark Theme**: Modern dark theme for reduced eye strain
- **Custom Theme**: Unique purple/blue color scheme

### File Operations

- **New/Open/Save**: Standard file operations with dialogs
- **Auto-save**: Settings persistence across sessions
- **File Types**: Native support for `.gom` Gulf of Mexico files

### Keyboard Shortcuts

- `Ctrl+N`: New file
- `Ctrl+O`: Open file
- `Ctrl+S`: Save file
- `Ctrl+Shift+S`: Save as
- `Ctrl+Q`: Exit
- `Ctrl+Z`: Undo
- `Ctrl+Y`: Redo
- `Ctrl+X/C/V`: Cut/Copy/Paste
- `Ctrl+F`: Find
- `Ctrl+H`: Replace
- `F5`: Run code
- `F10`: Debug code
- `Ctrl++`: Zoom in
- `Ctrl+-`: Zoom out

## Installation

The GUI IDE is included with the Gulf of Mexico Python package. Install dependencies:

```bash
pip install -e .
```

Or with Poetry:

```bash
poetry install
```

## Usage

### Running the IDE

```bash
# Direct execution
python dreamberd_ide.py

# Via Poetry script
poetry run dreamberd-ide
```

### Basic Workflow

1. **Write Code**: Use the editor pane to write Gulf of Mexico code
2. **Run Code**: Press F5 or click "Run" to execute
3. **View Output**: Check the output window for results
4. **Debug**: Use F10 to start debugging (framework ready)
5. **Save Work**: Use Ctrl+S to save your files

### Example Code

```dreamberd
const message = 'Hello, GulfOfMexico!'!
print message!
```

## Architecture

The IDE is built with Python's tkinter library for cross-platform GUI support:

- **Main Window**: Split-pane layout with editor and output areas
- **Tabbed Interface**: Output and Debug tabs
- **Threading**: Non-blocking code execution
- **Settings Persistence**: JSON-based configuration storage

## Future Enhancements

- Full debugger implementation
- Project management
- Syntax error highlighting
- Code completion
- Plugin system
