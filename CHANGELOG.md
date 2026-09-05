# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).


## [Unreleased](https://github.com/pack-it/contextdiff-parser/compare/0.0.1...HEAD)

### Added
- Test cases for the full parsing and translation from context diffs to unified diffs.

### Fixed
- Fix wrong expected hunk length for hunks of files that don't exist (new or removed files).


## [v0.0.1](https://github.com/pack-it/contextdiff-parser/releases/tag/0.0.1) - 2026-07-10

First release of contextdiff-parser, consisting of the basic context diff parser implementation and a simple translator to unified diff format.
