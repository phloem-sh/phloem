# Contributing to Phloem

Thank you for your interest in contributing to Phloem! We welcome contributions from the community and are excited to see what you'll build.

## 🚀 Getting Started

### Development Environment

1. **Fork and clone the repository:**
   ```bash
   git clone git@github.com:your-username/phloem.git
   cd phloem
   ```

2. **Set up Rust development:**
   ```bash
   # Install Rust if you haven't already
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Build the project
   cargo build
   ```

3. **Install Ollama (for AI functionality):**
   ```bash
   # Install Ollama
   curl -fsSL https://ollama.ai/install.sh | sh
   
   # Pull a model
   ollama pull llama2
   ```

4. **Run tests:**
   ```bash
   # Rust tests
   cargo test
   
   # Clippy linting
   cargo clippy
   
   # Format check
   cargo fmt --check
   ```

## 🎯 Ways to Contribute

### 🐛 Bug Reports

Found a bug? Please create an issue with:
- Clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- System information (`phloem doctor` output)
- Relevant logs from `~/.phloem/logs/`

### 💡 Feature Requests

Have an idea? Open an issue with:
- Clear description of the feature
- Use cases and motivation
- Proposed implementation approach
- Examples of how it would work

### 🔧 Code Contributions

#### Areas We Need Help With

- **Caching**: Improving cache hit rates and success prediction
- **Shell Integrations**: Better bash/zsh/fish completions and history parsing  
- **Executable Detection**: Enhanced system tool discovery and validation
- **Cross-platform**: Ensuring Windows and Linux compatibility
- **Performance**: Optimizing database queries and validation speed
- **Testing**: Improving test coverage and integration tests
- **Documentation**: Better examples and user guides

#### Development Workflow

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes:**
   - Follow existing code style
   - Add tests for new functionality
   - Update documentation as needed

3. **Test your changes:**
   ```bash
   # Run all tests
   cargo test
   cd python && pytest
   
   # Test the CLI manually
   cargo run -- "test command"
   ```

4. **Commit your changes:**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

5. **Push and create a PR:**
   ```bash
   git push origin feature/your-feature-name
   ```

## 📝 Code Style

### Rust Code Style

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow Rust naming conventions
- Add documentation for public APIs
- Prefer explicit error handling with `anyhow::Result`

Example:
```rust
/// Generate command suggestions for the given prompt
pub async fn generate_suggestions(
    &mut self,
    prompt: &str,
    options: PromptOptions,
) -> Result<Vec<Suggestion>> {
    // Implementation
}
```

### Python Code Style

- Use `black` for formatting
- Use `isort` for import sorting
- Use `mypy` for type checking
- Follow PEP 8 conventions
- Add type hints for all functions

Example:
```python
async def generate_text(
    self, 
    prompt: str, 
    max_tokens: int = 100
) -> str:
    """Generate text using the loaded model."""
    # Implementation
```

## 🧪 Testing

### Rust Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_function_name

# Run tests with output
cargo test -- --nocapture
```

### Python Tests

```bash
cd python
# Run all tests
pytest

# Run with coverage
pytest --cov=phloem_ml

# Run specific test
pytest tests/test_inference.py
```

### Integration Tests

Test the full pipeline:
```bash
# Build and test
cargo build --release
./target/release/phloem init
./target/release/phloem "test prompt"
```

## 📚 Documentation

### Code Documentation

- Document all public functions and structs
- Include examples where helpful
- Explain complex algorithms or business logic

### User Documentation

- Update README.md for user-facing changes
- Add examples for new features
- Update the docs/ folder for architectural changes

## 🏗️ Architecture Guidelines

### Rust Core Principles

- **Performance First**: Optimize for speed and memory usage
- **Error Handling**: Use `anyhow::Result` consistently
- **Modularity**: Keep modules focused and loosely coupled
- **Safety**: Leverage Rust's safety guarantees

### Python ML Layer Principles

- **Async by Default**: Use asyncio for all IO operations
- **Type Safety**: Use type hints and validate inputs
- **Error Handling**: Graceful degradation when models fail
- **Resource Management**: Properly manage model memory

### Database Guidelines

- **Schema Migrations**: Version all schema changes
- **Performance**: Index frequently queried columns
- **Data Integrity**: Use transactions for multi-step operations

## 📋 PR Guidelines

### Before Submitting

- [ ] Tests pass locally
- [ ] Code is formatted (`cargo fmt`, `black`)
- [ ] No lint errors (`cargo clippy`, `ruff`)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (if applicable)

### PR Description Template

```markdown
## What does this PR do?

Brief description of the changes.

## Type of change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing

- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist

- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or marked as such)
```

## 🔄 Release Process

### Versioning

We use [Semantic Versioning](https://semver.org/):
- `MAJOR.MINOR.PATCH`
- Major: Breaking changes
- Minor: New features (backwards compatible)
- Patch: Bug fixes

### Release Checklist

1. Update version in `Cargo.toml` and `pyproject.toml`
2. Update `CHANGELOG.md`
3. Create release PR
4. Tag release after merge
5. GitHub Actions builds and publishes binaries

## 🤝 Community Guidelines

### Code of Conduct

- Be respectful and inclusive
- Help others learn and grow
- Give constructive feedback
- Assume good intentions

### Communication

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Discord**: Real-time chat and community support

## ❓ Getting Help

### Documentation

- [Architecture Overview](docs/architecture.md)
- [Implementation Details](docs/rust-implementation.md)
- [Python ML Layer](docs/python-ml-layer.md)

### Asking Questions

1. Check existing issues and discussions
2. Search the documentation
3. Ask in GitHub Discussions
4. Join our Discord community

### Debugging Tips

```bash
# Enable debug logging
RUST_LOG=debug phloem "your prompt"

# Check system health
phloem doctor

# View logs
tail -f ~/.phloem/logs/phloem.log
tail -f ~/.phloem/logs/ml.log
```

## 🎉 Recognition

Contributors will be:
- Added to the README contributors section
- Mentioned in release notes
- Invited to join the core contributor team (for significant contributions)

Thank you for contributing to Phloem! 🚀