mod parser;
mod ratios;

// (Buf) Uncomment these lines to have the output buffered, this can provide
// better performance but is not always intuitive behaviour.
// use std::io::BufWriter;

use structopt::StructOpt;
use soup::prelude::*;
use std::collections::HashMap;
use std::fs;

// Our CLI arguments. (help and version are automatically generated)
// Documentation on how to use:
// https://docs.rs/structopt/0.2.10/structopt/index.html#how-to-derivestructopt
#[derive(StructOpt, Debug)]
struct Cli {
}

const HTML_TABLE:&'static str = include_str!("Weapons - SINoALICE Database.html");
const RELEVANT_HEADERS: &'static [&'static str] = &["name", "type", "ele.", "colo.skill", "c.aid skill"];

fn main() {
    let args = Cli::from_args();
    let soup = Soup::new(HTML_TABLE);

    let table = soup.class("fixedTable").find().expect("Couldn't find class 'fixedTable'");
    let head = table.tag("thead").find().expect("Couldn't find table head");
    let headers = head.tag("td").find_all();
    let mut header_indices = HashMap::<u32,String>::new();
    let mut index = 0;
    for header in headers {
        let header = header.text();
        if RELEVANT_HEADERS.contains(&header.to_lowercase().as_str()) {
            header_indices.insert(index, header.to_lowercase());
        }
        index+=1;
    }
    let mut weapons = HashMap::<String,sinoalice_core::Weapon>::new();
    let rows = table.tag("tbody").find().unwrap().tag("tr").find_all();
    for row in rows {
        let cols = row.tag("td").find_all();
        let mut name = None::<String>;
        let mut weapon_type = None::<sinoalice_core::WeaponType>;
        let mut ele = None::<sinoalice_core::WeaponEle>;
        let mut c_skill = None::<sinoalice_core::CSkill>;
        let mut c_aid_skill = None::<sinoalice_core::CAidSkill>;
        let mut index = 0;
        for col in cols {
            if !header_indices.contains_key(&index) {
                index+=1;
                continue;
            }
            let header_name = &header_indices[&index];
            if header_indices.contains_key(&index) {
                match header_name.as_str() {
                    "name" => name = Some(col.tag("a").find().unwrap().text()),
                    "type" => weapon_type = Some(sinoalice_core::WeaponType::from_str(col.text().as_str())),
                    "ele." => ele = Some(parser::parse_ele(col.tag("img").find().unwrap().get("src").unwrap())),
                    "colo.skill" => c_skill = Some(parser::parse_c_skill(col.class("tableDetail").find().unwrap().text())),
                    "c.aid skill" => c_aid_skill = Some(parser::parse_c_aid_skill(col.class("tableDetail").find().unwrap().text())),
                    _=>panic!("Unrecognized header name: {}", header_name)
                }
            }
            // if (&weapon_type).is_some() && weapon_type.as_ref().unwrap()!=&sinoalice_core::WeaponType::Tome && weapon_type.as_ref().unwrap()!=&sinoalice_core::WeaponType::Instr {
            //     break;
            // }
            index += 1;
        }
        if name.is_some() && weapon_type.is_some() && ele.is_some() && c_skill.is_some() && c_aid_skill.is_some() {
            let weapon = sinoalice_core::Weapon {name:name.as_ref().unwrap().clone(), weapon_type:weapon_type.unwrap(), weapon_ele:ele.unwrap(), c_skill:c_skill.unwrap(), c_aid_skill:c_aid_skill.unwrap() };
            let key = name.unwrap().to_lowercase();
            weapons.insert(key, weapon);
            //println!("{}", serde_json::to_string_pretty(&weapon).unwrap());
        }
    }
    fs::write("../weapons.json", serde_json::to_string_pretty(&weapons).unwrap()).unwrap();
}

