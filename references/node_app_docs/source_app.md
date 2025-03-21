
### Features

1. **File System Navigation**
- Interactive directory browsing with expandable folders
- Support for nested directory structures
- Respects `.gitignore` rules and excludes common directories (node_modules, .git, dist, build, coverage)
- File/folder sorting (directories first, then alphabetically)

2. **Search Functionality**
- Real-time search filtering of files and folders
- Case-insensitive search
- Automatic expansion of parent folders containing matches
- Maintains directory structure in search results

3. **Selection System**
- Multi-select capability for both files and folders
- Bulk selection/deselection of entire directories
- Visual indication of selected items
- Maintains selection state during navigation

4. **Navigation Controls**
- Up/Down arrows for item navigation
- Left/Right arrows for selection
- Tab key for expanding/collapsing folders
- Enter key to finalize selection and copy

5. **Output Generation**
- XML-based output format for selected files
- Automatic file content inclusion
- File path list generation
- Clipboard integration for easy sharing

### Algorithmic Process

1. **Initialization**
```typescript
1. Load and parse .gitignore rules if present
2. Initialize file system traversal from current working directory
3. Build initial directory tree structure
4. Setup UI components and state management
```

2. **File System Traversal**
```typescript
1. Recursively scan directories
2. Filter out excluded folders and gitignored paths
3. Generate unique IDs for each item
4. Create hierarchical structure with parent-child relationships
5. Sort items (directories first, then alphabetically)
```

3. **Search Algorithm**
```typescript
1. Convert search query to lowercase
2. Recursively traverse item tree
3. For files: Match name against search query
4. For directories:
   - Search children recursively
   - If children match, include parent with expanded state
   - Maintain directory structure for matches
5. Update UI with filtered results
```

4. **Selection Management**
```typescript
1. Track selected items in state
2. For file selection:
   - Toggle individual file state
3. For directory selection:
   - Get all items in directory recursively
   - Check if all items are selected
   - Toggle entire directory based on current state
4. Maintain selection state during navigation/search
```

5. **Output Generation**
```typescript
1. Collect all selected items
2. Generate XML structure:
   - Create folder nodes for directories
   - Create file nodes with content
3. Clean up file tree (remove duplicates)
4. Generate file path list
5. Copy to clipboard
6. Display success message with file count
```

6. **Navigation Logic**
```typescript
1. Maintain current item pointer
2. For next item:
   - If current item is expanded directory, move to first child
   - Otherwise, find next item in flattened list
3. For previous item:
   - Find previous item in flattened list
4. Handle edge cases (wrap-around at list boundaries)
```

The application is built using React with Ink for terminal UI rendering, and follows a component-based architecture with custom hooks for file handling and search functionality. It's currently at version 0.2.7 with a generateOutput version of 0.0.4.
