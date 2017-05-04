pub enum DocElem {
    Literal(&'static str),
    Text(String),
    Newline(i32),
    Flatable(Vec<DocElem>)
}

pub fn literal(s: &'static str) -> DocElem {DocElem::Literal(s)}
pub fn text(s: String) -> DocElem {DocElem::Text(s)}
pub fn newline(indent: i32) -> DocElem{DocElem::Newline(indent)}
pub fn flatable(ds: Vec<DocElem>) -> DocElem{DocElem::Flatable(ds)}


pub struct Doc(Vec<DocElem>);

impl Doc {
    pub fn new(x: Vec<DocElem>) -> Doc {Doc(x)}

    pub fn pretty(&self, width: i32) -> String {
        fn pretty_walk(ds: &Vec<DocElem>, width: i32, rest_width: &mut i32, indent: &mut i32, ret: &mut String) {
            for d in ds {
                match *d {
                    DocElem::Literal(ref s) => {
                        // println!("literal {} (rest_width: {}", s, rest_width);
                        *rest_width -= s.len() as i32;
                        ret.push_str(s);
                    }
                    DocElem::Text(ref s) => {
                        // println!("text {} (rest_width: {}", s, rest_width);
                        *rest_width -= s.len() as i32;
                        ret.push_str(s.as_str());
                    },
                    DocElem::Newline(i) => {
                        // println!("newline {} (rest_width: {}", i, rest_width);
                        *indent += i;
                        *rest_width = width - *indent;
                        ret.push('\n');
                        for _ in 0..*indent {ret.push(' ')}
                    },
                    DocElem::Flatable(ref ds2) => {
                        // println!("flat: ({} <= {}) `{}`", flat_doc_width(&ds2), rest_width, flatten_print(&ds2));
                        if flat_doc_width(&ds2) <= *rest_width {
                            let fstr = flatten_print(&ds2);
                            ret.push_str(fstr.as_str());
                            *rest_width -= ret.len() as i32;
                        } else {
                            pretty_walk(&ds2, width, rest_width, indent, ret)
                        }
                    }
                }
            }
        }
        let mut ret = String::new();
        pretty_walk(&self.0, width, &mut width.clone(), &mut 0, &mut ret);
        ret
    }
}

fn flatten_print(vdocs: &Vec<DocElem>) -> String {
    fn flatten_walk(ds: &Vec<DocElem>, ret: &mut String) {
        for d in ds {
            match *d {
                DocElem::Literal(ref s) => ret.push_str(s),
                DocElem::Text(ref s) => ret.push_str(s.as_ref()),
                DocElem::Newline(_) => ret.push(' '),
                DocElem::Flatable(ref ds2) => flatten_walk(&ds2, ret)
            }
        }
    }
    let mut ret = String::new();
    flatten_walk(vdocs, &mut ret);
    ret
}

fn flat_doc_width(vdocs: &Vec<DocElem>) -> i32 {
    fn flat_doc_width_walk(vdocs: &Vec<DocElem>) -> i32{
        let mut sum = 0;
        for d in vdocs.iter() {
            match *d {
                DocElem::Literal(ref s) => sum += s.len() as i32,
                DocElem::Text(ref s) => sum += s.len() as i32,
                DocElem::Newline(_) => sum += 1,
                DocElem::Flatable(ref ds) => sum += flat_doc_width_walk(&ds)
            }
        }
        sum
    }
    flat_doc_width_walk(vdocs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newline() {
        let doc = Doc::new(vec![newline(1), newline(1), newline(1)]);
        assert_eq!(doc.pretty(0), "\n \n  \n   ")
    }

    #[test]
    fn test_pretty() {
        let doc = Doc::new(vec![flatable(vec![
            text("BEGIN".to_string()),
            newline(2),
            flatable(vec![
                literal("foo {"), newline(2), literal("bar"), newline(-2), literal("}")
            ]),
            literal(","),
            newline(0),
            flatable(vec![
                literal("1"), newline(0), literal("2"), newline(0), literal("3"), newline(0), literal("4")
            ]),
            newline(-2),
            text("END".to_string())
        ])]);
        assert_eq! {
            doc.pretty(0),
            r#"BEGIN
  foo {
    bar
  },
  1
  2
  3
  4
END"#.to_string()
        }
        assert_eq! {
            doc.pretty(30),
            "BEGIN foo { bar }, 1 2 3 4 END".to_string()
        }
        assert_eq! {
            doc.pretty(29),
            r#"BEGIN
  foo { bar },
  1 2 3 4
END"#.to_string()
        }
        // assert_eq! {
        //     doc.pretty(9),
        //     "foo bar,\n  1 2 3 4".to_string()
        // }
    }
}
