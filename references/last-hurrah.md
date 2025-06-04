Okay, I've analyzed the `cargo clippy` output. There are quite a few `dead_code` warnings, which is common during refactoring. Many functions and fields have been moved to new modules but might not be fully integrated yet, or their callers haven't been updated. Some warnings might also be false positives due to Clippy's analysis limitations across modules.

Let's go through each warning group based on the rules provided:

**1. `dead_code` in `src/cli/options.rs`**

*   **Issues:**
    *   Methods `to_app_config`, `to_mode_config`, `effective_source_dir`, `to_cli_mode_options` in `impl CliOptions` are unused.
    *   Functions `load_merged_config`, `create_ignore_config`, `string_to_output_format`, `get_output_format` are unused.
    *   (Note: `load_config` and `config_file_path` are also listed but *are* used elsewhere (`src/main.rs` and `src/config/mod.rs`), so these specific warnings are likely false positives or based on incomplete context.)
*   **Analysis:**
    *   These are helper functions related to parsing CLI options and translating them into configuration structs, or loading configuration files.
    *   In the monolithic code (`monolithic/main.rs`), CLI parsing and config loading were handled directly within `main`.
    *   The `to_...` methods seem like convenience converters that aren't currently used.
    *   `load_merged_config` appears genuinely unused.
    *   `create_ignore_config`, `string_to_output_format`, and `get_output_format` seem like helpers intended for `run_cli_mode` (defined in `src/cli/mod.rs`), but `run_cli_mode` currently duplicates some of this logic internally rather than calling these helpers.
*   **Proposed Resolution:**
    *   **Remove** the unused functions: `load_merged_config`, `create_ignore_config`, `string_to_output_format`, `get_output_format` from `src/cli/options.rs` as their logic is either duplicated or not needed currently.
    *   **Keep** the unused methods `to_app_config`, `to_mode_config`, `effective_source_dir`, `to_cli_mode_options` for now, as they might be useful helpers during further refactoring. Add `#[allow(dead_code)]` to these methods to suppress the warnings temporarily.
    *   **Keep** `load_config` and `config_file_path` as they are used elsewhere.

**2. `dead_code` in `src/fs/mod.rs`**

*   **Issues:** Functions `list_files`, `is_excluded`, `is_binary_file` are unused.
*   **Analysis:**
    *   `list_files` and `is_excluded` existed in `monolithic/fs/mod.rs` and were used for a simpler file listing approach. The current TUI uses `add_items_iterative` and `is_path_ignored_for_iterative` for its file listing.
    *   `is_binary_file` existed in `monolithic/main.rs:1576` and is also defined (and used) in `src/output/format.rs:44`. This is duplication.
*   **Proposed Resolution:**
    *   **Remove** the `is_binary_file` function from `src/fs/mod.rs` to eliminate duplication. The version in `src/output/format.rs` is sufficient.
    *   **Keep** `list_files` and `is_excluded` for now, marked with `#[allow(dead_code)]`. They represent functionality from the monolithic version that might be reintroduced later or used in a different context.

**3. `dead_code` in `src/models/app_config.rs` (IgnoreConfig fields)**

*   **Issue:** Fields `use_default_ignores`, `use_gitignore`, `include_binary_files`, `extra_ignore_patterns` in `IgnoreConfig` are reported as never read.
*   **Analysis:**
    *   The `IgnoreConfig` struct holds settings for filtering files. It is passed to and used by functions like `is_path_ignored_for_iterative` (`src/fs/mod.rs:72`), `process_directory` (`src/output/format.rs:91`), `collect_all_subdirs` (`src/fs/mod.rs:123`), and `count_selection_items_async` (`src/fs/mod.rs:150`).
    *   These functions *do* access the fields mentioned in the warning. For example, `is_path_ignored_for_iterative` reads `use_default_ignores`, `use_gitignore`, and `extra_ignore_patterns`. `process_directory` reads `include_binary_files`.
*   **Proposed Resolution:**
    *   This appears to be a **false positive** from Clippy, potentially due to limitations in its cross-module analysis. The fields are indeed read and used.
    *   **Recommend ignoring** these specific warnings. Alternatively, if the warnings are disruptive, add `#[allow(dead_code)]` to the fields in `IgnoreConfig`, but be aware that the code *is* actually used.

**4. `dead_code` in `src/output/mod.rs`**

*   **Issue:** Function `format_selected_items` is unused.
*   **Analysis:**
    *   This function was intended as the main dispatcher to call specific formatting functions based on `OutputFormat`.
    *   In the monolithic code, `App::format_selected_items` (`monolithic/main.rs:1685`) served this purpose and was called by `App::copy_selected_to_clipboard`.
    *   The modular `ClipboardHandler::copy_selected_to_clipboard` (`src/tui/handlers/clipboard.rs:15`) now calls the specific format functions (`format_json_output`, `format_markdown_output`, etc.) directly, effectively bypassing this dispatcher. However, looking closer at `src/tui/handlers/clipboard.rs:57`, it seems `ClipboardHandler::format_selected_items` calls `crate::output::format_selected_items` which *is* this dispatcher.
*   **Proposed Resolution:**
    *   This seems to be another **false positive**. The dispatcher function `format_selected_items` in `src/output/mod.rs` is called by `ClipboardHandler::format_selected_items`.
    *   **Recommend ignoring** this warning or adding `#[allow(dead_code)]` temporarily.

**5. `dead_code` in `src/output/format.rs`**

*   **Issue:** Function `get_language_name` is unused.
*   **Analysis:**
    *   This helper function converts file extensions to language names. It existed in `monolithic/main.rs:2077` and was used by the LLM formatting logic (`format_llm_output`).
    *   The current modular `format_llm_output` in `src/output/llm.rs` is potentially incomplete or doesn't yet use this helper, although it likely should to replicate the monolithic functionality.
*   **Proposed Resolution:**
    *   **Keep** the `get_language_name` function, marked with `#[allow(dead_code)]`. It represents planned functionality derived from the monolithic codebase and will likely be needed when `format_llm_output` is fully implemented.

**6. `dead_code` in `src/output/llm.rs`**

*   **Issue:** Function `format_llm_output` is unused.
*   **Analysis:**
    *   This function is responsible for generating the LLM-specific output format.
    *   It is called by the `format_selected_items` dispatcher in `src/output/mod.rs:37` when the format is `OutputFormat::Llm`.
    *   As established in point 4, the dispatcher *is* called.
*   **Proposed Resolution:**
    *   This appears to be another **false positive**. The function is called via the dispatcher.
    *   **Recommend ignoring** this warning or adding `#[allow(dead_code)]` temporarily.

**7. `dead_code` in `src/tui/mod.rs`**

*   **Issue:** Type alias `AppResult` is unused.
*   **Analysis:** A simple type alias `std::result::Result<T, Box<dyn std::error::Error>>` that isn't currently referenced.
*   **Proposed Resolution:**
    *   **Remove** the unused type alias `AppResult`.

**8. `dead_code` in `src/tui/components/modal.rs`**

*   **Issues:** Fields `timestamp`, `width`, `height` in `Modal` are never read. Method `get_visible_content` is never used.
*   **Analysis:**
    *   `timestamp` is set but not used; might be for future features like auto-dismissal.
    *   `width` and `height` were used in the monolithic version (`monolithic/main.rs:480`, `monolithic/main.rs:1484`) along with `centered_rect` to control modal size and position. `get_visible_content` (`monolithic/main.rs:568`) was used for pagination within the modal.
    *   The current `Modal::render` (`src/tui/components/modal.rs:134`) doesn't use the passed `area`, the size fields, or pagination logic.
*   **Proposed Resolution:**
    *   **Enhance `Modal::render`**:
        *   Use `crate::utils::centered_rect` along with the `self.width` and `self.height` fields to calculate the rendering area within the `area` passed to the function.
        *   Call `self.get_visible_content(calculated_height)` to get the text for the current page.
        *   Render the content obtained from `get_visible_content`. This will naturally use the `page` field as well.
    *   **Keep** the `timestamp` field for now, marked with `#[allow(dead_code)]`, for potential future use.

**9. `dead_code` in `src/tui/handlers/clipboard.rs`**

*   **Issues:** Associated functions `new` and `count_selected_items` are never used.
*   **Analysis:**
    *   `ClipboardHandler` is used as a namespace for static methods (`copy_selected_to_clipboard`, `format_selected_items`), so an instance created by `new` is unnecessary.
    *   `count_selected_items` was a helper in the monolithic `App` (`monolithic/main.rs:1098`) called by `copy_selected_to_clipboard`. The modular version calculates copy statistics (`CopyStats`) within the formatting functions (`format_json_output`, etc.) and returns them.
*   **Proposed Resolution:**
    *   **Remove** the unused `new` and `count_selected_items` methods from `ClipboardHandler`.

**10. `dead_code` in `src/tui/handlers/file_ops.rs`**

*   **Issue:** Associated function `format_selected_items` is unused.
*   **Analysis:**
    *   This function seems redundant. Formatting logic is centralized in `src/output/mod.rs` and invoked via `ClipboardHandler`.
*   **Proposed Resolution:**
    *   **Remove** the unused `format_selected_items` method from `FileOpsHandler`.

**11. `dead_code` in `src/tui/handlers/search.rs`**

*   **Issue:** Associated function `clear_search` is never used.
*   **Analysis:**
    *   `clear_search` (`monolithic/main.rs:814`) likely corresponds to clearing the search query and exiting search mode, typically triggered by `Esc`.
    *   The current `KeyboardHandler::handle_key` (`src/tui/handlers/keyboard.rs:41`) calls `SearchHandler::toggle_search` when `Esc` is pressed during search, which only exits the mode but doesn't clear the query state properly.
*   **Proposed Resolution:**
    *   **Modify `KeyboardHandler::handle_key`**: Change the `KeyCode::Esc` match arm within the `if app_state.is_searching` block (`src/tui/handlers/keyboard.rs:42`) to call `SearchHandler::clear_search(app_state, search_state)` instead of `SearchHandler::toggle_search`.

**12 & 13. `dead_code` in `src/tui/state/app_state.rs` (MessageType variants & AppMessage::new)**

*   **Issues:** `MessageType` enum variants are never constructed. `AppMessage::new` function is never used.
*   **Analysis:**
    *   The `AppMessage` struct, `MessageType` enum, and associated `AppState` fields (`message`) and methods (`set_message`, `clear_message`) define a system for showing status messages to the user (Info, Success, Warning, Error).
    *   However, no part of the code currently calls `app_state.set_message(...)` to actually create and display these messages. This feature is defined but not implemented.
*   **Proposed Resolution:**
    *   **Keep** the `MessageType` enum, `AppMessage` struct, and `AppMessage::new` function, marked with `#[allow(dead_code)]`. This infrastructure is needed for planned user feedback functionality. Future work should involve calling `app_state.set_message` at appropriate points (e.g., after copy, save, on errors).

**14. `dead_code` in `src/tui/state/app_state.rs` (file_tree field)**

*   **Issue:** Field `file_tree: Option<Node>` is never read.
*   **Analysis:**
    *   The `Node` struct (`src/models/app_config.rs:73`) represents a node in a file tree, similar to `monolithic/main.rs:2618`.
    *   This tree structure was used in the monolithic version primarily for the `format_llm_output` function (`monolithic/main.rs:2366`) to provide context and potentially for dependency analysis.
    *   The field is unused because the corresponding LLM formatting logic in `src/output/llm.rs` is likely incomplete and doesn't build or use this tree yet.
*   **Proposed Resolution:**
    *   **Keep** the `file_tree` field in `AppState`, marked with `#[allow(dead_code)]`. It's necessary for planned functionality (LLM output formatting matching the monolithic version).

**15. `dead_code` in `src/tui/state/app_state.rs` (Multiple methods)**

*   **Issue:** Many methods like `selected_count`, `item_count`, `is_file_selected`, `update_search`, `load_items`, `copy_selected_to_clipboard`, `move_selection`, `toggle_selection`, `toggle_select_all`, `set_message`, etc., are reported as unused.
*   **Analysis:**
    *   `AppState` is the central state container passed mutably (`&mut AppState`) to various handlers (`KeyboardHandler`, `FileOpsHandler`, `SearchHandler`, `ClipboardHandler`) within the main application loop (`App::run`).
    *   These handlers *call* the methods on `AppState` to modify or query the state. For example, `KeyboardHandler` calls `move_selection`, `toggle_selection`, etc. `FileOpsHandler` calls `load_items`, `update_search`. `ClipboardHandler` calls `copy_selected_to_clipboard`.
*   **Proposed Resolution:**
    *   This appears to be a **false positive** due to Clippy's analysis scope. The methods are used externally by the handler modules, not necessarily within `AppState` itself.
    *   **Recommend ignoring** these warnings or adding `#[allow(dead_code)]` to the methods temporarily if they are disruptive.

**16. `dead_code` in `src/tui/state/search.rs` (Fields and Methods)**

*   **Issues:** Fields `is_searching`, `selected_items` are reported as never read. Methods like `toggle_search`, `clear_search`, `handle_search_input`, `toggle_selection`, `is_selected`, etc., are reported as unused. Function `perform_search` is unused.
*   **Analysis:**
    *   Similar to `AppState`, `SearchState` is passed to handlers. `KeyboardHandler` uses `is_searching` and calls `handle_search_input`. `FileOpsHandler::update_search` uses `search_query` and `create_matcher`. `toggle_search` is called by handlers. `clear_search` will be used after the change in point 11.
    *   However, `selected_items` in `SearchState` duplicates the primary selection state managed in `AppState`. The methods related to selection (`toggle_selection`, `toggle_select_all`, `is_selected`, `selected_count`, `clear_selections`, `get_selected_items`) are therefore redundant and operate on this duplicate state.
    *   `perform_search` is a standalone function for filtering, but the actual filtering logic is now integrated within `FileOpsHandler::update_search`.
*   **Proposed Resolution:**
    *   **Refactor `SearchState`**:
        *   **Remove** the redundant `selected_items` field.
        *   **Remove** the associated redundant selection methods: `toggle_selection`, `toggle_select_all`, `is_selected`, `selected_count`, `clear_selections`, `get_selected_items`. Selection should be managed solely through `AppState`/`SelectionState`.
        *   **Keep** `search_query` and `is_searching` fields.
        *   **Keep** methods `new`, `toggle_search`, `clear_search`, `handle_search_input`, `handle_backspace`, `create_matcher`. The warnings for these seem to be false positives as they are used by handlers. Add `#[allow(dead_code)]` temporarily if needed.
    *   **Remove** the unused standalone function `perform_search`.

**17. `dead_code` in `src/tui/state/selection.rs` (local_selected field)**

*   **Issue:** Field `local_selected: HashSet<PathBuf>` is never read.
*   **Analysis:**
    *   `SelectionState` primarily holds the `list_state` (UI cursor position). It also has this `local_selected` field.
    *   The main application state (`AppState`) already holds `selected_items: HashSet<PathBuf>`, which represents the canonical set of selected items.
    *   Methods within `SelectionState` like `toggle_selection` and `update_folder_selection` operate directly on `app_state.selected_items`, not `self.local_selected`.
*   **Proposed Resolution:**
    *   **Remove** the unused and redundant `local_selected` field from `SelectionState`.

**18. `dead_code` in `src/tui/views/message_view.rs`**

*   **Issue:** Method `set_message_duration` is never used.
*   **Analysis:** A configuration method for the view that allows changing how long messages are displayed. It's not currently called.
*   **Proposed Resolution:**
    *   **Keep** the `set_message_duration` method, marked with `#[allow(dead_code)]`. It provides a potentially useful configuration point for the future.

**19. `dead_code` in `src/tui/app.rs`**

*   **Issue:** Field `keyboard_handler: KeyboardHandler` is never read.
*   **Analysis:**
    *   An instance of `KeyboardHandler` is created in `App::new` and stored in this field.
    *   However, `App::run` calls the handler method statically: `KeyboardHandler::handle_key(...)`. The stored instance is never used.
*   **Proposed Resolution:**
    *   **Remove** the `keyboard_handler` field from the `App` struct.

**20. `dead_code` in `src/utils/mod.rs`**

*   **Issue:** Function `centered_rect` is unused.
*   **Analysis:**
    *   A utility function for calculating centered layout areas. It was used in monolithic (`monolithic/main.rs:2625`) for modals.
    *   The modular `MessageView` (`src/tui/views/message_view.rs:86`) currently defines its *own* local `centered_rect` function. The `Modal` view doesn't use centering yet.
*   **Proposed Resolution:**
    *   **Remove** the local `centered_rect` function definition from `src/tui/views/message_view.rs`.
    *   **Modify `MessageView::render`** (`src/tui/views/message_view.rs:30`) to call `crate::utils::centered_rect` instead of its local version.
    *   As part of resolving point 8, **modify `Modal::render`** to also use `crate::utils::centered_rect`.

**21. `clippy::unnecessary_mut_passed` in `src/tui/app.rs`**

*   **Issue:** `ClipboardHandler::format_selected_items(&mut self.state)` passes a mutable reference, but the function only requires an immutable `&AppState`.
*   **Analysis:** The function signature for `ClipboardHandler::format_selected_items` in `src/tui/handlers/clipboard.rs:57` is `fn format_selected_items(app_state: &AppState) -> io::Result<(String, CopyStats)>`. It does not need mutable access.
*   **Proposed Resolution:**
    *   **Modify the call** in `src/tui/app.rs:205` to pass an immutable reference: `ClipboardHandler::format_selected_items(&self.state).map(|(s, _stats)| s)`.

**22. `unused_must_use` in `src/tui/handlers/keyboard.rs`**

*   **Issue:** The `io::Result<()>` returned by `SelectionState::toggle_select_all` is ignored.
*   **Analysis:** `toggle_select_all` (`src/tui/state/selection.rs:127`) returns `io::Result<()>`. While the current implementation might not actually produce errors often, ignoring a `Result` can hide potential issues.
*   **Proposed Resolution:**
    *   **Acknowledge the result** in `src/tui/handlers/keyboard.rs:87` by assigning it to `_`: `let _ = SelectionState::toggle_select_all(selection_state, app_state);`.

These resolutions address the clippy warnings by removing genuinely dead code, fixing minor issues, keeping potentially useful but currently unused code (marked with `#[allow(dead_code)]`), enhancing incomplete features like the Modal rendering, and resolving likely false positives by ignoring them or using `allow`. Remember that `#[allow(dead_code)]` should ideally be temporary placeholders during active development.

Please let me know which of these resolutions you'd like me to implement first. According to the `pre-linting-errors.mdc` rule, we should implement one recommendation at a time.
