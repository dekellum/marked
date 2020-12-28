## 0.3.0 (unreleased)

* `Document::len()` now returns u32 for compatibility to `with_capacity`, etc.

* `Document::fold()` now replaces the node's data with `NodeData::Hole` and
  returns the original `NodeData`.

* `Document::detach()` now replaces all descendant node data with
  `NodeData::Hole` and returns a new independent `Document` fragment with must
  be used.  Added `Document::unlink()` for cases where a returned fragment is not
  required.

* Added `Document::attach_child()` and `attach_before_sibling()` as logical
  inverses to `detach()`.

* Add `Document::descendants()` as more general form of `nodes()`, as well as
  `NodeRef::descendants()`.

* Made `&self` lifetime more lenient for many `NodeRef` methods.

* Misc memory use optimizations in the form of better capacity guesses and
  selective application of `shrink_to_fit` based on tested cost, and the
  likelihood of the latter causing a memory move by the allocator.

## 0.2.0 (2020-4-12)

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
