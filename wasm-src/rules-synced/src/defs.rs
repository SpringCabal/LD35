use alloc::vec;
use alloc::vec::Vec;
use springcabal::defs::DefCache;

use partsneeded_shared::BodyPart;

#[derive(Clone, Copy, Debug)]
pub struct Defs {
    pub wisp: i32,
    pub npcwisp: i32,
    pub gate: i32,
    pub lever: i32,
    pub plate: i32,
    pub cage: i32,
    pub light: i32,
    pub parts: [i32; 6],
}

impl Defs {
    fn resolve() -> Self {
        let id = |name: &str| spring::get_unit_def_id_by_name(name).unwrap_or(-1);
        Self {
            wisp: id("wisp"),
            npcwisp: id("npcwisp"),
            gate: id("gatesmoth"),
            lever: id("lever"),
            plate: id("plate"),
            cage: id("cage"),
            light: id("light"),
            parts: BodyPart::ALL.map(|part| id(part.name())),
        }
    }

    pub fn by_name(&self, name: &str) -> Option<i32> {
        let def = match name {
            "wisp" => self.wisp,
            "npcwisp" => self.npcwisp,
            "gatesmoth" => self.gate,
            "lever" => self.lever,
            "plate" => self.plate,
            "cage" => self.cage,
            "light" => self.light,
            _ => BodyPart::ALL
                .into_iter()
                .find(|part| part.name() == name)
                .map(|part| self.part_def(part))?,
        };
        (def >= 0).then_some(def)
    }

    pub fn body_part(&self, def: i32) -> Option<BodyPart> {
        BodyPart::ALL
            .into_iter()
            .find(|part| self.parts[*part as usize] == def)
    }

    pub fn part_def(&self, part: BodyPart) -> i32 {
        self.parts[part as usize]
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DefParams {
    pub pinned_wall: bool,
    pub scenery_wall: bool,
    pub trigger: bool,
    pub dungeon_element: bool,
}

impl DefParams {
    fn resolve(def: i32) -> Self {
        let param = |key: &str| {
            spring::unit_defs::get_unit_def_custom_param(def, key)
                .ok()
                .filter(|value| !value.is_empty())
        };
        let wall = param("wall");
        Self {
            pinned_wall: wall
                .as_deref()
                .is_some_and(|value| value != "false" && value != "0"),
            scenery_wall: wall.is_some() && param("wall_blocking").is_none(),
            trigger: param("trigger").is_some(),
            dungeon_element: param("dungeonelement").is_some(),
        }
    }
}

fn resolve_params() -> Vec<DefParams> {
    let mut params = vec![DefParams::default()];
    while spring::unit_defs::get_unit_def_name_bytes(params.len() as i32).is_ok() {
        params.push(DefParams::resolve(params.len() as i32));
    }
    params
}

static DEFS: DefCache<Defs> = DefCache::new(Defs::resolve);
static PARAMS: DefCache<Vec<DefParams>> = DefCache::new(resolve_params);

pub fn defs() -> Defs {
    *DEFS.get()
}

pub fn params(def: i32) -> DefParams {
    usize::try_from(def)
        .ok()
        .and_then(|def| PARAMS.get().get(def).copied())
        .unwrap_or_default()
}

pub fn unit_def(unit: i32) -> i32 {
    spring::get_unit_def_id(unit).map_or(-1, |def| def.0)
}
