use to_query_params::QueryParams;

#[derive(QueryParams)]
struct Data {
    #[query(###)]
    number: f64,
}

fn main() {}
