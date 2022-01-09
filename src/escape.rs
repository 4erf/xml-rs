//! Contains functions for performing XML special characters escaping.

use std::borrow::Cow;
use std::collections::HashMap;

enum Value {
    Char(char),
    Str(&'static str)
}

impl Value {
    fn dispatch_for_attribute(c: char, extra: &HashMap<char, &'static str>) -> Value {
        match c {
            '<'  => Value::Str("&lt;"),
            '>'  => Value::Str("&gt;"),
            '"'  => Value::Str("&quot;"),
            '\'' => Value::Str("&apos;"),
            '&'  => Value::Str("&amp;"),
            '\n' => Value::Str("&#xA;"),
            '\r' => Value::Str("&#xD;"),
            _    => match extra.get(&c) {
                Some(value) => Value::Str(value),
                None => Value::Char(c)
            }
        }
    }

    fn dispatch_for_pcdata(c: char, extra: &HashMap<char, &'static str>) -> Value {
        match c {
            '<'  => Value::Str("&lt;"),
            '&'  => Value::Str("&amp;"),
            _    => match extra.get(&c) {
                Some(value) => Value::Str(value),
                None => Value::Char(c)
            }
        }
    }
}

enum Process<'a> {
    Borrowed(&'a str),
    Owned(String)
}

impl<'a> Process<'a> {
    fn process(&mut self, (i, next): (usize, Value)) {
        match next {
            Value::Str(s) => match *self {
                Process::Owned(ref mut o) => o.push_str(s),
                Process::Borrowed(b) => {
                    let mut r = String::with_capacity(b.len() + s.len());
                    r.push_str(&b[..i]);
                    r.push_str(s);
                    *self = Process::Owned(r);
                }
            },
            Value::Char(c) => match *self {
                Process::Borrowed(_) => {}
                Process::Owned(ref mut o) => o.push(c)
            }
        }
    }

    fn into_result(self) -> Cow<'a, str> {
        match self {
            Process::Borrowed(b) => Cow::Borrowed(b),
            Process::Owned(o) => Cow::Owned(o)
        }
    }
}

impl<'a> Extend<(usize, Value)> for Process<'a> {
    fn extend<I: IntoIterator<Item=(usize, Value)>>(&mut self, it: I) {
        for v in it.into_iter() {
            self.process(v);
        }
    }
}

fn escape_str<'a>(
    s: &'a str,
    dispatch: fn(char, &HashMap<char, &'static str>) -> Value,
    extra: &HashMap<char, &'static str>
) -> Cow<'a, str> {
    let mut p = Process::Borrowed(s);
    p.extend(s.char_indices().map(|(ind, c)| (ind, dispatch(c, extra))));
    p.into_result()
}

/// Performs escaping of common XML characters inside an attribute value.
///
/// This function replaces several important markup characters with their
/// entity equivalents:
///
/// * `<` → `&lt;`
/// * `>` → `&gt;`
/// * `"` → `&quot;`
/// * `'` → `&apos;`
/// * `&` → `&amp;`
///
/// The resulting string is safe to use inside XML attribute values or in PCDATA sections.
///
/// Does not perform allocations if the given string does not contain escapable characters.
#[inline]
pub fn escape_str_attribute<'a>(s: &'a str, extra: &HashMap<char, &'static str>) -> Cow<'a, str> {
    escape_str(s, Value::dispatch_for_attribute, extra)
}

/// Performs escaping of common XML characters inside PCDATA.
///
/// This function replaces several important markup characters with their
/// entity equivalents:
///
/// * `<` → `&lt;`
/// * `&` → `&amp;`
///
/// The resulting string is safe to use inside PCDATA sections but NOT inside attribute values.
///
/// Does not perform allocations if the given string does not contain escapable characters.
#[inline]
pub fn escape_str_pcdata<'a>(s: &'a str, extra: &HashMap<char, &'static str>) -> Cow<'a, str> {
    escape_str(s, Value::dispatch_for_pcdata, extra)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::{escape_str_pcdata, escape_str_attribute};

    // TODO: add more tests

    #[test]
    fn test_escape_multibyte_code_points() {
        assert_eq!(escape_str_attribute("☃<", &HashMap::new()), "☃&lt;");
        assert_eq!(escape_str_pcdata("☃<", &HashMap::new()), "☃&lt;");
        assert_eq!(escape_str_attribute(
            "☃<.",
            &HashMap::from([('.', "&period;")])
        ), "☃&lt;&period;");
        assert_eq!(escape_str_pcdata(
            "☃<.",
            &HashMap::from([('.', "&period;")])
        ), "☃&lt;&period;");
    }
}

