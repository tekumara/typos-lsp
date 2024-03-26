# Changelog

## [0.1.0](https://github.com/tekumara/typos-lsp/compare/v0.1.16...v0.1.0) (2024-03-26)


### Features

* add diagnostics ([a69923e](https://github.com/tekumara/typos-lsp/commit/a69923ed20d2a563c8464faa4041fee59259c594))
* add did_change ([c8f3fa1](https://github.com/tekumara/typos-lsp/commit/c8f3fa1176e60f1d8bdbfda8d5366c2fcd540e99))
* add did_close ([5ceb15a](https://github.com/tekumara/typos-lsp/commit/5ceb15aa128e0995d29737a8f4ba3ae33fb31df4))
* add quick fix ([6bc34d8](https://github.com/tekumara/typos-lsp/commit/6bc34d8d678f7f932d0215b782bab432519e4cc6))
* better error handling + loglevel ([42b36d1](https://github.com/tekumara/typos-lsp/commit/42b36d1cd55cd00b882babacf8c915eefd37f2b9))
* configurable diagnostic severity ([7e7e743](https://github.com/tekumara/typos-lsp/commit/7e7e74397e77bc23b07e3d10ea863af4cdc1dccb)), closes [#17](https://github.com/tekumara/typos-lsp/issues/17)
* drop vscode Windows 32-bit builds ([d7fedba](https://github.com/tekumara/typos-lsp/commit/d7fedba810fa4288f4adc3524b8028f7341bf299))
* hello world - initialize with test ([26b91bc](https://github.com/tekumara/typos-lsp/commit/26b91bc01fd1dd809f9758f1428d7be7b01cdab0))
* ignore typos in config files ([1841b4c](https://github.com/tekumara/typos-lsp/commit/1841b4ce12b64cc0000fa25765ad59aad1a37ac1)), closes [#47](https://github.com/tekumara/typos-lsp/issues/47)
* publish typos-lsp during release ([1b933c7](https://github.com/tekumara/typos-lsp/commit/1b933c7f9f044330c18fa3ad32976f1b1acc9c87))
* restart command + restart when config changes ([22d9eba](https://github.com/tekumara/typos-lsp/commit/22d9eba7b3d7eeb50c00e568b221df5adbd94580))
* spellcheck untitled docs ([fbcba05](https://github.com/tekumara/typos-lsp/commit/fbcba05b2374bfa2e5d56f3f3ba604b96b8a0a80))
* support bundled binary ([f980b47](https://github.com/tekumara/typos-lsp/commit/f980b4722e6e79d1bc66b1f505c168746a55dd11))
* support config files (typos.toml etc.) ([#8](https://github.com/tekumara/typos-lsp/issues/8)) ([f16a143](https://github.com/tekumara/typos-lsp/commit/f16a143ab660969e2162b8eb2d388f87a041ec59)), closes [#6](https://github.com/tekumara/typos-lsp/issues/6)
* support custom config file ([67886b9](https://github.com/tekumara/typos-lsp/commit/67886b961fe9238fb6af19414bc07f18ad65959f)), closes [#19](https://github.com/tekumara/typos-lsp/issues/19)
* typos 1.16.12 ([1426919](https://github.com/tekumara/typos-lsp/commit/1426919066d94bb36bb0bf292d03504177268669))
* typos 1.16.13 ([a67f844](https://github.com/tekumara/typos-lsp/commit/a67f844f5d369772dcd1be1d6eba89e607ccbe3e))
* typos 1.16.20 ([fb013ea](https://github.com/tekumara/typos-lsp/commit/fb013ea3e96172e0c4ce07019fbebd71a2d6329e))
* typos 1.16.23 ([ae1d36c](https://github.com/tekumara/typos-lsp/commit/ae1d36ca33d191b39d88d859ce6caf1864735498))
* typos 1.16.24 ([3006f24](https://github.com/tekumara/typos-lsp/commit/3006f2418e823902e7150d91b444d57eb78b7f64))
* typos 1.16.5 ([29e2eac](https://github.com/tekumara/typos-lsp/commit/29e2eacc78406d648b422c21b6349eaadfa97007))
* typos 1.17.1 with November/December 2023 dictionary changes ([76fc5cf](https://github.com/tekumara/typos-lsp/commit/76fc5cf2ff13b7f8e51a16276d5a7e7e0ecda470))
* typos 1.18.0 with January 2024 dictionary changes ([dfb3e55](https://github.com/tekumara/typos-lsp/commit/dfb3e55b91da6ee67085bfe660249384ffb07bd9))
* typos 1.19.0 Feb 2024 dictionary changes ([250390d](https://github.com/tekumara/typos-lsp/commit/250390d0e94ebe5172458dfdc65d160fb5968a81))
* update to typos 1.14.9 ([fd19082](https://github.com/tekumara/typos-lsp/commit/fd1908284a8ceb101a47f6dd89d4c4168fabfaa1))
* update to typos 1.16.1 ([b9062af](https://github.com/tekumara/typos-lsp/commit/b9062afd338fafb79ea0d67ccb171f90350e10b0))


### Bug Fixes

* build arm64 binaries using correct target ([6f2c9d9](https://github.com/tekumara/typos-lsp/commit/6f2c9d9f89c74d5c6b0a8a57f7653550193c54b0)), closes [#13](https://github.com/tekumara/typos-lsp/issues/13)
* config files are now used on windows ([35080b3](https://github.com/tekumara/typos-lsp/commit/35080b374af3674dcc34938fed660333b772a9df))
* corrections larger than the misspelling ([27f53ef](https://github.com/tekumara/typos-lsp/commit/27f53efe694fe6bdb3576970f352fb28807d9005))
* count positions as utf-16 code units ([de52345](https://github.com/tekumara/typos-lsp/commit/de523457fbc4aced4076f0ef61e5fb9e5f338b60)), closes [#22](https://github.com/tekumara/typos-lsp/issues/22)
* error LNK2019: unresolved external symbol _GetLogicalDrives ([6ec5abf](https://github.com/tekumara/typos-lsp/commit/6ec5abf032170a64a18131f7923cf30f7644bbfe)), closes [#33](https://github.com/tekumara/typos-lsp/issues/33)
* npm lints + test ([77a832a](https://github.com/tekumara/typos-lsp/commit/77a832a824b016f196764bb6586c5d6947744b1d))
* revert back to @types/vscode ^1.77.0 ([a90a48a](https://github.com/tekumara/typos-lsp/commit/a90a48a603b6768716fb0194de48969037406f8a))
* suppress code action warning ([cc3dbc0](https://github.com/tekumara/typos-lsp/commit/cc3dbc051bc3cdf8af9cd9ac06abcaad93259c24))
* trace server ([1f993a6](https://github.com/tekumara/typos-lsp/commit/1f993a6ea7e7543fe7a4e0a6e921491c931df175))
* typo start position corrected for multiple code point unicode ([e3d2752](https://github.com/tekumara/typos-lsp/commit/e3d2752a966889ba516f36e4c4de8c1ad48f9322)), closes [#22](https://github.com/tekumara/typos-lsp/issues/22)
* use config file when workspace folder absent ([79ae446](https://github.com/tekumara/typos-lsp/commit/79ae44600a58d27115793894dadca6dfad006869)), closes [#31](https://github.com/tekumara/typos-lsp/issues/31)
* use default policy for empty file uri ([34998d4](https://github.com/tekumara/typos-lsp/commit/34998d48567d0b59c432296142934381f0258c4e)), closes [#39](https://github.com/tekumara/typos-lsp/issues/39)
* use default policy for non-file uris ([0265ff1](https://github.com/tekumara/typos-lsp/commit/0265ff15e4af4a6274a7f5e77bd849a4299880c4)), closes [#36](https://github.com/tekumara/typos-lsp/issues/36)


### Chores

* add displayName ([3e3ba8b](https://github.com/tekumara/typos-lsp/commit/3e3ba8b4c2624cfa8bb9a977ccb785fe035dae7c))
* bootstrap releases for . and crates/typos-lsp ([#3](https://github.com/tekumara/typos-lsp/issues/3)) ([415de68](https://github.com/tekumara/typos-lsp/commit/415de680faef57f35711536852e5f99f261552ca))
* bump typos 1.14.8 ([457ea7e](https://github.com/tekumara/typos-lsp/commit/457ea7efe70a8b7462a3673e90df431c568d8cf2))
* bump typos-cli 1.18.2 ([58ccf55](https://github.com/tekumara/typos-lsp/commit/58ccf55454f922eb9b4e2b64cc4447d8ded8a3f3))
* **deps:** bump the lsp group with 1 update ([#46](https://github.com/tekumara/typos-lsp/issues/46)) ([8950859](https://github.com/tekumara/typos-lsp/commit/8950859ded9be8fc08dae577527a651a3888fdaa))
* **deps:** bump the lsp group with 4 updates ([#50](https://github.com/tekumara/typos-lsp/issues/50)) ([d084fc8](https://github.com/tekumara/typos-lsp/commit/d084fc8e7b1f441cd21d1901631db793c2b8c8de))
* **deps:** bump the vscode group with 14 updates ([#44](https://github.com/tekumara/typos-lsp/issues/44)) ([2b42b7f](https://github.com/tekumara/typos-lsp/commit/2b42b7fc6830737b933fa61245c4c35385d08ca0))
* fix merging in from https://github.com/crate-ci/typos/pull/710 ([99a02bb](https://github.com/tekumara/typos-lsp/commit/99a02bb732eb00538e2665e3c71b15d55b5d9886))
* **lsp:** bump the lsp group with 3 updates ([#43](https://github.com/tekumara/typos-lsp/issues/43)) ([9430d0b](https://github.com/tekumara/typos-lsp/commit/9430d0b29c8530f76cac4dc19a4b9e9d48add429))
* **main:** release 0.1.0 ([#1](https://github.com/tekumara/typos-lsp/issues/1)) ([fb0052a](https://github.com/tekumara/typos-lsp/commit/fb0052aeb630cdeeb4567a0c68f64345864872f2))
* **main:** release 0.1.1 ([#2](https://github.com/tekumara/typos-lsp/issues/2)) ([665f04d](https://github.com/tekumara/typos-lsp/commit/665f04d986cff9127230ec74d50fa66be5d4180f))
* **main:** release 0.1.10 ([#26](https://github.com/tekumara/typos-lsp/issues/26)) ([1e11cf2](https://github.com/tekumara/typos-lsp/commit/1e11cf24cb18f16b3aa5608f6e083dc41e6b074a))
* **main:** release 0.1.11 ([#30](https://github.com/tekumara/typos-lsp/issues/30)) ([389d2e0](https://github.com/tekumara/typos-lsp/commit/389d2e03de5cab1f61ff89a135290a6a98bc7bb4))
* **main:** release 0.1.12 ([#32](https://github.com/tekumara/typos-lsp/issues/32)) ([975fa04](https://github.com/tekumara/typos-lsp/commit/975fa04be6493e6468b66815f51b394d6e7e7d6f))
* **main:** release 0.1.12 ([#34](https://github.com/tekumara/typos-lsp/issues/34)) ([68f1d4c](https://github.com/tekumara/typos-lsp/commit/68f1d4ceda255e74ccdb6b6bb941262e1b050e16))
* **main:** release 0.1.13 ([#37](https://github.com/tekumara/typos-lsp/issues/37)) ([1276acd](https://github.com/tekumara/typos-lsp/commit/1276acd7c5a57bed8358cbc1104a1b736edaa659))
* **main:** release 0.1.14 ([#38](https://github.com/tekumara/typos-lsp/issues/38)) ([fc1cc02](https://github.com/tekumara/typos-lsp/commit/fc1cc0243a1a02e3a04d483874d8877c6f580b2f))
* **main:** release 0.1.15 ([#40](https://github.com/tekumara/typos-lsp/issues/40)) ([affa832](https://github.com/tekumara/typos-lsp/commit/affa8321fc587b76a780a32648017f1ca82e0582))
* **main:** release 0.1.16 ([#45](https://github.com/tekumara/typos-lsp/issues/45)) ([ef24d2d](https://github.com/tekumara/typos-lsp/commit/ef24d2da0b5dc1833d617529558d5aef14773852))
* **main:** release 0.1.16 ([#49](https://github.com/tekumara/typos-lsp/issues/49)) ([2b6287c](https://github.com/tekumara/typos-lsp/commit/2b6287c59e43ed27f44032a3f12c48486cc0aa48))
* **main:** release 0.1.16 ([#53](https://github.com/tekumara/typos-lsp/issues/53)) ([95af4b6](https://github.com/tekumara/typos-lsp/commit/95af4b6a53c47fe1e8562d5048ac83708b5dfefd))
* **main:** release 0.1.2 ([#7](https://github.com/tekumara/typos-lsp/issues/7)) ([09cf9ff](https://github.com/tekumara/typos-lsp/commit/09cf9ff6436cf41d70abcd1c9241bfa4e0141d17))
* **main:** release 0.1.3 ([#9](https://github.com/tekumara/typos-lsp/issues/9)) ([59e3b65](https://github.com/tekumara/typos-lsp/commit/59e3b651942d58c11f562bc9dc0752adf3a5e20d))
* **main:** release 0.1.4 ([#14](https://github.com/tekumara/typos-lsp/issues/14)) ([f7e5769](https://github.com/tekumara/typos-lsp/commit/f7e5769b32fe2c64dc299e6f5fb7e5ed23a9b87e))
* **main:** release 0.1.5 ([#15](https://github.com/tekumara/typos-lsp/issues/15)) ([d5b51bb](https://github.com/tekumara/typos-lsp/commit/d5b51bbd8d36bf207b21e134e59fefc16d4fc768))
* **main:** release 0.1.6 ([#16](https://github.com/tekumara/typos-lsp/issues/16)) ([38253bf](https://github.com/tekumara/typos-lsp/commit/38253bf707b5588b4fb82ce18d1d2d099df64963))
* **main:** release 0.1.7 ([#18](https://github.com/tekumara/typos-lsp/issues/18)) ([c5c9ea1](https://github.com/tekumara/typos-lsp/commit/c5c9ea12f0777e440f223c3bb0be3fb2fb0a8ecf))
* **main:** release 0.1.8 ([#21](https://github.com/tekumara/typos-lsp/issues/21)) ([05608bd](https://github.com/tekumara/typos-lsp/commit/05608bdbc8dc076f9a6fec92db80728968b66785))
* **main:** release 0.1.9 ([#24](https://github.com/tekumara/typos-lsp/issues/24)) ([29a1ab1](https://github.com/tekumara/typos-lsp/commit/29a1ab1568dd67c0c2125763395537949ab84f08))
* release 0.1.0 ([0d86be4](https://github.com/tekumara/typos-lsp/commit/0d86be45688ecbff756e270ab7d9c85392f88d78))
* typos 1.16.25 ([1645f75](https://github.com/tekumara/typos-lsp/commit/1645f75ea518e48c146388de94f50323b04cc5b8))

## [0.1.16](https://github.com/tekumara/typos-lsp/compare/v0.1.15...v0.1.16) (2024-03-26)


### Features

* drop vscode Windows 32-bit builds ([d7fedba](https://github.com/tekumara/typos-lsp/commit/d7fedba810fa4288f4adc3524b8028f7341bf299))
* ignore typos in config files ([1841b4c](https://github.com/tekumara/typos-lsp/commit/1841b4ce12b64cc0000fa25765ad59aad1a37ac1)), closes [#47](https://github.com/tekumara/typos-lsp/issues/47)


### Bug Fixes

* revert back to @types/vscode ^1.77.0 ([a90a48a](https://github.com/tekumara/typos-lsp/commit/a90a48a603b6768716fb0194de48969037406f8a))


### Chores

* **deps:** bump the lsp group with 1 update ([#46](https://github.com/tekumara/typos-lsp/issues/46)) ([8950859](https://github.com/tekumara/typos-lsp/commit/8950859ded9be8fc08dae577527a651a3888fdaa))
* **deps:** bump the lsp group with 4 updates ([#50](https://github.com/tekumara/typos-lsp/issues/50)) ([d084fc8](https://github.com/tekumara/typos-lsp/commit/d084fc8e7b1f441cd21d1901631db793c2b8c8de))
* **deps:** bump the vscode group with 14 updates ([#44](https://github.com/tekumara/typos-lsp/issues/44)) ([2b42b7f](https://github.com/tekumara/typos-lsp/commit/2b42b7fc6830737b933fa61245c4c35385d08ca0))
* **lsp:** bump the lsp group with 3 updates ([#43](https://github.com/tekumara/typos-lsp/issues/43)) ([9430d0b](https://github.com/tekumara/typos-lsp/commit/9430d0b29c8530f76cac4dc19a4b9e9d48add429))
* **main:** release 0.1.16 ([#45](https://github.com/tekumara/typos-lsp/issues/45)) ([ef24d2d](https://github.com/tekumara/typos-lsp/commit/ef24d2da0b5dc1833d617529558d5aef14773852))
* **main:** release 0.1.16 ([#49](https://github.com/tekumara/typos-lsp/issues/49)) ([2b6287c](https://github.com/tekumara/typos-lsp/commit/2b6287c59e43ed27f44032a3f12c48486cc0aa48))

## [0.1.15](https://github.com/tekumara/typos-lsp/compare/v0.1.14...v0.1.15) (2024-03-03)


### Bug Fixes

* use default policy for empty file uri ([34998d4](https://github.com/tekumara/typos-lsp/commit/34998d48567d0b59c432296142934381f0258c4e)), closes [#39](https://github.com/tekumara/typos-lsp/issues/39)

## [0.1.14](https://github.com/tekumara/typos-lsp/compare/v0.1.13...v0.1.14) (2024-03-03)


### Features

* typos 1.19.0 Feb 2024 dictionary changes ([250390d](https://github.com/tekumara/typos-lsp/commit/250390d0e94ebe5172458dfdc65d160fb5968a81))


### Bug Fixes

* use default policy for non-file uris ([0265ff1](https://github.com/tekumara/typos-lsp/commit/0265ff15e4af4a6274a7f5e77bd849a4299880c4)), closes [#36](https://github.com/tekumara/typos-lsp/issues/36)

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
