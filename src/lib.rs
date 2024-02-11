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

#[pg_extern(sql = "
CREATE OR REPLACE FUNCTION redis_tableam_handler(internal)
RETURNS table_am_handler AS 'redis_backed_storage', 'redis_tableam_handler'
LANGUAGE C STRICT;

CREATE ACCESS METHOD redis TYPE TABLE HANDLER redis_tableam_handler;
")]
fn redis_tableam_handler(_fcinfo: pg_sys::FunctionCallInfo) -> PgBox<pg_sys::TableAmRoutine> {
    let mut amroutine =
        unsafe { PgBox::<pg_sys::TableAmRoutine>::alloc_node(pg_sys::NodeTag::T_TableAmRoutine)};

        // need to implement these for storage engine (or phils eq)
        amroutine.slot_callbacks = None;
        amroutine.scan_begin = None;
        amroutine.scan_getnextslot = None;
        amroutine.tuple_insert = None;
        amroutine.relation_set_new_filelocator = None;
        
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
        amroutine.relation_set_new_filelocator = None;
        amroutine.relation_size = None;
        amroutine.relation_toast_am = None;
        amroutine.relation_vacuum = None;
        amroutine.scan_analyze_next_block = None;
        amroutine.scan_analyze_next_tuple = None;
        amroutine.scan_begin = None;
        amroutine.scan_bitmap_next_block = None;
        amroutine.scan_bitmap_next_tuple = None;
        amroutine.scan_end = None;
        amroutine.scan_getnextslot = None;
        amroutine.scan_getnextslot_tidrange = None;
        amroutine.scan_rescan = None;
        amroutine.scan_sample_next_block = None;
        amroutine.scan_sample_next_tuple = None;
        amroutine.scan_set_tidrange = None;
        amroutine.slot_callbacks = None;
        amroutine.tuple_complete_speculative = None;
        amroutine.tuple_delete = None;
        amroutine.tuple_fetch_row_version = None;
        amroutine.tuple_get_latest_tid = None;
        amroutine.tuple_insert = None;
        amroutine.tuple_insert_speculative = None;
        amroutine.tuple_lock = None;
        amroutine.tuple_satisfies_snapshot = None;
        amroutine.tuple_tid_valid = None;
        amroutine.tuple_update = None;
        
        


        // also need other functions (like the c implementation just needed stubs for its null checks)

    amroutine.into_pg_boxed()
}