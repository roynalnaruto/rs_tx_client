use graphql_client::{GraphQLQuery, Response};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/queries/schema.graphql",
    query_path = "src/queries/entities.graphql",
    response_derives = "Debug",
)]
pub struct Transactions;

pub fn query() {
    match query_transactions(transactions::Variables {
        first: 5
    }) {
        Ok(_) => println!("did ok"),
        Err(e) => eprintln!("got some error: {:?}", e)
    }
}

fn query_transactions(variables: transactions::Variables) -> Result<(), anyhow::Error> {
    let request_body = Transactions::build_query(variables);

    let client = reqwest::Client::new();
    let mut res = client
        .post("http://127.0.0.1:8000/subgraphs/name/roynalnaruto/rs_tx_subgraph")
        .json(&request_body)
        .send()?;

    let response_body: Response<transactions::ResponseData> = res.json()?;
    println!("{:#?}", response_body);

    Ok(())
}
