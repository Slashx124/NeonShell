# Contributing to NeonShell

First off, thank you for considering contributing to NeonShell! 🎉

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Pull Request Process](#pull-request-process)
- [Style Guidelines](#style-guidelines)

## Code of Conduct

This project adheres to a Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to conduct@neonshell.dev.

## How Can I Contribute?

### 🐛 Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates.

**When creating a bug report, include:**

- A clear, descriptive title
- Steps to reproduce the behavior
- Expected vs actual behavior
- Screenshots if applicable
- Your environment (OS, NeonShell version)
- Debug bundle export (use `Ctrl+\`` → Export Bundle)

### 💡 Suggesting Features

Feature suggestions are welcome! Please:

- Check if the feature is already on the [roadmap](README.md#-roadmap)
- Search existing issues for similar suggestions
- Describe the use case and expected behavior
- Explain why this would be useful to most users

### 🎨 Creating Themes

We love community themes! To submit a theme:

1. Create your theme in `~/.neonshell/themes/your-theme/`
2. Include `theme.json`, `styles.css`, and a `preview.png`
3. Test on light and dark backgrounds
4. Open a PR adding it to `themes/community/`

### 🔌 Building Plugins

Plugin contributions are welcome! See the [Plugin API documentation](docs/plugin-api.md).

### 📖 Improving Documentation

Documentation improvements are always appreciated:

- Fix typos and grammar
- Add examples and clarifications
- Translate to other languages
- Improve API documentation

## Development Setup

### Prerequisites

- **Rust** 1.75 or later
- **Node.js** 20 or later
- **pnpm** 8 or later
- Platform-specific dependencies (see [README](README.md#installation))

### Getting Started

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/neonshell.git
cd neonshell

# Add upstream remote
git remote add upstream https://github.com/yourorg/neonshell.git

# Install dependencies
pnpm install

# Run in development mode
pnpm dev
```

### Project Structure

```
NeonShell/
├── apps/desktop/          # Main Tauri application
│   ├── src-tauri/         # Rust backend
│   └── src/               # React frontend
├── packages/              # Shared packages
├── plugins/               # Example plugins
├── scripts/               # Build and utility scripts
├── themes/                # Bundled themes
└── docs/                  # Documentation
```

### Running Tests

```bash
# All tests
pnpm test

# Rust tests only
cd apps/desktop/src-tauri && cargo test

# Frontend tests only
cd apps/desktop && pnpm test

# Linting
pnpm lint
```

## Pull Request Process

### Before Submitting

1. **Fork** the repository and create your branch from `main`
2. **Test** your changes thoroughly
3. **Lint** your code: `pnpm lint`
4. **Update documentation** if needed
5. **Add tests** for new functionality

### Branch Naming

- `feature/description` – New features
- `fix/description` – Bug fixes
- `docs/description` – Documentation
- `refactor/description` – Code refactoring
- `theme/name` – New themes

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat` – New feature
- `fix` – Bug fix
- `docs` – Documentation
- `style` – Formatting (no code change)
- `refactor` – Code restructuring
- `test` – Adding tests
- `chore` – Maintenance

**Examples:**
```
feat(ssh): add jump host support
fix(theme): correct cursor color in Dracula theme
docs(readme): update installation instructions
```

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How did you test these changes?

## Screenshots
If applicable

## Checklist
- [ ] My code follows the project's style guidelines
- [ ] I have performed a self-review
- [ ] I have added tests for new functionality
- [ ] All tests pass locally
- [ ] I have updated documentation as needed
```

## Style Guidelines

### Rust

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting
- Use `clippy` for linting
- Document public APIs with doc comments
- **NEVER** log secrets (passwords, keys, tokens)

```rust
// Good
/// Creates a new SSH session.
///
/// # Arguments
/// * `config` - Session configuration
///
/// # Errors
/// Returns an error if connection fails.
pub fn create_session(config: SessionConfig) -> Result<Session> {
    // ...
}
```

### TypeScript/React

- Use TypeScript strict mode
- Prefer functional components with hooks
- Use Zustand for state management
- Follow React best practices

```typescript
// Good
interface Props {
  sessionId: string;
  onClose: () => void;
}

export function TerminalPane({ sessionId, onClose }: Props) {
  const { sendData } = useSessionStore();
  // ...
}
```

### CSS/Tailwind

- Use Tailwind utility classes
- Define custom properties in `index.css` for theming
- Keep component styles co-located

### Security Guidelines

⚠️ **Critical: Never expose secrets!**

- Never log passwords, private keys, or tokens
- Use the `sanitize()` function for any user-facing output
- Store secrets only in OS keychain
- Validate all user input
- Use parameterized queries/commands

## Questions?

Feel free to:
- Open a [Discussion](https://github.com/yourorg/neonshell/discussions)
- Join our [Discord](https://discord.gg/neonshell)
- Email maintainers@neonshell.dev

Thank you for contributing! 💜
