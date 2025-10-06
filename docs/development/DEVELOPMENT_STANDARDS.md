# MeshBBS Development Standards

This document outlines the development standards and practices for the MeshBBS project.

## Code Quality Requirements

### Zero Tolerance for Compiler Warnings

**CRITICAL POLICY**: We maintain a zero-tolerance policy for compiler warnings in all code.

- ✅ **All compiler warnings must be fixed** before committing code
- ✅ **All test warnings must be resolved** before merging
- ✅ **Build process must be warning-free** in both debug and release modes
- ✅ Use `cargo check` and `cargo test` to verify clean builds

#### Rationale
- Warnings often indicate potential bugs or code quality issues
- Warning-free code is more maintainable and professional
- Prevents warning buildup that can mask real issues
- Ensures consistent code quality across all contributors

#### Enforcement
- All pull requests must have clean builds with zero warnings
- CI/CD pipeline should fail on any warnings
- Use `#[allow(warning_type)]` sparingly and only with justification
- Document any necessary warning suppressions in code comments

### Testing Standards

- **100% test coverage requirement** for new features
- Unit tests must be included with all code contributions
- Integration tests required for complex workflows
- All tests must pass before merging

### Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Address all Clippy suggestions (`cargo clippy`)
- Write self-documenting code with clear variable names
- Add documentation for all public APIs

### TinyMUSH-Specific Standards

- All features must respect the 200-byte message limit
- Follow the phased implementation plan
- Update TODO.md as phases are completed
- Maintain backward compatibility where possible

## Contribution Workflow

1. **Before coding**: Ensure local environment is warning-free
2. **During development**: Fix warnings as they appear
3. **Before committing**: Run `cargo check` and `cargo test` 
4. **Pull request**: Must have zero warnings to be merged

## Tools and Commands

```bash
# Check for warnings
cargo check

# Run all tests (must be warning-free)
cargo test

# Format code
cargo fmt

# Check for common issues
cargo clippy

# Build release version (must be warning-free)
cargo build --release
```

## Violations

- Code with warnings will not be merged
- Pull requests must be updated to resolve all warnings
- Repeated violations may result in contribution restrictions

---

This policy ensures MeshBBS maintains the highest code quality standards and professional development practices.