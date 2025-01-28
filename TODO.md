# TODO List

## Path Handling Refactoring

### Issue: Inconsistent Path Type Usage

Currently, the codebase uses `&PathBuf` in several places where `&Path` would be more appropriate. While Clippy correctly suggests using `&Path` instead of `&PathBuf`, the refactoring is non-trivial due to the following reasons:

1. The codebase extensively uses `HashSet<PathBuf>` for storing paths
2. There are complex interactions between path comparisons and the HashSet operations
3. Path cloning and ownership patterns are deeply integrated into the current implementation

### Required Changes

To properly implement this refactoring:

1. Review and potentially redesign the path storage strategy
   - Consider if `HashSet<PathBuf>` is the optimal data structure
   - Evaluate alternatives that work better with borrowed `Path` references

2. Audit all path-related operations
   - Identify where path ownership is truly needed vs where borrows would suffice
   - Map out the lifecycle of path objects through the application

3. Modify function signatures
   - Change `&PathBuf` parameters to `&Path` where appropriate
   - Update corresponding function implementations to handle borrowed paths correctly

4. Update path comparison logic
   - Ensure path comparisons work correctly with both owned and borrowed paths
   - Maintain consistent behavior with the current implementation

### Benefits

1. More idiomatic Rust code
2. Better performance by reducing unnecessary cloning
3. More flexible API that accepts both `Path` and `PathBuf` arguments

### Potential Risks

1. Breaking changes to the API
2. Subtle behavioral changes in path comparison logic
3. Performance implications in hot paths

### Related Files

Primary files that need modification:
- `src/main.rs`
  - `fn count_selection_items`
  - `fn get_icon`
  - `fn is_binary_file`
  - `fn add_items_iterative`
  - Other path-handling functions

### Priority: Medium

This refactoring would improve code quality but isn't critical for functionality. Should be addressed in a dedicated refactoring pass with comprehensive testing. 