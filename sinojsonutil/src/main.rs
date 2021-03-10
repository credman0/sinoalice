
#[macro_use]
extern crate lazy_static;
use std::{
    fs,
    collections::HashMap
};

use sinoalice_core::{Weapon, LeveledWeapon, SimpleWeapon};

// (Buf) Uncomment these lines to have the output buffered, this can provide
// better performance but is not always intuitive behaviour.
// use std::io::BufWriter;

use structopt::StructOpt;

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
    }
}

