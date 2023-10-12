pub use query_params_macro::QueryParams;

pub trait ToQueryParams {
    fn to_query_params(&self) -> Vec<(&'static str, String)>;
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
    fn test_query_params_required_case() {
        let test_item = TestItem { a: 0, b: 1 };

        let expected = vec![("a", "0".to_string()), ("b", "1".to_string())];

        assert_eq!(test_item.to_query_params(), expected);
    }

    #[test]
    fn test_required_rename_case() {
        let test_item = TestItemRequiredRename { a: 0, b: 1 };

        let expected = vec![("alpha", "0".to_string()), ("b", "1".to_string())];

        assert_eq!(test_item.to_query_params(), expected);
    }

    #[test]
    fn test_query_params_optional_case() {
        let test_item = TestItemOptionals {
            a: Some("a".to_string()),
            b: None,
        };

        let expected = vec![("a", "a".to_string())];

        assert_eq!(test_item.to_query_params(), expected);
    }

    #[test]
    fn test_query_params_optional_rename_case() {
        let test_item = TestItemRename {
            a: Some("a".to_string()),
            b: Some(true),
        };

        let expected = vec![("alpha", "a".to_string()), ("beta", "true".to_string())];

        assert_eq!(test_item.to_query_params(), expected);
    }

    #[test]
    fn test_query_params_mixed_case() {
        let test_item = TestItemMixedRequiredOptionals {
            a: Some("a".to_string()),
            b: None,
            c: 42,
        };

        let expected = vec![("a", "a".to_string()), ("c", "42".to_string())];

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

        let expected = vec![("alpha", "a".to_string()), ("gamma", "42".to_string())];

        let mut actual = test_item.to_query_params();

        // order is not guaranteed due to splitting up optional and required params
        actual.sort();

        assert_eq!(actual, expected);
    }
}
