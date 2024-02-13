use pgrx::pg_sys::uint32;
use pgrx::pg_sys::BlockIdData;
use pgrx::pg_sys::BulkInsertStateData;
use pgrx::pg_sys::CommandId;
use pgrx::pg_sys::Item;
use pgrx::pg_sys::ItemPointerData;
use pgrx::pg_sys::MultiXactId;
use pgrx::pg_sys::ParallelTableScanDesc;
use pgrx::pg_sys::RelFileLocator;
use pgrx::pg_sys::Relation;
use pgrx::pg_sys::ScanDirection;
use pgrx::pg_sys::ScanKeyData;
use pgrx::pg_sys::Snapshot;
use pgrx::pg_sys::TTSOpsVirtual;
use pgrx::pg_sys::TableScanDesc;
use pgrx::pg_sys::TableScanDescData;
use pgrx::pg_sys::TransactionId;
use pgrx::pg_sys::TupleTableSlot;
use pgrx::pg_sys::TupleTableSlotOps;
use pgrx::prelude::*;
use pgrx::pg_sys;

pgrx::pg_module_magic!();

#[pg_extern]
fn hello_redis_backed_storage() -> &'static str {
    "Hello, redis_backed_storage"
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_hello_redis_backed_storage() {
        assert_eq!("Hello, redis_backed_storage", crate::hello_redis_backed_storage());
    }

}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}

// did c version of this
// kinda understand what functions we need to implement here
// (pgrx says not to use external calls, but we can figure smth out for redis)
// how can we pass redis config info in? (via CREATE EXTENSION /w args or pg_config)
// those + rust storage = redis backed pg SE


// next order of business:
// get this to compile (might need to add functions first due to compile time checks (v sexy, wish better documentation though))
// implement in memory
// redis
// boom done

unsafe extern "C" fn redis_slot_callbacks(_rel: Relation) -> *const TupleTableSlotOps {
    return &TTSOpsVirtual;
}

unsafe extern "C" fn redis_scan_begin(
    rel: Relation,
    snapshot: Snapshot,
    nkeys: ::std::os::raw::c_int,
    key: *mut ScanKeyData,
    pscan: ParallelTableScanDesc,
    flags: uint32,
) -> TableScanDesc {
    let desc = Box::leak(Box::new(TableScanDescData{ rs_rd: rel, 
        rs_snapshot: snapshot, rs_nkeys: nkeys, rs_key: key, 
        rs_mintid: ItemPointerData{ ip_blkid: BlockIdData{ bi_hi: 0, bi_lo: 0 }, ip_posid: 0 }, 
        rs_maxtid: ItemPointerData{ ip_blkid: BlockIdData{ bi_hi: 0, bi_lo: 0 }, ip_posid: 0 }, 
        rs_flags: flags, rs_parallel: pscan 
    }));
    desc
}

unsafe extern "C" fn redis_scan_getnextslot(
    _scan: TableScanDesc,
    _direction: ScanDirection,
    _slot: *mut TupleTableSlot,
) -> bool {
    false
}

unsafe extern "C" fn redis_tuple_insert(
    _rel: Relation,
    _slot: *mut TupleTableSlot,
    _cid: CommandId,
    _options: ::std::os::raw::c_int,
    _bistate: *mut BulkInsertStateData,
) {
    
}

unsafe extern "C" fn redis_relation_set_new_filelocator(
    _rel: Relation,
    _newrlocator: *const RelFileLocator,
    _persistence: ::std::os::raw::c_char,
    _freeze_xid: *mut TransactionId,
    _minmulti: *mut MultiXactId,
) {

}


// CREATE OR REPLACE FUNCTION amhandler(internal) RETURNS index_am_handler PARALLEL SAFE IMMUTABLE STRICT COST 0.0001 LANGUAGE c AS 'MODULE_PATHNAME', 'amhandler_wrapper';
// CREATE ACCESS METHOD zombodb TYPE INDEX HANDLER amhandler
#[pg_extern(sql = "
CREATE OR REPLACE FUNCTION redis_tableam_handler(internal)
RETURNS table_am_handler LANGUAGE c AS 'MODULE_PATHNAME', 'redis_tableam_handler_wrapper';

CREATE ACCESS METHOD redis TYPE TABLE HANDLER redis_tableam_handler;
")]
pub fn redis_tableam_handler(_fcinfo: pg_sys::FunctionCallInfo) -> PgBox<pg_sys::TableAmRoutine> {
    let mut amroutine =
        unsafe { PgBox::<pg_sys::TableAmRoutine>::alloc_node(pg_sys::NodeTag::T_TableAmRoutine)};

        // need to implement these for storage engine (or phils eq)
        amroutine.slot_callbacks = Some(redis_slot_callbacks);
        amroutine.scan_begin = Some(redis_scan_begin);
        amroutine.scan_getnextslot = Some(redis_scan_getnextslot);
        amroutine.tuple_insert = Some(redis_tuple_insert);
        amroutine.relation_set_new_filelocator = Some(redis_relation_set_new_filelocator);
        
        // gen rest? still does not work
        amroutine.finish_bulk_insert = None;
        amroutine.index_build_range_scan = None;
        amroutine.index_delete_tuples = None;
        amroutine.index_fetch_begin = None;
        amroutine.index_fetch_end = None;
        amroutine.index_fetch_reset = None;
        amroutine.index_fetch_tuple = None;
        amroutine.index_validate_scan = None;
        amroutine.multi_insert = None;
        amroutine.parallelscan_estimate = None;
        amroutine.parallelscan_initialize = None;
        amroutine.parallelscan_reinitialize = None;
        amroutine.relation_copy_data = None;
        amroutine.relation_copy_for_cluster = None;
        amroutine.relation_estimate_size = None;
        amroutine.relation_fetch_toast_slice = None;
        amroutine.relation_needs_toast_table = None;
        amroutine.relation_nontransactional_truncate = None;
        amroutine.relation_size = None;
        amroutine.relation_toast_am = None;
        amroutine.relation_vacuum = None;
        amroutine.scan_analyze_next_block = None;
        amroutine.scan_analyze_next_tuple = None;
        amroutine.scan_bitmap_next_block = None;
        amroutine.scan_bitmap_next_tuple = None;
        amroutine.scan_getnextslot_tidrange = None;
        amroutine.scan_rescan = None;
        amroutine.scan_sample_next_block = None;
        amroutine.scan_sample_next_tuple = None;
        amroutine.scan_set_tidrange = None;
        amroutine.tuple_complete_speculative = None;
        amroutine.tuple_delete = None;
        amroutine.tuple_fetch_row_version = None;
        amroutine.tuple_get_latest_tid = None;
        amroutine.tuple_insert_speculative = None;
        amroutine.tuple_lock = None;
        amroutine.tuple_satisfies_snapshot = None;
        amroutine.tuple_tid_valid = None;
        amroutine.tuple_update = None;
        
        


        // also need other functions (like the c implementation just needed stubs for its null checks)

    amroutine.into_pg_boxed()
}

#[pg_extern(sql = "
    CREATE OR REPLACE FUNCTION amhandler(internal) RETURNS index_am_handler PARALLEL SAFE IMMUTABLE STRICT COST 0.0001 LANGUAGE c AS 'MODULE_PATHNAME', '@FUNCTION_NAME@';
    CREATE ACCESS METHOD zombodb TYPE INDEX HANDLER amhandler;
")]
fn amhandler(_fcinfo: pg_sys::FunctionCallInfo) -> PgBox<pg_sys::IndexAmRoutine> {
    let mut amroutine =
        unsafe { PgBox::<pg_sys::IndexAmRoutine>::alloc_node(pg_sys::NodeTag::T_IndexAmRoutine) };

    amroutine.amstrategies = 4;
    amroutine.amsupport = 0;
    amroutine.amcanmulticol = true;
    amroutine.amsearcharray = true;

    amroutine.amkeytype = pg_sys::InvalidOid;

    amroutine.into_pg_boxed()
}

#[pg_guard]
pub extern "C" fn amvalidate(_opclassoid: pg_sys::Oid) -> bool {
    true
}
