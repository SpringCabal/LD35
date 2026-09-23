//! Unit def ids the UI tells apart, resolved once.

use core::cell::OnceCell;

#[derive(Clone, Copy, Debug)]
pub struct Defs {
    pub wisp: i32,
    pub npcwisp: i32,
    pub lever: i32,
    pub light: i32,
}

thread_local! {
    static DEFS: OnceCell<Defs> = const { OnceCell::new() };
}

pub fn defs() -> Defs {
    DEFS.with(|defs| {
        *defs.get_or_init(|| {
            let id = |name: &str| spring::get_unit_def_id_by_name(name).unwrap_or(-1);
            Defs {
                wisp: id("wisp"),
                npcwisp: id("npcwisp"),
                lever: id("lever"),
                light: id("light"),
            }
        })
    })
}

/// The def id of a unit, or -1.
pub fn unit_def(unit: i32) -> i32 {
    spring::get_unit_def_id(unit).map_or(-1, |def| def.0)
}
