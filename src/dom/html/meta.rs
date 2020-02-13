//! Static metadata for HTML elements and attributes.
//!
//! This file is generated via build/generate.rb and the build/meta.rs.erb
//! template. It should not be manually edited. To avoid any rust build-time
//! dependency however, the resulting source file (src/dom/html/meta.rs) is
//! also checked in.

use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::dom::LocalName;

lazy_static! {
    /// A static lookup table for metadata on known HTML tags.
    pub static ref TAG_META: HashMap<LocalName, TagMeta> = init_tag_metadata();
}

/// Metadata about HTML tags and their attributes.
pub struct TagMeta {
    is_empty: bool,
    is_deprecated: bool,
    is_inline: bool,
    is_meta: bool,
    is_banned: bool,
    basic_attrs: Vec<LocalName>,
}

impl TagMeta {
    /// If the element is defined to be empty: having no contents or end tag.
    pub fn is_empty(&self) -> bool {
        self.is_empty
    }

    /// If the tag is deprecated as of html5
    pub fn is_deprecated(&self) -> bool {
        self.is_deprecated
    }

    /// If the element is _inline_: not a block producing element under normal
    /// usage. Note that `<br>` is not considered inline.
    pub fn is_inline(&self) -> bool {
        self.is_inline
    }

    /// If the element represents metadata only, where any content is not
    /// displayed text. e.g. `<head>`.
    pub fn is_meta(&self) -> bool {
        self.is_meta
    }

    /// If the element is banned/blacklisted, where any content is not
    /// extracted, displayed, or otherwise used.
    pub fn is_banned(&self) -> bool {
        self.is_banned
    }

    /// Return true if the given name is part of the _basic_ set of known
    /// attributes for this element.
    ///
    /// This _basic set_ of attributes excludes those that are used exclusively
    /// for styling purposes.
    pub fn has_basic_attr(&self, name: &LocalName) -> bool {
        self.basic_attrs.binary_search(name).is_ok()
    }
}

impl Default for TagMeta {
    fn default() -> TagMeta {
        TagMeta {
            is_empty: false,
            is_deprecated: false,
            is_inline: false,
            is_meta: false,
            is_banned: false,
            basic_attrs: vec![],
        }
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

    /// Tag `<a>`: anchor.
    pub const A:             LocalName = lname!("a");
    /// Tag `<abbr>`: abbreviation.
    pub const ABBR:          LocalName = lname!("abbr");
    /// Tag `<acronym>`: acronym.
    pub const ACRONYM:       LocalName = lname!("acronym");
    /// Tag `<address>`: contact information for the author or owner.
    pub const ADDRESS:       LocalName = lname!("address");
    /// Tag `<applet>`: embedded applet.
    pub const APPLET:        LocalName = lname!("applet");
    /// Tag `<area>`: area inside an image-map.
    pub const AREA:          LocalName = lname!("area");
    /// Tag `<article>`: Structure: an independent content element.
    pub const ARTICLE:       LocalName = lname!("article");
    /// Tag `<aside>`: Structure: tengentially related content.
    pub const ASIDE:         LocalName = lname!("aside");
    /// Tag `<audio>`: Sound content.
    pub const AUDIO:         LocalName = lname!("audio");
    /// Tag `<b>`: bold text.
    pub const B:             LocalName = lname!("b");
    /// Tag `<base>`: default address or target for all links on a page.
    pub const BASE:          LocalName = lname!("base");
    /// Tag `<basefont>`: default font; color; or size for the text in a page.
    pub const BASEFONT:      LocalName = lname!("basefont");
    /// Tag `<bdi>`: Text isolated from surrounding for BIDI formatting.
    pub const BDI:           LocalName = lname!("bdi");
    /// Tag `<bdo>`: the text direction.
    pub const BDO:           LocalName = lname!("bdo");
    /// Tag `<big>`: big text.
    pub const BIG:           LocalName = lname!("big");
    /// Tag `<blink>`: blinking text.
    pub const BLINK:         LocalName = lname!("blink");
    /// Tag `<blockquote>`: long quotation.
    pub const BLOCKQUOTE:    LocalName = lname!("blockquote");
    /// Tag `<body>`: the document's body.
    pub const BODY:          LocalName = lname!("body");
    /// Tag `<br>`: single line break.
    pub const BR:            LocalName = lname!("br");
    /// Tag `<button>`: push button.
    pub const BUTTON:        LocalName = lname!("button");
    /// Tag `<canvas>`: canvas for drawing graphics and animations.
    pub const CANVAS:        LocalName = lname!("canvas");
    /// Tag `<caption>`: table caption.
    pub const CAPTION:       LocalName = lname!("caption");
    /// Tag `<center>`: centered text.
    pub const CENTER:        LocalName = lname!("center");
    /// Tag `<cite>`: citation.
    pub const CITE:          LocalName = lname!("cite");
    /// Tag `<code>`: computer code text.
    pub const CODE:          LocalName = lname!("code");
    /// Tag `<col>`: attribute values for one or more columns in a table.
    pub const COL:           LocalName = lname!("col");
    /// Tag `<colgroup>`: group of columns in a table for formatting.
    pub const COLGROUP:      LocalName = lname!("colgroup");
    /// Tag `<content>`: Shadow DOM content placeholder element.
    pub const CONTENT:       LocalName = lname!("content");
    /// Tag `<data>`: adds machine-oriented data representation.
    pub const DATA:          LocalName = lname!("data");
    /// Tag `<datalist>`: container for option elements.
    pub const DATALIST:      LocalName = lname!("datalist");
    /// Tag `<dd>`: description of a term in a definition list.
    pub const DD:            LocalName = lname!("dd");
    /// Tag `<del>`: deleted text.
    pub const DEL:           LocalName = lname!("del");
    /// Tag `<details>`: optional additional details (also: summary).
    pub const DETAILS:       LocalName = lname!("details");
    /// Tag `<dfn>`: definition term.
    pub const DFN:           LocalName = lname!("dfn");
    /// Tag `<dialog>`: dialog box or other interactive component.
    pub const DIALOG:        LocalName = lname!("dialog");
    /// Tag `<dir>`: directory list.
    pub const DIR:           LocalName = lname!("dir");
    /// Tag `<div>`: section in a document.
    pub const DIV:           LocalName = lname!("div");
    /// Tag `<dl>`: definition list.
    pub const DL:            LocalName = lname!("dl");
    /// Tag `<dt>`: term (an item) in a definition list.
    pub const DT:            LocalName = lname!("dt");
    /// Tag `<em>`: emphasized text.
    pub const EM:            LocalName = lname!("em");
    /// Tag `<embed>`: embed content by external app or plug-in.
    pub const EMBED:         LocalName = lname!("embed");
    /// Tag `<fieldset>`: border around elements in a form.
    pub const FIELDSET:      LocalName = lname!("fieldset");
    /// Tag `<figcaption>`: Structure: a figure caption.
    pub const FIGCAPTION:    LocalName = lname!("figcaption");
    /// Tag `<figure>`: Structure: self contained content that can be moved.
    pub const FIGURE:        LocalName = lname!("figure");
    /// Tag `<font>`: font; color; or size for text.
    pub const FONT:          LocalName = lname!("font");
    /// Tag `<footer>`: Structure: a footer of a section.
    pub const FOOTER:        LocalName = lname!("footer");
    /// Tag `<form>`: form for user input.
    pub const FORM:          LocalName = lname!("form");
    /// Tag `<frame>`: window (a frame) in a frameset.
    pub const FRAME:         LocalName = lname!("frame");
    /// Tag `<frameset>`: set of frames.
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
    pub const HEAD:          LocalName = lname!("head");
    /// Tag `<header>`: Structure: a header of a section.
    pub const HEADER:        LocalName = lname!("header");
    /// Tag `<hgroup>`: Structure: a group of headings.
    pub const HGROUP:        LocalName = lname!("hgroup");
    /// Tag `<hr>`: horizontal line.
    pub const HR:            LocalName = lname!("hr");
    /// Tag `<html>`: document.
    pub const HTML:          LocalName = lname!("html");
    /// Tag `<i>`: italic text.
    pub const I:             LocalName = lname!("i");
    /// Tag `<iframe>`: inline frame.
    pub const IFRAME:        LocalName = lname!("iframe");
    /// Tag `<img>`: image.
    pub const IMG:           LocalName = lname!("img");
    /// Tag `<input>`: input control.
    pub const INPUT:         LocalName = lname!("input");
    /// Tag `<ins>`: inserted text.
    pub const INS:           LocalName = lname!("ins");
    /// Tag `<isindex>`: searchable index related to a document.
    pub const ISINDEX:       LocalName = lname!("isindex");
    /// Tag `<kbd>`: keyboard text.
    pub const KBD:           LocalName = lname!("kbd");
    /// Tag `<label>`: label for input or other element.
    pub const LABEL:         LocalName = lname!("label");
    /// Tag `<legend>`: caption for a fieldset element.
    pub const LEGEND:        LocalName = lname!("legend");
    /// Tag `<li>`: list item.
    pub const LI:            LocalName = lname!("li");
    /// Tag `<listing>`: preformated text.
    pub const LISTING:       LocalName = lname!("listing");
    /// Tag `<link>`: relationship with an external resource.
    pub const LINK:          LocalName = lname!("link");
    /// Tag `<main>`: identify central topic/functional content.
    pub const MAIN:          LocalName = lname!("main");
    /// Tag `<map>`: image-map.
    pub const MAP:           LocalName = lname!("map");
    /// Tag `<mark>`: Text marked/highlighted for reference purposes.
    pub const MARK:          LocalName = lname!("mark");
    /// Tag `<menu>`: menu list.
    pub const MENU:          LocalName = lname!("menu");
    /// Tag `<menuitem>`: a command in a menu.
    pub const MENUITEM:      LocalName = lname!("menuitem");
    /// Tag `<meta>`: metadata.
    pub const META:          LocalName = lname!("meta");
    /// Tag `<meter>`: a linear guage for a scaler value.
    pub const METER:         LocalName = lname!("meter");
    /// Tag `<nav>`: Structure: container for navigational links.
    pub const NAV:           LocalName = lname!("nav");
    /// Tag `<nobr>`: contained text; white-space: nowrap.
    pub const NOBR:          LocalName = lname!("nobr");
    /// Tag `<noframes>`: alternate content where frames not supported.
    pub const NOFRAMES:      LocalName = lname!("noframes");
    /// Tag `<noscript>`: alternate content script not supported.
    pub const NOSCRIPT:      LocalName = lname!("noscript");
    /// Tag `<object>`: embedded object.
    pub const OBJECT:        LocalName = lname!("object");
    /// Tag `<ol>`: ordered list.
    pub const OL:            LocalName = lname!("ol");
    /// Tag `<optgroup>`: group of related options in a select list.
    pub const OPTGROUP:      LocalName = lname!("optgroup");
    /// Tag `<option>`: option in a select list.
    pub const OPTION:        LocalName = lname!("option");
    /// Tag `<output>`: content is (scripted) outcome of a user action..
    pub const OUTPUT:        LocalName = lname!("output");
    /// Tag `<p>`: paragraph.
    pub const P:             LocalName = lname!("p");
    /// Tag `<param>`: parameter for an object.
    pub const PARAM:         LocalName = lname!("param");
    /// Tag `<picture>`: container for multiple img/source DPI.
    pub const PICTURE:       LocalName = lname!("picture");
    /// Tag `<plaintext>`: like xmp; no close tag.
    pub const PLAINTEXT:     LocalName = lname!("plaintext");
    /// Tag `<pre>`: preformatted text.
    pub const PRE:           LocalName = lname!("pre");
    /// Tag `<progress>`: a progress bar.
    pub const PROGRESS:      LocalName = lname!("progress");
    /// Tag `<q>`: short quotation.
    pub const Q:             LocalName = lname!("q");
    /// Tag `<rb>`: ruby base text.
    pub const RB:            LocalName = lname!("rb");
    lazy_static::lazy_static! {
        /// Tag `<rbc>`: ruby base container (complex).
        ///
        /// This is a lazy static (struct) as its not defined by html5ever.
        pub static ref RBC: LocalName = "rbc".into();
    }
    /// Tag `<rp>`: ruby simple text container.
    pub const RP:            LocalName = lname!("rp");
    /// Tag `<rt>`: ruby annotation text.
    pub const RT:            LocalName = lname!("rt");
    /// Tag `<rtc>`: ruby text container (complex).
    pub const RTC:           LocalName = lname!("rtc");
    /// Tag `<ruby>`: ruby pronunciation aid.
    pub const RUBY:          LocalName = lname!("ruby");
    /// Tag `<s>`: strikethrough text.
    pub const S:             LocalName = lname!("s");
    /// Tag `<samp>`: sample computer code.
    pub const SAMP:          LocalName = lname!("samp");
    /// Tag `<script>`: client-side script.
    pub const SCRIPT:        LocalName = lname!("script");
    /// Tag `<section>`: Structure: generic document/application section.
    pub const SECTION:       LocalName = lname!("section");
    /// Tag `<select>`: select list (drop-down list).
    pub const SELECT:        LocalName = lname!("select");
    /// Tag `<slot>`: (Shadow) DOM placeholder element.
    pub const SLOT:          LocalName = lname!("slot");
    /// Tag `<small>`: small text.
    pub const SMALL:         LocalName = lname!("small");
    /// Tag `<source>`: source for picture/audio/video elements.
    pub const SOURCE:        LocalName = lname!("source");
    /// Tag `<span>`: section in a document.
    pub const SPAN:          LocalName = lname!("span");
    /// Tag `<strike>`: strikethrough text.
    pub const STRIKE:        LocalName = lname!("strike");
    /// Tag `<strong>`: strong text.
    pub const STRONG:        LocalName = lname!("strong");
    /// Tag `<style>`: style information for a document.
    pub const STYLE:         LocalName = lname!("style");
    /// Tag `<sub>`: subscripted text.
    pub const SUB:           LocalName = lname!("sub");
    /// Tag `<summary>`: summary of details element.
    pub const SUMMARY:       LocalName = lname!("summary");
    /// Tag `<sup>`: superscripted text.
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
    pub const TEMPLATE:      LocalName = lname!("template");
    /// Tag `<textarea>`: multi-line text input control.
    pub const TEXTAREA:      LocalName = lname!("textarea");
    /// Tag `<tfoot>`: Groups the footer content in a table.
    pub const TFOOT:         LocalName = lname!("tfoot");
    /// Tag `<th>`: header cell in a table.
    pub const TH:            LocalName = lname!("th");
    /// Tag `<thead>`: Groups the header content in a table.
    pub const THEAD:         LocalName = lname!("thead");
    /// Tag `<time>`: A date or time.
    pub const TIME:          LocalName = lname!("time");
    /// Tag `<title>`: the title of a document.
    pub const TITLE:         LocalName = lname!("title");
    /// Tag `<tr>`: row in a table.
    pub const TR:            LocalName = lname!("tr");
    /// Tag `<tt>`: teletype text.
    pub const TT:            LocalName = lname!("tt");
    /// Tag `<u>`: underlined text.
    pub const U:             LocalName = lname!("u");
    /// Tag `<ul>`: unordered list.
    pub const UL:            LocalName = lname!("ul");
    /// Tag `<var>`: variable part of a text.
    pub const VAR:           LocalName = lname!("var");
    /// Tag `<video>`: video container.
    pub const VIDEO:         LocalName = lname!("video");
    /// Tag `<wbr>`: A line break opportunity.
    pub const WBR:           LocalName = lname!("wbr");
    /// Tag `<xmp>`: preformatted text.
    pub const XMP:           LocalName = lname!("xmp");
}

/// HTML attribute constants
pub mod a {
    use html5ever::local_name as lname;
    use crate::dom::LocalName;

    pub const CLASS:             LocalName = lname!("class");
    pub const ID:                LocalName = lname!("id");
    pub const STYLE:             LocalName = lname!("style");
    /// Attribute hidden: hidden element.
    pub const HIDDEN:            LocalName = lname!("hidden");
    /// Attribute title: extra title.
    pub const TITLE:             LocalName = lname!("title");
    /// Attribute dir: Text direction; ltr or rtl.
    pub const DIR:               LocalName = lname!("dir");
    /// Attribute lang: language_code; also xml:lang.
    pub const LANG:              LocalName = lname!("lang");
    /// Attribute base: inherited from xml:base (deprecated).
    pub const BASE:              LocalName = lname!("base");
    /// Attribute http-equiv: HTTP Header name.
    pub const HTTP_EQUIV:        LocalName = lname!("http-equiv");
    /// Attribute content: text.
    pub const CONTENT:           LocalName = lname!("content");
    /// Attribute scheme: format URI.
    pub const SCHEME:            LocalName = lname!("scheme");
    /// Attribute charset: encoding of link or (meta) document.
    pub const CHARSET:           LocalName = lname!("charset");
    /// Attribute coords: coordinates; i.e. image map.
    pub const COORDS:            LocalName = lname!("coords");
    /// Attribute hreflang: language_code of referent.
    pub const HREFLANG:          LocalName = lname!("hreflang");
    /// Attribute href: URL.
    pub const HREF:              LocalName = lname!("href");
    pub const MEDIA:             LocalName = lname!("media");
    /// Attribute name: section_name anchor.
    pub const NAME:              LocalName = lname!("name");
    pub const REL:               LocalName = lname!("rel");
    pub const REV:               LocalName = lname!("rev");
    pub const SHAPE:             LocalName = lname!("shape");
    pub const TARGET:            LocalName = lname!("target");
    pub const TYPE:              LocalName = lname!("type");
    pub const SRC:               LocalName = lname!("src");
    pub const DATA:              LocalName = lname!("data");
    pub const ALT:               LocalName = lname!("alt");
    pub const HEIGHT:            LocalName = lname!("height");
    pub const WIDTH:             LocalName = lname!("width");
    lazy_static::lazy_static! {
        /// Attribute decoding: preferred method to decode.
        ///
        /// This is a lazy static (struct) as its not defined by html5ever.
        pub static ref DECODING: LocalName = "decoding".into();
    }
    pub const ABBR:              LocalName = lname!("abbr");
    pub const ALIGN:             LocalName = lname!("align");
    pub const AXIS:              LocalName = lname!("axis");
    pub const BGCOLOR:           LocalName = lname!("bgcolor");
    pub const BORDER:            LocalName = lname!("border");
    pub const CELLPADDING:       LocalName = lname!("cellpadding");
    pub const CELLSPACING:       LocalName = lname!("cellspacing");
    pub const CHAR:              LocalName = lname!("char");
    pub const CHAROFF:           LocalName = lname!("charoff");
    pub const COLSPAN:           LocalName = lname!("colspan");
    pub const FRAME:             LocalName = lname!("frame");
    pub const HEADERS:           LocalName = lname!("headers");
    pub const NOWRAP:            LocalName = lname!("nowrap");
    pub const ROWSPAN:           LocalName = lname!("rowspan");
    pub const RULES:             LocalName = lname!("rules");
    pub const SCOPE:             LocalName = lname!("scope");
    pub const SPAN:              LocalName = lname!("span");
    pub const SUMMARY:           LocalName = lname!("summary");
    pub const VALIGN:            LocalName = lname!("valign");
    pub const VALUE:             LocalName = lname!("value");
    /// Attribute accept: (file) types accepted.
    pub const ACCEPT:            LocalName = lname!("accept");
    pub const ACCEPT_CHARSET:    LocalName = lname!("accept-charset");
    pub const CITE:              LocalName = lname!("cite");
    pub const COLOR:             LocalName = lname!("color");
    pub const CONTROLS:          LocalName = lname!("controls");
    pub const DATETIME:          LocalName = lname!("datetime");
    pub const LABEL:             LocalName = lname!("label");
}

fn init_tag_metadata() -> HashMap<LocalName, TagMeta> {
    let mut tag_meta = HashMap::with_capacity(135);

    {
        let mut basic_attrs = vec![
            a::ID, a::TITLE, a::DIR, a::LANG, a::BASE, a::CHARSET, a::HREFLANG, a::HREF, a::MEDIA, a::NAME, a::REL, a::REV, a::TYPE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::A, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::ABBR, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::ACRONYM, TagMeta {
            is_deprecated: true,
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::ADDRESS, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::APPLET, TagMeta {
            is_deprecated: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::MEDIA, a::ALT
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::AREA, TagMeta {
            is_empty: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::ARTICLE, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::ASIDE, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::SRC
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::AUDIO, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::B, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::BASE, a::HREF
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::BASE, TagMeta {
            is_empty: true,
            is_meta: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::BASEFONT, TagMeta {
            is_empty: true,
            is_deprecated: true,
            is_inline: true,
            is_meta: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::BDI, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::BDO, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::BIG, TagMeta {
            is_deprecated: true,
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::BLINK, TagMeta {
            is_deprecated: true,
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::CITE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::BLOCKQUOTE, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::BODY, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::BR, TagMeta {
            is_empty: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::BUTTON, TagMeta {
            is_inline: true,
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::CANVAS, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::CAPTION, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::CENTER, TagMeta {
            is_deprecated: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::CITE, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::CODE, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::SPAN
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::COL, TagMeta {
            is_empty: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::SPAN
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::COLGROUP, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::CONTENT, TagMeta {
            is_deprecated: true,
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::VALUE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::DATA, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::DATALIST, TagMeta {
            is_inline: true,
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::DD, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::CITE, a::DATETIME
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::DEL, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::DETAILS, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::DFN, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::DIALOG, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::DIR, TagMeta {
            is_deprecated: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::DIV, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::DL, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::DT, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::EM, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::TYPE, a::SRC, a::HEIGHT, a::WIDTH
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::EMBED, TagMeta {
            is_empty: true,
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::FIELDSET, TagMeta {
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::FIGCAPTION, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::FIGURE, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::FONT, TagMeta {
            is_deprecated: true,
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::FOOTER, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::ACCEPT, a::ACCEPT_CHARSET
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::FORM, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::BASE, a::SRC
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::FRAME, TagMeta {
            is_empty: true,
            is_deprecated: true,
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::FRAMESET, TagMeta {
            is_deprecated: true,
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::H1, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::H2, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::H3, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::H4, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::H5, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::H6, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::HEAD, TagMeta {
            is_meta: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::HEADER, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::HGROUP, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::HR, TagMeta {
            is_empty: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::HTML, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::I, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::BASE, a::ALIGN
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::IFRAME, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::SRC, a::ALT, a::HEIGHT, a::WIDTH, a::DECODING.clone()
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::IMG, TagMeta {
            is_empty: true,
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::ALT, a::ACCEPT
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::INPUT, TagMeta {
            is_empty: true,
            is_inline: true,
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::CITE, a::DATETIME
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::INS, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::ISINDEX, TagMeta {
            is_deprecated: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::KBD, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::LABEL, TagMeta {
            is_inline: true,
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::LEGEND, TagMeta {
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::LI, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::LISTING, TagMeta {
            is_deprecated: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::CHARSET, a::HREFLANG, a::HREF, a::MEDIA, a::REL, a::REV, a::TYPE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::LINK, TagMeta {
            is_empty: true,
            is_meta: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::MAIN, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::MAP, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::MARK, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::MENU, TagMeta {
            is_deprecated: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::MENUITEM, TagMeta {
            is_empty: true,
            is_deprecated: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::DIR, a::LANG, a::BASE, a::HTTP_EQUIV, a::CONTENT, a::SCHEME, a::CHARSET
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::META, TagMeta {
            is_empty: true,
            is_meta: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::METER, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::NAV, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::NOBR, TagMeta {
            is_deprecated: true,
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::NOFRAMES, TagMeta {
            is_deprecated: true,
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::NOSCRIPT, TagMeta {
            is_inline: true,
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::TYPE, a::DATA, a::ALIGN
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::OBJECT, TagMeta {
            is_inline: true,
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::OL, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::LABEL
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::OPTGROUP, TagMeta {
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::LABEL
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::OPTION, TagMeta {
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::OUTPUT, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::P, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::BASE, a::NAME, a::VALUE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::PARAM, TagMeta {
            is_empty: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::HEIGHT, a::WIDTH
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::PICTURE, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::PLAINTEXT, TagMeta {
            is_deprecated: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::PRE, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::PROGRESS, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::CITE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::Q, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::RB, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::RBC.clone(), TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::RP, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::RT, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::RTC, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::RUBY, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::S, TagMeta {
            is_deprecated: true,
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::SAMP, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::SCRIPT, TagMeta {
            is_inline: true,
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::SECTION, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::SELECT, TagMeta {
            is_inline: true,
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::SLOT, TagMeta {
            is_inline: true,
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::SMALL, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::TYPE, a::SRC
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::SOURCE, TagMeta {
            is_empty: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::SPAN, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::STRIKE, TagMeta {
            is_deprecated: true,
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::STRONG, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::STYLE, TagMeta {
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::SUB, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::SUMMARY, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::SUP, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::HEIGHT, a::WIDTH
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::SVG, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::ALIGN, a::SUMMARY
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::TABLE, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::ALIGN
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::TBODY, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::ALIGN, a::CHAR, a::CHAROFF, a::COLSPAN, a::HEADERS, a::ROWSPAN, a::SCOPE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::TD, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::TEMPLATE, TagMeta {
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::TEXTAREA, TagMeta {
            is_inline: true,
            is_banned: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::ALIGN
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::TFOOT, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::ABBR, a::ALIGN, a::AXIS, a::CHAR, a::CHAROFF, a::COLSPAN, a::ROWSPAN, a::SCOPE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::TH, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::ALIGN
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::THEAD, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::DATETIME
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::TIME, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::TITLE, TagMeta {
            is_meta: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::ABBR, a::ALIGN, a::AXIS, a::CHAR, a::CHAROFF, a::COLSPAN, a::HEADERS, a::ROWSPAN, a::SCOPE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::TR, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::TT, TagMeta {
            is_deprecated: true,
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::U, TagMeta {
            is_deprecated: true,
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::UL, TagMeta {
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::VAR, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE, a::HEIGHT, a::WIDTH
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::VIDEO, TagMeta {
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::WBR, TagMeta {
            is_empty: true,
            is_inline: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    {
        let mut basic_attrs = vec![
            a::TITLE, a::DIR, a::LANG, a::BASE
        ];
        basic_attrs.sort();
        basic_attrs.dedup();

        tag_meta.insert(t::XMP, TagMeta {
            is_deprecated: true,
            basic_attrs,
            .. TagMeta::default()
        });
    }
    tag_meta
}
