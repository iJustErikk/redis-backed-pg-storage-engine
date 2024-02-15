use std::ptr;

use pgrx::pg_sys::int32;
use pgrx::pg_sys::uint32;
use pgrx::pg_sys::uint64;
use pgrx::pg_sys::uint8;
use pgrx::pg_sys::varlena;
use pgrx::pg_sys::BlockIdData;
use pgrx::pg_sys::BlockNumber;
use pgrx::pg_sys::BufferAccessStrategy;
use pgrx::pg_sys::BulkInsertStateData;
use pgrx::pg_sys::CommandId;
use pgrx::pg_sys::ForkNumber;
use pgrx::pg_sys::IndexBuildCallback;
use pgrx::pg_sys::IndexFetchTableData;
use pgrx::pg_sys::IndexInfo;
use pgrx::pg_sys::Item;
use pgrx::pg_sys::ItemPointer;
use pgrx::pg_sys::ItemPointerData;
use pgrx::pg_sys::LockTupleMode;
use pgrx::pg_sys::LockWaitPolicy;
use pgrx::pg_sys::MultiXactId;
use pgrx::pg_sys::Oid;
use pgrx::pg_sys::ParallelTableScanDesc;
use pgrx::pg_sys::RelFileLocator;
use pgrx::pg_sys::Relation;
use pgrx::pg_sys::SampleScanState;
use pgrx::pg_sys::ScanDirection;
use pgrx::pg_sys::ScanKeyData;
use pgrx::pg_sys::Size;
use pgrx::pg_sys::Snapshot;
use pgrx::pg_sys::TBMIterateResult;
use pgrx::pg_sys::TM_FailureData;
use pgrx::pg_sys::TM_IndexDeleteOp;
use pgrx::pg_sys::TM_Result;
use pgrx::pg_sys::TTSOpsVirtual;
use pgrx::pg_sys::TU_UpdateIndexes;
use pgrx::pg_sys::TableScanDesc;
use pgrx::pg_sys::TableScanDescData;
use pgrx::pg_sys::TransactionId;
use pgrx::pg_sys::TupleTableSlot;
use pgrx::pg_sys::TupleTableSlotOps;
use pgrx::pg_sys::VacuumParams;
use pgrx::pg_sys::ValidateIndexState;
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
        
        // don't care about these
        amroutine.scan_end = Some(redis_scan_end);
        amroutine.finish_bulk_insert = Some(redis_finish_bulk_insert);
        amroutine.index_build_range_scan = Some(redis_index_build_range_scan);
        amroutine.index_delete_tuples = Some(redis_index_delete_tuples);
        amroutine.index_fetch_begin = Some(redis_index_fetch_begin);
        amroutine.index_fetch_end = Some(redis_index_fetch_end);
        amroutine.index_fetch_reset = Some(redis_index_fetch_reset);
        amroutine.index_fetch_tuple = Some(redis_index_fetch_tuple);
        amroutine.index_validate_scan = Some(redis_index_validate_scan);
        amroutine.multi_insert = Some(redis_multi_insert);
        amroutine.parallelscan_estimate = Some(redis_parallelscan_estimate);
        amroutine.parallelscan_initialize = Some(redis_parallelscan_initialize);
        amroutine.parallelscan_reinitialize = Some(redis_parallelscan_reinitialize);
        amroutine.relation_copy_data = Some(redis_relation_copy_data);
        amroutine.relation_copy_for_cluster = Some(redis_relation_copy_for_cluster);
        amroutine.relation_estimate_size = Some(redis_relation_estimate_size);
        amroutine.relation_fetch_toast_slice = Some(redis_relation_fetch_toast_slice);
        amroutine.relation_needs_toast_table = Some(redis_relation_needs_toast_table);
        amroutine.relation_nontransactional_truncate = Some(redis_relation_nontransactional_truncate);
        amroutine.relation_size = Some(redis_relation_size);
        amroutine.relation_toast_am = Some(redis_relation_toast_am);
        amroutine.relation_vacuum = Some(redis_relation_vacuum);
        amroutine.scan_analyze_next_block = Some(redis_scan_analyze_next_block);
        amroutine.scan_analyze_next_tuple = Some(redis_scan_analyze_next_tuple);
        amroutine.scan_bitmap_next_block = Some(redis_scan_bitmap_next_block);
        amroutine.scan_bitmap_next_tuple = Some(redis_scan_bitmap_next_tuple);
        amroutine.scan_getnextslot_tidrange = Some(redis_scan_getnextslot_tidrange);
        amroutine.scan_rescan = Some(redis_scan_rescan);
        amroutine.scan_sample_next_block = Some(redis_scan_sample_next_block);
        amroutine.scan_sample_next_tuple = Some(redis_scan_sample_next_tuple);
        amroutine.scan_set_tidrange = Some(redis_scan_set_tidrange);
        amroutine.tuple_complete_speculative = Some(redis_tuple_complete_speculative);
        amroutine.tuple_delete = Some(redis_tuple_delete);
        amroutine.tuple_fetch_row_version = Some(redis_tuple_fetch_row_version);
        amroutine.tuple_get_latest_tid = Some(redis_tuple_get_latest_tid);
        amroutine.tuple_insert_speculative = Some(redis_tuple_insert_speculative);
        amroutine.tuple_lock = Some(redis_tuple_lock);
        amroutine.tuple_satisfies_snapshot = Some(redis_tuple_satisfies_snapshot);
        amroutine.tuple_tid_valid = Some(redis_tuple_tid_valid);
        amroutine.tuple_update = Some(redis_tuple_update);
        
        


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


// functions I don't care about that Postgres very kindly checks if they are null
// maybe not all but it does not hurt to add them
unsafe extern "C" fn redis_finish_bulk_insert(_rel: Relation, _options: ::std::os::raw::c_int) {}

unsafe extern "C" fn redis_index_build_range_scan(
    _table_rel: Relation,
    _index_rel: Relation,
    _index_info: *mut IndexInfo,
    _allow_sync: bool,
    _anyvisible: bool,
    _progress: bool,
    _start_blockno: BlockNumber,
    _numblocks: BlockNumber,
    _callback: IndexBuildCallback,
    _callback_state: *mut ::std::os::raw::c_void,
    _scan: TableScanDesc,
) -> f64 {return 0f64;}

unsafe extern "C" fn redis_index_delete_tuples(_rel: Relation, _delstate: *mut TM_IndexDeleteOp) -> TransactionId {
  return 0u32;
}

unsafe extern "C" fn redis_index_fetch_begin(_rel: Relation) -> *mut IndexFetchTableData {
  return ptr::null_mut::<IndexFetchTableData>();
}

unsafe extern "C" fn redis_index_fetch_end(_data: *mut IndexFetchTableData) {

}

unsafe extern "C" fn redis_index_fetch_reset(_data: *mut IndexFetchTableData) {

}

unsafe extern "C" fn redis_index_fetch_tuple(
    _scan: *mut IndexFetchTableData,
    _tid: ItemPointer,
    _snapshot: Snapshot,
    _slot: *mut TupleTableSlot,
    _call_again: *mut bool,
    _all_dead: *mut bool,
) -> bool {return false;}

unsafe extern "C" fn redis_index_validate_scan(
    _table_rel: Relation,
    _index_rel: Relation,
    _index_info: *mut IndexInfo,
    _snapshot: Snapshot,
    _state: *mut ValidateIndexState,
) {}

unsafe extern "C" fn redis_multi_insert(
    _rel: Relation,
    _slots: *mut *mut TupleTableSlot,
    _nslots: ::std::os::raw::c_int,
    _cid: CommandId,
    _options: ::std::os::raw::c_int,
    _bistate: *mut BulkInsertStateData,
) {}

unsafe extern "C" fn redis_parallelscan_estimate(_: Relation) -> Size {
    return 0usize;
}

unsafe extern "C" fn redis_parallelscan_initialize(_: Relation, _: ParallelTableScanDesc) -> Size {return 0usize;}

unsafe extern "C" fn redis_parallelscan_reinitialize(_: Relation, _: ParallelTableScanDesc) {}

unsafe extern "C" fn redis_relation_copy_data(_: Relation, _: *const RelFileLocator) {}

unsafe extern "C" fn redis_relation_copy_for_cluster(
    _: Relation,
    _: Relation,
    _: Relation,
    _: bool,
    _: TransactionId,
    _xid_cutoff: *mut TransactionId,
    _multi_cutoff: *mut MultiXactId,
    _num_tuples: *mut f64,
    _tups_vacuumed: *mut f64,
    _tups_recently_dead: *mut f64,
) {}

unsafe extern "C" fn redis_relation_estimate_size(
 _: Relation,
 _: *mut int32,
 _: *mut BlockNumber,
 _: *mut f64,
 _: *mut f64,
) {}

unsafe extern "C" fn redis_relation_fetch_toast_slice(
 _: Relation,
 _: Oid,
 _: int32,
 _: int32,
 _: int32,
 _: *mut varlena,
) {}

unsafe extern "C" fn redis_relation_needs_toast_table(rel: Relation) -> bool {
    return false;
}

unsafe extern "C" fn redis_relation_nontransactional_truncate(rel: Relation) {}

unsafe extern "C" fn redis_relation_size(rel: Relation, fork_number: ForkNumber) -> uint64 {return 0u64;}

unsafe extern "C" fn redis_relation_toast_am(rel: Relation) -> Oid {return Oid::INVALID;}

unsafe extern "C" fn redis_relation_vacuum(
 _: Relation,
 _: *mut VacuumParams,
 _: BufferAccessStrategy,
) {}

unsafe extern "C" fn redis_scan_analyze_next_block(
 _: TableScanDesc,
 _: BlockNumber,
 _: BufferAccessStrategy,
) -> bool {return false;}

unsafe extern "C" fn redis_scan_analyze_next_tuple(
 _: TableScanDesc,
 _: TransactionId,
 _: *mut f64,
 _: *mut f64,
 _: *mut TupleTableSlot,
) -> bool {return false;}

unsafe extern "C" fn redis_scan_bitmap_next_block(scan: TableScanDesc, tbmres: *mut TBMIterateResult) -> bool {return false;}

unsafe extern "C" fn redis_scan_bitmap_next_tuple(
 _: TableScanDesc,
 _: *mut TBMIterateResult,
 _: *mut TupleTableSlot,
) -> bool {return false;}

unsafe extern "C" fn redis_scan_getnextslot_tidrange(
 _: TableScanDesc,
 _: ScanDirection,
 _: *mut TupleTableSlot,
) -> bool {return false;}

unsafe extern "C" fn redis_scan_rescan(
 _: TableScanDesc,
 _: *mut ScanKeyData,
 _: bool,
 _: bool,
 _: bool,
 _: bool,
) {}

unsafe extern "C" fn redis_scan_sample_next_block(scan: TableScanDesc, scanstate: *mut SampleScanState) -> bool {return false;}


unsafe extern "C" fn redis_scan_sample_next_tuple(
 _: TableScanDesc,
 _: *mut SampleScanState,
 _: *mut TupleTableSlot,
) -> bool {return false;}

unsafe extern "C" fn redis_scan_set_tidrange(scan: TableScanDesc, mintid: ItemPointer, maxtid: ItemPointer) {}

unsafe extern "C" fn redis_tuple_complete_speculative(
 _: Relation,
 _: *mut TupleTableSlot,
 _: uint32,
 _: bool,
) {}

unsafe extern "C" fn redis_tuple_delete(
 _: Relation,
 _: ItemPointer,
 _: CommandId,
 _: Snapshot,
 _: Snapshot,
 _: bool,
 _: *mut TM_FailureData,
 _: bool,
) -> TM_Result {
  return 0u32;
}

unsafe extern "C" fn redis_tuple_fetch_row_version(
 _: Relation,
 _: ItemPointer,
 _: Snapshot,
 _: *mut TupleTableSlot,
) -> bool {return false;}

unsafe extern "C" fn redis_tuple_get_latest_tid(scan: TableScanDesc, tid: ItemPointer) {}

unsafe extern "C" fn redis_tuple_insert_speculative(
 _: Relation,
 _: *mut TupleTableSlot,
 _: CommandId,
 _: std::os::raw::c_int,
 _: *mut BulkInsertStateData,
 _: uint32,
) {}

unsafe extern "C" fn redis_tuple_lock(
 _: Relation,
 _: ItemPointer,
 _: Snapshot,
 _: *mut TupleTableSlot,
 _: CommandId,
 _: LockTupleMode,
 _: LockWaitPolicy,
 _: uint8,
 _: *mut TM_FailureData,
) -> TM_Result {
    return 0u32;
}

unsafe extern "C" fn redis_tuple_satisfies_snapshot(_: Relation, _: *mut TupleTableSlot, _: Snapshot) -> bool {
    return false;
}

unsafe extern "C" fn redis_tuple_tid_valid(_: TableScanDesc, _: ItemPointer) -> bool {
    return false;
}
unsafe extern "C" fn redis_tuple_update(
 _: Relation,
 _: ItemPointer,
 _: *mut TupleTableSlot,
 _: CommandId,
 _: Snapshot,
 _: Snapshot,
 _: bool,
 _: *mut TM_FailureData,
 _: *mut LockTupleMode,
 _: *mut TU_UpdateIndexes,
) -> TM_Result {return 0u32;}

unsafe extern "C" fn redis_scan_end(_: TableScanDesc) {}