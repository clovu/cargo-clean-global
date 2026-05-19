## [1.2.2](https://github.com/clovu/cargo-clean-global/compare/v1.2.1..v1.2.2) - 2026-05-19

### 🚜 Refactor

- Make cleanup callback optional - ([c437203](https://github.com/clovu/cargo-clean-global/commit/c437203a154e10750ce173f30a2ca7ac2c612625))

### 📚 Documentation

- Add changelog - ([23b135e](https://github.com/clovu/cargo-clean-global/commit/23b135edc8ed20d1df442393b3d84e806271a750))
- Link changelog entries - ([2e63ad4](https://github.com/clovu/cargo-clean-global/commit/2e63ad405d448da04d8ffd8fba10ab32551cd284))

### 🧪 Testing

- Cover cleanup dry-run behavior - ([dce974c](https://github.com/clovu/cargo-clean-global/commit/dce974c13871088f6a25c5762deb78831407def3))
- Cover cleanup deletion behavior - ([26bc227](https://github.com/clovu/cargo-clean-global/commit/26bc227f1537a2d711da8064cd20552fa8675eba))
- Cover missing cleanup target - ([fd02af3](https://github.com/clovu/cargo-clean-global/commit/fd02af3b6659ea7e43dfdf6562331ab9cbd9726d))
- Cover non-directory cleanup target - ([dede278](https://github.com/clovu/cargo-clean-global/commit/dede2785c7ca1413283a6134301530d2a4ef8e1d))
- Cover symlink cleanup target - ([7100357](https://github.com/clovu/cargo-clean-global/commit/71003579cd46f186cf91bc72d2b3a8f1c5b37301))

### ⚙️ Miscellaneous Tasks

- Update deps - ([707f62d](https://github.com/clovu/cargo-clean-global/commit/707f62d553238a2bd1d220805435d6d9ea726d49))
- Remove redundant license-file from Cargo.toml - ([d059d3f](https://github.com/clovu/cargo-clean-global/commit/d059d3ff3595494b5f298fea2eef88ee005cc766))
- Disable automatic spelling correction in commit messages - ([ed32129](https://github.com/clovu/cargo-clean-global/commit/ed32129c6e1ba1782d3b04856654e88178c88a05))
## [1.2.1](https://github.com/clovu/cargo-clean-global/compare/v1.2.0..v1.2.1) - 2026-04-16

### 🐛 Bug Fixes

- Correct license declaration in Cargo.toml - ([edb08f8](https://github.com/clovu/cargo-clean-global/commit/edb08f8d5339424e0fc322597fa759e142a6399c))

### ⚙️ Miscellaneous Tasks

- Uptd deps - ([9d0433f](https://github.com/clovu/cargo-clean-global/commit/9d0433f0eca046ba3ea805a69bd1b4bcab124ebf))
- Bump version to 1.2.1 - ([d1ffba1](https://github.com/clovu/cargo-clean-global/commit/d1ffba1a2c63f9d13bb67c355d3dad5661b1f056))
## [1.2.0](https://github.com/clovu/cargo-clean-global/compare/v1.1.0..v1.2.0) - 2026-04-15

### 🚀 Features

- *(config)* Support default scan roots from global Cargo config #2 - ([5657ceb](https://github.com/clovu/cargo-clean-global/commit/5657cebb6e87349eecf64dd5834bfaf74817474f))

### 🐛 Bug Fixes

- Correct license specification in Cargo.toml - ([3a08641](https://github.com/clovu/cargo-clean-global/commit/3a08641a122a43f0ec7c909a68f0eefc7d2b2a47))

### 📚 Documentation

- Update README - ([594c93d](https://github.com/clovu/cargo-clean-global/commit/594c93d344ebdb61f33a5e1bf70923ecd01e8e50))

### ⚙️ Miscellaneous Tasks

- Add "indicatif" to cSpell words list in settings.json - ([dbe2e46](https://github.com/clovu/cargo-clean-global/commit/dbe2e4684c776e5e8508dd7002ebefc5ed32f4cf))
- Add "pathbuf" to cSpell words list in settings.json - ([38d19ca](https://github.com/clovu/cargo-clean-global/commit/38d19ca4abf4520334f967e76725e714454aa078))
- Bump version to 1.2.0 - ([a805d93](https://github.com/clovu/cargo-clean-global/commit/a805d93ed00c7a2c3108580a6d7e6f2926bb9c6f))
## [1.1.0](https://github.com/clovu/cargo-clean-global/compare/v1.0.1..v1.1.0) - 2026-04-02

### 🚀 Features

- *(cli)* Improve terminal output readability and error highlighting - ([fb6f455](https://github.com/clovu/cargo-clean-global/commit/fb6f45533c1af52c57833055549052cebe59ad85))
- *(cli)* Polish scan/cleanup console output formatting #1 - ([b45b59c](https://github.com/clovu/cargo-clean-global/commit/b45b59cf10d947ccbeac3090d6cd43de2f98013e))
- *(progress)* Unify spinner status with scan and cleanup size metrics - ([ec65bc0](https://github.com/clovu/cargo-clean-global/commit/ec65bc08ee986536435961952c41551c1a36d46d))

### 🐛 Bug Fixes

- *(cli)* Allow debug runs without cargo subcommand forwarding - ([b85fe68](https://github.com/clovu/cargo-clean-global/commit/b85fe684fd32bfd6dd78e741d649d25fc11729a1))
- *(cli)* Preserve cargo-forwarded subcommand parsing in debug mode - ([89cdcf2](https://github.com/clovu/cargo-clean-global/commit/89cdcf212cdd55c28d31444d64a58ab75212b8f6))

### ⚙️ Miscellaneous Tasks

- Fmt - ([2ebf173](https://github.com/clovu/cargo-clean-global/commit/2ebf1732e353891917615e9cd4ef444630c38dd2))
- Bump version to 1.1.0 - ([9d5cbd4](https://github.com/clovu/cargo-clean-global/commit/9d5cbd4730e05e356dc71bf49af4d944a7b1d851))
## [1.0.1](https://github.com/clovu/cargo-clean-global/compare/v1.0.0..v1.0.1) - 2026-03-28

### 🐛 Bug Fixes

- *(paths)* Skip names with common prefixes - ([d676cd1](https://github.com/clovu/cargo-clean-global/commit/d676cd1b0129fda05e6d264ad98bb42301b26c56))

### ⚙️ Miscellaneous Tasks

- Update version to 1.0.1 - ([1581b97](https://github.com/clovu/cargo-clean-global/commit/1581b9758a02e8e79cb6cb171d2b0d998b95d0ae))
## [1.0.0](https://github.com/clovu/cargo-clean-global/compare/v0.1.0..v1.0.0) - 2026-03-19

### 🐛 Bug Fixes

- Make cargo clean-global work as a Cargo subcommand - ([e546c74](https://github.com/clovu/cargo-clean-global/commit/e546c744dbab12bbeedb51daae0fc21ba6d0df7c))

### 🚜 Refactor

- Simplify scan root current-path check - ([33c02ff](https://github.com/clovu/cargo-clean-global/commit/33c02fffff9ca8b67c6be4dafa53a37a98c31308))

### ⚙️ Miscellaneous Tasks

- Release version 1.0.0 - ([49c691b](https://github.com/clovu/cargo-clean-global/commit/49c691b5369d3b967ec3d0a1b81cd4469bd41db6))
## [0.1.0] - 2026-03-19

### 🚀 Features

- Clean global crates - ([6d1c244](https://github.com/clovu/cargo-clean-global/commit/6d1c244c140131ed45ab99837f18a76010f60bf1))

### ⚙️ Miscellaneous Tasks

- Config file - ([599669e](https://github.com/clovu/cargo-clean-global/commit/599669e31bed16fae63c6dcfd2ef1aad0126f916))
- Publish crate to crates.io from release tags - ([cd4ab36](https://github.com/clovu/cargo-clean-global/commit/cd4ab3656e9ee0e34fe7933bbbd3e71d0f97c6c0))

## New Contributors ❤️

* @clovu made their first contribution