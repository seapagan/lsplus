# Changelog

This is an auto-generated log of all the changes that have been made to the
project since the first release.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.7.0](https://github.com/seapagan/lsplus/releases/tag/0.7.0) (2025-09-08)

**Closed Issues**

- Passing a link to a folder will show the link file instead of the contents ([#48](https://github.com/seapagan/lsplus/issues/48)) by [seapagan](https://github.com/seapagan)

**New Features**

- Add a github action to build a release for linux and mac ([#93](https://github.com/seapagan/lsplus/pull/93)) by [seapagan](https://github.com/seapagan)
- Update rust edition from 2021 to 2024 ([#91](https://github.com/seapagan/lsplus/pull/91)) by [seapagan](https://github.com/seapagan)
- Move from dependabot to renovate ([#82](https://github.com/seapagan/lsplus/pull/82)) by [seapagan](https://github.com/seapagan)

**Bug Fixes**

- Fix some icons and file associations ([#81](https://github.com/seapagan/lsplus/pull/81)) by [seapagan](https://github.com/seapagan)
- Properly list the contents of a symlink to folder ([#52](https://github.com/seapagan/lsplus/pull/52)) by [seapagan](https://github.com/seapagan)

**Refactoring**

- Perform some minor refactoring ([#59](https://github.com/seapagan/lsplus/pull/59)) by [seapagan](https://github.com/seapagan)

**Automatic Testing**

- Add test suite ([#51](https://github.com/seapagan/lsplus/pull/51)) by [seapagan](https://github.com/seapagan)

**Dependency Updates**

- Update actions/checkout action to v5 ([#94](https://github.com/seapagan/lsplus/pull/94)) by [renovate[bot]](https://github.com/apps/renovate)
- Update Rust crate chrono to v0.4.42 ([#92](https://github.com/seapagan/lsplus/pull/92)) by [renovate[bot]](https://github.com/apps/renovate)
- Update actions/checkout action to v5 ([#90](https://github.com/seapagan/lsplus/pull/90)) by [renovate[bot]](https://github.com/apps/renovate)
- Update Rust crate tempfile to v3.21.0 ([#89](https://github.com/seapagan/lsplus/pull/89)) by [renovate[bot]](https://github.com/apps/renovate)
- Update Rust crate glob to v0.3.3 ([#88](https://github.com/seapagan/lsplus/pull/88)) by [renovate[bot]](https://github.com/apps/renovate)
- Update Rust crate filetime to v0.2.26 ([#87](https://github.com/seapagan/lsplus/pull/87)) by [renovate[bot]](https://github.com/apps/renovate)
- Update Rust crate config to v0.15.15 ([#86](https://github.com/seapagan/lsplus/pull/86)) by [renovate[bot]](https://github.com/apps/renovate)
- Update Rust crate clap to v4.5.47 ([#84](https://github.com/seapagan/lsplus/pull/84)) by [renovate[bot]](https://github.com/apps/renovate)
- Update Rust crate assert_cmd to v2.0.17 ([#83](https://github.com/seapagan/lsplus/pull/83)) by [renovate[bot]](https://github.com/apps/renovate)
- Bump chrono from 0.4.39 to 0.4.41 ([#80](https://github.com/seapagan/lsplus/pull/80)) by [dependabot[bot]](https://github.com/apps/dependabot)
- *and 11 more dependency updates*

[`Full Changelog`](https://github.com/seapagan/lsplus/compare/0.6.0...0.7.0) | [`Diff`](https://github.com/seapagan/lsplus/compare/0.6.0...0.7.0.diff) | [`Patch`](https://github.com/seapagan/lsplus/compare/0.6.0...0.7.0.patch)

## [0.6.0](https://github.com/seapagan/lsplus/releases/tag/0.6.0) (2024-12-30)

**Closed Issues**

- Short mode should not show link targets ([#27](https://github.com/seapagan/lsplus/issues/27)) by [seapagan](https://github.com/seapagan)
- Some icons truncated under `kitty` terminal ([#18](https://github.com/seapagan/lsplus/issues/18)) by [seapagan](https://github.com/seapagan)

**New Features**

- Don't show link target in short mode ([#45](https://github.com/seapagan/lsplus/pull/45)) by [seapagan](https://github.com/seapagan)

**Bug Fixes**

- Bugfix: inconsistent spacing in short mode ([#46](https://github.com/seapagan/lsplus/pull/46)) by [seapagan](https://github.com/seapagan)

[`Full Changelog`](https://github.com/seapagan/lsplus/compare/0.5.0...0.6.0) | [`Diff`](https://github.com/seapagan/lsplus/compare/0.5.0...0.6.0.diff) | [`Patch`](https://github.com/seapagan/lsplus/compare/0.5.0...0.6.0.patch)

## [0.5.0](https://github.com/seapagan/lsplus/releases/tag/0.5.0) (2024-11-18)

**Closed Issues**

- Panic if a file named exactly the same as a registered extension exists in the folder ([#36](https://github.com/seapagan/lsplus/issues/36)) by [seapagan](https://github.com/seapagan)

**Bug Fixes**

- Fix bug causing panic if file named as extension exists ([#38](https://github.com/seapagan/lsplus/pull/38)) by [seapagan](https://github.com/seapagan)

**Dependency Updates**

- Bump serde from 1.0.210 to 1.0.215 ([#37](https://github.com/seapagan/lsplus/pull/37)) by [dependabot[bot]](https://github.com/apps/dependabot)
- Bump config from 0.14.0 to 0.14.1 ([#35](https://github.com/seapagan/lsplus/pull/35)) by [dependabot[bot]](https://github.com/apps/dependabot)
- Bump clap from 4.5.18 to 4.5.20 ([#33](https://github.com/seapagan/lsplus/pull/33)) by [dependabot[bot]](https://github.com/apps/dependabot)
- Bump clap from 4.5.17 to 4.5.18 ([#31](https://github.com/seapagan/lsplus/pull/31)) by [dependabot[bot]](https://github.com/apps/dependabot)
- Bump clap from 4.5.15 to 4.5.17 ([#30](https://github.com/seapagan/lsplus/pull/30)) by [dependabot[bot]](https://github.com/apps/dependabot)
- Bump serde from 1.0.209 to 1.0.210 ([#29](https://github.com/seapagan/lsplus/pull/29)) by [dependabot[bot]](https://github.com/apps/dependabot)

[`Full Changelog`](https://github.com/seapagan/lsplus/compare/0.4.0...0.5.0) | [`Diff`](https://github.com/seapagan/lsplus/compare/0.4.0...0.5.0.diff) | [`Patch`](https://github.com/seapagan/lsplus/compare/0.4.0...0.5.0.patch)

## [0.4.0](https://github.com/seapagan/lsplus/releases/tag/0.4.0) (2024-08-29)

**New Features**

- Add a configuration file to enable setting defaults for the CLI options ([#26](https://github.com/seapagan/lsplus/pull/26)) by [seapagan](https://github.com/seapagan)

[`Full Changelog`](https://github.com/seapagan/lsplus/compare/0.3.1...0.4.0) | [`Diff`](https://github.com/seapagan/lsplus/compare/0.3.1...0.4.0.diff) | [`Patch`](https://github.com/seapagan/lsplus/compare/0.3.1...0.4.0.patch)

## [0.3.1](https://github.com/seapagan/lsplus/releases/tag/0.3.1) (2024-08-16)

There were no merged pull requests or closed issues for this release.

See the Full Changelog below for details.

[`Full Changelog`](https://github.com/seapagan/lsplus/compare/0.3.0...0.3.1) | [`Diff`](https://github.com/seapagan/lsplus/compare/0.3.0...0.3.1.diff) | [`Patch`](https://github.com/seapagan/lsplus/compare/0.3.0...0.3.1.patch)

## [0.3.0](https://github.com/seapagan/lsplus/releases/tag/0.3.0) (2024-08-16)

**New Features**

- Allow using wildcards for the path ([#22](https://github.com/seapagan/lsplus/pull/22)) by [seapagan](https://github.com/seapagan)
- Show executable files as green and bold under unix systems ([#19](https://github.com/seapagan/lsplus/pull/19)) by [seapagan](https://github.com/seapagan)
- Add a '--fuzzy-time' option for file modified time ([#17](https://github.com/seapagan/lsplus/pull/17)) by [seapagan](https://github.com/seapagan)
- Implement error handling for main function ([#16](https://github.com/seapagan/lsplus/pull/16)) by [seapagan](https://github.com/seapagan)

**Dependency Updates**

- Bump clap from 4.5.13 to 4.5.15 ([#24](https://github.com/seapagan/lsplus/pull/24)) by [dependabot[bot]](https://github.com/apps/dependabot)
- Bump clap from 4.5.11 to 4.5.13 ([#23](https://github.com/seapagan/lsplus/pull/23)) by [dependabot[bot]](https://github.com/apps/dependabot)
- Bump clap from 4.5.9 to 4.5.11 ([#20](https://github.com/seapagan/lsplus/pull/20)) by [dependabot[bot]](https://github.com/apps/dependabot)

[`Full Changelog`](https://github.com/seapagan/lsplus/compare/0.2.0...0.3.0) | [`Diff`](https://github.com/seapagan/lsplus/compare/0.2.0...0.3.0.diff) | [`Patch`](https://github.com/seapagan/lsplus/compare/0.2.0...0.3.0.patch)

## [0.2.0](https://github.com/seapagan/lsplus/releases/tag/0.2.0) (2024-07-23)

**New Features**

- Add file and folder icons, make them optional ([#14](https://github.com/seapagan/lsplus/pull/14)) by [seapagan](https://github.com/seapagan)

[`Full Changelog`](https://github.com/seapagan/lsplus/compare/0.1.0...0.2.0) | [`Diff`](https://github.com/seapagan/lsplus/compare/0.1.0...0.2.0.diff) | [`Patch`](https://github.com/seapagan/lsplus/compare/0.1.0...0.2.0.patch)

## [0.1.0](https://github.com/seapagan/lsplus/releases/tag/0.1.0) (2024-07-22)

**New Features**

- Human readable file size ([#9](https://github.com/seapagan/lsplus/pull/9)) by [seapagan](https://github.com/seapagan)
- Only show dot-files when passed the '-a' or '--all' flag ([#8](https://github.com/seapagan/lsplus/pull/8)) by [seapagan](https://github.com/seapagan)
- Provide a custom version command ([#7](https://github.com/seapagan/lsplus/pull/7)) by [seapagan](https://github.com/seapagan)
- Show item type (dir, symlink, file) before the attrs. ([#5](https://github.com/seapagan/lsplus/pull/5)) by [seapagan](https://github.com/seapagan)
- Change the CLI definition to use 'derive' syntax ([#4](https://github.com/seapagan/lsplus/pull/4)) by [seapagan](https://github.com/seapagan)
- Add flag to sort folders before files ([#2](https://github.com/seapagan/lsplus/pull/2)) by [seapagan](https://github.com/seapagan)
- Colorize the output ([#1](https://github.com/seapagan/lsplus/pull/1)) by [seapagan](https://github.com/seapagan)

**Bug Fixes**

- BUGFIX: crashes if the supplied `path` is a file and not a folder ([#3](https://github.com/seapagan/lsplus/pull/3)) by [seapagan](https://github.com/seapagan)

**Refactoring**

- Refactor the project layout, split into modules by functionality ([#12](https://github.com/seapagan/lsplus/pull/12)) by [seapagan](https://github.com/seapagan)
- Rename project to `lsplus` ([#6](https://github.com/seapagan/lsplus/pull/6)) by [seapagan](https://github.com/seapagan)

**Documentation**

- Add first docs and a gh action to publish ([#10](https://github.com/seapagan/lsplus/pull/10)) by [seapagan](https://github.com/seapagan)

**Dependency Updates**

- Bump actions/checkout from 2 to 4 ([#11](https://github.com/seapagan/lsplus/pull/11)) by [dependabot[bot]](https://github.com/apps/dependabot)

---
*This changelog was generated using [github-changelog-md](http://changelog.seapagan.net/) by [Seapagan](https://github.com/seapagan)*
