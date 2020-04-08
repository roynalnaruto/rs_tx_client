use graphql_client::{GraphQLQuery, Response};

use web3::types::U64;

use crate::errors::Error;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/queries/schema.graphql",
    query_path = "src/queries/entities.graphql",
    response_derives = "Debug",
    variables_derives = "Debug",
    normalization = "rust"
)]
pub struct Transactions;

#[derive(Clone, Debug)]
pub struct RsTxTransaction {
    pub id: String,
    pub block: U64,
    pub nonce_point: String,
    pub encrypted_recipient: Vec<u8>,
}

pub fn query(block_number: i64) -> Vec<RsTxTransaction> {
    let query_vars = transactions::Variables {
        block_gt: block_number
    };

    match query_transactions(query_vars) {
        Ok(rs_tx_txs) => rs_tx_txs,
        Err(_) => vec![]
    }
}

fn query_transactions(variables: transactions::Variables) -> Result<Vec<RsTxTransaction>, anyhow::Error> {
    let request_body = Transactions::build_query(variables);

    let client = reqwest::Client::new();
    let mut res = client
        .post("http://127.0.0.1:8000/subgraphs/name/roynalnaruto/rs_tx_subgraph")
        .json(&request_body)
        .send()?;

    let response_body: Response<transactions::ResponseData> = res.json()?;

    let txs = match response_body.errors {
        Some(errors) => {
            eprintln!("[query] ERRORS:");
            for error in errors {
                eprintln!("{:?}", error);
            }

            vec![]
        },
        None => {
            let response_data: transactions::ResponseData = response_body.data.expect("[query] Missing response data");
            let rs_tx_transactions = response_data
                .transactions
                .expect("[query] No transactions found")
                .iter()
                .map(|tx| convert_fields(&tx))
                .filter_map(Result::ok)
                .collect();

            rs_tx_transactions
        }
    };

    Ok(txs)
}

fn convert_fields(tx: &transactions::TransactionsTransactions) -> Result<RsTxTransaction, Error> {
    let block = U64::from_dec_str(tx.block.clone().as_str())?;
    let nonce_point_str = &tx.nonce_point.as_str()[2..];
    let encrypted_recipient: Vec<u8> = hex::decode(&tx.encrypted_recipient.as_str()[2..])?;

    let rs_tx = RsTxTransaction {
        id: tx.id.clone(),
        block: block,
        nonce_point: String::from(nonce_point_str),
        encrypted_recipient: encrypted_recipient
    };

    Ok(rs_tx)
}
