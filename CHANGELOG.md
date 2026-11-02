# Changelog

Notable changes to this project are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0]

### Added

- Simurgh, a fork of git-koi that draws a pixel-art Simurgh instead of a koi. The
  collection, history, development, and streak logic and the GitHub Action are
  carried over; the renderer, the motion, the plumage, and the behaviors are new.
- Six bodies: `zarrin`, `firuzeh`, `lajvard`, `sepid`, `shab`, and `arghavan`.
- Tail streamers colored by the account's languages, in proportion to their
  shares, and a golden farr for the current streak.
- Five behaviors: glide, soar, perch, seed fall, and sleep.
- English and Persian READMEs (`README.md`, `README.fa.md`) and getting-started
  guides, with growth
  stages, behaviors, the farr, and sample GIFs.
- `generate-docs`, an example that regenerates the README pictures.

### Changed

- The Action reads its own repository from GitHub instead of `Cargo.toml`, so a
  fork works without editing it. Publishing to crates.io is opt in through the
  `PUBLISH_TO_CRATES_IO` repository variable.
