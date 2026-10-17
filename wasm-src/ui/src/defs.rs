use springcabal::defs::DefCache;

#[derive(Clone, Copy, Debug)]
pub struct Defs {
    pub wisp: i32,
    pub npcwisp: i32,
    pub lever: i32,
    pub light: i32,
}

static DEFS: DefCache<Defs> = DefCache::new();

pub fn defs(spring: &spring::Spring<'_, spring::Ui>) -> Defs {
    *DEFS.get_or_init(|| {
        let id = |name: &str| {
            spring
                .unit_defs()
                .get_unit_def_id_by_name(name)
                .unwrap_or(-1)
        };
        Defs {
            wisp: id("wisp"),
            npcwisp: id("npcwisp"),
            lever: id("lever"),
            light: id("light"),
        }
    })
}

pub fn unit_def(spring: &spring::Spring<'_, spring::Ui>, unit: i32) -> i32 {
    spring.units_info().get_unit_def_id(unit).unwrap_or(-1)
}
