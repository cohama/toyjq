use super::parsercombinator::*;
use super::prettyprinter::*;

#[derive(Debug, PartialEq)]
pub enum Json<'a> {
    JNumber(f64),
    JString(&'a str),
    JBool(bool),
    JNull,
    JArray(Vec<Json<'a>>),
    JObject(Vec<(&'a str, Json<'a>)>) // To preserve input order, use Vec instead of HashMap
}

impl <'a> Json<'a> {
    pub fn from_str(s: &str) -> Result<Json, ParseError> {
        parse_json().parse(s)
    }

    pub fn pretty_print(&self, width: i32) -> String {
        Doc::new(vec![json_to_doc_elem(&self)]).pretty(width)
    }
}

fn parse_json<'a>() -> Parser<'a, Json<'a>> {
    parse_jarray()
        .or_lazy(||parse_jobject())
        .or_lazy(||parse_jstring())
        .or_lazy(||parse_jnull())
        .or_lazy(||parse_jbool())
        .or_lazy(||parse_jnumber())
}

fn parse_jbool<'a>() -> Parser<'a, Json<'a>> {
    string("true").map(|_|Json::JBool(true)).try()
        .or(string("false").map(|_|Json::JBool(false))).try()
}

fn parse_jnull<'a>() -> Parser<'a, Json<'a>> {
    string("null").map(|_|Json::JNull).try()
}

fn parse_jnumber<'a>() -> Parser<'a, Json<'a>> {
    or_from("-0123456789.Ee+".chars().map(chr))
        .many().try().flat_map(|v| {
            let s: String = v.iter().collect();
            if let Ok(d) = s.as_str().parse::<f64>() {
                unit(d).map(Json::JNumber)
            } else {
                failure(format!("Unable to parse a number: {}", s)).map(|_| Json::JNull)
            }
        })
}

fn parse_string<'a>() -> Parser<'a, &'a str> {
    chr('"').then_lazy(||until("\"")).skip(chr('"'))
}

fn parse_jstring<'a>() -> Parser<'a, Json<'a>> {
    parse_string().map(Json::JString)
}

fn parse_keyvalue<'a>() -> Parser<'a, (&'a str, Json<'a>)> {
    parse_string().skip(chr(':').with_spaces()).and_lazy(||parse_json())
}

fn parse_jobject<'a>() -> Parser<'a, Json<'a>> {
    chr('{').with_spaces().then_lazy(||
        parse_keyvalue().sep_by(chr(',').with_spaces())
    ).skip(chr('}').with_spaces()).map(|v|Json::JObject(v.into_iter().collect()))
}

fn parse_jarray<'a>() -> Parser<'a, Json<'a>> {
    chr('[').with_spaces().then_lazy(||
        parse_json().sep_by(chr(',').with_spaces())
    ).skip(chr(']').with_spaces()).map(Json::JArray)
}

const INDENT_DEPTH: i32 = 2;

fn json_to_doc_elem(json: &Json) -> DocElem {
    match *json {
        Json::JNumber(v) => text(format!("{}", v)),
        Json::JString(s) => text(format!("\"{}\"", s)),
        Json::JBool(true) => literal("true"),
        Json::JBool(false) => literal("false"),
        Json::JNull => literal("null"),
        Json::JArray(ref jsons) => json_vec_to_flatable_doc_elem(jsons),
        Json::JObject(ref obj) => json_object_to_flatable_doc_elem(obj)
    }
}

fn json_vec_to_flatable_doc_elem(jsons: &Vec<Json>) -> DocElem {
    if jsons.is_empty() {
        literal("[]")
    } else {
        let mut it = jsons.iter();
        let mut ret = vec![literal("["), newline(INDENT_DEPTH)];
        ret.push(json_to_doc_elem(it.next().unwrap()));
        while let Some(j) = it.next() {
            ret.push(literal(","));
            ret.push(newline(0));
            ret.push(json_to_doc_elem(j));
        }
        ret.push(newline(-2));
        ret.push(literal("]"));
        flatable(ret)
    }
}

fn json_object_to_flatable_doc_elem(obj: &Vec<(&str, Json)>) -> DocElem {
    if obj.is_empty() {
        literal("{}")
    } else {
        let mut it = obj.iter();
        let mut ret = vec![literal("{"), newline(INDENT_DEPTH)];
        let kv0 = it.next().unwrap();
        ret.append(&mut json_keyvalue_to_doc_elems(kv0));
        while let Some(kv) = it.next() {
            ret.push(literal(","));
            ret.push(newline(0));
            ret.append(&mut json_keyvalue_to_doc_elems(kv));
        }
        ret.push(newline(-2));
        ret.push(literal("}"));
        flatable(ret)
    }
}

fn json_keyvalue_to_doc_elems(keyvalue: &(&str, Json)) -> Vec<DocElem> {
    let (ref k, ref v) = *keyvalue;
    vec![
        text(format!("\"{}\"", k)),
        literal(": "),
        json_to_doc_elem(v)
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_pretty_print() {
        use self::Json::*;
        let json = JArray(vec![
            JNumber(42f64),
            JString("foo"),
            JBool(true),
            JBool(false),
            JArray(vec![]),
            JArray(vec![JNull]),
            JObject(vec![]),
            JObject(vec![("poem", JString("Lorem ipsum"))]),
            JObject(vec![
                ("a", JNumber(1f64)),
                ("foo-bar-baz", JString("1 2 Fizz 4 Buzz 6 7 8 Fizz Buzz")),
                ("Numbers", JArray((1..20).map(|i: i32| JNumber(i as f64)).collect()))
            ])
        ]);
        assert_eq! {
            json.pretty_print(1),
            r#"[
  42,
  "foo",
  true,
  false,
  [],
  [
    null
  ],
  {},
  {
    "poem": "Lorem ipsum"
  },
  {
    "a": 1,
    "foo-bar-baz": "1 2 Fizz 4 Buzz 6 7 8 Fizz Buzz",
    "Numbers": [
      1,
      2,
      3,
      4,
      5,
      6,
      7,
      8,
      9,
      10,
      11,
      12,
      13,
      14,
      15,
      16,
      17,
      18,
      19
    ]
  }
]"#
        }
        assert_eq! {
            json.pretty_print(84),
            r#"[
  42,
  "foo",
  true,
  false,
  [],
  [ null ],
  {},
  { "poem": "Lorem ipsum" },
  {
    "a": 1,
    "foo-bar-baz": "1 2 Fizz 4 Buzz 6 7 8 Fizz Buzz",
    "Numbers": [ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19 ]
  }
]"#
        }
        assert_eq! {
            json.pretty_print(215),
            r#"[ 42, "foo", true, false, [], [ null ], {}, { "poem": "Lorem ipsum" }, { "a": 1, "foo-bar-baz": "1 2 Fizz 4 Buzz 6 7 8 Fizz Buzz", "Numbers": [ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19 ] } ]"#
        }
    }

    #[test]
    fn test_parse_json() {
        assert_eq! {
            Json::from_str("123").unwrap(),
            Json::JNumber(123f64)
        }
        assert_eq! {
            Json::from_str("\"fooo\"").unwrap(),
            Json::JString("fooo")
        }
        assert_eq! {
            Json::from_str("[1, -2, 3.0E4, true, false, null]").unwrap(),
            Json::JArray(vec! {
                Json::JNumber(1f64),
                Json::JNumber(-2f64),
                Json::JNumber(30000f64),
                Json::JBool(true),
                Json::JBool(false),
                Json::JNull,
            })
        }
        assert_eq! {
            Json::from_str("{\"key1\" : 123, \"key2\" : \"foo\"}").unwrap(),
            Json::JObject(vec! {
                ("key1", Json::JNumber(123f64)),
                ("key2", Json::JString("foo"))
            })
        }
        assert_eq! {
            Json::from_str(r#"
[
    {
        "key1" : 123,
        "key2" : "foo"
    },
    123,
    ["foo", true]
]
"#).unwrap(),
            Json::JArray(vec! {
                Json::JObject(vec! {
                    ("key1", Json::JNumber(123f64)),
                    ("key2", Json::JString("foo"))
                }),
                Json::JNumber(123f64),
                Json::JArray(vec! {
                    Json::JString("foo"),
                    Json::JBool(true)
                })
            })
        }
        assert_eq! {
            {
                let ParseError {retry, message: _, pos} = Json::from_str("[[null, null ],[null ,null      null] , [ null ] ] ").unwrap_err();
                (retry, pos)
            },
            (false, 26)
        }
    }
}
