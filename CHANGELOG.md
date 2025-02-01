# Changelog

## Unreleased

### Changed
 - Removed the dependency on the unmaintained `encoding` create and switched to `encoding_rs`. See
   [RUSTSEC-2021-0153](https://rustsec.org/advisories/RUSTSEC-2021-0153).
 - The `Text::to_string()` method no longer takes a `DecoderTrap` argument (due to the removal of the `encoding`
   dependency).
