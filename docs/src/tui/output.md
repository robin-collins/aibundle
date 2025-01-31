# Output Options 🎨

## Format Selection 📝

### Available Formats
Toggle with 'f' key:

1. XML Format (Default)
```xml
<file name="src/main.rs">
fn main() {
    println!("Hello!");
}
</file>
```

2. Markdown Format
````text
```rust:src/main.rs
fn main() {
    println!("Hello!");
}
```
````

3. JSON Format
```json
[{
  "type": "file",
  "path": "src/main.rs",
  "content": "fn main() {\n    println!(\"Hello!\");\n}"
}]
```

## Line Numbers 📊

### Toggle Line Numbers
- Press 'n' to toggle
- Applies to XML and Markdown
- Not available in JSON mode

Example with line numbers:
```
     1 | fn main() {
     2 |     println!("Hello!");
     3 | }
```

## Binary File Handling 📦

### Binary File Options
- Toggle with 'b' key
- Include/exclude in output
- Size indication
- Type detection

Binary file representation:
```xml
<file name="image.png">
<binary file>
</file>
```

## Directory Structure 📂

### Hierarchy Preservation
```
[X] 📁 src/
    [X] 📄 main.rs
    [X] 📁 modules/
        [X] 📄 mod.rs
```

Output reflects structure:
```xml
<folder name="src">
    <file name="main.rs">
        // content
    </file>
    <folder name="modules">
        <file name="mod.rs">
            // content
        </file>
    </folder>
</folder>
```

## Status Indicators 📊

### Format Status
- Current format shown in status bar
- Line number state
- Binary file handling
- Selection count

### Copy Statistics
```
Files: 5
Folders: 2
Lines: 150
Size: 4.2 KB
Format: XML
```

## Best Practices 💡

1. Choose format based on target platform
2. Enable line numbers for code review
3. Consider binary handling for mixed content
4. Watch status bar for current settings
5. Use appropriate format for file types
