
#[macro_use]
extern crate lazy_static;
use std::{
    fs,
    collections::HashMap
};

use sinoalice_core::{Weapon, LeveledWeapon, SimpleWeapon, WeaponEle, WeaponType};

// (Buf) Uncomment these lines to have the output buffered, this can provide
// better performance but is not always intuitive behaviour.
// use std::io::BufWriter;

use structopt::StructOpt;
use serde::{Serialize, Deserialize};

// Our CLI arguments. (help and version are automatically generated)
// Documentation on how to use:
// https://docs.rs/structopt/0.2.10/structopt/index.html#how-to-derivestructopt
#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(subcommand)]
    command: Command
}

#[derive(StructOpt, Debug)]
enum Command {
    Reduce {
        filename:String,
        output:String
    },
    Expand {
        filename:String,
        output:String
    },
    /// Count the elements and weapon types of each player in the specified expanded json
    Count {
        filename:String,
        output:String
    }
}

#[derive(PartialEq,Clone,Serialize,Deserialize, Debug)]
struct CountResult {
    elements:ElementCount,
    weapons:WeaponCount
}

#[derive(PartialEq,Clone,Serialize,Deserialize, Debug)]
struct ElementCount {
    fire:u32,
    wind:u32,
    water:u32
}
impl ElementCount {
    fn new() -> ElementCount {
        return ElementCount {fire:0, wind:0, water:0}
    }
}


#[derive(PartialEq,Clone,Serialize,Deserialize, Debug)]
struct WeaponCount {
    tome:u32,
    instr:u32,
    staff:u32,
    focus:u32,
    blade:u32,
    hammer:u32,
    ranged:u32,
    polearm:u32

}

impl WeaponCount {
    fn new() -> WeaponCount {
        return WeaponCount {
            tome:0,
            instr:0,
            staff:0,
            focus:0,
            blade:0,
            hammer:0,
            ranged:0,
            polearm:0
        }
    }
}

const WEAPON_JSON:&'static str = include_str!("weapons.json");
lazy_static! {
    static ref WEAPONS:HashMap::<String, Weapon> = serde_json::from_str(WEAPON_JSON).unwrap();
}

fn main() {
    let args = Cli::from_args();
    match args.command {
        Command::Reduce{filename, output} => {
            let content = fs::read_to_string(filename).expect("Unable to open file");
            let content:serde_json::Value = serde_json::from_str(&content).unwrap();
            let content = content.as_object().unwrap();
            let mut all_players_simple:HashMap<String,Vec<SimpleWeapon>> = HashMap::new();
            for player in content {
                let player_name = player.0;
                let grid = player.1;
                let grid = grid.as_array().unwrap();
                let mut simple_vec:Vec<SimpleWeapon> = vec![];
                for leveled_weapon in grid {
                    let weapon = leveled_weapon["weapon"].as_object().unwrap();
                    let name = weapon["name"].as_str().unwrap().to_string().to_lowercase();
                    let c_lvl = leveled_weapon["c_skill_lvl"].as_u64();
                    let c_lvl = c_lvl.map(|x| x as u32);
                    let c_aid_lvl = leveled_weapon["c_aid_skill_lvl"].as_u64();
                    let c_aid_lvl = c_aid_lvl.map(|x| x as u32);
                    simple_vec.push(SimpleWeapon{name:name, c_lvl:c_lvl, c_aid_lvl:c_aid_lvl});
                }
                all_players_simple.insert(player_name.clone(), simple_vec);
            }
            let result = serde_json::to_string_pretty(&all_players_simple).unwrap();
            fs::write(output, result).unwrap();
        },
        Command::Expand{filename, output} => {
            let content = fs::read_to_string(filename).expect("Unable to open file");
            let content:HashMap<String,Vec<SimpleWeapon>> = serde_json::from_str(&content).expect("Invalid json format");
            let mut all_players:HashMap<String,Vec<LeveledWeapon>> = HashMap::new();
            for player in content {
                let player_name = player.0;
                let simple_grid = player.1;
                let mut complete_grid:Vec<LeveledWeapon> = vec![];
                for weapon in simple_grid {
                    let complete_weapon = WEAPONS[&weapon.name].clone();
                    complete_grid.push(LeveledWeapon{weapon:complete_weapon, c_skill_lvl:weapon.c_lvl,c_aid_skill_lvl:weapon.c_aid_lvl});
                }
                all_players.insert(player_name, complete_grid);
            }
            let result = serde_json::to_string_pretty(&all_players).unwrap();
            fs::write(output, result).unwrap();
        },
        Command::Count{filename, output} => {
            let content = fs::read_to_string(filename).expect("Unable to open file");
            let content:HashMap<String,Vec<LeveledWeapon>> = serde_json::from_str(&content).expect("Invalid json format");
            let mut all_results:HashMap<String,CountResult> = HashMap::new();
            for player in content {
                let player_name = player.0;
                let grid = player.1;
                let mut count_result = CountResult{elements:ElementCount::new(), weapons:WeaponCount::new()};
                for weapon in grid {
                    let weapon = weapon.weapon;
                    match weapon.weapon_ele {
                        WeaponEle::Fire => count_result.elements.fire+=1,
                        WeaponEle::Wind => count_result.elements.wind+=1,
                        WeaponEle::Water => count_result.elements.water+=1,
                    }
                    match weapon.weapon_type {
                        WeaponType::Tome => count_result.weapons.tome+=1,
                        WeaponType::Instr => count_result.weapons.instr+=1,
                        WeaponType::Staff => count_result.weapons.staff+=1,
                        WeaponType::Focus => count_result.weapons.focus+=1,
                        WeaponType::Blade => count_result.weapons.blade+=1,
                        WeaponType::Hammer => count_result.weapons.hammer+=1,
                        WeaponType::Ranged => count_result.weapons.ranged+=1,
                        WeaponType::Polearm => count_result.weapons.polearm+=1
                    }
                }
                all_results.insert(player_name, count_result);
            }
            let result = serde_json::to_string_pretty(&all_results).unwrap();
            fs::write(output, result).unwrap();
        }
    }
}

