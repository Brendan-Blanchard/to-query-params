//! `to-query-params` exports the [`QueryParams`] derive macro for public consumption, and the
//! [`ToQueryParams`] trait that it derives.
#[doc(inline)]
pub use query_params_macro::QueryParams;

#[doc(hidden)]
pub use urlencoding;

extern crate self as to_query_params;

/// [`ToQueryParams`] contains two methods, `to_query_params` and `to_encoded_params`, which each
/// produce a `Vec<(String, String)>` representing the struct as query parameters, either un-encoded
/// or url-encoded respectively.
///
pub trait ToQueryParams {
    /// Creates a `Vec<(String, String)>` as the un-encoded (key, value) pairs for query parameters.
    fn to_query_params(&self) -> Vec<(String, String)>;

    /// Creates a `Vec<(String, String)>` as the url-encoded (key, value) pairs for query parameters.
    fn to_encoded_params(&self) -> Vec<(String, String)> {
        self.to_query_params()
            .iter()
            .map(|(k, v)| {
                (
                    urlencoding::encode(k).to_string(),
                    urlencoding::encode(v).to_string(),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(QueryParams, Debug, PartialEq)]
    struct TestItem {
        #[query(required)]
        a: i32,
        #[query(required)]
        b: i32,
    }

    #[derive(QueryParams, Debug, PartialEq)]
    struct TestExcludeItem {
        #[query(required)]
        a: i32,
        #[query(exclude)]
        b: i32,
        #[query(exclude)]
        c: Option<i32>,
        #[query(required)]
        d: i32,
    }

    #[derive(QueryParams, Debug, PartialEq)]
    struct TestStringItem {
        #[query(required)]
        a: String,
        #[query(required, rename = "please encode")]
        b: String,
    }

    #[derive(QueryParams, Debug, PartialEq)]
    struct TestItemRequiredRename {
        #[query(required, rename = "alpha")]
        a: i32,
        #[query(required)]
        b: i32,
    }

    #[derive(QueryParams, Debug, PartialEq)]
    struct TestItemOptionals {
        a: Option<String>,
        b: Option<bool>,
    }

    #[derive(QueryParams, Debug, PartialEq)]
    struct TestItemRename {
        #[query(rename = "alpha")]
        a: Option<String>,
        #[query(rename = "beta")]
        b: Option<bool>,
    }

    #[derive(QueryParams, Debug, PartialEq)]
    struct TestItemMixedRequiredOptionals {
        a: Option<String>,
        b: Option<bool>,
        #[query(required)]
        c: i32,
    }

    #[derive(QueryParams, Debug, PartialEq)]
    struct TestItemMixedRequiredOptionalsAndRename {
        #[query(rename = "alpha")]
        a: Option<String>,
        b: Option<bool>,
        #[query(rename = "gamma")]
        #[query(required)]
        c: i32,
    }

    #[test]
    fn test_developer_experience() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/*.rs");
    }

    #[test]
    fn test_query_params_required_case() {
        let test_item = TestItem { a: 0, b: 1 };

        let expected = vec![
            ("a".to_string(), "0".to_string()),
            ("b".to_string(), "1".to_string()),
        ];

        assert_eq!(test_item.to_query_params(), expected);
    }

    #[test]
    fn test_exclude_attribute() {
        let test_item = TestExcludeItem {
            a: 0,
            b: 1,
            c: Some(2),
            d: 3,
        };

        let expected = vec![
            ("a".to_string(), "0".to_string()),
            ("d".to_string(), "3".to_string()),
        ];

        assert_eq!(test_item.to_query_params(), expected);
    }

    #[test]
    fn test_query_params_encoding() {
        let test_item = TestStringItem {
            a: "please encode me".into(),
            b: "this works?".into(),
        };

        let expected = vec![
            ("a".to_string(), "please%20encode%20me".to_string()),
            ("please%20encode".to_string(), "this%20works%3F".to_string()),
        ];

        assert_eq!(test_item.to_encoded_params(), expected);
    }

    #[test]
    fn test_required_rename_case() {
        let test_item = TestItemRequiredRename { a: 0, b: 1 };

        let expected = vec![
            ("alpha".to_string(), "0".to_string()),
            ("b".to_string(), "1".to_string()),
        ];

        assert_eq!(test_item.to_query_params(), expected);
    }

    #[test]
    fn test_query_params_optional_case() {
        let test_item = TestItemOptionals {
            a: Some("a".to_string()),
            b: None,
        };

        let expected = vec![("a".to_string(), "a".to_string())];

        assert_eq!(test_item.to_query_params(), expected);
    }

    #[test]
    fn test_query_params_optional_rename_case() {
        let test_item = TestItemRename {
            a: Some("a".to_string()),
            b: Some(true),
        };

        let expected = vec![
            ("alpha".to_string(), "a".to_string()),
            ("beta".to_string(), "true".to_string()),
        ];

        assert_eq!(test_item.to_query_params(), expected);
    }

    #[test]
    fn test_query_params_mixed_case() {
        let test_item = TestItemMixedRequiredOptionals {
            a: Some("a".to_string()),
            b: None,
            c: 42,
        };

        let expected = vec![
            ("a".to_string(), "a".to_string()),
            ("c".to_string(), "42".to_string()),
        ];

        let mut actual = test_item.to_query_params();

        // order is not guaranteed due to splitting up optional and required params
        actual.sort();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_query_params_mixed_case_with_rename() {
        let test_item = TestItemMixedRequiredOptionalsAndRename {
            a: Some("a".to_string()),
            b: None,
            c: 42,
        };

        let expected = vec![
            ("alpha".to_string(), "a".to_string()),
            ("gamma".to_string(), "42".to_string()),
        ];

        let mut actual = test_item.to_query_params();

        // order is not guaranteed due to splitting up optional and required params
        actual.sort();

        assert_eq!(actual, expected);
    }
}
