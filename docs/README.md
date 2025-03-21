# AIBundle Documentation 📚

[![Version](https://img.shields.io/badge/version-0.6.13-blue.svg)](https://crates.io/crates/aibundle)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](../LICENSE)

This directory contains the documentation for AIBundle, built with [mdBook](https://rust-lang.github.io/mdBook/). The documentation is organized to provide comprehensive information about AIBundle's features, usage, and configuration.

## Documentation Structure 📂

- `src/` - Source markdown files for the documentation
- `theme/` - Custom theming and page layout components
- `book.toml` - mdBook configuration file
- `custom.css` - Additional styling

## Documentation Tooling 🛠️

AIBundle documentation uses the following tools:

- **[mdBook](https://rust-lang.github.io/mdBook/)** - Main documentation generator
- **Custom Theme** - Enhanced page navigation with table of contents
- **GitHub Pages** - Hosting platform for the published documentation

## Building the Documentation 🏗️

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) installed
- mdBook installed: `cargo install mdbook`

### Build Commands

```bash
# Navigate to the docs directory
cd docs

# Build the documentation
mdbook build

# The output will be in the 'book' directory
# To view locally:
mdbook serve --open
```

## Development Workflow 🔄

### Local Development

```bash
# Watch for changes and rebuild automatically
mdbook serve

# Access the documentation at http://localhost:3000
```

### Adding New Content

1. Add your markdown file to the appropriate directory in `src/`
2. Update `src/SUMMARY.md` to include your new page
3. Run `mdbook build` to verify the changes

## Documentation Best Practices 📝

1. **Structure**: Follow the existing organization pattern
2. **Consistency**: Maintain consistent formatting and style
3. **Examples**: Include practical examples for features
4. **Screenshots**: Add screenshots for UI features when relevant
5. **Cross-references**: Link related sections for better navigation
6. **Versioning**: Note version-specific features clearly

## Publishing the Documentation 📤

The documentation is automatically published to GitHub Pages when changes are pushed to the main branch.

Manual publishing:

```bash
# Build the documentation
mdbook build

# Deploy to GitHub Pages (if you have appropriate permissions)
# This assumes you have a gh-pages branch set up
git worktree add gh-pages gh-pages
cp -r book/* gh-pages/
cd gh-pages
git add .
git commit -m "Update documentation"
git push origin gh-pages
cd ..
git worktree remove gh-pages
```

## Checking for Issues ✅

```bash
# Validate links and references
mdbook test

# Check for markdown formatting issues
# (requires markdownlint-cli installed)
markdownlint src/**/*.md
```

## Contributing to Documentation 🤝

Contributions to improve the documentation are welcome! Please follow these steps:

1. Fork the repository
2. Create your feature branch (`git checkout -b docs/improve-installation`)
3. Make your changes
4. Test your changes locally with `mdbook serve`
5. Commit your changes (`git commit -m 'Improve installation documentation'`)
6. Push to the branch (`git push origin docs/improve-installation`)
7. Open a Pull Request

## License 📄

This documentation is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

---

Made with ❤️ by the AIBundle Team 