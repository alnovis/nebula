# Release Notes

## v0.2.19

**Release Date:** 2025-01-27

### Highlights

This release adds proper favicon support and improves developer experience with automated formatting.

### New Features

- **Favicon** — Multiple sizes for all platforms (browsers, iOS, Android)
- **Root favicon route** — `/favicon.ico` served from root for better crawler support
- **Git hooks** — Pre-commit hook runs `cargo fmt` automatically
- **Documentation** — Updated README, added CHANGELOG and RELEASE_NOTES

### Improvements

- README now includes full feature documentation
- Project structure diagram updated
- Tech stack table added

### Migration

No breaking changes. To enable git hooks after cloning:

```bash
git config core.hooksPath .hooks
```

---

## Previous Releases

See [CHANGELOG.md](CHANGELOG.md) for full version history.
