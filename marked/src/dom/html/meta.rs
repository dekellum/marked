//! Static metadata for HTML elements and attributes.
//!
//! This file is generated via build/generate.rb and the build/meta.rs.erb
//! template. It should not be manually edited. To avoid any rust build-time
//! dependency however, the resulting source file (src/dom/html/meta.rs) is
//! also checked in.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::dom::LocalName;

/// A static lookup table for metadata on known HTML tags.
pub static TAG_META: LazyLock<HashMap<LocalName, TagMeta>> = LazyLock::new(init_tag_metadata);

/// Metadata about HTML tags and their attributes.
#[derive(Default)]
pub struct TagMeta {
    is_empty: bool,
    is_deprecated: bool,
    is_inline: bool,
    is_meta: bool,
    is_banned: bool,
    basic_attrs: Vec<LocalName>,
}

impl TagMeta {
    /// Return true if the element is defined to be empty: having no contents
    /// or end tag.
    ///
    /// Tags include: `area base basefont br col embed frame hr img input link menuitem meta param source wbr`.
    pub fn is_empty(&self) -> bool {
        self.is_empty
    }

    /// Return true if the tag is deprecated as of html5.
    ///
    /// Tags include: `acronym applet basefont big blink center content dir font frame frameset isindex listing menu menuitem nobr noframes plaintext s strike tt u xmp`.
    pub fn is_deprecated(&self) -> bool {
        self.is_deprecated
    }

    /// Return true if the tag reprsents an _inline_ element: is not a block
    /// layout producing element under normal use.
    ///
    /// Because HTML 5 no longer specifies this property, this is a
    /// somewhat arbitrary distinction maintained here, loosely based on HTML 4
    /// but extending for new tags. One noteworthy exception is that `<br>` is
    /// not considered inline.
    ///
    /// Tags include: `a abbr acronym audio b basefont bdi bdo big blink button canvas cite code data datalist del dfn em embed font i iframe img input ins kbd label map mark meter nobr noscript object output picture progress q ruby s samp script select slot small span strike strong sub sup textarea time tt u var video wbr`.
    pub fn is_inline(&self) -> bool {
        self.is_inline
    }

    /// Return true if the tag represents metadata only, where any content is
    /// not displayed text. e.g. `<head>`.
    ///
    /// Tags include: `base basefont head link meta title`.
    pub fn is_meta(&self) -> bool {
        self.is_meta
    }

    /// Return true if the tag is banned/blacklisted: where no content should
    /// be extracted, displayed, or otherwise used.
    ///
    /// Tags include: `button content datalist fieldset frame frameset input label legend noframes noscript object optgroup option script select slot style template textarea`.
    pub fn is_banned(&self) -> bool {
        self.is_banned
    }

    /// Return true if the given name is part of the _basic_ set of known
    /// attributes for this element.
    ///
    /// This _basic set_ of attributes excludes, among other things, attributes
    /// that are used exclusively for styling purposes.
    pub fn has_basic_attr(&self, name: &LocalName) -> bool {
        self.basic_attrs.binary_search(name).is_ok()
    }
}



/// `Namespace` constants
pub mod ns {
    use html5ever::ns;
    use crate::dom::Namespace;

    pub const HTML:           Namespace = ns!(html);
}

/// HTML tag constants
pub mod t {
    use html5ever::local_name as lname;
    use crate::dom::LocalName;
    use std::sync::LazyLock;

    /// Tag `<a>`: anchor.
    /// (meta: inline)
    pub const A:             LocalName = lname!("a");
    /// Tag `<abbr>`: abbreviation.
    /// (meta: inline)
    pub const ABBR:          LocalName = lname!("abbr");
    /// Tag `<acronym>`: acronym.
    /// (meta: deprecated inline)
    pub const ACRONYM:       LocalName = lname!("acronym");
    /// Tag `<address>`: contact information for the author or owner.
    pub const ADDRESS:       LocalName = lname!("address");
    /// Tag `<applet>`: embedded applet.
    /// (meta: deprecated)
    pub const APPLET:        LocalName = lname!("applet");
    /// Tag `<area>`: area inside an image-map.
    /// (meta: empty)
    pub const AREA:          LocalName = lname!("area");
    /// Tag `<article>`: Structure: an independent content element.
    pub const ARTICLE:       LocalName = lname!("article");
    /// Tag `<aside>`: Structure: tengentially related content.
    pub const ASIDE:         LocalName = lname!("aside");
    /// Tag `<audio>`: Sound content.
    /// (meta: inline)
    pub const AUDIO:         LocalName = lname!("audio");
    /// Tag `<b>`: bold text.
    /// (meta: inline)
    pub const B:             LocalName = lname!("b");
    /// Tag `<base>`: default address or target for all links on a page.
    /// (meta: empty meta)
    pub const BASE:          LocalName = lname!("base");
    /// Tag `<basefont>`: default font; color; or size for the text in a page.
    /// (meta: empty deprecated inline meta)
    pub const BASEFONT:      LocalName = lname!("basefont");
    /// Tag `<bdi>`: Text isolated from surrounding for BIDI formatting.
    /// (meta: inline)
    pub const BDI:           LocalName = lname!("bdi");
    /// Tag `<bdo>`: the text direction.
    /// (meta: inline)
    pub const BDO:           LocalName = lname!("bdo");
    /// Tag `<big>`: big text.
    /// (meta: deprecated inline)
    pub const BIG:           LocalName = lname!("big");
    /// Tag `<blink>`: blinking text.
    /// (meta: deprecated inline)
    pub const BLINK:         LocalName = lname!("blink");
    /// Tag `<blockquote>`: long quotation.
    pub const BLOCKQUOTE:    LocalName = lname!("blockquote");
    /// Tag `<body>`: the document's body.
    pub const BODY:          LocalName = lname!("body");
    /// Tag `<br>`: single line break.
    /// (meta: empty)
    pub const BR:            LocalName = lname!("br");
    /// Tag `<button>`: push button.
    /// (meta: inline banned)
    pub const BUTTON:        LocalName = lname!("button");
    /// Tag `<canvas>`: canvas for drawing graphics and animations.
    /// (meta: inline)
    pub const CANVAS:        LocalName = lname!("canvas");
    /// Tag `<caption>`: table caption.
    pub const CAPTION:       LocalName = lname!("caption");
    /// Tag `<center>`: centered text.
    /// (meta: deprecated)
    pub const CENTER:        LocalName = lname!("center");
    /// Tag `<cite>`: citation.
    /// (meta: inline)
    pub const CITE:          LocalName = lname!("cite");
    /// Tag `<code>`: computer code text.
    /// (meta: inline)
    pub const CODE:          LocalName = lname!("code");
    /// Tag `<col>`: attribute values for one or more columns in a table.
    /// (meta: empty)
    pub const COL:           LocalName = lname!("col");
    /// Tag `<colgroup>`: group of columns in a table for formatting.
    pub const COLGROUP:      LocalName = lname!("colgroup");
    /// Tag `<content>`: Shadow DOM content placeholder element.
    /// (meta: deprecated banned)
    pub const CONTENT:       LocalName = lname!("content");
    /// Tag `<data>`: adds machine-oriented data representation.
    /// (meta: inline)
    pub const DATA:          LocalName = lname!("data");
    /// Tag `<datalist>`: container for option elements.
    /// (meta: inline banned)
    pub const DATALIST:      LocalName = lname!("datalist");
    /// Tag `<dd>`: description of a term in a definition list.
    pub const DD:            LocalName = lname!("dd");
    /// Tag `<del>`: deleted text.
    /// (meta: inline)
    pub const DEL:           LocalName = lname!("del");
    /// Tag `<details>`: optional additional details (also: summary).
    pub const DETAILS:       LocalName = lname!("details");
    /// Tag `<dfn>`: definition term.
    /// (meta: inline)
    pub const DFN:           LocalName = lname!("dfn");
    /// Tag `<dialog>`: dialog box or other interactive component.
    pub const DIALOG:        LocalName = lname!("dialog");
    /// Tag `<dir>`: directory list.
    /// (meta: deprecated)
    pub const DIR:           LocalName = lname!("dir");
    /// Tag `<div>`: section in a document.
    pub const DIV:           LocalName = lname!("div");
    /// Tag `<dl>`: definition list.
    pub const DL:            LocalName = lname!("dl");
    /// Tag `<dt>`: term (an item) in a definition list.
    pub const DT:            LocalName = lname!("dt");
    /// Tag `<em>`: emphasized text.
    /// (meta: inline)
    pub const EM:            LocalName = lname!("em");
    /// Tag `<embed>`: embed content by external app or plug-in.
    /// (meta: empty inline)
    pub const EMBED:         LocalName = lname!("embed");
    /// Tag `<fieldset>`: border around elements in a form.
    /// (meta: banned)
    pub const FIELDSET:      LocalName = lname!("fieldset");
    /// Tag `<figcaption>`: Structure: a figure caption.
    pub const FIGCAPTION:    LocalName = lname!("figcaption");
    /// Tag `<figure>`: Structure: self contained content that can be moved.
    pub const FIGURE:        LocalName = lname!("figure");
    /// Tag `<font>`: font; color; or size for text.
    /// (meta: deprecated inline)
    pub const FONT:          LocalName = lname!("font");
    /// Tag `<footer>`: Structure: a footer of a section.
    pub const FOOTER:        LocalName = lname!("footer");
    /// Tag `<form>`: form for user input.
    pub const FORM:          LocalName = lname!("form");
    /// Tag `<frame>`: window (a frame) in a frameset.
    /// (meta: empty deprecated banned)
    pub const FRAME:         LocalName = lname!("frame");
    /// Tag `<frameset>`: set of frames.
    /// (meta: deprecated banned)
    pub const FRAMESET:      LocalName = lname!("frameset");
    /// Tag `<h1>`: heading level 1.
    pub const H1:            LocalName = lname!("h1");
    /// Tag `<h2>`: heading level 2.
    pub const H2:            LocalName = lname!("h2");
    /// Tag `<h3>`: heading level 3.
    pub const H3:            LocalName = lname!("h3");
    /// Tag `<h4>`: heading level 4.
    pub const H4:            LocalName = lname!("h4");
    /// Tag `<h5>`: heading level 5.
    pub const H5:            LocalName = lname!("h5");
    /// Tag `<h6>`: heading level 6.
    pub const H6:            LocalName = lname!("h6");
    /// Tag `<head>`: information about the document.
    /// (meta: meta)
    pub const HEAD:          LocalName = lname!("head");
    /// Tag `<header>`: Structure: a header of a section.
    pub const HEADER:        LocalName = lname!("header");
    /// Tag `<hgroup>`: Structure: a group of headings.
    pub const HGROUP:        LocalName = lname!("hgroup");
    /// Tag `<hr>`: horizontal line.
    /// (meta: empty)
    pub const HR:            LocalName = lname!("hr");
    /// Tag `<html>`: document.
    pub const HTML:          LocalName = lname!("html");
    /// Tag `<i>`: italic text.
    /// (meta: inline)
    pub const I:             LocalName = lname!("i");
    /// Tag `<iframe>`: inline frame.
    /// (meta: inline)
    pub const IFRAME:        LocalName = lname!("iframe");
    /// Tag `<img>`: image.
    /// (meta: empty inline)
    pub const IMG:           LocalName = lname!("img");
    /// Tag `<input>`: input control.
    /// (meta: empty inline banned)
    pub const INPUT:         LocalName = lname!("input");
    /// Tag `<ins>`: inserted text.
    /// (meta: inline)
    pub const INS:           LocalName = lname!("ins");
    /// Tag `<isindex>`: searchable index related to a document.
    /// (meta: deprecated)
    pub const ISINDEX:       LocalName = lname!("isindex");
    /// Tag `<kbd>`: keyboard text.
    /// (meta: inline)
    pub const KBD:           LocalName = lname!("kbd");
    /// Tag `<label>`: label for input or other element.
    /// (meta: inline banned)
    pub const LABEL:         LocalName = lname!("label");
    /// Tag `<legend>`: caption for a fieldset element.
    /// (meta: banned)
    pub const LEGEND:        LocalName = lname!("legend");
    /// Tag `<li>`: list item.
    pub const LI:            LocalName = lname!("li");
    /// Tag `<link>`: relationship with an external resource.
    /// (meta: empty meta)
    pub const LINK:          LocalName = lname!("link");
    /// Tag `<listing>`: preformated text.
    /// (meta: deprecated)
    pub const LISTING:       LocalName = lname!("listing");
    /// Tag `<main>`: identify central topic/functional content.
    pub const MAIN:          LocalName = lname!("main");
    /// Tag `<map>`: image-map.
    /// (meta: inline)
    pub const MAP:           LocalName = lname!("map");
    /// Tag `<mark>`: Text marked/highlighted for reference purposes.
    /// (meta: inline)
    pub const MARK:          LocalName = lname!("mark");
    /// Tag `<menu>`: menu list.
    /// (meta: deprecated)
    pub const MENU:          LocalName = lname!("menu");
    /// Tag `<menuitem>`: a command in a menu.
    /// (meta: empty deprecated)
    pub const MENUITEM:      LocalName = lname!("menuitem");
    /// Tag `<meta>`: metadata.
    /// (meta: empty meta)
    pub const META:          LocalName = lname!("meta");
    /// Tag `<meter>`: a linear guage for a scaler value.
    /// (meta: inline)
    pub const METER:         LocalName = lname!("meter");
    /// Tag `<nav>`: Structure: container for navigational links.
    pub const NAV:           LocalName = lname!("nav");
    /// Tag `<nobr>`: contained text; white-space: nowrap.
    /// (meta: deprecated inline)
    pub const NOBR:          LocalName = lname!("nobr");
    /// Tag `<noframes>`: alternate content where frames not supported.
    /// (meta: deprecated banned)
    pub const NOFRAMES:      LocalName = lname!("noframes");
    /// Tag `<noscript>`: alternate content script not supported.
    /// (meta: inline banned)
    pub const NOSCRIPT:      LocalName = lname!("noscript");
    /// Tag `<object>`: embedded object.
    /// (meta: inline banned)
    pub const OBJECT:        LocalName = lname!("object");
    /// Tag `<ol>`: ordered list.
    pub const OL:            LocalName = lname!("ol");
    /// Tag `<optgroup>`: group of related options in a select list.
    /// (meta: banned)
    pub const OPTGROUP:      LocalName = lname!("optgroup");
    /// Tag `<option>`: option in a select list.
    /// (meta: banned)
    pub const OPTION:        LocalName = lname!("option");
    /// Tag `<output>`: content is (scripted) outcome of a user action..
    /// (meta: inline)
    pub const OUTPUT:        LocalName = lname!("output");
    /// Tag `<p>`: paragraph.
    pub const P:             LocalName = lname!("p");
    /// Tag `<param>`: parameter for an object.
    /// (meta: empty)
    pub const PARAM:         LocalName = lname!("param");
    /// Tag `<picture>`: container for multiple img/source DPI.
    /// (meta: inline)
    pub const PICTURE:       LocalName = lname!("picture");
    /// Tag `<plaintext>`: like xmp; no close tag.
    /// (meta: deprecated)
    pub const PLAINTEXT:     LocalName = lname!("plaintext");
    /// Tag `<pre>`: preformatted text.
    pub const PRE:           LocalName = lname!("pre");
    /// Tag `<progress>`: a progress bar.
    /// (meta: inline)
    pub const PROGRESS:      LocalName = lname!("progress");
    /// Tag `<q>`: short quotation.
    /// (meta: inline)
    pub const Q:             LocalName = lname!("q");
    /// Tag `<rb>`: ruby base text.
    pub const RB:            LocalName = lname!("rb");
    /// Tag `<rbc>`: ruby base container (complex).
    /// (meta: undefined)
    ///
    /// This is a lazy static (struct) as its not defined by html5ever.
    pub static RBC: LazyLock<LocalName> = LazyLock::new(|| "rbc".into());
    /// Tag `<rp>`: ruby simple text container.
    pub const RP:            LocalName = lname!("rp");
    /// Tag `<rt>`: ruby annotation text.
    pub const RT:            LocalName = lname!("rt");
    /// Tag `<rtc>`: ruby text container (complex).
    pub const RTC:           LocalName = lname!("rtc");
    /// Tag `<ruby>`: ruby pronunciation aid.
    /// (meta: inline)
    pub const RUBY:          LocalName = lname!("ruby");
    /// Tag `<s>`: strikethrough text.
    /// (meta: deprecated inline)
    pub const S:             LocalName = lname!("s");
    /// Tag `<samp>`: sample computer code.
    /// (meta: inline)
    pub const SAMP:          LocalName = lname!("samp");
    /// Tag `<script>`: client-side script.
    /// (meta: inline banned)
    pub const SCRIPT:        LocalName = lname!("script");
    /// Tag `<section>`: Structure: generic document/application section.
    pub const SECTION:       LocalName = lname!("section");
    /// Tag `<select>`: select list (drop-down list).
    /// (meta: inline banned)
    pub const SELECT:        LocalName = lname!("select");
    /// Tag `<slot>`: (Shadow) DOM placeholder element.
    /// (meta: inline banned)
    pub const SLOT:          LocalName = lname!("slot");
    /// Tag `<small>`: small text.
    /// (meta: inline)
    pub const SMALL:         LocalName = lname!("small");
    /// Tag `<source>`: source for picture/audio/video elements.
    /// (meta: empty)
    pub const SOURCE:        LocalName = lname!("source");
    /// Tag `<span>`: section in a document.
    /// (meta: inline)
    pub const SPAN:          LocalName = lname!("span");
    /// Tag `<strike>`: strikethrough text.
    /// (meta: deprecated inline)
    pub const STRIKE:        LocalName = lname!("strike");
    /// Tag `<strong>`: strong text.
    /// (meta: inline)
    pub const STRONG:        LocalName = lname!("strong");
    /// Tag `<style>`: style information for a document.
    /// (meta: banned)
    pub const STYLE:         LocalName = lname!("style");
    /// Tag `<sub>`: subscripted text.
    /// (meta: inline)
    pub const SUB:           LocalName = lname!("sub");
    /// Tag `<summary>`: summary of details element.
    pub const SUMMARY:       LocalName = lname!("summary");
    /// Tag `<sup>`: superscripted text.
    /// (meta: inline)
    pub const SUP:           LocalName = lname!("sup");
    /// Tag `<svg>`: inline scalable vector graphics.
    pub const SVG:           LocalName = lname!("svg");
    /// Tag `<table>`: table.
    pub const TABLE:         LocalName = lname!("table");
    /// Tag `<tbody>`: Groups the body content in a table.
    pub const TBODY:         LocalName = lname!("tbody");
    /// Tag `<td>`: cell in a table.
    pub const TD:            LocalName = lname!("td");
    /// Tag `<template>`: html sub-tree notrenderered except by script.
    /// (meta: banned)
    pub const TEMPLATE:      LocalName = lname!("template");
    /// Tag `<textarea>`: multi-line text input control.
    /// (meta: inline banned)
    pub const TEXTAREA:      LocalName = lname!("textarea");
    /// Tag `<tfoot>`: Groups the footer content in a table.
    pub const TFOOT:         LocalName = lname!("tfoot");
    /// Tag `<th>`: header cell in a table.
    pub const TH:            LocalName = lname!("th");
    /// Tag `<thead>`: Groups the header content in a table.
    pub const THEAD:         LocalName = lname!("thead");
    /// Tag `<time>`: A date or time.
    /// (meta: inline)
    pub const TIME:          LocalName = lname!("time");
    /// Tag `<title>`: the title of a document.
    /// (meta: meta)
    pub const TITLE:         LocalName = lname!("title");
    /// Tag `<tr>`: row in a table.
    pub const TR:            LocalName = lname!("tr");
    /// Tag `<tt>`: teletype text.
    /// (meta: deprecated inline)
    pub const TT:            LocalName = lname!("tt");
    /// Tag `<u>`: underlined text.
    /// (meta: deprecated inline)
    pub const U:             LocalName = lname!("u");
    /// Tag `<ul>`: unordered list.
    pub const UL:            LocalName = lname!("ul");
    /// Tag `<var>`: variable part of a text.
    /// (meta: inline)
    pub const VAR:           LocalName = lname!("var");
    /// Tag `<video>`: video container.
    /// (meta: inline)
    pub const VIDEO:         LocalName = lname!("video");
    /// Tag `<wbr>`: A line break opportunity.
    /// (meta: empty inline)
    pub const WBR:           LocalName = lname!("wbr");
    /// Tag `<xmp>`: preformatted text.
    /// (meta: deprecated)
    pub const XMP:           LocalName = lname!("xmp");
}

/// HTML attribute constants
pub mod a {
    use html5ever::local_name as lname;
    use crate::dom::LocalName;
    use std::sync::LazyLock;

    pub const ABBR:              LocalName = lname!("abbr");
    /// Attribute accept: (file) types accepted.
    pub const ACCEPT:            LocalName = lname!("accept");
    pub const ACCEPT_CHARSET:    LocalName = lname!("accept-charset");
    pub const ALIGN:             LocalName = lname!("align");
    pub const ALT:               LocalName = lname!("alt");
    pub const AXIS:              LocalName = lname!("axis");
    /// Attribute base: inherited from xml:base (deprecated).
    pub const BASE:              LocalName = lname!("base");
    pub const BGCOLOR:           LocalName = lname!("bgcolor");
    pub const BORDER:            LocalName = lname!("border");
    pub const CELLPADDING:       LocalName = lname!("cellpadding");
    pub const CELLSPACING:       LocalName = lname!("cellspacing");
    pub const CHAR:              LocalName = lname!("char");
    pub const CHAROFF:           LocalName = lname!("charoff");
    /// Attribute charset: encoding of link or (meta) document.
    pub const CHARSET:           LocalName = lname!("charset");
    pub const CITE:              LocalName = lname!("cite");
    pub const CLASS:             LocalName = lname!("class");
    pub const COLOR:             LocalName = lname!("color");
    pub const COLSPAN:           LocalName = lname!("colspan");
    /// Attribute content: text.
    pub const CONTENT:           LocalName = lname!("content");
    pub const CONTROLS:          LocalName = lname!("controls");
    /// Attribute coords: coordinates; i.e. image map.
    pub const COORDS:            LocalName = lname!("coords");
    pub const DATA:              LocalName = lname!("data");
    pub const DATETIME:          LocalName = lname!("datetime");
    /// Attribute decoding: preferred method to decode.
    ///
    /// This is a lazy static (struct) as its not defined by html5ever.
    pub static DECODING: LazyLock<LocalName> = LazyLock::new(|| "decoding".into());
    /// Attribute dir: Text direction; ltr or rtl.
    pub const DIR:               LocalName = lname!("dir");
    pub const FRAME:             LocalName = lname!("frame");
    pub const HEADERS:           LocalName = lname!("headers");
    pub const HEIGHT:            LocalName = lname!("height");
    /// Attribute hidden: hidden element.
    pub const HIDDEN:            LocalName = lname!("hidden");
    /// Attribute href: URL.
    pub const HREF:              LocalName = lname!("href");
    /// Attribute hreflang: language_code of referent.
    pub const HREFLANG:          LocalName = lname!("hreflang");
    /// Attribute http-equiv: HTTP Header name.
    pub const HTTP_EQUIV:        LocalName = lname!("http-equiv");
    pub const ID:                LocalName = lname!("id");
    pub const LABEL:             LocalName = lname!("label");
    /// Attribute lang: language_code; also xml:lang.
    pub const LANG:              LocalName = lname!("lang");
    pub const MEDIA:             LocalName = lname!("media");
    /// Attribute name: section_name anchor.
    pub const NAME:              LocalName = lname!("name");
    pub const NOWRAP:            LocalName = lname!("nowrap");
    pub const REL:               LocalName = lname!("rel");
    pub const REV:               LocalName = lname!("rev");
    pub const ROWSPAN:           LocalName = lname!("rowspan");
    pub const RULES:             LocalName = lname!("rules");
    /// Attribute scheme: format URI.
    pub const SCHEME:            LocalName = lname!("scheme");
    pub const SCOPE:             LocalName = lname!("scope");
    pub const SHAPE:             LocalName = lname!("shape");
    pub const SPAN:              LocalName = lname!("span");
    pub const SRC:               LocalName = lname!("src");
    pub const STYLE:             LocalName = lname!("style");
    pub const SUMMARY:           LocalName = lname!("summary");
    pub const TARGET:            LocalName = lname!("target");
    /// Attribute title: extra title.
    pub const TITLE:             LocalName = lname!("title");
    pub const TYPE:              LocalName = lname!("type");
    pub const VALIGN:            LocalName = lname!("valign");
    pub const VALUE:             LocalName = lname!("value");
    pub const WIDTH:             LocalName = lname!("width");
}

fn init_tag_metadata() -> HashMap<LocalName, TagMeta> {
    let mut tag_meta = HashMap::with_capacity(135);

    tag_meta.insert(t::A, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::CHARSET, a::DIR, a::HREF, a::HREFLANG, a::ID, a::LANG, a::MEDIA, a::NAME, a::REL, a::REV, a::TITLE, a::TYPE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::ABBR, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::ACRONYM, TagMeta {
        is_deprecated: true,
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::ADDRESS, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::APPLET, TagMeta {
        is_deprecated: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::AREA, TagMeta {
        is_empty: true,
        basic_attrs: vec![
            a::ALT, a::BASE, a::DIR, a::LANG, a::MEDIA, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::ARTICLE, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::ASIDE, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::AUDIO, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::SRC, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::B, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::BASE, TagMeta {
        is_empty: true,
        is_meta: true,
        basic_attrs: vec![
            a::BASE, a::HREF
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::BASEFONT, TagMeta {
        is_empty: true,
        is_deprecated: true,
        is_inline: true,
        is_meta: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::BDI, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::BDO, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::BIG, TagMeta {
        is_deprecated: true,
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::BLINK, TagMeta {
        is_deprecated: true,
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::BLOCKQUOTE, TagMeta {
        basic_attrs: vec![
            a::BASE, a::CITE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::BODY, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::BR, TagMeta {
        is_empty: true,
        basic_attrs: vec![
            a::BASE, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::BUTTON, TagMeta {
        is_inline: true,
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::CANVAS, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::CAPTION, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::CENTER, TagMeta {
        is_deprecated: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::CITE, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::CODE, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::COL, TagMeta {
        is_empty: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::SPAN, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::COLGROUP, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::SPAN, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::CONTENT, TagMeta {
        is_deprecated: true,
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::DATA, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE, a::VALUE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::DATALIST, TagMeta {
        is_inline: true,
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::DD, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::DEL, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::CITE, a::DATETIME, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::DETAILS, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::DFN, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::DIALOG, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::DIR, TagMeta {
        is_deprecated: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::DIV, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::DL, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::DT, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::EM, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::EMBED, TagMeta {
        is_empty: true,
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::HEIGHT, a::LANG, a::SRC, a::TITLE, a::TYPE, a::WIDTH
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::FIELDSET, TagMeta {
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::FIGCAPTION, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::FIGURE, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::FONT, TagMeta {
        is_deprecated: true,
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::FOOTER, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::FORM, TagMeta {
        basic_attrs: vec![
            a::ACCEPT, a::ACCEPT_CHARSET, a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::FRAME, TagMeta {
        is_empty: true,
        is_deprecated: true,
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::SRC, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::FRAMESET, TagMeta {
        is_deprecated: true,
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::H1, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::H2, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::H3, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::H4, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::H5, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::H6, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::HEAD, TagMeta {
        is_meta: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::HEADER, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::HGROUP, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::HR, TagMeta {
        is_empty: true,
        basic_attrs: vec![
            a::BASE, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::HTML, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::I, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::IFRAME, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::ALIGN, a::BASE, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::IMG, TagMeta {
        is_empty: true,
        is_inline: true,
        basic_attrs: vec![
            a::ALT, a::BASE, a::DECODING.clone(), a::DIR, a::HEIGHT, a::LANG, a::SRC, a::TITLE, a::WIDTH
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::INPUT, TagMeta {
        is_empty: true,
        is_inline: true,
        is_banned: true,
        basic_attrs: vec![
            a::ACCEPT, a::ALT, a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::INS, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::CITE, a::DATETIME, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::ISINDEX, TagMeta {
        is_deprecated: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::KBD, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::LABEL, TagMeta {
        is_inline: true,
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::LEGEND, TagMeta {
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::LI, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::LINK, TagMeta {
        is_empty: true,
        is_meta: true,
        basic_attrs: vec![
            a::BASE, a::CHARSET, a::DIR, a::HREF, a::HREFLANG, a::LANG, a::MEDIA, a::REL, a::REV, a::TITLE, a::TYPE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::LISTING, TagMeta {
        is_deprecated: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::MAIN, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::MAP, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::MARK, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::MENU, TagMeta {
        is_deprecated: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::MENUITEM, TagMeta {
        is_empty: true,
        is_deprecated: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::META, TagMeta {
        is_empty: true,
        is_meta: true,
        basic_attrs: vec![
            a::BASE, a::CHARSET, a::CONTENT, a::DIR, a::HTTP_EQUIV, a::LANG, a::SCHEME
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::METER, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::NAV, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::NOBR, TagMeta {
        is_deprecated: true,
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::NOFRAMES, TagMeta {
        is_deprecated: true,
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::NOSCRIPT, TagMeta {
        is_inline: true,
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::OBJECT, TagMeta {
        is_inline: true,
        is_banned: true,
        basic_attrs: vec![
            a::ALIGN, a::BASE, a::DATA, a::DIR, a::LANG, a::TITLE, a::TYPE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::OL, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::OPTGROUP, TagMeta {
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LABEL, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::OPTION, TagMeta {
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LABEL, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::OUTPUT, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::P, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::PARAM, TagMeta {
        is_empty: true,
        basic_attrs: vec![
            a::BASE, a::NAME, a::VALUE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::PICTURE, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::HEIGHT, a::LANG, a::TITLE, a::WIDTH
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::PLAINTEXT, TagMeta {
        is_deprecated: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::PRE, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::PROGRESS, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::Q, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::CITE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::RB, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::RBC.clone(), TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::RP, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::RT, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::RTC, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::RUBY, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::S, TagMeta {
        is_deprecated: true,
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::SAMP, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::SCRIPT, TagMeta {
        is_inline: true,
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::SECTION, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::SELECT, TagMeta {
        is_inline: true,
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::SLOT, TagMeta {
        is_inline: true,
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::SMALL, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::SOURCE, TagMeta {
        is_empty: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::SRC, a::TITLE, a::TYPE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::SPAN, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::STRIKE, TagMeta {
        is_deprecated: true,
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::STRONG, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::STYLE, TagMeta {
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::SUB, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::SUMMARY, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::SUP, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::SVG, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::HEIGHT, a::LANG, a::TITLE, a::WIDTH
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::TABLE, TagMeta {
        basic_attrs: vec![
            a::ALIGN, a::BASE, a::DIR, a::LANG, a::SUMMARY, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::TBODY, TagMeta {
        basic_attrs: vec![
            a::ALIGN, a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::TD, TagMeta {
        basic_attrs: vec![
            a::ALIGN, a::BASE, a::COLSPAN, a::DIR, a::HEADERS, a::LANG, a::ROWSPAN, a::SCOPE, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::TEMPLATE, TagMeta {
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::TEXTAREA, TagMeta {
        is_inline: true,
        is_banned: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::TFOOT, TagMeta {
        basic_attrs: vec![
            a::ALIGN, a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::TH, TagMeta {
        basic_attrs: vec![
            a::ABBR, a::ALIGN, a::AXIS, a::BASE, a::COLSPAN, a::DIR, a::LANG, a::ROWSPAN, a::SCOPE, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::THEAD, TagMeta {
        basic_attrs: vec![
            a::ALIGN, a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::TIME, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DATETIME, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::TITLE, TagMeta {
        is_meta: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::TR, TagMeta {
        basic_attrs: vec![
            a::ABBR, a::ALIGN, a::AXIS, a::BASE, a::COLSPAN, a::DIR, a::HEADERS, a::LANG, a::ROWSPAN, a::SCOPE, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::TT, TagMeta {
        is_deprecated: true,
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::U, TagMeta {
        is_deprecated: true,
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::UL, TagMeta {
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::VAR, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::VIDEO, TagMeta {
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::HEIGHT, a::LANG, a::TITLE, a::WIDTH
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::WBR, TagMeta {
        is_empty: true,
        is_inline: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });
    tag_meta.insert(t::XMP, TagMeta {
        is_deprecated: true,
        basic_attrs: vec![
            a::BASE, a::DIR, a::LANG, a::TITLE
        ],
        .. TagMeta::default()
    });

    tag_meta
}
