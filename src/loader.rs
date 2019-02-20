use super::schema::key_blocks::dsl::*;
use super::schema::transactions::dsl::*;
use chashmap::*;
use diesel::Connection;
use diesel::pg::PgConnection;
use diesel::query_dsl::QueryDsl;
use diesel::sql_query;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use epoch::*;
use models::*;
use serde_json;
use middleware_result::*;
use middleware_result::MiddlewareResult;

use std::slice::SliceConcatExt;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use PGCONNECTION;
use SQLCONNECTION;

use super::websocket;

pub struct BlockLoader {
    epoch: Epoch,
    rx: std::sync::mpsc::Receiver<i64>,
    pub tx: std::sync::mpsc::Sender<i64>,
}

pub static BACKLOG_CLEARED: i64 = -1;

lazy_static! {
    static ref tx_queue: CHashMap<i64, bool>  = CHashMap::<i64, bool>::new();
}

fn is_in_queue(_height: i64) -> bool {
    match tx_queue.get(&_height) {
        None => false,
        _ => true,
    }
}

fn remove_from_queue(_height: i64) {
    info!("tx_queue -> {}", _height);
    tx_queue.remove(&_height);
    info!("tx_queue len={}", tx_queue.len());

}
fn add_to_queue(_height: i64) {
    info!("tx_queue <- {}", _height);
    tx_queue.insert(_height, true);
}

pub fn queue(_height: i64, _tx: &std::sync::mpsc::Sender<i64>) -> Result<(), std::sync::mpsc::SendError<i64>> {
    info!("tx_queue len={}", tx_queue.len());

    if is_in_queue(_height) {
        info!("tx_queue already has {}", _height);
        return Ok(());
    }
    _tx.send(_height)?;
    add_to_queue(_height);
    Ok(())
}


/*
* You may notice the use of '_tx' as a variable name in this file,
* rather than the more natural 'tx'. This is because the macros which
* diesel generates from the code in models.rs cause every occurrence
* of their field names to be replaced, which obviously causes
* problems.
*/

impl BlockLoader {
    /*
     * Makes a new BlockLoader object, initializes its DB pool.
     */
    pub fn new(epoch_url: String) -> BlockLoader {
        let (_tx, rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();
        let epoch = Epoch::new(epoch_url.clone());
        BlockLoader { epoch, rx, tx: _tx }
    }

    pub fn start_fork_detection(
        epoch: &Epoch,
        _tx: &std::sync::mpsc::Sender<i64>)
    {
        let settings = [(1,10),(11,50),(51,500)];
        for setting in settings.iter() {
            let _tx = _tx.clone();
            let epoch = epoch.clone();
            let start = setting.0;
            let end = setting.1;
            thread::spawn(move || {
                loop {
                    let epoch = epoch.clone();
                    let _tx = _tx.clone();
                    let handle = thread::spawn(move || {
                        match BlockLoader::detect_forks(&epoch, start, end, &_tx) {
                            Ok(x) => {
                                if x {
                                    info!("Fork detected");
                                }
                            },
                            Err(x) => error!("Error in fork detection {}", x),
                        }
                    });
                    match handle.join() {
                        Ok(_) => {
                            error!("Thread exited, respawning");
                            continue;
                        },
                        Err(_) => {
                            error!("Error creating fork detection thread, exiting");
                            break;
                        },
                    };
                }
            });
        }
    }

    /*
     * We walk backward through the chain loading generations from the
     * DB, and requesting them from the chain. We pause 1 second
     * between each check, and only check 500 blocks (~1 day)
     * back. For each pair of blocks we compare them using their eq()
     * mehods. If false we delete the block from the DB (which
     * cascades to delete the microblocks and transactions), and put
     * the height onto the load queue.
     *
     * TODO: disassociate the TXs from the micro-blocks and keep them
     * for reporting purposes.
     */
    pub fn detect_forks(
        epoch: &Epoch,
        from: i64,
        to: i64,
        _tx: &std::sync::mpsc::Sender<i64>,
    ) -> MiddlewareResult<bool> {
        let conn = PGCONNECTION.get()?;
        let mut fork_detected = false;
        let mut _height = KeyBlock::top_height(&conn)? - from;
        let mut stop_height = _height - to;
        loop {
            // first time through fork_detected will be false, on subsequent trips it will have the value
            // of the last iteration. Putting it here so we can just continue when we find a fork, to save
            // time.
            if fork_detected {
                info!("In fork: invalidating block at height {}", _height);
                BlockLoader::invalidate_block_at_height(_height, &conn, &_tx)?;
                fork_detected = false;
            } else {
                debug!("Block checks out at height {}", _height);
                thread::sleep(std::time::Duration::new(2, 0));
            }

            _height -= 1;

            if _height <= stop_height {
                _height = KeyBlock::top_height(&conn)?;
                stop_height = _height - to;
                debug!("Resetting fork detection loop: now from {} to {}",
                       _height, stop_height);
            }

            let jg: JsonGeneration = match JsonGeneration::get_generation_at_height(
                &*SQLCONNECTION.get()?,
                &conn,
                _height,
            ) {
                Some(x) => x,
                None => {
                    error!("Couldn't load generation {} from DB", _height);
                    fork_detected = true;
                    continue;
                }
            };

            let gen_from_server: JsonGeneration =
                serde_json::from_value(epoch.get_generation_at_height(_height)?)?;
            if !jg.eq(&gen_from_server) {
                debug!("Generations don't match at height {}", _height);
                fork_detected = true;
                continue;
            }

            for i in 0..jg.micro_blocks.len() {
                let differences = BlockLoader::compare_micro_blocks(
                    &epoch,
                    &conn,
                    _height,
                    jg.micro_blocks[i].clone(),
                    jg.micro_blocks[i].clone(),
                )?;
                if differences.len() != 0 {
                    info!("Microblocks differ: {:?}", differences);
                    fork_detected = true;
                }
            }
        }
        Ok(fork_detected)
    }

    /*
     * Delete a key block at height (which causes the deletion of the
     * associated micro blocks and transaactions, and then adds the
     * height to the queue of heights to be loaded.
     */

    pub fn invalidate_block_at_height(
        _height: i64,
        _conn: &PgConnection,
        _tx: &std::sync::mpsc::Sender<i64>,
    ) -> MiddlewareResult<bool> {
        debug!("Invalidating block at height {}", _height);
//        diesel::delete(key_blocks.filter(height.eq(&_height))).execute(conn)?;
        match queue(_height, _tx) {
            Ok(()) => (),
            Err(e) => {
                error!("Error queuing block at height {}: {:?}", _height, e);
                BlockLoader::recover_from_db_error();
            }
        };
        Ok(true)
    }

    /*
     * Load the mempool from the node--this will only be possible if
     * the debug endpoint is also available on the main URL. Otherwise
     * it harmlessly explodes.
     */
    pub fn load_mempool(&self, _epoch: &Epoch) -> MiddlewareResult<u64> {
        let conn = PGCONNECTION.get()?;
        let trans: JsonTransactionList =
            serde_json::from_value(self.epoch.get_pending_transaction_list()?)?;
        let mut hashes_in_mempool = vec![];
        for i in 0..trans.transactions.len() {
            match self.store_or_update_transaction(&conn, &trans.transactions[i], None) {
                Ok(_) => (),
                Err(x) => error!(
                    "Failed to insert transaction {} with error {}",
                    trans.transactions[i].hash, x
                ),
            }
            hashes_in_mempool.push(format!("'{}'", trans.transactions[i].hash));
        }
        let sql = format!(
            "UPDATE transactions SET VALID=FALSE WHERE \
             micro_block_id IS NULL AND \
             hash NOT IN ({})",
            hashes_in_mempool.join(", ")
        );
        Ok(SQLCONNECTION.get()?.execute(&sql, &[])?)
    }

    pub fn start_scan_thread(epoch: &Epoch, _tx: &std::sync::mpsc::Sender<i64>)
    {
        let _tx = _tx.clone();
        let epoch = epoch.clone();
        thread::spawn(move || {
            loop {
                match BlockLoader::scan(&epoch, &_tx) {
                    Ok(count) => debug!("BlockLoader::scan() added {} blocks to loading queue",
                                        count),
                    Err(x) => debug!("BlockLoader::scan() returned an error: {}", x),
                };
                thread::sleep(std::time::Duration::new(5, 0));
            }
        });
    }


    /*
     * this method scans the blocks from the heighest reported by the
     * node to the highest iairport n the DB, filling in the gaps.
     */
    pub fn scan(epoch: &Epoch, _tx: &std::sync::mpsc::Sender<i64>) ->
        MiddlewareResult<i32>
    {
        let connection = PGCONNECTION.get()?;
        let top_block_chain = key_block_from_json(epoch.latest_key_block()?)?;
        let top_block_db = KeyBlock::top_height(&connection)?;
        let mut blocks_changed = 0;
        if top_block_chain.height == top_block_db {
            trace!("Up-to-date");
            return Ok(0);
        }
        debug!(
            "Reading blocks {} to {}",
            top_block_db + 1,
            top_block_chain.height
        );
        let mut _height = top_block_chain.height;
        loop {
            if _height <= top_block_db {
                break;
            }
            if !KeyBlock::height_exists(&connection, _height) {
                debug!("Queuing block {}", _height);
                match queue(_height, _tx) {
                    Ok(x) => {
                        debug!("Success: {:?}", x);
                        blocks_changed += 1;
                    }
                    Err(e) => {
                        error!("Error queuing block at height {}: {:?}", _height, e);
                        BlockLoader::recover_from_db_error();
                    }
                };
            } else {
                info!("Block already in DB at height {}", _height);
            }
            _height -= 1;
        }
        Ok(blocks_changed)
    }

    /*
     * In fact there is no recovery currently--we just exit, and someone else can restart
     * the process.
     */
    pub fn recover_from_db_error() {
        error!("Quitting...");
        ::std::process::exit(-1);
    }

    /*
     * At this height, load the key block, using the generations call
     * to grab the block and all of its microblocks.
     */

    fn load_blocks(&self, _height: i64) -> MiddlewareResult<(i32, i32)> {
        let connection = PGCONNECTION.get()?;
        let result = connection.transaction::<(i32, i32), MiddlewareError, _>(|| {
            self.internal_load_block(&connection, _height)
        });
        result
    }

    fn internal_load_block(&self, connection: &PgConnection, _height: i64) -> MiddlewareResult<(i32,i32)> {
        let mut count = 0;
        // clear out the block at this height, and any with the same hash, to prevent key violations.
        diesel::delete(key_blocks.filter(height.eq(&_height))).execute(connection)?;
        let generation: JsonGeneration =
            serde_json::from_value(self.epoch.get_generation_at_height(_height)?)?;
        diesel::delete(key_blocks.filter(super::schema::key_blocks::dsl::hash.eq(&generation.key_block.hash))).execute(connection)?;
        let ib: InsertableKeyBlock =
            InsertableKeyBlock::from_json_key_block(&generation.key_block)?;
        let key_block_id = ib.save(&connection)? as i32;
        websocket::broadcast_ws(WsPayload::key_blocks, &json!(&generation.key_block))?; //broadcast key_block
        for mb_hash in &generation.micro_blocks {
            let mut mb: InsertableMicroBlock =
                serde_json::from_value(self.epoch.get_micro_block_by_hash(&mb_hash)?)?;
            mb.key_block_id = Some(key_block_id);
            websocket::broadcast_ws(WsPayload::micro_blocks, &json!(&mb))?; //broadcast micro_block
            let _micro_block_id = mb.save(&connection)? as i32;
            let trans: JsonTransactionList =
                serde_json::from_value(self.epoch.get_transaction_list_by_micro_block(&mb_hash)?)?;
            for i in 0..trans.transactions.len() {
                self.store_or_update_transaction(
                    &connection,
                    &trans.transactions[i],
                    Some(_micro_block_id),
                )?;
            }
            count += 1;
        }
        self.refresh_accounts_view(connection); //refresh accounts view
        Ok((key_block_id, count))
    }

    /*
     * transactions in the mempool won't have a micro_block_id, so as we scan the chain we may
     * need to insert them, or update them with the id of the micro block with which they're
     * now associated. We may also need to move them to a different micro block, in the event
     * of a fork.
     */
    pub fn store_or_update_transaction(
        &self,
        conn: &PgConnection,
        trans: &JsonTransaction,
        _micro_block_id: Option<i32>,
    ) -> MiddlewareResult<i32> {
        let sql = format!(
            "select * from transactions where hash = '{}' limit 1",
            &trans.hash
        );
        debug!("{}", sql);
        let mut results: Vec<Transaction> = sql_query(sql).
            // bind::<diesel::sql_types::Text, _>(trans.hash.clone()).
            // TODO: fix ^^^^^^^^^^^^^^
            get_results(conn)?;
        match results.pop() {
            Some(x) => {
                debug!("Updating transaction with hash {}", &trans.hash);
                diesel::update(&x).set(micro_block_id.eq(_micro_block_id));
                websocket::broadcast_ws(WsPayload::tx_update, &json!(&x))?; //broadcast updated transaction
                Ok(x.id)
            }
            None => {
                debug!("Inserting transaction with hash {}", &trans.hash);
                let _tx_type: String = from_json(&serde_json::to_string(&trans.tx["type"])?);
                let _tx: InsertableTransaction = InsertableTransaction::from_json_transaction(
                    &trans,
                    _tx_type,
                    _micro_block_id,
                )?;
                websocket::broadcast_ws(WsPayload::transactions, &json!(&trans))?; //broadcast updated transaction
                _tx.save(conn)
            }
        }
    }

    /*
     * The very simple function which pulls heights from the queue and
     * loads them into the DB
     */
    pub fn start(&self) {
        for b in &self.rx {
            debug!("Pulling height {} from queue for storage", b);
            if b == BACKLOG_CLEARED {
                debug!("Backlog cleared, now launching fork detection & scanning threads");
                BlockLoader::start_fork_detection(&self.epoch, &self.tx);
                BlockLoader::start_scan_thread(&self.epoch, &self.tx);
            } else {
                debug!("Loading block at height {} into DB", b);
                match self.load_blocks(b) {
                    Ok(x) => info!(
                        "Saved {} micro blocks in total at height {}, key block id is {} ",
                        x.1, b, x.0
                    ),
                    Err(x) => error!("Error loading blocks {}", x),
                };
                remove_from_queue(b);
            }
        }
        // if we fall through here something has gone wrong. Let's quit!
        error!("Failed to read from the queue, quitting.");
        BlockLoader::recover_from_db_error();
    }

    /*
     * Verify the DB, printing results to stdout.
     *
     * methodology:
     * pick the highest of DB height and height reported by node
     * load blocks from DB and from node
     * compare the microblocks and transactions in each
     * for each, log with - if missing from DB, + if present in DB but not on chain
     * - key block
     * - micro blocks
     * - transactions
     */
    pub fn verify(&self) -> MiddlewareResult<i64>
    {
        let top_chain = self.epoch.latest_key_block()?["height"]
            .as_i64()?;
        let mut _verified: i64 = 0;
        let conn = PGCONNECTION.get()?;
        let top_db = KeyBlock::top_height(&conn)?;
        let top_max = std::cmp::max(top_chain, top_db);
        let mut i = top_max;
        loop {
            self.compare_chain_and_db(i, &conn)?;
            i -= 1;
            _verified += 1;
        }
    }

    pub fn compare_chain_and_db(&self, _height: i64, conn: &PgConnection) ->
        MiddlewareResult<bool>
    {
        let block_chain = self.epoch.get_key_block_by_height(_height);
        let block_db = KeyBlock::load_at_height(conn, _height);
        match block_chain {
            Ok(x) => {
                match block_db {
                    Some(y) => {
                        return Ok(self.compare_key_blocks(conn, x, y)? == 0);
                    }
                    None => {
                        println!("{} missing from DB", _height);
                        return Ok(false);
                    }
                };
            }
            Err(_) => {
                match block_db {
                    Some(_) => {
                        println!("{} present in DB not in chain", _height);
                        return Ok(false);
                    }
                    None => {
                        println!("Not found at either, something weird happened");
                        return Ok(false);
                    }
                };
            }
        };
    }

    pub fn compare_key_blocks(
        &self,
        conn: &PgConnection,
        block_chain: serde_json::Value,
       block_db: KeyBlock,
    ) -> MiddlewareResult<i64> {
        let chain_hash = block_chain["hash"].as_str()?;
        if !block_db.hash.eq(&chain_hash) {
            let err = format!(
                "{} Hashes differ: {} chain vs {} block",
                block_db.height, chain_hash, block_db.hash
            );
            return Err(MiddlewareError::new(&err));
        }
        let mut db_mb_hashes = MicroBlock::get_microblock_hashes_for_key_block_hash(
            &*SQLCONNECTION.get()?,
            &String::from(chain_hash),
        )?;
        db_mb_hashes.sort_by(|a, b| a.cmp(b));
        let chain_gen = self
            .epoch
            .get_generation_at_height(block_db.height)?;
        let mut chain_mb_hashes = chain_gen["micro_blocks"].as_array()?.clone();
        chain_mb_hashes.sort_by(|a, b| a.as_str().unwrap().cmp(b.as_str().unwrap()));
        if db_mb_hashes.len() != chain_mb_hashes.len() {
            println!(
                "{} Microblock array size differs: {} chain vs {} db",
                block_db.height,
                chain_mb_hashes.len(),
                block_db.hash.len()
            );
        } else {
            let mut all_good = true;
            for i in 0..db_mb_hashes.len() {
                let chain_mb_hash = String::from(chain_mb_hashes[i].as_str()?);
                let db_mb_hash = db_mb_hashes[i].clone();
                let differences = BlockLoader::compare_micro_blocks(
                    &self.epoch,
                    &conn,
                    block_db.height,
                    db_mb_hash,
                    chain_mb_hash,
                )?;
                if differences.len() != 0 {
                    println!("Transactions differ: {:?}", differences);
                    all_good = false;
                }
            }
            if all_good {
                println!("{} OK", block_db.height);
            }
        }
        Ok(block_db.height)
    }

    pub fn compare_micro_blocks(
        epoch: &Epoch,
        conn: &PgConnection,
        _height: i64,
        db_mb_hash: String,
        chain_mb_hash: String,
    ) -> MiddlewareResult<Vec<String>> {
        let mut differences: Vec<String> = vec![];
        if db_mb_hash != chain_mb_hash {
            differences.push(format!(
                "{} micro block hashes don't match: {} chain vs {} db",
                _height, chain_mb_hash, db_mb_hash
            ));
        }
        let mut db_transactions = match Transaction::load_for_micro_block(conn, &db_mb_hash) {
            Some(x) => x,
            None => {
                error!("Couldn't load transaction list from DB {}", db_mb_hash);
                return Ok(differences);
            }
        };
        db_transactions.sort_by(|a, b| a.hash.cmp(&b.hash));
        // well this fails when Epoch barfs on us.
        let ct = match epoch.get_transaction_list_by_micro_block(&chain_mb_hash) {
            Ok(x) => x,
            Err(y) => {
                error!("Couldn't load transaction list from Epoch {}", y);
                return Ok(differences); // TODO should force reload?
            }
        };
        let mut chain_transactions = ct["transactions"].as_array()?.clone();
        chain_transactions
            .sort_by(|a, b| a["hash"].as_str().unwrap().cmp(b["hash"].as_str().unwrap()));
        if db_transactions.len() != chain_transactions.len() {
            differences.push(format!(
                "{} micro block {} transaction count differs: {} chain vs {} DB",
                _height,
                db_mb_hash,
                chain_transactions.len(),
                db_transactions.len()
            ));
        }
        let mut diffs: Vec<(usize, String, String)> = vec![];
        for i in 0..db_transactions.len() {
            let chain_tx_hash = chain_transactions[i]["hash"].as_str()?;
            if db_transactions[i].hash != chain_tx_hash {
                diffs.push((
                    i,
                    db_transactions[i].hash.clone(),
                    chain_tx_hash.to_string(),
                ));
            }
        }
        if diffs.len() != 0 {
            for i in 0 .. diffs.len() {
                differences.push(format!(
                        "{} micro block {} transaction {} of {} hashes differ: {} chain vs {} DB",
                        _height,
                        db_mb_hash,
                        diffs[i].0,
                        diffs.len(),
                        diffs[i].2,
                        diffs[i].1
                ));
            }
        }
        Ok(differences)
    }

    pub fn refresh_accounts_view(&self, connection: &PgConnection) {
        let result = sql_query("refresh materialized view public.account_spendtx_balance;")
                    .execute(connection);
    }
}
