# to-query-params
A procedural macro and trait for converting arbitrary structs into `Vec<(String, String)>` for use as query parameters, 
simplifying the conversion of arbitrary structs into query parameters, largely for use with the [Hyper](https://crates.io/crates/hyper) HTTP framework.

![badge](https://github.com/Brendan-Blanchard/to-query-params/actions/workflows/main.yml/badge.svg) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Usage:

```rust
use query_params::{ToQueryParams, QueryParams};

 // Eq and PartialEq are just for assertions
 #[derive(QueryParams, Debug, PartialEq, Eq)]
 struct ProductRequest {
     #[query(required)] // field that aren't Option<T> must be marked as required
     id: i32,
     #[query(required, rename = "type")]
     product_type: String,
     min_price: Option<i32>,
     max_price: Option<i32>,
 }

 pub fn main() {
     let request = ProductRequest {
         id: 999,
         product_type: "accessory".to_string(),
         min_price: None,
         max_price: Some(100),
     };

     let expected = vec![
         ("id".into(), "999".into()), 
         ("type".into(), "accessory".into()), 
         ("max_price".into(), "100".into())
     ];
     
     let query_params = request.to_query_params();

     assert_eq!(expected, query_params);
 }
```