use {
    crate::{CStr, direct::argc_argv, helpers::try_to_str, iter::len},
    alloc::vec::Vec,
    std::{
        clone::Clone,
        cmp::min,
        collections::{BTreeMap, HashMap},
        fmt::{Debug, Formatter, Result as FmtRes},
        hint::unreachable_unchecked,
        iter::{ExactSizeIterator, Iterator},
        marker::Copy,
        mem::transmute,
        ops::{Fn, RangeBounds},
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
            Err(err) => return Err(Error::InvalidStr($i, err))
        }
    };
    (opt_err:$e:expr,$err:expr) => {
        match $e {
            Some(val) => val,
            None => return Err($err)
        }
    };
    (unrp $e:expr) => {
        match $e {
            Some(val) => val,
            None => unreachable_unchecked()
        }
    };
}

fn null_slice() -> *const [*const u8] {
    ptr::slice_from_raw_parts(null(), 0)
}

// indicator of the start of a short, two for a long argument
const INDICATOR: char = '-';
const EMPTY_STR: &str = "";

/// A parser that indexes program arguments for named access.
///
/// Parses once and stores results for fast lookup. Allocates per argument kind.
///
/// # Heap memory usage (approx., 64-bit):
/// - Positional arguments: stored as &str in a Vec, ~16 bytes per positional plus amortized `Vec`
///   capacity.
/// - Options: stored in a BTreeMap keyed by rule name, ~80–120 bytes per present option.
/// - Named positionals: stored in a HashMap<&'static str, usize>, ~24–40 bytes per named entry.
///
/// Additionally, the temporary HashMap used during `parse` to determine whether required arguments
/// were found adds roughly ~24 bytes per *required* option. Actual numbers depend on target pointer
/// width, allocator, etc. If you need low memory usage, benchmark to determine actual memory usage
/// first.
pub struct IndexingParser {
    // program name. empty = None
    prog: &'static str,
    // map correlating option names to their values. BTreeMap for its auto-sorting properties,
    // since HashMap leads to terrible Debug output.
    option_index: BTreeMap<&'static str, Argument>,
    // the values of positionals. elem 0 = first positional, elem 1 = second, etc.
    positionals: Vec<&'static str>,
    // map correlating the names of named positionals to their indexes.
    positional_names: HashMap<&'static str, usize>
}

impl IndexingParser {
    /// Creates a new `IndexingParser`.
    ///
    /// Does not parse arguments. Call [`parse`] before accessing results.
    #[allow(clippy::new_without_default, clippy::inline_always)]
    #[must_use]
    #[inline(always)]
    pub fn new() -> IndexingParser {
        IndexingParser {
            prog: EMPTY_STR,
            option_index: BTreeMap::new(),
            positionals: Vec::new(),
            positional_names: HashMap::new()
        }
    }

    /// Clear parsed index and reset parser state.
    pub fn reset(&mut self) {
        self.prog = EMPTY_STR;
        self.option_index.clear();
        self.positionals.clear();
        self.positional_names.clear();
    }

    // TODO: use a IndexingParserBuilder or something so parse can have defaults and be simpler.
    //  like IndexingParser::builder().rules(&[...]).build() where build just creates an
    //  IndexingParser, calls parse with its stored values, and returns the IndexingParser
    /// Parses program arguments using the provided rules.
    // ///
    // /// Prefer using [`IndexingParserBuilder`] as it greatly simplifies this and provides
    // /// defaults.
    ///
    /// - `rules`: slice of `OptRule`s that describes recognized options.
    /// - `positional_range`: range of valid positional counts.
    /// - `positional_names`: names associated with positional indices, allowing for access via
    ///   [`named_positional`](IndexingParser::named_positional).
    /// - `is_first_prog`: callback that identifies the program executable in the first arg.
    /// - `allow_multiple_short_vals`: whether to allow "-nm 100 100" syntax (`true`) or "-n1000"
    ///   (`false`) syntax. For a parser more similar to existing standards, this should be `false`.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidStr`] if any argument contains invalid UTF-8. Parsing aborted.
    /// - [`Error::WrongPositionalCount(n)`] if the number of found positionals was not in
    ///   `positional_range`. Parsing was otherwise successful.
    /// - [`Error::MissingRequired(missing)`] if any required options were missing. Parsing was
    ///   otherwise successful.
    pub fn parse(
        &mut self,
        rules: &[OptRule],
        positional_range: impl RangeBounds<usize>,
        positional_names: &[(&'static str, usize)],
        is_first_prog: impl Fn(&'static str) -> bool,
        allow_multiple_short_vals: bool
    ) -> Result<(), Error> {
        if !self.option_index.is_empty() {
            // already parsed
            return Ok(());
        }
        let (argc, argv) = argc_argv();
        let len_1 = argc as usize;
        let len = len_1 - 1;

        let mut i = 0;
        let mut end_of_args = false;

        // so we can reuse the pre-str'd next which we need for using values
        let mut next = None;

        let mut found_required = rules
            .iter()
            .filter_map(|r| if r.required() { Some((r.name, false)) } else { None })
            .collect::<HashMap<_, _>>();
        self.positional_names = positional_names
            .iter()
            .copied()
            .filter(|(_, i)| positional_range.contains(i))
            .collect::<HashMap<_, _>>();

        unsafe {
            loop {
                let current_raw = argv.add(i);
                let current = current_raw.read();
                // TODO: maybe allow non-UTF8?
                let str = if let Some(next) = next {
                    next
                } else {
                    tri!(str:i CStr::from_ptr(current).to_stdlib().to_str())
                };

                if i < len {
                    let i = i + 1;
                    next = Some(
                        tri!(str:i CStr::from_ptr(current_raw.add(1).read()).to_stdlib().to_str())
                    );
                }

                if i == 0 && is_first_prog(str) {
                    self.prog = str;
                } else if end_of_args {
                    self.push_positional(str);
                } else {
                    let mut chars = str.chars();
                    match (chars.next(), chars.next(), chars.next()) {
                        (Some(INDICATOR), Some(INDICATOR), Some(_)) => {
                            // long
                            self.push_long(
                                str,
                                current_raw,
                                rules,
                                &mut found_required,
                                len - i,
                                &mut i,
                                next
                            );
                        }
                        (Some(INDICATOR), Some(INDICATOR), None) => {
                            // end-of-args marker --
                            end_of_args = true;
                        }
                        (Some(INDICATOR), Some(_), _) => {
                            // single short
                            self.push_short(
                                str,
                                current_raw,
                                rules,
                                &mut found_required,
                                len - i,
                                &mut i,
                                next,
                                allow_multiple_short_vals
                            );
                        }
                        // no need for (Some('-'), None, None), the stdin shorthand as it's just a
                        //  positional, so the below catches it
                        _ => {
                            self.push_positional(str);
                        }
                    }
                }

                i += 1;
                if i == len_1 {
                    let missing = found_required
                        .iter()
                        .filter_map(|(name, found)| if *found { None } else { Some(*name) });
                    if missing.clone().count() != 0 {
                        return Err(Error::MissingRequired(missing.collect()));
                    } else if !positional_range.contains(&self.positional_count()) {
                        return Err(Error::WrongPositionalCount(self.positional_count()));
                    }
                    return Ok(());
                }
            }
        }
    }

    // accessors

    /// Returns program name if detected by `is_first_prog` during `parse`.
    /// Otherwise `None`.
    #[must_use]
    #[inline]
    pub const fn prog_name(&self) -> Option<&'static str> {
        if self.prog.is_empty() { None } else { Some(self.prog) }
    }

    /// Returns the number of positional arguments parsed.
    #[must_use]
    #[inline]
    pub fn positional_count(&self) -> usize {
        self.positionals.len()
    }

    /// Returns the `n`th positional argument, or `None` if it does not exist.
    #[must_use]
    #[inline]
    pub fn positional(&self, n: usize) -> Option<&'static str> {
        self.positionals.get(n).copied()
    }

    /// Returns the positional with the given name. This will return `None` if there is no
    /// positional with that name or the index of the positional with that name does not exist.
    ///
    /// # Errors
    ///
    /// - [`Error::NotFound`] if no positional has the requested name.
    /// - [`Error::NoValue`] if the positional index correlated with that name has no value.
    #[inline]
    pub fn named_positional(&self, name: &'static str) -> Result<&'static str, Error> {
        self.positional_names
            .get(name)
            .map_or(Err(Error::NotFound), |n| self.positional(*n).ok_or(Error::NoValue))
    }
    // TODO: make these part of a trait

    /// Returns a slice of all indexed positionals.
    #[must_use]
    #[inline]
    pub fn positionals(&self) -> &[&'static str] {
        &self.positionals
    }

    /// Returns `true` if an option with `name` was present.
    ///
    /// Note: this treats options with attached values as flags.
    #[must_use]
    #[inline]
    pub fn flag(&self, name: &'static str) -> bool {
        for id in self.option_index.keys() {
            if *id == name {
                return true;
            }
        }
        false
    }

    /// Returns an iterator over values for `name` if any.
    ///
    /// # Errors
    ///
    /// - <code>Err([Error::NotFound])</code> if no positional has the given name.
    /// - <code>Err([Error::NoValue])</code> if there is no value at the positional index correlated
    ///   with the given name.
    #[inline]
    pub fn option(&self, name: &'static str) -> Result<OptValues, Error> {
        for (id, arg) in &self.option_index {
            if *id == name {
                let val = tri!(opt_err:arg.val(), Error::NoValue);

                return Ok(OptValues {
                    cur: val.cast::<*const u8>(),
                    end: unsafe { val.cast::<*const u8>().add((&*val).len()) },
                    offset: arg.val_offset()
                });
            }
        }
        Err(Error::NotFound)
    }

    // helpers

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn push_positional(&mut self, s: &'static str) {
        self.positionals.push(s);
    }

    // TODO: don't allow this, do something better
    #[allow(clippy::too_many_arguments)]
    #[inline]
    fn push_long(
        &mut self,
        s: &'static str,
        raw: *const *const u8,
        rules: &[OptRule],
        found_required: &mut HashMap<&'static str, bool>,
        remaining: usize,
        i: &mut usize,
        next_peek: Option<&str>
    ) {
        let eq_form = s.find('=');
        for rule in rules {
            match rule.long() {
                Some(rule_s) if rule_s == eq_form.map_or_else(|| &s[2..], |eq| &s[2..eq]) => {
                    let ((val, enough_vals), val_offset) = eq_form.map_or_else(
                        || (IndexingParser::parse_vals(raw, rule, remaining, i, next_peek), 0),
                        |i| ((ptr::slice_from_raw_parts(raw, 1), rule.val_count() == 1), i + 1)
                    );
                    if rule.required() {
                        // SAFETY: if the rule is required, it must be in the found_required map
                        // from the start.
                        unsafe {
                            *tri!(unrp found_required.get_mut(rule.name())) = enough_vals;
                        }
                    }
                    self.option_index.insert(rule.name(), Argument::new_maybe_opt(val, val_offset));
                }
                _ => {}
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[inline]
    fn push_short(
        &mut self,
        s: &'static str,
        raw: *const *const u8,
        rules: &[OptRule],
        found_required: &mut HashMap<&'static str, bool>,
        remaining: usize,
        i: &mut usize,
        next_peek: Option<&str>,
        allow_multiple_vals: bool
    ) {
        // cut off '-'
        let cut = &s[1..];

        for (c_i, c) in cut.char_indices().map(|(i, c)| (i + 1, c)) {
            // TODO: more efficient rule matching than a for loop (both in here and in push_long)
            //  already tried a HashMap but it was slower (25x slower). might have done smth wrong
            for rule in rules {
                match rule.short() {
                    Some(rule_c) if rule_c == c => {
                        // if it has a value, we end the bundle and parse the rest of the arg or the
                        // next as the value. this allows for "-vn1000" but not "-nm 100 100", more
                        // standard and expected behavior
                        let ((val, enough_vals), val_offset, consumed_remaining_arg) =
                            match (allow_multiple_vals, rule.val_count() != 0, c_i < cut.len()) {
                                // we only use the rest of the argument if the current both has a
                                // value, there are more characters, and the caller doesn't want
                                // -nm 100 100 syntax.
                                (false, true, true) => (
                                    (ptr::slice_from_raw_parts(raw, 1), rule.val_count() == 1),
                                    c_i + 1,
                                    true
                                ),
                                // otherwise, we use the next argument (if the current has a value).
                                // parse_vals will handle the case where it has no value, slightly
                                // faster than handling it here despite slight redundancy.
                                _ => (
                                    IndexingParser::parse_vals(raw, rule, remaining, i, next_peek),
                                    0,
                                    false
                                )
                            };
                        if rule.required() {
                            // SAFETY: if the rule is required, it must be in the found_required map
                            // from the start.
                            unsafe {
                                *tri!(unrp found_required.get_mut(rule.name())) = enough_vals;
                            }
                        }
                        self.option_index
                            .insert(rule.name(), Argument::new_maybe_opt(val, val_offset));
                        if consumed_remaining_arg {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    #[inline]
    fn next_is_special(peek: Option<&str>) -> bool {
        peek.map_or(false, |s| {
            let mut chars = s.chars();
            match (chars.next(), chars.next(), chars.next()) {
                // longs, shorts, bundles, and end-of-args are special
                (Some('-'), Some('-'), Some(_)) | (Some('-'), Some(_), _) => true,
                // anything else (including -/stdin shorthand and empty arg) aren't
                _ => false
            }
        })
    }

    #[inline]
    fn parse_vals(
        raw: *const *const u8,
        rule: &OptRule,
        remaining: usize,
        i: &mut usize,
        next_peek: Option<&str>
    ) -> (*const [*const u8], bool) {
        if IndexingParser::next_is_special(next_peek) {
            return (null_slice(), false);
        }

        match rule.val_count() {
            0 => (null_slice(), false),
            n => unsafe {
                let cnt = min(n, remaining);
                if cnt == 0 {
                    return (null_slice(), false);
                }
                *i += cnt;
                (ptr::slice_from_raw_parts(raw.add(1), cnt), cnt == n)
            }
        }
    }

    // TODO: make below better in general.

    fn write_vals(f: &mut Formatter<'_>, vals: *const [*const u8], val_offset: usize) -> FmtRes {
        let slice = unsafe { &*vals };
        write!(f, "[")?;
        for (i, &p) in slice.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            match try_to_str(unsafe { p.add(val_offset) }) {
                Some(s) => write!(f, "{:?}", s)?,
                None => write!(f, "{:p}", p)?
            }
        }
        write!(f, "]")
    }

    fn debug_alt(&self, f: &mut Formatter<'_>) -> FmtRes {
        writeln!(f, "IndexingParser(")?;

        if !self.prog.is_empty() {
            writeln!(f, "    Program executable: {}", self.prog)?;
        }
        for (i, arg) in self.positionals.iter().enumerate() {
            writeln!(f, "    Positional #{}: {}", i, arg)?;
        }
        for (id, arg) in &self.option_index {
            if let Some(val) = arg.val() {
                write!(f, "    ?Option?: \"{}\": ", id)?;
                IndexingParser::write_vals(f, val, arg.val_offset())?;
                writeln!(f)?;
            } else {
                writeln!(f, "    ?Flag?: \"{}\"", id)?;
            }
        }

        writeln!(f, ")")
    }

    fn debug_norm(&self, f: &mut Formatter<'_>) -> FmtRes {
        fn write_sep(first: &mut bool, f: &mut Formatter<'_>) -> FmtRes {
            if *first {
                *first = false;
            } else {
                write!(f, ", ")?;
            }
            Ok(())
        }

        write!(f, "IndexingParser(")?;
        let mut first = true;

        if !self.prog.is_empty() {
            write_sep(&mut first, f)?;
            write!(f, "program={:?}", self.prog)?;
        }
        for (i, arg) in self.positionals.iter().enumerate() {
            write_sep(&mut first, f)?;
            write!(f, "{}={:?}", i, arg)?;
        }
        for (id, arg) in &self.option_index {
            write_sep(&mut first, f)?;

            if let Some(val) = arg.val() {
                write!(f, "{}=", id)?;
                IndexingParser::write_vals(f, val, arg.val_offset())?;
            } else {
                write!(f, "?flag?=\"{}\"", id)?;
            }
        }

        write!(f, ")")
    }
}

impl Debug for IndexingParser {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtRes {
        if self.option_index.is_empty() && self.positionals.is_empty() && self.prog.is_empty() {
            return write!(f, "IndexingParser(unparsed)");
        }

        if f.alternate() {
            return self.debug_alt(f);
        }

        self.debug_norm(f)
    }
}

/// A parsing rule that describes one option. This includes the following metadata:
///
/// - `name`: internal lookup name.
/// - `long`: optional long form (for example `verbose`).
/// - `short`: optional short form (for example `v`).
/// - `val_count`: number of following values. Zero means this is a flag.
/// - `required`: whether the option is required.
pub struct OptRule {
    name: &'static str,
    // if non-zero, the option accepts up to val_count following arguments
    val_count: usize,
    required: bool,
    // below are optional, where:
    // (_, 0) == None
    long: (*const u8, usize),
    // 0 == None
    short: char /* why? users have no need for a null pointer, 0-sized long, or null short, and
                 *  making them Options adds 2 bytes */
}

impl OptRule {
    /// Creates an `OptRule` with `name`. No short or long identifier is set.
    #[must_use]
    pub const fn new(name: &'static str) -> OptRule {
        OptRule { name, long: (null(), 0), short: '\0', val_count: 0, required: false }
    }

    /// Creates an `OptRule` whose long identifier equals `name`.
    #[must_use]
    pub const fn new_auto_long(name: &'static str) -> OptRule {
        OptRule {
            name,
            long: (name.as_ptr(), name.len()),
            short: '\0',
            val_count: 0,
            required: false
        }
    }

    /// Creates an `OptRule` with long equal to `name` and short set to the first character of
    /// `name`.
    #[must_use]
    pub const fn new_auto(name: &'static str) -> OptRule {
        OptRule {
            name,
            long: (name.as_ptr(), name.len()),
            // unsafe as this assumes first char is ascii, lazy impl which will be done better later
            short: {
                const CONT_MASK: u8 = 0b0011_1111;

                #[inline]
                const fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
                    (ch << 6) | (byte & CONT_MASK) as u32
                }

                #[inline]
                const fn first_char(bytes: &[u8]) -> u32 {
                    // Decode UTF-8
                    let x = bytes[0];
                    if x < 128 {
                        return x as u32;
                    }

                    // Multibyte case follows
                    // Decode from a byte combination out of: [[[x y] z] w]
                    // NOTE: Performance is sensitive to the exact formulation here
                    let init = (x & (0x7F >> 2)) as u32;
                    let y = bytes[1];
                    let mut ch = utf8_acc_cont_byte(init, y);
                    if x >= 0xE0 {
                        // [[x y z] w] case
                        // 5th bit in 0xE0 .. 0xEF is always clear, so `init` is still valid
                        let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, bytes[2]);
                        ch = init << 12 | y_z;
                        if x >= 0xF0 {
                            // [x y z w] case
                            // use only the lower 3 bits of `init`
                            ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, bytes[3]);
                        }
                    }

                    ch
                }

                // janky way to get a const-stable transmute in 1.48.0
                const unsafe fn transmute<Src: Copy, Dst: Copy>(s: Src) -> Dst {
                    *(&s as *const Src).cast::<Dst>()
                }

                unsafe {
                    #[allow(unnecessary_transmutes)]
                    transmute::<u32, char>(first_char(name.as_bytes()))
                }
            },
            val_count: 0,
            required: false
        }
    }

    /// Sets the long identifier.
    #[must_use]
    pub const fn set_long(mut self, long: &'static str) -> OptRule {
        self.long = (long.as_ptr(), long.len());
        self
    }

    /// Sets the short identifier.
    #[must_use]
    pub const fn set_short(mut self, short: char) -> OptRule {
        self.short = short;
        self
    }

    /// Sets the number of values this option accepts.
    #[must_use]
    pub const fn set_val_count(mut self, val_count: usize) -> OptRule {
        self.val_count = val_count;
        self
    }

    /// Sets whether this option is required.
    #[must_use]
    pub const fn set_required(mut self, required: bool) -> OptRule {
        self.required = required;
        self
    }

    /// Returns the rule's internal name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the long identifier, if any.
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

    /// Returns the short identifier, if any.
    #[must_use]
    pub const fn short(&self) -> Option<char> {
        if self.short == '\0' { None } else { Some(self.short) }
    }

    /// Returns how many values this option accepts.
    #[must_use]
    pub const fn val_count(&self) -> usize {
        self.val_count
    }

    /// Gets whether this option is required.
    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }
}

enum Argument {
    Flag,
    Opt {
        val: *const [*const u8],
        // for long=value form. the index of the first char following the = sign.
        val_offset: usize
    }
}

#[allow(clippy::inline_always)]
impl Argument {
    fn new_maybe_opt(val: *const [*const u8], val_offset: usize) -> Argument {
        if val.is_null() { Argument::Flag } else { Argument::Opt { val, val_offset } }
    }

    #[inline(always)]
    const fn val(&self) -> Option<*const [*const u8]> {
        match self {
            Argument::Flag => None,
            Argument::Opt { val, .. } => Some(*val)
        }
    }

    #[inline(always)]
    const fn val_offset(&self) -> usize {
        match self {
            Argument::Flag => 0,
            Argument::Opt { val_offset, .. } => *val_offset
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
/// An error which can occur while parsing arguments.
pub enum Error {
    /// An argument contained invalid UTF-8.
    InvalidStr(usize, Utf8Error),
    /// Parsing was successful, but by the end there were too few or too many positionals.
    WrongPositionalCount(usize),
    /// The contained required options were missing.
    MissingRequired(Vec<&'static str>),
    /// If attempting to access an option, it has no values; it was used as a flag. If attempting to
    /// access a named positional, there was no value at the correlated index.
    NoValue,
    /// If attempting to access an option, it was not specified. If attempting to access a named
    /// positional, the name was not found.
    NotFound
}

/// An iterator over the values of an option.
pub struct OptValues {
    cur: *const *const u8,
    end: *const *const u8,
    offset: usize
}

impl OptValues {
    /// Gets the element at index `i`, or `None` if the index is out-of-bounds. This does _not_
    /// consume elements like `nth`.
    #[must_use]
    #[inline]
    pub fn get(&self, i: usize) -> Option<CStr<'static>> {
        if self.len() > i { Some(unsafe { self.get_unchecked(i) }) } else { None }
    }

    /// Gets the element at index `i`. This does _not_ consume elements.
    ///
    /// # Safety
    ///
    /// The caller must ensure the element at index `i` exists and is in bounds.
    #[must_use]
    #[inline]
    pub unsafe fn get_unchecked(&self, i: usize) -> CStr<'static> {
        #[allow(clippy::cast_ptr_alignment)]
        self.cur.add(i).cast::<CStr<'static>>().read()
    }
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

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // this is probably cheaper than checking with an if statement whether offset > 0 and using
        // (1, Some(1)) in that case, so this works.
        let len = unsafe { len(self.cur, self.end) };
        (len, Some(len))
    }

    // TODO: other methods. default implementations should suck for this but i don't feel like
    //  implementing them rn
}

impl ExactSizeIterator for OptValues {
    fn len(&self) -> usize {
        unsafe { len(self.cur, self.end) }
    }
}
