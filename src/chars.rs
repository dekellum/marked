use std::mem;

use tendril::StrTendril;

/// Replace ws and/or control characters with a single U+0020 SPACE, and
/// optionally remove leading/trailing spaces.
pub(crate) fn replace_chars(
    st: &mut StrTendril,
    ws: bool,
    control: bool,
    trim_start: bool,
    trim_end: bool)
{
    let mut last = 0;
    let mut replacing = false;
    let mut ost = StrTendril::with_capacity(st.len32());
    // FIXME: Optimize (no-alloc, replace) for cases where there is no real
    // change

    let ins = st.as_ref();
    for (i, ch) in ins.char_indices() {
        if do_replace(ch, ws, control) {
            if !replacing {
                ost.push_slice(&ins[last..i]);
                replacing = true;
            }
        } else if replacing {
            if ost.len32() > 0 || !trim_start {
                ost.push_char(' ');
            }
            last = i;
            replacing = false;
        }
    }
    if replacing {
        if !trim_end {
            ost.push_char(' ');
        }
    } else {
        ost.push_slice(&ins[last..]);
    }
    mem::replace(st, ost);
}

fn do_replace(c: char, ws: bool, control: bool) -> bool {
    use CharClass::*;
    match char_class(c) {
        ZeroSpace | Control if control => true,
        WhiteSpace if ws => true,
        _ => false,
    }
}

#[derive(Debug, Eq, PartialEq)]
enum CharClass {
    Unclassified,
    Control,
    WhiteSpace,
    ZeroSpace
}

/// Return true if char is a control, Unicode whitespace, BOM, or otherwise
/// invalid.
fn char_class(c: char) -> CharClass {
    use CharClass::*;
    match c {
        '\u{0000}'..='\u{0008}' => Control,    // C0 (XML disallowed)
        '\u{0009}'              | // HT
        '\u{000A}'              | // LF
        '\u{000B}'              => WhiteSpace, // VT
        '\u{000C}'              => Control,    // FF (C0)
        '\u{000D}'              => WhiteSpace, // CR
        '\u{000E}'..='\u{001F}' => Control,    // C0
        '\u{0020}'              => WhiteSpace, // SPACE

        '\u{007F}'              | // DEL (C0)
        '\u{0080}'..='\u{009F}' => Control,    // C1 (XML disallowed)
        '\u{00A0}'              => WhiteSpace, // NO-BREAK SPACE (NBSP)

        // Not white, rendered with a line:
        // '\u{1680}'           => Un-         // OGHAM SPACE MARK

        // Effects subsequent characters in mongolion:
        // '\u{180E}'           => Un-         // MONGOLIAN VOWEL SEPARATOR

        '\u{2000}'..='\u{200A}' => WhiteSpace, // EN QUAD..HAIR SPACE
        '\u{200B}'              | // ZERO WIDTH SPACE
        '\u{200C}'              => ZeroSpace,  // ZERO WIDTH NON-JOINER

        '\u{2028}'              | // LINE SEPARATOR
        '\u{2029}'              | // PARAGRAPH SEPARATOR

        '\u{202F}'              | // NARROW NO-BREAK SPACE

        '\u{205F}'              => WhiteSpace, // MEDIUM MATHEMATICAL SPACE
        '\u{2060}'              => ZeroSpace,  // WORD JOINER

        '\u{3000}'              => WhiteSpace, // IDEOGRAPHIC SPACE

        '\u{FEFF}'              => ZeroSpace,  // BOM or ZERO WIDTH NON-BREAKING
        '\u{FFFE}'              | // Bad BOM (not assigned)
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
        assert_eq!(Control,      char_class('\u{0008}'));
        assert_eq!(Unclassified, char_class('x'));
        assert_eq!(WhiteSpace,   char_class('\n'));
        assert_eq!(WhiteSpace,   char_class('\t'));
    }

    #[test]
    fn replace() {
        assert_clean("",  "" );
        assert_clean(" ", " ");

        assert_clean("x",   "x"   );
        assert_clean(" x ", " x  ");
        assert_clean(" x",  " x"  );
        assert_clean("x ",  "x "  );

        assert_clean("aa b c ", "aa b c "     );
        assert_clean("aa b c",  "aa \t b c"   );
        assert_clean(" aa b c", "\t aa \t b c");

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

        assert_clean_trim("aa b c", "aa b c "     );
        assert_clean_trim("aa b c", "aa \t b c"   );
        assert_clean_trim("aa b c", "\t aa \t b c");
    }

    fn assert_clean_trim(exp: &str, src: &str) {
        let mut st = src.to_tendril();
        replace_chars(&mut st, true, true, true, true);
        assert_eq!(exp, st.as_ref());
    }

    fn assert_clean(exp: &str, src: &str) {
        let mut st = src.to_tendril();
        replace_chars(&mut st, true, true, false, false);
        assert_eq!(exp, st.as_ref());
    }
}
