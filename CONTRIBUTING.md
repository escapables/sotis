---
summary: 'Contribution guidelines for sotis including quality gates and PR workflow.'
read_when:
  - Preparing your first contribution.
  - Opening a pull request.
  - Understanding project conventions.
---

# Contributing to sotis

Thank you for your interest in contributing! This guide covers how to contribute effectively.

## Quick Start

1. Fork and clone the repository:
```bash
git clone https://github.com/escapables/sotis.git
cd sotis
```

2. Build the project:
```bash
make build
```

3. Run tests:
```bash
make test
```

4. Make your changes and test locally.

## Pull Request Workflow

### Before Submitting

- [ ] Code compiles without warnings: `cargo build --workspace`
- [ ] Tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace -- -D warnings`
- [ ] Code formatted: `cargo fmt --all -- --check`
- [ ] Documentation updated (if behavior changes)
- [ ] Commit messages are clear and descriptive

### Opening a PR

1. Create a feature branch:
```bash
git checkout -b feature/my-feature
```

2. Make commits with clear messages:
```bash
git commit -m "feat: add PDF text extraction"
```

3. Push and open PR:
```bash
git push origin feature/my-feature
```

4. In your PR description:
   - Explain what changed and why
   - Reference any related issues
   - Note any breaking changes

### PR Review Process

- All PRs require at least one review
- Address review feedback promptly
- CI must pass before merge
- Maintainers may squash commits on merge

## Local Quality Gates

Run these before pushing:

```bash
# Full check
make check

# Individual gates
make build
make test
make clippy
make fmt-check
bin/validate-docs
```

## Code Style

- Follow Rust idioms and `rustfmt` conventions
- Run `cargo clippy` and fix all warnings
- Keep functions focused and small
- Use meaningful variable names
- See `docs/STYLE.md` for full conventions

## Documentation

If your change affects user-facing behavior:

- Update `README.md` for CLI changes
- Update `docs/PRIMARY_TODO.md` if architecture changes
- Add/update examples if applicable

## Getting Help

- Open an issue for bugs or feature requests
- Join discussions in existing issues
- Check `docs/` for architecture and setup information

## License

By contributing, you agree that your contributions will be licensed under the project's license.
