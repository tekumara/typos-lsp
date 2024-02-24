# Changelog

## [0.1.13](https://github.com/tekumara/typos-lsp/compare/v0.1.12...v0.1.13) (2024-02-24)


### Chores

* bump typos-cli 1.18.2 ([58ccf55](https://github.com/tekumara/typos-lsp/commit/58ccf55454f922eb9b4e2b64cc4447d8ded8a3f3))

## [0.1.12](https://github.com/tekumara/typos-lsp/compare/v0.1.11...v0.1.12) (2024-02-05)


### Features

* typos 1.18.0 with January 2024 dictionary changes ([dfb3e55](https://github.com/tekumara/typos-lsp/commit/dfb3e55b91da6ee67085bfe660249384ffb07bd9))


### Bug Fixes

* error LNK2019: unresolved external symbol _GetLogicalDrives ([6ec5abf](https://github.com/tekumara/typos-lsp/commit/6ec5abf032170a64a18131f7923cf30f7644bbfe)), closes [#33](https://github.com/tekumara/typos-lsp/issues/33)
* use config file when workspace folder absent ([79ae446](https://github.com/tekumara/typos-lsp/commit/79ae44600a58d27115793894dadca6dfad006869)), closes [#31](https://github.com/tekumara/typos-lsp/issues/31)

## [0.1.11](https://github.com/tekumara/typos-lsp/compare/v0.1.10...v0.1.11) (2024-01-17)


### Features

* typos 1.17.1 with November/December 2023 dictionary changes ([76fc5cf](https://github.com/tekumara/typos-lsp/commit/76fc5cf2ff13b7f8e51a16276d5a7e7e0ecda470))

## [0.1.10](https://github.com/tekumara/typos-lsp/compare/v0.1.9...v0.1.10) (2023-12-26)


### Bug Fixes

* count positions as utf-16 code units ([de52345](https://github.com/tekumara/typos-lsp/commit/de523457fbc4aced4076f0ef61e5fb9e5f338b60)), closes [#22](https://github.com/tekumara/typos-lsp/issues/22)

## [0.1.9](https://github.com/tekumara/typos-lsp/compare/v0.1.8...v0.1.9) (2023-12-10)


### Bug Fixes

* typo start position corrected for multiple code point unicode ([e3d2752](https://github.com/tekumara/typos-lsp/commit/e3d2752a966889ba516f36e4c4de8c1ad48f9322)), closes [#22](https://github.com/tekumara/typos-lsp/issues/22)

## [0.1.8](https://github.com/tekumara/typos-lsp/compare/v0.1.7...v0.1.8) (2023-12-10)


### Features

* support custom config file ([67886b9](https://github.com/tekumara/typos-lsp/commit/67886b961fe9238fb6af19414bc07f18ad65959f)), closes [#19](https://github.com/tekumara/typos-lsp/issues/19)
* typos 1.16.23 ([ae1d36c](https://github.com/tekumara/typos-lsp/commit/ae1d36ca33d191b39d88d859ce6caf1864735498))
* typos 1.16.24 ([3006f24](https://github.com/tekumara/typos-lsp/commit/3006f2418e823902e7150d91b444d57eb78b7f64))

## [0.1.7](https://github.com/tekumara/typos-lsp/compare/v0.1.6...v0.1.7) (2023-10-22)


### Features

* configurable diagnostic severity ([7e7e743](https://github.com/tekumara/typos-lsp/commit/7e7e74397e77bc23b07e3d10ea863af4cdc1dccb)), closes [#17](https://github.com/tekumara/typos-lsp/issues/17)
* typos 1.16.20 ([fb013ea](https://github.com/tekumara/typos-lsp/commit/fb013ea3e96172e0c4ce07019fbebd71a2d6329e))

## [0.1.6](https://github.com/tekumara/typos-lsp/compare/v0.1.5...v0.1.6) (2023-09-23)


### Features

* typos 1.16.13 ([a67f844](https://github.com/tekumara/typos-lsp/commit/a67f844f5d369772dcd1be1d6eba89e607ccbe3e))

## [0.1.5](https://github.com/tekumara/typos-lsp/compare/v0.1.4...v0.1.5) (2023-09-23)


### Features

* publish typos-lsp during release ([1b933c7](https://github.com/tekumara/typos-lsp/commit/1b933c7f9f044330c18fa3ad32976f1b1acc9c87))
* typos 1.16.12 ([1426919](https://github.com/tekumara/typos-lsp/commit/1426919066d94bb36bb0bf292d03504177268669))

## [0.1.4](https://github.com/tekumara/typos-lsp/compare/v0.1.3...v0.1.4) (2023-09-04)


### Bug Fixes

* build arm64 binaries using correct target ([6f2c9d9](https://github.com/tekumara/typos-lsp/commit/6f2c9d9f89c74d5c6b0a8a57f7653550193c54b0)), closes [#13](https://github.com/tekumara/typos-lsp/issues/13)

## [0.1.3](https://github.com/tekumara/typos-lsp/compare/v0.1.2...v0.1.3) (2023-08-15)


### Features

* support config files (typos.toml etc.) ([#8](https://github.com/tekumara/typos-lsp/issues/8)) ([f16a143](https://github.com/tekumara/typos-lsp/commit/f16a143ab660969e2162b8eb2d388f87a041ec59)), closes [#6](https://github.com/tekumara/typos-lsp/issues/6)
* typos 1.16.5 ([29e2eac](https://github.com/tekumara/typos-lsp/commit/29e2eacc78406d648b422c21b6349eaadfa97007))


### Bug Fixes

* config files are now used on windows ([35080b3](https://github.com/tekumara/typos-lsp/commit/35080b374af3674dcc34938fed660333b772a9df))

## [0.1.2](https://github.com/tekumara/typos-lsp/compare/v0.1.1...v0.1.2) (2023-07-29)


### Features

* update to typos 1.14.9 ([fd19082](https://github.com/tekumara/typos-lsp/commit/fd1908284a8ceb101a47f6dd89d4c4168fabfaa1))
* update to typos 1.16.1 ([b9062af](https://github.com/tekumara/typos-lsp/commit/b9062afd338fafb79ea0d67ccb171f90350e10b0))

## 0.1.1 (2023-05-01)


### Features

* Initial release with diagnostics and quick fixes
