use super::*;

use std::collections::HashMap;

pub fn parse_json<'a>() -> Parser<'a, Json<'a>> {
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
    chr('"').try().then_lazy(||until("\"")).skip(chr('"'))
}

fn parse_jstring<'a>() -> Parser<'a, Json<'a>> {
    parse_string().map(Json::JString)
}

fn parse_keyvalue<'a>() -> Parser<'a, (&'a str, Json<'a>)> {
    parse_string().skip(chr(':').with_spaces()).and_lazy(||parse_json())
}

fn parse_jobject<'a>() -> Parser<'a, Json<'a>> {
    chr('{').with_spaces().try().then_lazy(||
        parse_keyvalue().sep_by(chr(',').with_spaces())
    ).skip(chr('}').with_spaces()).map(|v|Json::JObject(v.into_iter().collect()))
}

fn parse_jarray<'a>() -> Parser<'a, Json<'a>> {
    chr('[').with_spaces().try().then_lazy(||
        parse_json().sep_by(chr(',').with_spaces())
    ).skip(chr(']').with_spaces()).map(Json::JArray)
}

#[derive(Debug, PartialEq)]
pub enum Json<'a> {
    JNumber(f64),
    JString(&'a str),
    JBool(bool),
    JNull,
    JArray(Vec<Json<'a>>),
    JObject(HashMap<&'a str, Json<'a>>)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::iter::FromIterator;

    #[test]
    fn test_parse_json() {
        assert_eq! {
            parse_json().parse("123").unwrap(),
            Json::JNumber(123f64)
        }
        assert_eq! {
            parse_json().parse("\"fooo\"").unwrap(),
            Json::JString("fooo")
        }
        assert_eq! {
            parse_json().parse("[1, -2, 3.0E4, true, false, null]").unwrap(),
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
            parse_json().parse("{\"key1\" : 123, \"key2\" : \"foo\"}").unwrap(),
            Json::JObject(HashMap::from_iter(vec! {
                ("key1", Json::JNumber(123f64)),
                ("key2", Json::JString("foo"))
            }))
        }
        assert_eq! {
            parse_json().parse("[{\"key1\" : 123, \"key2\" : \"foo\"}, 123, [\"foo\", true]]").unwrap(),
            Json::JArray(vec! {
                Json::JObject(HashMap::from_iter(vec! {
                    ("key1", Json::JNumber(123f64)),
                    ("key2", Json::JString("foo"))
                })),
                Json::JNumber(123f64),
                Json::JArray(vec! {
                    Json::JString("foo"),
                    Json::JBool(true)
                })
            })
        }
    }
}
