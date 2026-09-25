//! Unit def ids the UI tells apart, resolved once.

use springcabal::defs::DefCache;

#[derive(Clone, Copy, Debug)]
pub struct Defs {
    pub wisp: i32,
    pub npcwisp: i32,
    pub lever: i32,
    pub light: i32,
}

static DEFS: DefCache<Defs> = DefCache::new(|| {
    let id = |name: &str| spring::get_unit_def_id_by_name(name).unwrap_or(-1);
    Defs {
        wisp: id("wisp"),
        npcwisp: id("npcwisp"),
        lever: id("lever"),
        light: id("light"),
    }
});

pub fn defs() -> Defs {
    *DEFS.get()
}

/// The def id of a unit, or -1.
pub fn unit_def(unit: i32) -> i32 {
    spring::get_unit_def_id(unit).map_or(-1, |def| def.0)
}
