//! Permanent Hangar Shop catalog and overflow-safe cost calculations.

#[derive(Clone, Copy, Debug)]
pub struct ShopItem {
    pub id: u8,
    pub tab: u8, // 0: BOJOWE, 1: SURVIVAL, 2: ENDGAME
    pub name: &'static str,
    pub bonus: &'static str,
    pub base_cost: u32,
    pub max_lvl: u32,
    pub quadratic: bool,
}

pub const SHOP_ITEMS: [ShopItem; 15] = [
    ShopItem {
        id: 0,
        tab: 0,
        name: "MOC RDZENIA",
        bonus: "+15% DMG",
        base_cost: 15,
        max_lvl: 15,
        quadratic: false,
    },
    ShopItem {
        id: 1,
        tab: 0,
        name: "SZYBKOSTRZELNOSC",
        bonus: "+9% ROF",
        base_cost: 15,
        max_lvl: 12,
        quadratic: false,
    },
    ShopItem {
        id: 2,
        tab: 0,
        name: "KRYTYCZNY RDZEN",
        bonus: "+8% CRIT 2x",
        base_cost: 20,
        max_lvl: 8,
        quadratic: false,
    },
    ShopItem {
        id: 3,
        tab: 0,
        name: "DRON STARTOWY",
        bonus: "+1 Ostrze start",
        base_cost: 40,
        max_lvl: 4,
        quadratic: false,
    },
    ShopItem {
        id: 4,
        tab: 0,
        name: "DOPALACZ DASH",
        bonus: "-12% CD uniku",
        base_cost: 20,
        max_lvl: 6,
        quadratic: false,
    },
    ShopItem {
        id: 5,
        tab: 1,
        name: "PANCERZ KADLUBA",
        bonus: "+30 Max HP",
        base_cost: 15,
        max_lvl: 15,
        quadratic: false,
    },
    ShopItem {
        id: 6,
        tab: 1,
        name: "REGENERACJA NANO",
        bonus: "+1.5 HP/s",
        base_cost: 20,
        max_lvl: 10,
        quadratic: false,
    },
    ShopItem {
        id: 7,
        tab: 1,
        name: "SILNIKI MANEWROWE",
        bonus: "+8% Predkosc",
        base_cost: 15,
        max_lvl: 10,
        quadratic: false,
    },
    ShopItem {
        id: 8,
        tab: 1,
        name: "MAGNES ZBIERANIA",
        bonus: "+35% Zasieg",
        base_cost: 15,
        max_lvl: 8,
        quadratic: false,
    },
    ShopItem {
        id: 9,
        tab: 1,
        name: "EKSTRAKTOR SHARDOW",
        bonus: "+20% Shards/XP",
        base_cost: 25,
        max_lvl: 8,
        quadratic: false,
    },
    ShopItem {
        id: 10,
        tab: 2,
        name: "TARCZA PLAZMOWA",
        bonus: "+35 Max Tarczy + Regen",
        base_cost: 400,
        max_lvl: 10,
        quadratic: true,
    },
    ShopItem {
        id: 11,
        tab: 2,
        name: "OVERCLOCK SALWY",
        bonus: "+1 Salwa start (+Przebicie)",
        base_cost: 600,
        max_lvl: 5,
        quadratic: true,
    },
    ShopItem {
        id: 12,
        tab: 2,
        name: "PODWOJNA SUPERNOVA",
        bonus: "EMP niszczy pociski + Start",
        base_cost: 450,
        max_lvl: 8,
        quadratic: true,
    },
    ShopItem {
        id: 13,
        tab: 2,
        name: "REAKTOR OVERDRIVE",
        bonus: "+1.2s, +12% DMG, Boss OD",
        base_cost: 350,
        max_lvl: 8,
        quadratic: true,
    },
    ShopItem {
        id: 14,
        tab: 2,
        name: "RDZEN PRESTIZU",
        bonus: "+10% DMG/+8% HP/+10% Shards",
        base_cost: 500,
        max_lvl: 25,
        quadratic: true,
    },
];

impl ShopItem {
    /// Overflow-safe shard cost for the next level from `current_lvl`.
    #[inline]
    pub fn next_cost(self, current_lvl: u32) -> u32 {
        let next = current_lvl.saturating_add(1);
        if self.quadratic {
            next.saturating_mul(next).saturating_mul(self.base_cost)
        } else {
            next.saturating_mul(self.base_cost)
        }
    }
}

pub fn max_level_for_id(id: u8) -> u32 {
    SHOP_ITEMS
        .iter()
        .find(|item| item.id == id)
        .map_or(25, |item| item.max_lvl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shop_cost_matches_linear_and_quadratic_formulas() {
        let linear = SHOP_ITEMS[0]; // base 15, linear
        assert_eq!(linear.next_cost(0), 15);
        assert_eq!(linear.next_cost(4), 75);

        let quad = SHOP_ITEMS[14]; // base 500, quadratic
        assert_eq!(quad.next_cost(0), 500);
        assert_eq!(quad.next_cost(1), 2000);
        assert_eq!(quad.next_cost(2), 4500);
    }

    #[test]
    fn shop_cost_never_overflows_on_extreme_levels() {
        let quad = SHOP_ITEMS[14];
        assert_eq!(quad.next_cost(u32::MAX), u32::MAX);
        assert_eq!(quad.next_cost(100_000), u32::MAX);
    }
}
