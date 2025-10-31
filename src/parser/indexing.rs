use {
    crate::{CStr, cmdline::helpers::try_to_str, direct::argc_argv, iter::helpers::len},
    std::{
        cmp::{Ord, min},
        collections::BTreeMap,
        fmt::{Debug, Formatter, Result as FmtRes},
        iter::Iterator,
        mem::transmute,
        ops::Fn,
        option::Option::{self, None, Some},
        ptr::{self, null},
        result::Result::{self, Err, Ok},
        slice,
        str::Utf8Error
    }
};

macro_rules! tri {
    (str:$i:ident $e:expr) => {
        match $e {
            Ok(val) => val,
            Err(err) => return Err(ParseError::InvalidStr($i, err))
        }
    };
    (opt:$e:expr) => {
        match $e {
            Some(val) => val,
            None => return None
        }
    };
}

fn null_slice() -> *const [*const u8] {
    ptr::slice_from_raw_parts(null(), 0)
}

// indicator of the start of a short, two for a long argument
const INDICATOR: char = '-';

pub struct IndexingParser {
    index: BTreeMap<Ident, Argument>,
    positionals: usize
}

impl IndexingParser {
    #[allow(clippy::new_without_default, clippy::inline_always)]
    #[must_use]
    #[inline(always)]
    pub fn new() -> IndexingParser {
        IndexingParser { index: BTreeMap::new(), positionals: 0 }
    }

    pub fn reset(&mut self) {
        self.index.clear();
    }

    pub fn parse(
        &mut self,
        rules: &[OptRule],
        is_first_progname: impl Fn(&'static str) -> bool
    ) -> Result<(), ParseError> {
        if !self.index.is_empty() {
            // already parsed
            return Ok(());
        }
        let (argc, argv) = argc_argv();
        let len = argc as usize;

        let mut positionals = 0;
        let mut i = 0;
        let mut end_of_args = false;

        // so we can reuse the pre-str'd next which we need for using values
        let mut next = None;

        unsafe {
            loop {
                let current_raw = argv.add(i);
                let current = current_raw.read();
                let str = if let Some(next) = next {
                    next
                } else {
                    tri!(str:i CStr::from_ptr(current).to_stdlib().to_str())
                };

                if i < len {
                    let i = i + 1;
                    next = Some(tri!(str:i CStr::from_ptr(current).to_stdlib().to_str()))
                }

                if i == 0 && is_first_progname(str) {
                    self.index.insert(Ident::__Prog, Argument::ProgFlagOrPos(str));
                } else if end_of_args {
                    self.push_positional(&mut positionals, str);
                } else {
                    let mut chars = str.chars();
                    match (chars.next(), chars.next(), chars.next()) {
                        (Some(INDICATOR), Some(INDICATOR), Some(_)) => {
                            // long
                            self.push_long(str, current_raw, rules, len - i - 1, &mut i, next);
                        }
                        (Some(INDICATOR), Some(INDICATOR), None) => {
                            // end-of-args marker --
                            end_of_args = true;
                        }
                        (Some(INDICATOR), Some(_), _) => {
                            // single short
                            self.push_short(str, current_raw, rules, len - i - 1, &mut i, next);
                        }
                        // no need for (Some('-'), None, None), the stdin shorthand as it's just a
                        //  positional, so the below catches it
                        (Some(_), _, _) => {
                            self.push_positional(&mut positionals, str);
                        }
                        // under normal circumstances, no argument will be zero-length. Chars is
                        // Fused so 1 None means all None
                        (None, _, _) => {}
                    }
                }

                i += 1;
                if i == len {
                    self.positionals = positionals;
                    return Ok(());
                }
            }
        }
    }

    // accessors

    #[must_use]
    #[inline]
    pub fn positional(&self, idx: usize) -> Option<&'static str> {
        if idx >= self.positionals {
            return None;
        }
        for (id, arg) in &self.index {
            match id {
                Ident::Positional(n) if *n == idx => {
                    return Some(arg.opt());
                }
                _ => {}
            }
        }
        None
    }

    // this does have a small caveat that any option (e.g. -n 100) may be accessible as a flag,
    //  even though it is clearly an option.
    #[must_use]
    #[inline]
    pub fn flag(&self, name: &'static str) -> bool {
        for id in self.index.keys() {
            match id {
                Ident::Option(rule_name) if *rule_name == name => {
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    #[must_use]
    #[inline]
    pub fn option(&self, name: &'static str) -> Option<OptValues> {
        for (id, arg) in &self.index {
            match id {
                Ident::Option(rule_name) if *rule_name == name => {
                    let val = tri!(opt:arg.val());

                    return Some(OptValues {
                        cur: val.cast::<*const u8>(),
                        end: unsafe { val.cast::<*const u8>().add((&*val).len()) },
                        offset: arg.val_offset().unwrap_or(0)
                    });
                }
                _ => {}
            }
        }
        None
    }

    #[must_use]
    #[inline]
    pub fn option_ptrs(&self, name: &'static str) -> Option<&'static [CStr<'static>]> {
        for (id, arg) in &self.index {
            match id {
                Ident::Option(rule_name) if *rule_name == name => {
                    return Some(unsafe { &*(tri!(opt:arg.val()) as *const [CStr<'static>]) });
                }
                _ => {}
            }
        }
        None
    }

    // helpers

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn push_positional(&mut self, positionals: &mut usize, s: &'static str) {
        // positional
        self.index.insert(Ident::Positional(*positionals), Argument::ProgFlagOrPos(s));
        *positionals += 1;
    }

    #[inline]
    fn push_long(
        &mut self,
        s: &'static str,
        raw: *const *const u8,
        rules: &[OptRule],
        remaining: usize,
        i: &mut usize,
        next_peek: Option<&str>
    ) {
        let eq_form = s.chars().position(|c| c == '=');
        for rule in rules {
            match rule.long() {
                Some(rule_s) if rule_s == eq_form.map_or_else(|| &s[2..], |eq| &s[2..eq]) => {
                    let (val, val_offset) = eq_form.map_or_else(
                        || (IndexingParser::parse_vals(raw, rule, remaining, i, next_peek), 0),
                        |i| (ptr::slice_from_raw_parts(raw, 1), i + 1)
                    );
                    self.index.insert(
                        Ident::Option(rule.name()),
                        Argument::Opt { opt: s, val, val_offset }
                    );
                }
                _ => {}
            }
        }
    }

    #[inline]
    fn push_short(
        &mut self,
        s: &'static str,
        raw: *const *const u8,
        rules: &[OptRule],
        remaining: usize,
        i: &mut usize,
        next_peek: Option<&str>
    ) {
        // cut off '-'
        let cut = &s[1..];

        for (c_i, c) in cut.char_indices() {
            // TODO: more efficient rule matching than a for loop (both in here and in push_long)
            //  already tried a HashMap but it was slower (25x slower). might have done smth wrong
            for rule in rules {
                match rule.short() {
                    Some(rule_c) if rule_c == c => {
                        self.index.insert(
                            Ident::Option(rule.name()),
                            Argument::Opt {
                                opt: &cut[c_i..=c_i],
                                // this does in theory allow for things like "-nm 100 100", while
                                //  the gnu/posix standards don't
                                val: IndexingParser::parse_vals(raw, rule, remaining, i, next_peek),
                                val_offset: 0
                            }
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    #[inline]
    fn next_is_special(peek: Option<&str>) -> bool {
        match peek {
            // no next, can't be special
            None => false,
            Some(s) => {
                let mut chars = s.chars();
                match (chars.next(), chars.next(), chars.next()) {
                    // longs are special
                    (Some('-'), Some('-'), Some(_)) => true,
                    // eoa is not (TODO: decide whether it should be)
                    (Some('-'), Some('-'), None) => false,
                    // shorts and bundles are special
                    (Some('-'), Some(_), _) => true,
                    // anything else (including -/stdin shorthand and empty arg) isn't
                    _ => false
                }
            }
        }
    }

    #[inline]
    fn parse_vals(
        raw: *const *const u8,
        rule: &OptRule,
        remaining: usize,
        i: &mut usize,
        next_peek: Option<&str>
    ) -> *const [*const u8] {
        if IndexingParser::next_is_special(next_peek) {
            return null_slice();
        }

        match rule.val_count() {
            0 => null_slice(),
            n => unsafe {
                let cnt = min(n, remaining);
                if cnt == 0 {
                    return null_slice();
                }
                *i += cnt;
                // TODO: don't count as a value if it's special like another option or something
                ptr::slice_from_raw_parts(raw.add(1), cnt)
            }
        }
    }
    fn write_vals(f: &mut Formatter<'_>, vals: *const [*const u8]) -> FmtRes {
        let slice = unsafe { &*vals };
        write!(f, "[")?;
        for (i, &p) in slice.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            match try_to_str(p) {
                Some(s) => write!(f, "{:?}", s)?,
                None => write!(f, "{:p}", p)?
            }
        }
        write!(f, "]")
    }

    fn debug_alt(&self, f: &mut Formatter<'_>) -> FmtRes {
        writeln!(f, "IndexingParser(")?;
        for (id, arg) in &self.index {
            match id {
                Ident::__Prog => writeln!(f, "    Program executable: {}", arg.opt())?,
                Ident::Positional(n) => writeln!(f, "    Positional #{}: {}", n, arg.opt())?,
                Ident::Option(name) => {
                    if let Some(val) = arg.val() {
                        write!(f, "    ?Option?: \"{}\": ", name)?;
                        IndexingParser::write_vals(f, val)?;
                        writeln!(f)?;
                    } else {
                        writeln!(f, "    ?Flag?: \"{}\"", name)?;
                    }
                }
            }
        }
        writeln!(f, ")")
    }

    fn debug_norm(&self, f: &mut Formatter<'_>) -> FmtRes {
        write!(f, "IndexingParser(")?;
        let mut first = true;
        for (id, arg) in &self.index {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }

            match id {
                Ident::__Prog => write!(f, "program={:?}", arg.opt())?,
                Ident::Positional(n) => write!(f, "{}={:?}", n, arg.opt())?,
                Ident::Option(name) => {
                    if let Some(val) = arg.val() {
                        write!(f, "{}=", name)?;
                        IndexingParser::write_vals(f, val)?;
                    } else {
                        write!(f, "?flag?=\"{}\"", name)?;
                    }
                }
            }
        }
        write!(f, ")")
    }
}

impl Debug for IndexingParser {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtRes {
        if self.index.is_empty() {
            return write!(f, "IndexingParser(unparsed)");
        }

        if f.alternate() {
            return self.debug_alt(f);
        }

        self.debug_norm(f)
    }
}

pub struct OptRule {
    name: &'static str,
    // below are optional, where:
    // (_, 0) == None
    long: (*const u8, usize),
    // 0 == None
    short: char,
    // why? users have no need for a null pointer, 0-sized long, or null short, and
    //  making them Options adds 2 bytes

    // if non-zero, the option accepts up to val_count following arguments
    val_count: usize
}

impl OptRule {
    #[must_use]
    pub const fn new(name: &'static str) -> OptRule {
        OptRule { name, long: (null(), 0), short: '\0', val_count: 0 }
    }

    #[must_use]
    pub const fn new_auto_long(name: &'static str) -> OptRule {
        OptRule { name, long: (name.as_ptr(), name.len()), short: '\0', val_count: 0 }
    }

    #[must_use]
    pub const unsafe fn new_auto(name: &'static str) -> OptRule {
        OptRule {
            name,
            long: (name.as_ptr(), name.len()),
            // unsafe as this assumes first char is ascii, lazy impl which will be done better later
            short: *name.as_ptr() as char,
            val_count: 0
        }
    }

    #[must_use]
    pub const fn set_long(mut self, long: &'static str) -> OptRule {
        self.long = (long.as_ptr(), long.len());
        self
    }

    #[must_use]
    pub const fn set_short(mut self, short: char) -> OptRule {
        self.short = short;
        self
    }

    #[must_use]
    pub const fn set_val_count(mut self, val_count: usize) -> OptRule {
        self.val_count = val_count;
        self
    }

    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    #[must_use]
    pub fn long(&self) -> Option<&'static str> {
        if self.long.1 == 0 {
            None
        } else {
            Some(unsafe {
                #[allow(clippy::transmute_bytes_to_str)]
                transmute::<&[u8], &str>(slice::from_raw_parts(self.long.0, self.long.1))
            })
        }
    }

    #[must_use]
    pub const fn short(&self) -> Option<char> {
        if self.short == '\0' { None } else { Some(self.short) }
    }

    #[must_use]
    pub const fn val_count(&self) -> usize {
        self.val_count
    }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ident {
    /// The program name.
    __Prog,
    Positional(usize),
    Option(&'static str)
}

enum Argument {
    ProgFlagOrPos(&'static str),
    Opt {
        opt: &'static str,
        val: *const [*const u8],
        // for long=value form. the index of the first char following the = sign.
        val_offset: usize
    }
}

#[allow(clippy::inline_always)]
impl Argument {
    #[inline(always)]
    const fn opt(&self) -> &'static str {
        match self {
            Argument::ProgFlagOrPos(opt) | Argument::Opt { opt, .. } => opt
        }
    }

    #[inline(always)]
    const fn val(&self) -> Option<*const [*const u8]> {
        match self {
            Argument::ProgFlagOrPos(_) => None,
            Argument::Opt { val, .. } => Some(*val)
        }
    }

    #[inline(always)]
    const fn val_offset(&self) -> Option<usize> {
        match self {
            Argument::ProgFlagOrPos(_) => None,
            Argument::Opt { val_offset, .. } => Some(*val_offset)
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    InvalidStr(usize, Utf8Error)
}

pub struct OptValues {
    cur: *const *const u8,
    end: *const *const u8,
    offset: usize
}

impl Iterator for OptValues {
    type Item = &'static str;

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self) -> Option<&'static str> {
        if self.cur == self.end {
            return None;
        }
        let p = self.cur;
        self.cur = unsafe { self.cur.add(1) };

        try_to_str(unsafe { p.read().add(self.offset) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = unsafe { len(self.cur, self.end) };
        (len, Some(len))
    }

    // TODO: other methods. default implementations should suck for this but i don't feel like
    //  implementing them rn
}

// TODO: iterators for positionals, etc.
