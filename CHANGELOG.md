# Changelog

### v0.1.0
- Add a default impl for `ToQueryParams.to_encoded_params`, remove from macro
- Use fully qualified `to_query_params::QueryParams` to avoid callers needing to import `ToQueryParams`
- Update to Rust 2024 Edition
- Bump dependencies