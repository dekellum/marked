use std::mem;

use tendril::StrTendril;

/// Replace or remove sequences of white-space and/or control characters, and
/// optionally remove leading/trailing spaces.
///
/// _What_ char classes to replace is given via `ws` and `ctrl` flags. If a
/// sequence is _all_ control or zero-width spaces, then it is simple removed
/// (without replacement). If there is at least one non-zero width white-space
/// character then the sequence is replaces with U+0020 SPACE.  The string (st)
/// is only lazily re-allocated (replaced) if a change is required.
pub(crate) fn replace_chars(
    st: &mut StrTendril,
    ws: bool,
    ctrl: bool,
    trim_start: bool,
    trim_end: bool)
{
    let mut last = 0;
    let mut ost = None; // output lazy allocated
    let mut replacing = 0u8;

    let ins = st.as_ref();
    for (i, ch) in ins.char_indices() {
        let rmask = replace_mask(ch, ws, ctrl);
        if rmask > 0 {
            if replacing == 0 {
                if ost.is_none() {
                    ost = Some(StrTendril::with_capacity(st.len32()));
                }
                ost.as_mut().unwrap().push_slice(&ins[last..i]);
            }
            replacing |= rmask;
        } else if replacing > 0 {
            if  replacing >= 2 &&
                (ost.as_ref().unwrap().len32() > 0 || !trim_start)
            {
                ost.as_mut().unwrap().push_char(' ');
            }
            last = i;
            replacing = 0;
        }
    }
    if replacing > 0 {
        if replacing >= 2 && !trim_end {
            ost.as_mut().unwrap().push_char(' ');
        }
    } else if ost.is_some() {
        ost.as_mut().unwrap().push_slice(&ins[last..]);
    }
    if ost.is_some() {
        mem::replace(st, ost.take().unwrap());
    }
}

// Compare CharClass to flags and return bit-1 (control or zero-width) or bit-2
// (whitespace).
fn replace_mask(c: char, ws: bool, ctrl: bool) -> u8 {
    use CharClass::*;
    match char_class(c) {
        ZeroSpace | Control if ctrl => 1,
        WhiteSpace if ws => 2,
        _ => 0,
    }
}

// Character classes of internal interest (not the same as Unicode classes).
#[derive(Debug, Eq, PartialEq)]
enum CharClass {
    Unclassified,
    WhiteSpace,
    ZeroSpace,
    Control,
}

/// True if all contained characters are classfied as whitespace or controls.
pub(crate) fn is_all_ctrl_ws(st: &StrTendril) -> bool {
    st.as_ref().chars().all(|c| char_class(c) != CharClass::Unclassified)
}

// Return CharClass for a char
fn char_class(c: char) -> CharClass {
    use CharClass::*;
    match c {
        '\u{0000}'..='\u{0008}' => Control,    // C0 (XML disallowed)
        '\u{0009}'              |              // HT
        '\u{000A}'              |              // LF
        '\u{000B}'              => WhiteSpace, // VT
        '\u{000C}'              => Control,    // FF (C0)
        '\u{000D}'              => WhiteSpace, // CR
        '\u{000E}'..='\u{001F}' => Control,    // C0
        '\u{0020}'              => WhiteSpace, // SPACE

        '\u{007F}'              |              // DEL (C0)
        '\u{0080}'..='\u{009F}' => Control,    // C1 (XML disallowed)
        '\u{00A0}'              => WhiteSpace, // NO-BREAK SPACE (NBSP)

        // Not always (zero) white; shows hypen when line is wrapped.
        // '\u{00AD}'           => Un-         // SOFT HYPHEN,

        // Not white, rendered with a line:
        // '\u{1680}'           => Un-         // OGHAM SPACE MARK

        // Effects subsequent characters in mongolion:
        // '\u{180E}'           => Un-         // MONGOLIAN VOWEL SEPARATOR

        '\u{2000}'..='\u{200A}' => WhiteSpace, // EN QUAD..HAIR SPACE
        '\u{200B}'              |              // ZERO WIDTH SPACE
        '\u{200C}'              => ZeroSpace,  // ZERO WIDTH NON-JOINER

        '\u{2028}'              |              // LINE SEPARATOR
        '\u{2029}'              |              // PARAGRAPH SEPARATOR

        '\u{202F}'              |              // NARROW NO-BREAK SPACE

        '\u{205F}'              => WhiteSpace, // MEDIUM MATHEMATICAL SPACE
        '\u{2060}'              => ZeroSpace,  // WORD JOINER

        '\u{3000}'              => WhiteSpace, // IDEOGRAPHIC SPACE

        '\u{FEFF}'              => ZeroSpace,  // BOM or ZERO WIDTH NON-BREAKING
        '\u{FFFE}'              |              // Bad BOM (not assigned)
        '\u{FFFF}'              => Control,    // Not assigned (invalid)
        _ => Unclassified,
    }

    // FIXME: see markup5ever/data/mod.rs: C1_REPLACEMENTS replaced with
    // alt higher unicode characters. This should occur _before_ above
    // transform, at least for HTML?
}

#[cfg(test)]
mod tests {
    use super::*;
    use tendril::SliceExt;

    #[test]
    fn test_char_class() {
        use CharClass::*;
        assert_eq!(Unclassified, char_class('x'));
        assert_eq!(Control,      char_class('\u{0008}'));
        assert_eq!(ZeroSpace,    char_class('\u{2060}'));
        assert_eq!(WhiteSpace,   char_class('\n'));
        assert_eq!(WhiteSpace,   char_class('\n'));
    }

    #[test]
    fn replace() {
        assert_clean("",  "" );
        assert_clean("",  "\u{2060}" );
        assert_clean(" ", " ");
        assert_clean(" ", "\t \r\n");

        assert_clean("x",   "x"   );
        assert_clean(" x ", " x  ");
        assert_clean(" x",  " x\u{2060}"  );
        assert_clean("x ",  "x "  );

        assert_clean("aa b ",  "\u{009F}a\u{009F}a  b " );

        assert_clean("aa b c ", "aa b c "     );
        assert_clean("aa b c",  "aa \t b c"   );
        assert_clean(" aa b c", "\t aa \t b c");
    }

    #[test]
    fn replace_ctrl_only() {
        assert_clean_ctrl("",  "" );
        assert_clean_ctrl("",  "\u{2060}" );
        assert_clean_ctrl(" ", " ");

        assert_clean_ctrl("x",   "x"   );
        assert_clean_ctrl(" x  ", " x  ");
        assert_clean_ctrl(" x",  " x\u{2060}"  );
        assert_clean_ctrl("x ",  "x "  );

        assert_clean_ctrl("aaa  b ",  "\u{009F}a\u{009F}aa  b " );

        assert_clean_ctrl("aa b c ", "aa b c "     );
        assert_clean_ctrl("aa \t b c",  "aa \t b c"   );
        assert_clean_ctrl("\t aa \t b c", "\t aa \t b c");
    }

    #[test]
    fn replace_trim() {
        assert_clean_trim("", "");
        assert_clean_trim("", "\t \r\n");
        assert_clean_trim("", "\u{0000}"); //NUL
        assert_clean_trim("", "\u{FFFE}"); //BAD BOM
        assert_clean_trim("", "\u{00A0}\u{2007}\u{202F}");

        assert_clean_trim("x",  "x"  );
        assert_clean_trim("x", " x  ");
        assert_clean_trim("x", " x"  );
        assert_clean_trim("x",  "x " );

        assert_clean_trim("aa b",  " a\u{009F}a\u{009F}  b " );

        assert_clean_trim("aa b c", "aa b c "     );
        assert_clean_trim("aa b c", "aa \t b c"   );
        assert_clean_trim("aa b c", "\t aa \t b c");
    }

    #[test]
    fn replace_trim_left() {
        assert_clean_trim_l("", "");
        assert_clean_trim_l(" ", " ");
        assert_clean_trim_l(" ", "\t \r\n");
    }

    #[test]
    fn replace_trim_right() {
        assert_clean_trim_r("", "");
        assert_clean_trim_r("", " ");
        assert_clean_trim_r("", "\t \r\n");
    }

    fn assert_clean_trim(exp: &str, src: &str) {
        let mut st = src.to_tendril();
        replace_chars(&mut st, true, true, true, true);
        assert_eq!(exp, st.as_ref());
    }

    fn assert_clean_trim_l(exp: &str, src: &str) {
        let mut st = src.to_tendril();
        replace_chars(&mut st, true, true, true, false);
        assert_eq!(exp, st.as_ref());
    }

    fn assert_clean_trim_r(exp: &str, src: &str) {
        let mut st = src.to_tendril();
        replace_chars(&mut st, true, true, false, true);
        assert_eq!(exp, st.as_ref());
    }

    fn assert_clean(exp: &str, src: &str) {
        let mut st = src.to_tendril();
        replace_chars(&mut st, true, true, false, false);
        assert_eq!(exp, st.as_ref());
    }

    fn assert_clean_ctrl(exp: &str, src: &str) {
        let mut st = src.to_tendril();
        replace_chars(&mut st, false, true, false, false);
        assert_eq!(exp, st.as_ref());
    }
}
