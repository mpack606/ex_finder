# ex_finder: Agent Guidelines & Architecture

This document codifies the architectural patterns and coding standards used in `ex_finder`. AI agents and human contributors should follow these guidelines to ensure consistency and maintain the project's KISS (Keep It Simple, Stupid) philosophy.

## 1. Project Context
`ex_finder` is a lightweight macOS Finder replacement built with Rust and the `iced` GUI library. It focuses on speed, simplicity, and a modular architecture.

## 2. Architecture

### Vertical Slice Architecture
Features are isolated into modular "slices" located in the `src/` directory (e.g., `sidebar.rs`, `grid_view.rs`, `address_bar.rs`). Each slice is responsible for its own view and message definitions, keeping the codebase flat and modular.

### Elm Architecture (MVU)
The application follows the Model-View-Update pattern required by `iced`:
- **Model (`App` struct in `app.rs`)**: Centralized state management.
- **Update (`App::update`)**: Logic for transitioning state based on `Message`s.
- **View (`App::view`)**: UI representation of the current state.

## 3. Component Development Pattern

### Message Definition
Each UI component defines its own `Message` enum in its respective file:
```rust
pub enum SidebarMessage {
    SelectPath(PathBuf),
    AddCurrentPath(PathBuf),
    RemovePath(PathBuf),
}
```

### View Implementation
Components provide a `view` function that returns an `Element` specialized for the component's message type:
```rust
pub fn view(...) -> Element<'static, SidebarMessage> {
    // ...
}
```
*Note: Components that do not emit messages (like `bottom_bar.rs`) should be generic over their message type: `pub fn view<Message: 'static>(...) -> Element<'static, Message>`.*

### Integration in `App`
1.  **Wrap Messages**: Add the component's message to the global `Message` enum in `src/app.rs`.
    ```rust
    pub enum Message {
        Sidebar(sidebar::SidebarMessage),
        // ...
    }
    ```
2.  **Delegate Update**: Handle the wrapped message in `App::update` by matching on it. Actual logic should be contained in specific feature file.
3.  **Map View**: Call the component's view in `App::view` and use `.map(Message::[Variant])` to convert messages back to the global type.
    ```rust
    sidebar::view(...).map(Message::Sidebar)
    ```

## 4. Coding Standards

### Naming Conventions
- **Messages**: Use `[Component]Message` (e.g., `GridMessage`).
- **State**: Keep state fields in the `App` struct unless a slice becomes complex enough to warrant its own `[Component]State` struct.

### Styling
- **Theme Palette**: ALWAYS use `theme.extended_palette()` for colors. Avoid hardcoded hex/RGB values to ensure light/dark mode compatibility.
- **Consistency**: Use `12.0.into()` for `Border` radius on interactive elements (buttons, inputs).
- **Alignment**: Use `.align_y(iced::Alignment::Center)` for elements within a `row!`.

### Persistence
- Application settings (window size, bookmarks, etc.) are handled in `src/settings.rs` using TOML and stored at `~/.ex_finder.toml`.
- Always save settings after modification using `settings::save_settings(&self.settings)`.

### macOS Specifics
- `src/app_icons.rs` handles fetching system icons for file extensions using macOS CLI tools (`swift`, `defaults`, `sips`).

## 5. File Structure Guide
- `src/main.rs`: Entry point, window configuration, and subscription setup.
- `src/app.rs`: The "brain" of the app; contains state and orchestrates message handling.
- `src/settings.rs`: Persistence logic.
- `src/icons.rs`: Embedded SVG constants.
- `src/sidebar.rs`, `src/grid_view.rs`, `src/address_bar.rs`, etc.: Modular UI slices.

## 6. Guidelines for Agents
- **Maintain KISS**: Prioritize simplicity. Avoid deep abstraction layers unless strictly necessary.
- **Stay Modular**: When adding a new feature (e.g., Tabs), create a new file and follow the Message/View delegation pattern.
- **Check TODO.md**: Consult the roadmap before proposing major architectural changes.
- **Icons**: Add new SVG icons to `src/icons.rs` as `pub const [NAME]_SVG: &[u8] = br##"..."##;`.
- **Async Operations**: Current IO is mostly blocking. When implementing new features, prefer using `iced::Task` for heavy operations (like reading large directories).
