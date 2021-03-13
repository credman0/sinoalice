#[macro_use]
extern crate lazy_static;

use std::{
    fs,
    collections::HashMap,
    ops
};

use sinoalice_core::{LeveledWeapon, StatModifier, SkillEffect, Weapon};
use serde::{Serialize, Deserialize};

// (Buf) Uncomment these lines to have the output buffered, this can provide
// better performance but is not always intuitive behaviour.
// use std::io::BufWriter;

use structopt::StructOpt;

const WEAPON_JSON:&'static str = include_str!("../../weapons.json");
lazy_static! {
    static ref WEAPONS:HashMap::<String, Weapon> = serde_json::from_str(WEAPON_JSON).unwrap();
}

// Our CLI arguments. (help and version are automatically generated)
// Documentation on how to use:
// https://docs.rs/structopt/0.2.10/structopt/index.html#how-to-derivestructopt
#[derive(StructOpt, Debug)]
struct Cli {
    /// JSON containing all players' expanded grids
    grids_file: String,
    /// Player name of grid to optimize
    player:String,
    /// Weapon to compare adding to the grid
    expirimental_weapon:Option<String>,
    /// The level of the test weapon
    level:Option<u32>
}

#[derive(PartialEq,Clone,Serialize,Deserialize, Debug)]
struct CumulativeEffect {
   buffs:StatModifier,
   debuffs:StatModifier,
   recover:f32,
   pdamage:f32,
   mdamage:f32
}

impl CumulativeEffect {
    fn new() -> CumulativeEffect {
        return CumulativeEffect {buffs:StatModifier::new(), debuffs:StatModifier::new(), recover:0., pdamage:0., mdamage:0.}
    }

    fn evaluate(&self) -> f32 {
        return self.buffs.sum() + self.debuffs.sum() + self.recover + self.pdamage + self.mdamage;
    }
}

impl ops::Add<&SkillEffect> for CumulativeEffect {
    type Output = CumulativeEffect;
    fn add (self, effect:&SkillEffect) -> CumulativeEffect {
        match effect {
            SkillEffect::Buff(modifier) => {
                let mut out = self.clone();
                out.buffs = &out.buffs+&modifier;
                return out;
            },
            SkillEffect::Debuff(modifier) => {
                let mut out = self.clone();
                out.debuffs = &out.debuffs+&modifier;
                return out;
            },
            SkillEffect::Recover(amount) => {
                let mut out = self.clone();
                out.recover = out.recover+amount;
                return out;
            },
            SkillEffect::Damage(amount,damage_type) => {
                let mut out = self.clone();
                match damage_type {
                    sinoalice_core::DamageType::Magic => {
                        out.mdamage = out.mdamage+amount;
                    },
                    sinoalice_core::DamageType::Physical => {
                        out.pdamage = out.pdamage+amount;
                    },
                }
                return out;
            },
        }
    }
}


impl ops::Add<&CumulativeEffect> for CumulativeEffect {
    type Output = CumulativeEffect;
    fn add (self, other:&CumulativeEffect) -> CumulativeEffect {
        let mut out = self.clone();
        out.buffs = &out.buffs + &other.buffs;
        out.debuffs = &out.debuffs + &other.debuffs;
        out.recover += other.recover;
        out.pdamage += other.pdamage;
        out.mdamage += other.mdamage;
        return out;
    }
}

impl ops::Div<f32> for CumulativeEffect {
    type Output = CumulativeEffect;
    fn div (self, div:f32) -> CumulativeEffect {
        let mut out = self.clone();
        out.buffs = &out.buffs/div;
        out.debuffs = &out.debuffs/div;
        out.recover /= div;
        out.pdamage /= div;
        out.mdamage /= div;
        return out;
    }
}

fn calc_aid_effect(used_weapon:&LeveledWeapon, support_weapon:&LeveledWeapon) -> CumulativeEffect {
    let support_skill = &support_weapon.weapon.c_aid_skill;
    let active_skill = used_weapon.scaled_skill();
    match &support_skill.trigger {
        sinoalice_core::Trigger::All => {},
        sinoalice_core::Trigger::Support => {
            if !active_skill.effect.is_support() {
                return CumulativeEffect::new();
            }
        },
        sinoalice_core::Trigger::Recover => {
            if !active_skill.effect.is_recover() {
                return CumulativeEffect::new();
            }
        },
        sinoalice_core::Trigger::Attack => {
            if !active_skill.effect.is_attack() {
                return CumulativeEffect::new();
            }
        }
    }
    let activation_chance = LeveledWeapon::get_c_aid_skill_level_chance(support_weapon.c_aid_skill_lvl.unwrap_or(0));
    match &support_skill.aid_effect {
        sinoalice_core::AidEffect::Amplify(amount) => {
            return CumulativeEffect::new() + &(&active_skill.expected_effect_positive()*(activation_chance*amount));
        },
        sinoalice_core::AidEffect::Buff(modifier) => {
            return CumulativeEffect::new() + &(&SkillEffect::Buff(modifier.clone())*activation_chance);
        },
        sinoalice_core::AidEffect::Debuff(modifier) => {
            return CumulativeEffect::new() + &(&SkillEffect::Debuff(modifier.clone())*activation_chance);
        },
        sinoalice_core::AidEffect::RestoreSp(_) => {
            return CumulativeEffect::new();
        },
    }
}

fn main() {
    let args = Cli::from_args();
    let content = fs::read_to_string(args.grids_file).expect("Unable to open file");
    let content:HashMap<String,Vec<LeveledWeapon>> = serde_json::from_str(&content).expect("Invalid json format");
    let player_grid = content[&args.player].clone();

    let calculated_weapons = compute_grid(&player_grid);

    let mut collected_grid:Vec<(&String,&CumulativeEffect)> = calculated_weapons.iter().collect();
    collected_grid.sort_by(|a,b| a.1.evaluate().partial_cmp(&b.1.evaluate()).unwrap());

    let mut sum = 0.;
    for grid in &collected_grid {
        println!("{}: {}", &grid.0, &grid.1.evaluate());
        sum+=&grid.1.evaluate();
    }
    println!("Sum: {}", sum);
    
    if args.expirimental_weapon.is_some() {
        let mut test_grid = player_grid.clone();
        let worst_weapon = collected_grid[0].0;
        let mut worst_weapon_index = None::<usize>;
        for i in 0..test_grid.len() {
            let weapon = &test_grid[i];
            if &weapon.weapon.name == worst_weapon {
                worst_weapon_index = Some(i);
                break;
            }
        }
        test_grid.remove(worst_weapon_index.unwrap());
        test_grid.push(LeveledWeapon{weapon:WEAPONS[&args.expirimental_weapon.unwrap().to_lowercase()].clone(), c_skill_lvl:args.level, c_aid_skill_lvl:args.level});
    
        let calculated_weapons = compute_grid(&test_grid);
    
        let mut collected_grid:Vec<(&String,&CumulativeEffect)> = calculated_weapons.iter().collect();
        collected_grid.sort_by(|a,b| a.1.evaluate().partial_cmp(&b.1.evaluate()).unwrap());
    
        let mut sum = 0.;
        for grid in &collected_grid {
            println!("{}: {}", &grid.0, &grid.1.evaluate());
            sum+=&grid.1.evaluate();
        }
        println!("Sum: {}", sum);
    }

}

fn compute_grid(grid:&Vec<LeveledWeapon>) -> HashMap<String,CumulativeEffect> {
    let mut calculated_weapons:HashMap<String,CumulativeEffect> = HashMap::new();
    let mut sum_effects = CumulativeEffect::new();
    for leveled_weapon in grid {
        let weapon_effect = leveled_weapon.scaled_skill().expected_effect_positive();
        let effect = CumulativeEffect::new() + &weapon_effect;
        // println!("{}: {}", &leveled_weapon.weapon.name, LeveledWeapon::get_c_skill_level_multiplier(leveled_weapon.c_skill_lvl.unwrap_or(1)));
        // println!("{}: {}", &leveled_weapon.weapon.name, serde_json::to_string_pretty(&effect).unwrap());
        sum_effects = sum_effects + &effect;
        calculated_weapons.insert(leveled_weapon.weapon.name.clone(), effect);
    }
    for support_weapon in grid {
        let mut weapon_support_effect = CumulativeEffect::new();
        for leveled_weapon in grid {
            let effect = calc_aid_effect(&leveled_weapon,&support_weapon);
            weapon_support_effect = weapon_support_effect + &effect;
        }
        sum_effects = sum_effects + &weapon_support_effect;
        let previous_effect = calculated_weapons.get_mut(&support_weapon.weapon.name).unwrap();
        *previous_effect = weapon_support_effect+&previous_effect.clone();
    }
    let avg_effects = sum_effects/(grid.len() as f32);
    println!("{}", serde_json::to_string_pretty(&avg_effects).unwrap());
    return calculated_weapons;
}
