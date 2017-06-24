#[macro_export]
macro_rules! try_opt {
    ($expr:expr) => (match $expr {
        Option::Some(val) => val,
        Option::None => {
            return Option::None
        }
    })
}

////////////////////////////////////////////////////////////////////////////////

pub struct InsertIterator<'iter, A: 'iter> {
    insert_point: usize,
    current_idx: usize,
    iter_orig: &'iter mut Iterator<Item=A>,
    iter_insert: &'iter mut Iterator<Item=A>,
}

impl<'iter, A> Iterator for InsertIterator<'iter, A> {
    type Item = A;

    fn next(&mut self) -> Option<A> {
        if self.current_idx >= self.insert_point {
            if let Some(a) = self.iter_insert.next() {
                Some(a)
            } else {
                self.iter_orig.next()
            }
        } else {
            self.current_idx += 1;
            self.iter_orig.next()
        }
    }
}

pub fn insert_iter<'iter, A>(iter_orig: &'iter mut Iterator<Item=A>,
                             iter_insert: &'iter mut Iterator<Item=A>,
                             insert_point: usize)
                             -> InsertIterator<'iter, A> {
    InsertIterator {
        insert_point: insert_point,
        current_idx: 0,
        iter_orig: iter_orig,
        iter_insert: iter_insert,
    }
}

////////////////////////////////////////////////////////////////////////////////

use std::str::SplitWhitespace;

/// Like `std::str::SplitWhitespace`, but returns beginning indices rather than slices.
pub struct SplitWhitespaceIndices<'a> {
    inner: SplitWhitespace<'a>,
    str: &'a str,
}

impl<'a> Iterator for SplitWhitespaceIndices<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        match self.inner.next() {
            Some(str) =>
                self.str.as_ptr().offset_to(str.as_ptr()).map(|i| i as usize),
            None =>
                None,
        }
    }
}

pub fn split_whitespace_indices(str: &str) -> SplitWhitespaceIndices {
    SplitWhitespaceIndices {
        inner: str.split_whitespace(),
        str: str
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct SplitNicks<'a> {
    str: &'a str,
}

// forall s . split_nicks(s).join("") == s
impl<'a> Iterator for SplitNicks<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let mut cis = self.str.char_indices();
        match cis.next() {
            None => None,
            Some((mut i0, c)) => {
                if is_nick_char(c) {
                    while let Some((i, c)) = cis.next() {
                        if !is_nick_char(c) {
                            break;
                        }
                        i0 = i;
                    }
                } else {
                    while let Some((i, c)) = cis.next() {
                        if is_nick_char(c) {
                            break;
                        }
                        i0 = i;
                    }
                }
                let ret = &self.str[..i0 + c.len_utf8()];
                self.str = &self.str[i0 + c.len_utf8() ..];
                Some(ret)
            }
        }
    }
}

pub fn split_nicks(str: &str) -> SplitNicks {
    SplitNicks {
        str: str,
    }
}

pub fn is_nick_char(c: char) -> bool {
    (c >= 'a' && c <= 'z')
        || (c >= 'A' && c <= 'Z')
        || (c >= '0' && c <= '9')
        || c == '-'
        || c == '['
        || c == ']'
        || c == '\\'
        || c == '`'
        || c == '^'
        || c == '{'
        || c == '}'
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {

    extern crate test;

    use super::*;

    #[test]
    fn insert_iter_test() {
        let mut range1 = 0 .. 5;
        let mut range2 = 5 .. 10;
        let iter = insert_iter(&mut range1, &mut range2, 3);
        assert_eq!(iter.collect::<Vec<i32>>(), vec![0, 1, 2, 5, 6, 7, 8, 9, 3, 4])
    }

    #[test]
    fn split_ws_idx() {
        let str = "x y z";
        let idxs: Vec<usize> = split_whitespace_indices(str).into_iter().collect();
        assert_eq!(idxs, vec![0, 2, 4]);

        let str = "       ";
        let idxs: Vec<usize> = split_whitespace_indices(str).into_iter().collect();
        let expected: Vec<usize> = vec![];
        // Next line fails when I inline `expected`. Probably a rustc bug.
        assert_eq!(idxs, expected);

        let str = "  foo    bar  \n\r   baz     ";
        let idxs: Vec<usize> = split_whitespace_indices(str).into_iter().collect();
        assert_eq!(idxs, vec![2, 9, 19]);
    }

    #[test]
    fn split_nicks_test() {
        let str = "a";
        let nicks: Vec<&str> = split_nicks(str).into_iter().collect();
        assert_eq!(nicks, vec!["a"]);

        let str = "foo bar baz";
        let nicks: Vec<&str> = split_nicks(str).into_iter().collect();
        assert_eq!(nicks, vec!["foo", " ", "bar", " ", "baz"]);

        let str = "";
        let nicks: Vec<&str> = split_nicks(str).into_iter().collect();
        let expected: Vec<&str> = vec![];
        assert_eq!(nicks, expected);

        let str = "  foo  bar   baz  ";
        let nicks: Vec<&str> = split_nicks(str).into_iter().collect();
        assert_eq!(nicks, vec!["  ", "foo", "  ", "bar", "   ", "baz", "  "]);
    }
}
