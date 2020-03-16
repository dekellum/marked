# The Marked Project

[![deps status](https://deps.rs/repo/github/dekellum/marked/status.svg)](https://deps.rs/repo/github/dekellum/marked)
[![Travis CI Build](https://travis-ci.org/dekellum/marked.svg?branch=master)](https://travis-ci.org/dekellum/marked)

A rust language project for parsing, filtering, selecting and serializing HTML
and XML mark-up.

See the _[marked]_ crate or _[marked-cli]_ crates or the README(s) and
CHANGELOG(s) under this ([github hosted]) source tree and cargo workspace.

## Feature Overview

Currently implemented features:

### A vector-allocated, indexed, DOM-like tree structure

The `marked::Document` is a DOM-like tree structure suitable for HTML and
XML. This was forked from the _[victor]_ project (same author as _html5ever_)
and further optimized.  It is implemented as a (std) `Vec` of `Node` types,
which references parent, siblings and children via (std) `NonZeroU32` indexes
for space efficiency.

### _html5ever_ integration

Including HTML5 document and fragment parsing and HTML5 serialization (mark-up
output). With the `marked::Document` (DOM), parsing and serialization is
measurably faster (see benchmarks in source tree) than the `RcDom` previously
included with *html5ever* associated crates, and mutating the `Document` is
more straightforward, via a mutable reference.

### _xml-rs_ integration

Strict, UTF-8 XML parsing to `marked::Document` is currently supported by
integration of the _[xml-rs]_ crate.

### Legacy character encoding support

An estimated 5% of the web remains in encodings other than UTF-8. That is too
common to be treated as as an error. Via `marked::html::parse_buffered`:

* Decoding via _encoding_rs_ which implements _[The Encoding Standard]_ including
  alternative names (labels) for supported encodings.

* HTML5 parsing restart from initial (4k) buffer with new encoding hints
  obtained from \<head>/\<meta> `charset` or an `http-equiv` `content-type` with
  charset.

* Byte-Order-Mark BOM sniffing as high priority `EncodingHint` for UTF-8, UTF-16
  Big-Endian and UTF-16 Little-Endian.

* "Impossible" hints from the above are ignored. For example, if we read a hint
  from UTF-8 that says its UTF-16LE (which would make it impossible to
  read the same hint if it was used).

(Note that the _detection_ features are not currently provided by _html5ever_ and
associated crates.)

### Rust "selectors" API

A `NodeRef` type with "CSS selectors"-like methods to recursively `select` and
`find` elements using closure predicates.  We prefer direct rust language
compiler support for writing such selection logic, over CSS or other
interpreted DSL.

### HTML tag and attribute metadata

See `marked::html::t` (tags) and `marked::html::a` (attributes) modules.

### Tree walking filters API

Bulk modifications to the DOM is easily and efficiently achieved with mutating
filter functions/closures and a tree walker (depth or breadth-first)
implementation in _marked_. This style of interface is sometimes called the
"visitor pattern". See `Document::filter_at` for details.  The crate also
includes the following built-in filters (a partial list):

`detach_banned_element`
: `Detach` known banned (via metadata) and unknown elements

`retain_basic_attributes`
: Remove all attributes that are not part of the "basic" logical set (via metadata)

`fold_empty_inline`
: `Fold` empty or meaninglessly "inline" elements

`text_normalize`
: Normalize text nodes by merging, replacing control characters and minimizing white-space.

An unreleased example, compatibility test and benchmark of _ammonia_ crate
equivalent filtering (for hygiene and safety) is included in the source tree
([./ammonia-compare])

## Roadmap

Features incomplete or unstarted which may be included in this project in the
future (PRs welcome):

* Complete (faster, more correct, legacy encodings) strict-mode XML parsing

* Lenient-mode XML parsing

* Optional (opt-in) direct charset detection (initial read buffer or entire
  document) via something like [chardet], integrated as high priority
  _EncodingHint_.

* XML/HTML pretty-indenting serialization (combines well with the existing white-space
  normalization features)

* XML (and XHTML) serialization

## License

This project is dual licensed under either of following:

* The Apache License, version 2.0
  ([LICENSE-APACHE] or http://www.apache.org/licenses/LICENSE-2.0)

* The MIT License
  ([LICENSE-MIT] or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the _marked_ project by you, as defined by the Apache License,
shall be dual licensed as above, without any additional terms or conditions.

[github hosted]: https://github.com/dekellum/marked
[marked]: https://docs.rs/crate/marked
[marked-cli]: https://crates.io/crates/marked-cli
[The Encoding Standard]: https://encoding.spec.whatwg.org/
[./ammonia-compare]: https://github.com/dekellum/marked/tree/master/ammonia-compare
[victor]: https://github.com/SimonSapin/victor
[chardet]: https://crates.io/crates/chardet
[xml-rs]: https://crates.io/crates/xml-rs
[LICENSE-APACHE]: https://github.com/dekellum/marked/tree/master/LICENSE-APACHE
[LICENSE-MIT]: https://github.com/dekellum/marked/tree/master/LICENSE-MIT
