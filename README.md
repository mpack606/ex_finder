# ex_finder

A lightweight, premium macOS Finder replacement written in **Rust** using the **Iced** GUI library. 

Developed with a clean **Vertical Slice Architecture** keeping codebase features completely modular, independent, and simple (KISS).

## Core Features

- **Responsive Grid View**: Automatically wraps and adjusts file/folder columns depending on window width.
- **Scrollable & Double-Click Navigation**:
  - Single-click to select files or folders.
  - Double-click a folder to navigate inside it.
  - Double-click a file to launch it using the system default application.
- **Address Bar**: Browser-like URL bar centered at the top for manual path input/pasting. Displays a red highlight and error icon on invalid paths.
- **Quick Access Sidebar**: Prepopulated with standard macOS folders (`Home`, `Desktop`, `Documents`, `Downloads`, `Applications`). Supports pinning the current directory or unpinning bookmarks.
- **TOML Configuration**: App preferences (window dimensions, quick access paths, last opened folder) are persisted locally at `~/.ex_finder.toml`.

## Installation & Running

Ensure you have Rust installed. Clone the repository and execute:

```bash
cargo run
```
