#![allow(missing_docs, unused_variables, trivial_casts)]

#[allow(unused_extern_crates)]
extern crate swagger;
#[allow(unused_extern_crates)]
extern crate futures;
#[macro_use]
extern crate swagger_client;
#[allow(unused_extern_crates)]
extern crate uuid;
extern crate clap;
extern crate tokio_core;
extern crate crypto;
extern crate hex;
extern crate blake2b;
extern crate rand;

extern crate regex;
use regex::Regex;

extern crate curl;
use curl::easy::Easy;

extern crate rust_base58;
//use rust_base58::{ToBase58, FromBase58};

extern crate rust_sodium;

extern crate serde_json;
use serde_json::{Value};

#[allow(unused_imports)]
use futures::{Future, future, Stream, stream};
use tokio_core::reactor;

pub mod transaction;

#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod schema;


use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

pub mod models;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

use swagger_client::{ ContextWrapperExt,ApiError,
                      CallContractResponse, CompileContractResponse,
                      EncodeCalldataResponse,
                      GetAccountBalanceResponse,
                      GetAccountsBalancesResponse,
                      GetBlockByHashResponse,
                      GetBlockByHeightResponse,
                      GetBlockGenesisResponse, GetBlockLatestResponse,
                      GetBlockPendingResponse,
                      GetCommitmentHashResponse,
                      GetContractCallFromTxResponse,
                      GetHeaderByHashResponse,
                      GetHeaderByHeightResponse, GetInfoResponse,
                      GetNameResponse, GetPeerKeyResponse,
                      GetTopResponse, GetTxResponse, GetTxsResponse,
                      GetVersionResponse, PostBlockResponse,
                      PostChannelCloseMutualResponse,
                      PostChannelCloseSoloResponse,
                      PostChannelCreateResponse,
                      PostChannelDepositResponse,
                      PostChannelSettleResponse,
                      PostChannelSlashResponse,
                      PostChannelWithdrawalResponse,
                      PostContractCallResponse,
                      PostContractCallComputeResponse,
                      PostContractCreateResponse,
                      PostNameClaimResponse, PostNamePreclaimResponse,
                      PostNameRevokeResponse,
                      PostNameTransferResponse,
                      PostNameUpdateResponse,
                      PostOracleExtendResponse,
                      PostOracleQueryResponse,
                      PostOracleRegisterResponse,
                      PostOracleResponseResponse, PostSpendResponse,
                      PostTxResponse,
                      GetActiveRegisteredOraclesResponse,
                      GetBlockNumberResponse,
                      GetBlockTxsCountByHashResponse,
                      GetBlockTxsCountByHeightResponse,
                      GetGenesisBlockTxsCountResponse,
                      GetLatestBlockTxsCountResponse,
                      GetOracleQuestionsResponse, GetPeersResponse,
                      GetPendingBlockTxsCountResponse,
                      GetPubKeyResponse,
                      GetTransactionFromBlockHashResponse,
                      GetTransactionFromBlockHeightResponse,
                      GetTransactionFromBlockLatestResponse,
                      GetTxsListFromBlockRangeByHashResponse,
                      GetTxsListFromBlockRangeByHeightResponse,
                      PostNameClaimTxResponse,
                      PostNamePreclaimTxResponse,
                      PostNameRevokeTxResponse,
                      PostNameTransferTxResponse,
                      PostNameUpdateTxResponse,
                      PostOracleExtendTxResponse,
                      PostOracleQueryTxResponse,
                      PostOracleRegisterTxResponse,
                      PostOracleResponseTxResponse,
                      PostSpendTxResponse };

pub struct Epoch {
    client: swagger_client::client::Client,
    base_uri: String,
}

impl Epoch {
    fn new(base_url: String) -> Epoch {
        let core = reactor::Core::new().unwrap();
        let client;
        if base_url.starts_with("https://") {
            client = swagger_client::client::Client::try_new_https(&base_url, "test/ca.pem").expect("Failed to connect");
        } else {
            client = swagger_client::client::Client::try_new_http(&base_url).expect("Failed to connect");
        }
        let context = swagger_client::Context::new_with_span_id(self::uuid::Uuid::new_v4().to_string());

        Epoch { client: client, base_uri: base_url } }

    fn top(&self) -> Option<serde_json::Value> {
            self.get(&String::from("/top"))
    }

    fn get(&self, operation: &String) -> Option<serde_json::Value> {
        let uri = self.base_uri.clone() + "/v2" + operation;
        println!("{}", uri);
            let mut data = Vec::new();
            let mut handle = Easy::new();
            handle.url(&uri).unwrap();
            {
                let mut transfer = handle.transfer();
                transfer.write_function(|new_data| {
                    data.extend_from_slice(new_data);
                    Ok(new_data.len())
                }).unwrap();
                transfer.perform().unwrap();
            }
            let value: Value = serde_json::from_str(std::str::from_utf8(&data).unwrap()).unwrap();
            Some(value)
    }

    fn get_block_at_height(&self, height: i64) ->
        Option<serde_json::Value> {
            self.get(&format!("{}{}", String::from("/block/height/"),&height.to_string()))
        }

    fn get_block_by_hash(&self, hash: &String) ->
        Option<serde_json::Value> {
            self.get(&format!("{}{}", String::from("/block/hash/"),&hash))
        }

    fn save_block(&self, conn: PgConnection, block: serde_json::Value) {
        use schema::blocks::dsl::*;
        use models::Block;
        let newblock = Block {
            hash: block["hash"].to_string(),
            height: block["height"].as_i64().unwrap(),
            miner: block["miner"].to_string(),
            nonce: block["nonce"].as_i64().unwrap(),
            prev_hash: block["prev_hash"].to_string(),
            state_hash: block["state_hash"].to_string(),
            txs_hash: block["txs_hash"].to_string(),
            target: block["target"].as_i64().unwrap(),
            time_: block["time"].as_i64().unwrap(),
            version: block["version"].as_i64().unwrap() as i32,
        };
        
        diesel::insert_into(blocks)
            .values(&newblock).execute(&conn);
    }
}

fn from_json(val: &String) -> String {
    let foo = "^\"(.*)\"$";
    println!("{}", foo);
    let re = Regex::new(foo).unwrap();
    match re.captures(val) {
        Some(matches) => {
            println!("Match: {:?}", String::from(&matches[1]));
            String::from(&matches[1])
        }
        None => val.clone()
    }
}
    
fn main() {
    let connection = establish_connection();
    
    let epoch = Epoch::new(String::from("https://sdk-testnet.aepps.com"));
    println!("Top: {:?}", epoch.top());
    let top_response = epoch.top().unwrap();
    let mut _hash = from_json(&top_response["hash"].to_string());
    loop  {
        let result = epoch.get_block_by_hash(&_hash);
        match result {
            Some(block) => {
                _hash = from_json(&block["prev_hash"].to_string());
                println!("{:?}", block);
            }
            None => {
                break;
            }
        }
    }
        
}

#[cfg(test)]
mod tests {
    use transaction::KeyPair;
    #[test]
    fn test_read_sign_verify() {
        // Read a key pair from a file (these were generated by the JS
        // SDK so this also tests ineroperability. Sign and check
        // verification works
        let key_pair = KeyPair::read_from_files(&String::from("test/keys/testkey.pub"),
                                                &String::from("test/keys/testkey"),
                                                &String::from(""));
        let msg = b"this is a test thing";
        let mut bytes = key_pair.sign(msg).unwrap();
        println!("Sig: {:?}", KeyPair::bytes_to_hex(bytes));
        key_pair.verify(&bytes, msg).unwrap();
    }
    #[test]
    #[should_panic(expected = "Verification failed")]
    fn test_generate_sign_verify() {
        // generate 2 key pairs. Generate with one, verify with the
        // other. Should blow up!
        let key_pair = KeyPair::generate().unwrap();
        let new_key_pair = KeyPair::generate().unwrap();
        let msg  =b"this is a test thing";
        let bytes = new_key_pair.sign(msg).unwrap();
        key_pair.verify(&bytes, msg).unwrap();
    }

    #[test]
    fn test_write_sign_verify() {
        // generate a key pair, write it to a file. Read from the file
        // into a new variable, sign with one and check that
        // verification with the other works
        let new_key_pair = KeyPair::generate().unwrap();
        new_key_pair.write_to_files(&String::from("test/keys/new.pub"),
                                    &String::from("test/keys/new")).unwrap();
        let msg  =b"this is a test thing";
        let bytes = new_key_pair.sign(msg).unwrap();
        let loaded_key_pair = KeyPair::read_from_files(&String::from("test/keys/new.pub"),
                                                       &String::from("test/keys/new"),
                                                       &String::from(""));
        loaded_key_pair.verify(&bytes, msg).unwrap();
    }

    use diesel::prelude::*;
    use diesel::pg::PgConnection;
    use dotenv::dotenv;
    use std::env;
    
    pub fn establish_connection() -> PgConnection {
        dotenv().ok();
        
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url))
    }
    
    #[test]
    fn test_save_block() {
        extern crate diesel;
        let conn = establish_connection();
        use schema::blocks::dsl::*;
        use models::Block;
        let newblock = Block {
            hash: String::from("bh$abcdef0123456789abcdef0123456789abcdef0123456789"),
            height: 123456,
            miner: String::from("ak$abcdef0123456789abcdef0123456789abcdef0123456789"),
            nonce: 567876876876,
            prev_hash: String::from("bh$abcdef0123456789abcdef0123456789abcdef0123456789"),
            state_hash: String::from("sh$abcdef0123456789abcdef0123456789abcdef0123456789"),
            txs_hash: String::from("th$abcdef0123456789abcdef0123456789abcdef0123456789"),
            target: 12345676,
            time_: 78798797987,
            version: 1,
        };
        use diesel::{insert_into};
        use diesel::prelude::*;
        use diesel::pg::PgConnection;
        diesel::insert_into(blocks)
            .values(&newblock).execute(&conn);
    }

    
}

