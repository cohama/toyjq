
#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    pub retry: bool,
    pub message: String
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrStream<'a> {
    body: &'a str,
    pos: usize
}

impl <'a> StrStream<'a> {
    fn new(body: &'a str) -> StrStream<'a> {
        StrStream {body, pos: 0}
    }

    fn can_advance(&self) -> bool {
        self.pos < self.body.len()
    }

    fn current(&self) -> &str {
        &self.body[self.pos..self.body.len()]
    }

    fn take(&'a self, n: usize) -> &'a str {
        use std::cmp::min;
        let cr = self.current();
        &cr[0..min(cr.len(), n)]
    }

    fn advance(mut self, n: usize) -> StrStream<'a> {
        self.pos += n;
        self
    }
}

impl <'a> Into<StrStream<'a>> for &'a str {
    fn into(self) -> StrStream<'a> {
        StrStream::new(self)
    }
}

impl <'a> Into<StrStream<'a>> for &'a String {
    fn into(self) -> StrStream<'a> {
        StrStream::new(self.as_str())
    }
}



type ParseResult<'a, T> = Result<(StrStream<'a>, T), ParseError>;

pub struct Parser<'a, T>(Box<Fn(StrStream<'a>) -> ParseResult<'a, T> + 'a>);


/// Creates a new Parser which returns the specified value.
///
/// ```
/// # use toyjq::parser::*;
/// assert_eq!(unit(42).parse("").unwrap(), 42);
/// ```
pub fn unit<'a, T>(x: T) -> Parser<'a, T>
    where T: Copy + 'a
{
    Parser(Box::new(move |i| {
        Ok((i, x))
    }))
}

/// Parses literal string.
///
/// ```
/// # use toyjq::parser::*;
/// assert_eq!(string("foo").parse("fooo").unwrap(), "foo");
/// ```
pub fn string<'a>(s: &'static str) -> Parser<'a, &'static str> {
    Parser(Box::new(move |input| {
        if input.can_advance() {
            let len = s.len();
            let heads = input.take(len);
            if s == heads {
                Ok((input.advance(len), s))
            } else {
                Err(ParseError {
                    retry: false,
                    message: format!("Expected `{}` but actual is `{}`.", s, heads)
                })
            }
        } else {
            Err(ParseError {
                retry: false,
                message: "Reaches end.".to_string()
            })
        }
    }))
}

/// Parsers single character.
///
/// ```
/// # use toyjq::parser::*;
/// assert_eq!(chr('f').parse("foo").unwrap(), 'f');
/// ```
pub fn chr<'a>(c: char) -> Parser<'a, char> {
    Parser(Box::new(move |input| {
        if input.can_advance() {
            let head = input.take(1).chars().next().unwrap();
            if c == head {
                Ok((input.advance(1), c))
            } else {
                Err(ParseError {
                    retry: false,
                    message: format!("Expected `{}` but actual is `{}`.", c, head)
                })
            }
        } else {
            Err(ParseError {
                retry: false,
                message: "Reaches end.".to_string()
            })
        }
    }))
}

/// ```
/// # use toyjq::parser::*;
/// assert_eq!(failure(format!("failed")).parse("").unwrap_err().message, "failed");
/// ```
pub fn failure<'a>(message: String) -> Parser<'a, ()> {
    Parser(Box::new(move |_| {
        Err(ParseError {
            retry: false,
            message: message.clone()
        })
    }))
}


/// Parses any string till the specified string appears.
///
/// ```
/// # use toyjq::parser::*;
/// assert_eq!(until("!").parse("foo bar!").unwrap(), "foo bar");
/// ```
pub fn until<'a>(s: &'a str) -> Parser<'a, &'a str> {
    Parser(Box::new(move |input| {
        let initpos = input.pos;
        let mut i = input;
        while i.can_advance() {
            let len = s.len();
            if s == i.take(len) {
                return Ok((i, &i.body[initpos..i.pos]))
            } else {
                i = i.advance(1);
            }
        }
        Err(ParseError {
            retry: false,
            message: "Reaches end.".to_string()
        })
    }))
}


/// Chains `or` opeartion
///
/// ```
/// # use toyjq::parser::*;
/// assert_eq!(or_from("abcdef".chars().map(chr)).parse("fff").unwrap(), 'f');
/// ```
pub fn or_from<'a, T, Ps>(ps: Ps) -> Parser<'a, T>
    where Ps: IntoIterator<Item = Parser<'a, T>>,
          T: 'a
{
    let mut piter = ps.into_iter();
    let p0 = piter.next().unwrap();
    piter.fold(p0, |acc, p| {
        acc.try().or(p)
    })
}


impl <'a, T> Parser<'a, T>
    where T: 'a
{

    fn run(&self, input: StrStream<'a>) -> ParseResult<'a, T> {
        (self.0)(input)
    }

    /// Runs parser with the specified input.
    /// input type will be &str or &String. (these implement Into<StrStream>)
    pub fn parse<S>(&self, input: S) -> Result<T, ParseError>
        where S: Into<StrStream<'a>>
    {
        let (_, v) = self.run(input.into())?;
        Ok(v)
    }

    /// ```
    /// # use toyjq::parser::*;
    /// assert_eq!(unit(42).map(|x|x+1).parse("").unwrap(), 43);
    /// ```
    pub fn map<F, U>(self, f: F) -> Parser<'a, U>
        where F: Fn(T) -> U + 'a,
              U: 'a
    {
        Parser(Box::new(move |input| {
            let (input2, x) = self.run(input)?;
            Ok((input2, f(x)))
        }))
    }

    /// Like `map` but do not use former result.
    ///
    /// ```
    /// # use toyjq::parser::*;
    /// assert_eq!(unit(42).map_(1).parse("").unwrap(), 1);
    /// ```
    pub fn map_<U>(self, x: U) -> Parser<'a, U>
        where U: Copy + 'a
    {
        Parser(Box::new(move |input| {
            let (input2, _) = self.run(input)?;
            Ok((input2, x))
        }))
    }

    /// ```
    /// # use toyjq::parser::*;
    /// assert_eq!(unit('f').flat_map(chr).parse("foo").unwrap(), 'f');
    /// ```
    pub fn flat_map<F, U>(self, f: F) -> Parser<'a, U>
        where F: Fn(T) -> Parser<'a, U> + 'a,
              U: 'a
    {
        Parser(Box::new(move |input| {
            let (input2, o) = try! { self.run(input) };
            f(o).run(input2)
        }))
    }

    /// p1 then p2
    /// First, parses p1 but abandons its result and then parses p2.
    ///
    /// ```
    /// # use toyjq::parser::*;
    /// assert_eq!(chr('[').then(string("foo")).parse("[foo]").unwrap(), "foo");
    /// ```
    pub fn then<U>(self, p: Parser<'a, U>) -> Parser<'a, U>
        where U: 'a
    {
        Parser(Box::new(move |input| {
            let (input2, _) = try! { self.run(input) };
            p.run(input2)
        }))
    }

    /// Like then but be lazy.
    ///
    /// ```
    /// # use toyjq::parser::*;
    /// assert_eq!(chr('[').then_lazy(||string("foo")).parse("[foo]").unwrap(), "foo");
    /// ```
    pub fn then_lazy<F, U>(self, f: F) -> Parser<'a, U>
        where F: Fn() -> Parser<'a, U> + 'a,
              U: 'a
    {
        self.flat_map(move |_|f())
    }

    /// ```
    /// # use toyjq::parser::*;
    /// assert_eq!(string("foo").skip(chr(';')).parse("foo;").unwrap(), "foo");
    /// ```
    pub fn skip<U>(self, p: Parser<'a, U>) -> Parser<'a, T>
        where U: 'a
    {
        Parser(Box::new(move |input| {
            match self.run(input) {
                Ok((input2, v)) => p.run(input2).map(|(input3, _)| (input3, v)),
                Err(e) => Err(e)
            }
        }))
    }

    /// p1 and p2
    /// parse both p1 and p2 and make tuple from these results.
    ///
    /// ```
    /// # use toyjq::parser::*;
    /// assert_eq!(chr('[').and(string("foo")).parse("[foo]").unwrap(), ('[', "foo"));
    /// ```
    pub fn and<U>(self, p: Parser<'a, U>) -> Parser<'a, (T, U)>
        where U: 'a
    {
        Parser(Box::new(move |input| {
            let (input2, o) = try!{ self.run(input) };
            let (input3, o2) = try!{ p.run(input2) };
            Ok((input3, (o, o2)))
        }))
    }


    /// Like and but be lazy.
    ///
    /// ```
    /// # use toyjq::parser::*;
    /// assert_eq!(chr('[').and_lazy(||string("foo")).parse("[foo]").unwrap(), ('[', "foo"));
    /// ```
    pub fn and_lazy<F, U>(self, f: F) -> Parser<'a, (T, U)>
        where F: Fn() -> Parser<'a, U> + 'a,
              U: 'a
    {
        Parser(Box::new(move |input| {
            let (input2, o) = try!{ self.run(input) };
            let (input3, o2) = try!{ f().run(input2) };
            Ok((input3, (o, o2)))
        }))
    }


    /// p1 or p2
    /// when p1 is failed and retry flag is true, then p2 will run.
    ///
    /// ```
    /// # use toyjq::parser::*;
    /// assert_eq!(string("foo").try().or(string("bar")).parse("bar").unwrap(), "bar");
    /// ```
    pub fn or(self, that: Self) -> Self {
        Parser(Box::new(move |input| {
            match self.run(input) {
                Ok(o) => Ok(o),
                Err(ParseError {retry, message: _}) if retry => that.run(input),
                Err(e) => Err(e)
            }
        }))
    }

    /// Like `or` but be lazy.
    ///
    /// ```
    /// # use toyjq::parser::*;
    /// assert_eq!(string("foo").try().or_lazy(||string("bar")).parse("bar").unwrap(), "bar");
    /// ```
    pub fn or_lazy<F>(self, that: F) -> Self
        where F: Fn() -> Self + 'a
    {
        Parser(Box::new(move |input| {
            match self.run(input) {
                Ok(o) => Ok(o),
                Err(ParseError {retry, message: _}) if retry => that().run(input),
                Err(e) => Err(e)
            }
        }))
    }

    /// Parses optional phrase.
    ///
    /// ```
    /// # use toyjq::parser::*;
    /// let p = chr('-').or_not().and(string("123"));
    /// assert_eq!(p.parse("-123").unwrap(), (Some('-'), "123"));
    /// assert_eq!(p.parse("123").unwrap(), (None, "123"));
    /// ```
    pub fn or_not(self) -> Parser<'a, Option<T>> {
        Parser(Box::new(move |input| {
            match self.run(input) {
                Ok((input2, v)) => Ok((input2, Some(v))),
                Err(_) => Ok((input, None))
            }
        }))
    }

    /// Parsing with backtracking.
    ///
    /// ```
    /// # use toyjq::parser::*;
    /// assert!(string("foo").or(string("bar")).parse("bar").is_err());
    /// assert_eq!(string("foo").try().or(string("bar")).parse("bar").unwrap(), "bar");
    /// ```
    pub fn try(self) -> Parser<'a, T> {
        Parser(Box::new(move |input| {
            self.run(input).map_err(|ParseError {retry: _, message}| {
                ParseError {retry: true, message}
            })
        }))
    }

    /// Parses any phrase repeatedly (0 or more)
    ///
    /// ```
    /// # use toyjq::parser::*;
    /// assert_eq!(string("foo").many().parse("foofoofoo").unwrap(), vec!["foo", "foo", "foo"]);
    /// ```
    pub fn many(self) -> Parser<'a, Vec<T>> {
        Parser(Box::new(move |input| {
            let mut v = vec![];
            let mut i = input;
            loop {
                match self.run(i) {
                    Ok((input2, o)) => {
                        v.push(o);
                        i = input2;
                    },
                    Err(_) => break
                }
            }
            Ok((i, v))
        }))
    }

    /// Parses any phrase separated by delimitor repeatedly (0 or more).
    ///
    /// ```
    /// # use toyjq::parser::*;
    /// assert_eq!(string("foo").sep_by(string(", ")).parse("foo, foo, foo").unwrap(), vec!["foo", "foo", "foo"]);
    /// ```
    pub fn sep_by<O2>(self, delim: Parser<'a, O2>) -> Parser<'a, Vec<T>>
        where O2: 'a
    {
        Parser(Box::new(move |input| {
            let mut v = vec![];
            let mut i = input;
            loop {
                match self.run(i) {
                    Ok((input2, o)) => {
                        v.push(o);
                        i = input2;
                        match delim.run(i) {
                            Ok((input3, _)) => i = input3,
                            Err(_) => break
                        }
                    },
                    Err(_) => break
                }
            }
            Ok((i, v))
        }))
    }

    pub fn with_spaces(self) -> Self {
        chr(' ').many().then(self).skip(chr(' ').many())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum Expr {
        Num(i32),
        Add(Box<(Expr, Expr)>)
    }

    fn parse_digit<'a>() -> Parser<'a, i32> {
        chr('0').map_(0).try().or(
            chr('-').or_not()
            .and(or_from("123456789".chars().map(chr)))
            .and(or_from("0123456789".chars().map(chr)).many())
            .map(|((negate, head), tail)| {
                let mut st = String::new();
                let minus = if negate.is_some() { "-" } else { "" };
                st.push_str(minus);
                st.push(head);
                for c in tail {
                    st.push(c)
                }
                i32::from_str_radix(st.as_str(), 10).unwrap()
            })
        )
    }

    fn parse_num<'a>() -> Parser<'a, Expr> {
        parse_digit().map(Expr::Num)
    }

    fn parse_add<'a>() -> Parser<'a, Expr> {
        chr('(').with_spaces().then_lazy(|| {
            parse_expr().and_lazy(||
                chr('+').with_spaces()
                .then(parse_expr())
            ).map(|(lhs, rhs)| {
                    Expr::Add(Box::new((lhs, rhs)))
            })
        }).skip(chr(')'))
    }

    fn parse_expr<'a>() -> Parser<'a, Expr> {
        parse_add().try().or_lazy(||parse_num())
    }

    #[test]
    fn test_parser() {
        assert_eq!(parse_digit().parse("0").unwrap(), 0);
        assert_eq!(parse_digit().parse("1").unwrap(), 1);
        assert_eq!(parse_digit().parse("123").unwrap(), 123);
        assert_eq!(parse_digit().parse("-999123").unwrap(), -999123);

        assert_eq!(parse_num().parse("-987654321").unwrap(), Expr::Num(-987654321));

        assert_eq! {
            parse_expr().parse("-987654321").unwrap(),
            Expr::Num(-987654321)
        };
        assert_eq! {
            parse_expr().parse("(1 + 2)").unwrap(),
            Expr::Add(Box::new((Expr::Num(1), Expr::Num(2))))
        };
        assert_eq! {
            parse_expr().parse("((1 + 2) + ((3 + 4) + 5))").unwrap(),
            Expr::Add(Box::new((
                Expr::Add(Box::new((
                    Expr::Num(1), Expr::Num(2)
                ))),
                Expr::Add(Box::new((
                    Expr::Add(Box::new((
                        Expr::Num(3), Expr::Num(4)
                    ))),
                    Expr::Num(5)
                )))
            )))
        };
    }

}
