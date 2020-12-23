# marked

[![Rustdoc](https://docs.rs/marked/badge.svg)](https://docs.rs/marked)
[![Change Log](https://img.shields.io/crates/v/marked.svg?maxAge=3600&label=change%20log&color=9cf)](https://github.com/dekellum/marked/blob/master/marked/CHANGELOG.md)
[![Crates.io](https://img.shields.io/crates/v/marked.svg?maxAge=3600)](https://crates.io/crates/marked)
[![CI Status](https://github.com/dekellum/marked/workflows/CI/badge.svg?branch=master)](https://github.com/dekellum/marked/actions?query=workflow%3ACI)

Parsing, filtering, selecting and serializing HTML/XML markup.

See the above linked rustdoc or The Märkəd Project [../README] for a feature
overview.

## Optional Features

The following features may be enabled at build time. **All are disabled by
default, unless otherwise noted.**

_xml_
: Includes `marked::xml` module for xml support via the _xml-rs_ crate.

## Minimum supported rust version

MSRV := 1.38.0

The crate will fail fast on any lower rustc (via a build.rs version
check) and is also CI tested on this version.

Certain non-default features (e.g. _xml_) may include dependencies which have
higher MSRV requirements.

## License

This project is dual licensed under either of following:

* The Apache License, version 2.0
  ([../LICENSE-APACHE] or http://www.apache.org/licenses/LICENSE-2.0)

* The MIT License
  ([../LICENSE-MIT] or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in _märkəd_ by you, as defined by the Apache License, shall be
dual licensed as above, without any additional terms or conditions.

[../README]: https://github.com/dekellum/marked#readme
[../LICENSE-APACHE]: https://github.com/dekellum/marked/tree/master/LICENSE-APACHE
[../LICENSE-MIT]: https://github.com/dekellum/marked/tree/master/LICENSE-MIT
