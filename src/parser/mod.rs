mod error;

pub use self::error::QueryParserError;
pub use self::error::QueryParserResult;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
pub enum Occurance {
    Must,
    MustNot,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
pub enum Token {
    Text {
        occurance: Option<Occurance>,
        strings: Vec<String>,
    },
    FilterEquals {
        occurance: Option<Occurance>,
        field: String,
        value: String,
    },
    FilterRange {
        occurance: Option<Occurance>,
        field: String,
        left_bound: Option<String>,
        right_bound: Option<String>,
    },
}

pub fn parse_query(query: &str) -> QueryParserResult<Vec<Token>> {
    let mut result = Vec::new();

    for part in query.split(char::is_whitespace).filter(|s| !s.is_empty()) {
        let (occurance, part) = match part.chars().next() {
            Some('+') => (Some(Occurance::Must), &part[1..]),
            Some('-') => (Some(Occurance::MustNot), &part[1..]),
            _ => (None, part),
        };

        let token = if let Some(index) = part.find(':') {
            let field = &part[..index];
            let value = &part[index + 1..];

            parse_filter(occurance, field, value)?
        } else {
            parse_text(occurance, part)?
        };

        result.push(token);
    }

    Ok(result)
}

fn parse_filter(
    occurance: Option<Occurance>,
    field: &str,
    value: &str,
) -> QueryParserResult<Token> {
    let field = field.into();

    if let Some(index) = value.find("..") {
        let left_bound = &value[..index];
        let right_bound = &value[index + 2..];

        Ok(Token::FilterRange {
            occurance,
            field,
            left_bound: parse_number(left_bound)?,
            right_bound: parse_number(right_bound)?,
        })
    } else {
        Ok(Token::FilterEquals {
            occurance,
            field,
            value: value.into(),
        })
    }
}

fn parse_number(value: &str) -> QueryParserResult<Option<String>> {
    if value.is_empty() {
        Ok(None)
    } else if value.chars().all(char::is_numeric) {
        Ok(Some(value.into()))
    } else {
        Err(QueryParserError::new(format!(
            "Value `{}` is not a number",
            value
        )))
    }
}

fn parse_text(occurance: Option<Occurance>, value: &str) -> QueryParserResult<Token> {
    let strings = as_strings(value);

    Ok(Token::Text { occurance, strings })
}

fn as_strings(value: &str) -> Vec<String> {
    value
        .split('_')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_query;
    use super::Occurance;
    use super::Token;

    #[test]
    fn should_parse_empty_query() {
        let tokens = parse_query("");

        assert_eq!(tokens.is_ok(), true);
        assert_eq!(tokens.unwrap(), vec![]);
    }

    #[test]
    fn should_parse_text_token() {
        let tokens = parse_query("_test_line_");

        assert_eq!(tokens.is_ok(), true);
        assert_eq!(
            tokens.unwrap(),
            vec![Token::Text {
                occurance: None,
                strings: ["test", "line"].iter().cloned().map(String::from).collect(),
            }]
        );
    }

    #[test]
    fn should_parse_two_tokens() {
        let tokens = parse_query("test line");

        assert_eq!(tokens.is_ok(), true);
        assert_eq!(
            tokens.unwrap(),
            vec![
                Token::Text {
                    occurance: None,
                    strings: ["test"].iter().cloned().map(String::from).collect(),
                },
                Token::Text {
                    occurance: None,
                    strings: ["line"].iter().cloned().map(String::from).collect(),
                }
            ]
        );
    }

    #[test]
    fn should_parse_must_occuranceance() {
        let tokens = parse_query("+test");

        assert_eq!(tokens.is_ok(), true);
        assert_eq!(
            tokens.unwrap(),
            vec![Token::Text {
                occurance: Some(Occurance::Must),
                strings: ["test"].iter().cloned().map(String::from).collect(),
            }]
        );
    }

    #[test]
    fn should_parse_must_not_occuranceance() {
        let tokens = parse_query("-test");

        assert_eq!(tokens.is_ok(), true);
        assert_eq!(
            tokens.unwrap(),
            vec![Token::Text {
                occurance: Some(Occurance::MustNot),
                strings: ["test"].iter().cloned().map(String::from).collect(),
            }]
        );
    }

    #[test]
    fn should_parse_filter_field_equals() {
        let tokens = parse_query("field:_test_value_");

        assert_eq!(tokens.is_ok(), true);
        assert_eq!(
            tokens.unwrap(),
            vec![Token::FilterEquals {
                occurance: None,
                field: "field".into(),
                value: "_test_value_".into(),
            }]
        );
    }

    #[test]
    fn should_parse_filter_field_greater() {
        let tokens = parse_query("field:10..");

        assert_eq!(tokens.is_ok(), true);
        assert_eq!(
            tokens.unwrap(),
            vec![Token::FilterRange {
                occurance: None,
                field: "field".into(),
                left_bound: Some("10".into()),
                right_bound: None,
            }]
        );
    }

    #[test]
    fn should_parse_filter_field_less() {
        let tokens = parse_query("field:..20");

        assert_eq!(tokens.is_ok(), true);
        assert_eq!(
            tokens.unwrap(),
            vec![Token::FilterRange {
                occurance: None,
                field: "field".into(),
                left_bound: None,
                right_bound: Some("20".into()),
            }]
        );
    }

    #[test]
    fn should_parse_filter_field_between() {
        let tokens = parse_query("field:10..20");

        assert_eq!(tokens.is_ok(), true);
        assert_eq!(
            tokens.unwrap(),
            vec![Token::FilterRange {
                occurance: None,
                field: "field".into(),
                left_bound: Some("10".into()),
                right_bound: Some("20".into()),
            }]
        );
    }

    #[test]
    fn should_fail_when_left_bound_not_number() {
        let tokens = parse_query("field:a..20");

        assert_eq!(tokens.is_err(), true);
    }

    #[test]
    fn should_fail_when_right_bound_not_number() {
        let tokens = parse_query("field:10..a");

        assert_eq!(tokens.is_err(), true);
    }
}
