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

// #[pg_extern]
// fn amhandler(_fcinfo: pg_sys::FunctionCallInfo) -> PgBox<pg_sys::TableAmRoutine> {
//     let mut amroutine =
//         unsafe { PgBox::<pg_sys::TableAmRoutine>::alloc_node(pg_sys::NodeTag_T_TableAmRoutine) };


//         // need to implement these for storage engine (or phils eq)
//         amroutine.slot_callbacks = None;
//         amroutine.scan_begin = None;
//         amroutine.scan_getnextslot = None;
//         amroutine.tuple_insert = None;
//         amroutine.relation_set_new_filelocator = None;

//         // also need other functions (like the c implementation just needed stubs for its null checks)

//     amroutine.into_pg_boxed()
// }