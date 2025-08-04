# Development Scripts

This project includes a utility script to help developers quickly understand the codebase structure.

## Code Analysis Script

### `./find-definitions.sh`
Shows all Rust definitions in file order:
- Lists structs, enums, traits, functions, impl blocks, and modules
- Displays them in the exact order they appear in each file
- Shows line numbers and file paths for easy navigation
- Uses icons to distinguish different types of definitions:
  - 🏗️ struct
  - 🔢 enum
  - 🎯 trait
  - 🧩 impl
  - ⚙️ fn
  - 📦 mod
- Shows which functions belong to which impl blocks
- Provides summary counts at the end

This is perfect for:
- Getting oriented at the start of a coding session
- Understanding the structure and organization of the codebase
- Seeing the relationship between structs, their impl blocks, and methods
- Finding specific definitions quickly

## Usage

```bash
./find-definitions.sh
```

The script uses `ripgrep` (rg) for fast searching and requires it to be installed.
