## 0.2.0 (unreleased)
* The `marked::xml` module and xml-rs dependency is now under a non-default
  _xml_ feature. The xml-rs crate appears to not manage or test MSRV. Patch
  release 0.8.1 of xml-rs no longer builds on rust 0.38.0 (our MSRV). A
  workaround for our users is to constrain to xml-rs 0.8.0.

* Replace unnamed `NodeData` enum structs with `DocumentType` and
  `ProcessingInstruction`. To these types and `Element`, add a private
  zero-size member for future proofing.

* Add `Document::with_capacity` constructor, `Document::len` and
  `Document::is_empty` for visibility to capacity and occupied length of
  `Node`s.

* Expose `Document::append_deep_clone` for appending a sub-graph from another
  document.

* Add in-place `Document::compact`, found more efficient than `deep_clone` and
  drop of the original.

* `Node` now implements `Clone` including parent/child/sibling references.

* Add `Document:bulk_clone` for a faster clone without removing non-reachable
  nodes.

* Properly constrain dependencies (#1)

* Added various structural debug asserts. For example, for which `NodeData`
  variants can have children nodes, or where `NodeData::Document` and `Hole`
  nodes are applicable.

* XML `Whitespace` events (a subcase of text) are now ignored on parse.

## 0.1.0 (2020-3-15)
* Initial release.
