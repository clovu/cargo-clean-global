## [1.2.1] - 2026-04-16

### 🐛 Bug Fixes

- Correct license declaration in Cargo.toml

### ⚙️ Miscellaneous Tasks

- Uptd deps
- Bump version to 1.2.1
## [1.2.0] - 2026-04-15

### 🚀 Features

- *(config)* Support default scan roots from global Cargo config #2

### 🐛 Bug Fixes

- Correct license specification in Cargo.toml

### 📚 Documentation

- Update README

### ⚙️ Miscellaneous Tasks

- Add "indicatif" to cSpell words list in settings.json
- Add "pathbuf" to cSpell words list in settings.json
- Bump version to 1.2.0
## [1.1.0] - 2026-04-02

### 🚀 Features

- *(progress)* Unify spinner status with scan and cleanup size metrics
- *(cli)* Improve terminal output readability and error highlighting
- *(cli)* Polish scan/cleanup console output formatting #1

### 🐛 Bug Fixes

- *(cli)* Allow debug runs without cargo subcommand forwarding
- *(cli)* Preserve cargo-forwarded subcommand parsing in debug mode

### ⚙️ Miscellaneous Tasks

- Fmt
- Bump version to 1.1.0
## [1.0.1] - 2026-03-28

### 🐛 Bug Fixes

- *(paths)* Skip names with common prefixes

### ⚙️ Miscellaneous Tasks

- Update version to 1.0.1
## [1.0.0] - 2026-03-19

### 🐛 Bug Fixes

- Make cargo clean-global work as a Cargo subcommand

### 🚜 Refactor

- Simplify scan root current-path check

### ⚙️ Miscellaneous Tasks

- Release version 1.0.0
## [0.1.0] - 2026-03-19

### 🚀 Features

- Clean global crates

### ⚙️ Miscellaneous Tasks

- Config file
- Publish crate to crates.io from release tags
