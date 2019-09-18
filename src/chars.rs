use std::mem;

use tendril::StrTendril;

/// Replace `is_ctrl_ws` characters with single space, and optionally
/// leading/trailing spaces.
pub(crate) fn replace_ctrl_ws(
    st: &mut StrTendril,
    trim_start: bool,
    trim_end: bool)
{
    let mut last = 0;
    let mut ws = false;
    let mut ost = StrTendril::with_capacity(st.len32());
    // FIXME: Optimize (no-alloc, replace) for cases where there is no real
    // change

    let ins = st.as_ref();
    for (i, ch) in ins.char_indices() {
        if is_ctrl_ws(ch) {
            if !ws {
                ost.push_slice(&ins[last..i]);
                ws = true;
            }
        } else if ws {
            if ost.len32() > 0 || !trim_start {
                ost.push_char(' ');
            }
            last = i;
            ws = false;
        }
    }
    if ws {
        if !trim_end {
            ost.push_char(' ');
        }
    } else {
        ost.push_slice(&ins[last..]);
    }
    mem::replace(st, ost);
}

/// Return true if char is a control, Unicode whitespace, BOM, or otherwise
/// invalid.
fn is_ctrl_ws(c: char) -> bool {
    // FIXME: Should probably break this up into several different
    // classifications. For now we treat all of maches as something to replace
    // with a single space.
     match c {
        '\u{0000}'..='\u{0008}' | // C0 control chars (XML disallowed)
        '\u{0009}'              | // HT
        '\u{000A}'              | // LF
        '\u{000B}'              | // VT
        '\u{000C}'              | // FF (C0)
        '\u{000D}'              | // CR
        '\u{000E}'..='\u{001F}' | // C0 control chars (XML disallowed)
        '\u{0020}'              | // SPACE

        '\u{007F}'              | // DEL (C0)
        '\u{0080}'..='\u{009F}' | // C1 control chars (XML disallowed)

        '\u{00A0}'              | // NO-BREAK SPACE (NBSP)

        '\u{1680}'              | // OGHAM SPACE MARK
        '\u{180E}'              | // MONGOLIAN VOWEL SEPARATOR

        '\u{2000}'..='\u{200A}' | // EN QUAD..HAIR SPACE exotics
        '\u{200B}'              | // ZERO WIDTH SPACE

        '\u{2028}'              | // LINE SEPARATOR
        '\u{2029}'              | // PARAGRAPH SEPARATOR

        '\u{202F}'              | // NARROW NO-BREAK SPACE

        '\u{205F}'              | // MEDIUM MATHEMATICAL SPACE
        '\u{2060}'              | // WORD JOINER

        '\u{3000}'              | // IDEOGRAPHIC SPACE

        '\u{FEFF}'              | // BOM
        '\u{FFFE}'              | // Bad BOM (not assigned)
        '\u{FFFF}'                // Not assigned (invalid Unicode)
            => return true,
        _ => return false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tendril::SliceExt;

    #[test]
    fn ctrl_ws() {
        assert!(is_ctrl_ws('\u{0008}'));
        assert!(!is_ctrl_ws('x'));
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
        replace_ctrl_ws(&mut st, true, true);
        assert_eq!(exp, st.as_ref());
    }

    fn assert_clean(exp: &str, src: &str) {
        let mut st = src.to_tendril();
        replace_ctrl_ws(&mut st, false, false);
        assert_eq!(exp, st.as_ref());
    }

}
