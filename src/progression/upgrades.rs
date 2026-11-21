//! In-run level-up upgrade cards and deterministic shuffle selection.

use crate::rng::GameRng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpgradeCard {
    pub id: u8,
    pub title: &'static str,
    pub desc: &'static str,
}

pub const ALL_UPGRADES: [UpgradeCard; 12] = [
    UpgradeCard {
        id: 0,
        title: "ROZDZIELACZ LUF",
        desc: "+1 Pocisk na salwe",
    },
    UpgradeCard {
        id: 1,
        title: "PLAZMA OVERCHARGE",
        desc: "+25% Obrazen broni",
    },
    UpgradeCard {
        id: 2,
        title: "CEWKI CYKLERA",
        desc: "+20% Szybkostrzelnosci",
    },
    UpgradeCard {
        id: 3,
        title: "PENETRATOR",
        desc: "+1 Przebicie wrogow",
    },
    UpgradeCard {
        id: 4,
        title: "PILA ORBITALNA",
        desc: "+1 Wirujace ostrze",
    },
    UpgradeCard {
        id: 5,
        title: "LUK TESLI",
        desc: "Blyskawice lancuchowe",
    },
    UpgradeCard {
        id: 6,
        title: "IMPULS EMP",
        desc: "Radialna fala uderzeniowa",
    },
    UpgradeCard {
        id: 7,
        title: "CHLODZENIE DASH",
        desc: "-25% Cooldownu uniku",
    },
    UpgradeCard {
        id: 8,
        title: "NANONAPRAWA",
        desc: "+40% HP i +25 Max HP",
    },
    UpgradeCard {
        id: 9,
        title: "MAGNES RDZENIA",
        desc: "+45% Zasieg zbierania",
    },
    UpgradeCard {
        id: 10,
        title: "TURBO KRYTYK",
        desc: "+15% Szansy na CRIT 2x",
    },
    UpgradeCard {
        id: 11,
        title: "AUTONAPRAWA",
        desc: "+3 HP/s regeneracji",
    },
];

/// Draws 3 distinct upgrade cards using Fisher-Yates shuffle.
pub fn roll_three_upgrades(rng: &mut GameRng) -> [UpgradeCard; 3] {
    let mut pool = ALL_UPGRADES;
    for i in (1..pool.len()).rev() {
        let j = rng.range_usize(0, i + 1);
        pool.swap(i, j);
    }
    [pool[0], pool[1], pool[2]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_three_upgrades_always_returns_three_distinct_cards() {
        let mut rng = GameRng::seeded(999);
        for _ in 0..50 {
            let cards = roll_three_upgrades(&mut rng);
            assert_ne!(cards[0].id, cards[1].id);
            assert_ne!(cards[1].id, cards[2].id);
            assert_ne!(cards[0].id, cards[2].id);
        }
    }
}
