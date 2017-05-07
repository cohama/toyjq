use super::jsonparser::*;
use super::*;

const INDENT_DEPTH: i32 = 2;

pub fn print_json(json: &Json, width: i32) -> String {
    Doc::new(vec![json_to_doc_elem(json)]).pretty(width)
}

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
    fn test_print_json() {
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
            print_json(&json, 1),
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
            print_json(&json, 84),
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
            print_json(&json, 215),
            r#"[ 42, "foo", true, false, [], [ null ], {}, { "poem": "Lorem ipsum" }, { "a": 1, "foo-bar-baz": "1 2 Fizz 4 Buzz 6 7 8 Fizz Buzz", "Numbers": [ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19 ] } ]"#
        }
    }
}
