mod adb;
#[macro_use]
extern crate lazy_static;

// (Buf) Uncomment these lines to have the output buffered, this can provide
// better performance but is not always intuitive behaviour.
// use std::io::BufWriter;

use structopt::StructOpt;
use std::{thread, time};
use regex::Regex;
use std::collections::{HashSet,HashMap};
use sinoalice_core::Weapon;
use leptess::tesseract;
use edit_distance::edit_distance;
// Our CLI arguments. (help and version are automatically generated)
// Documentation on how to use:
// https://docs.rs/structopt/0.2.10/structopt/index.html#how-to-derivestructopt
#[derive(StructOpt, Debug)]
struct Cli {
}

struct ChangeChecker {
    last_seen:Vec<String>
}

const WEAPON_JSON:&'static str = include_str!("weapons.json");
lazy_static! {
    static ref WEAPONS:HashMap::<String, Weapon> = serde_json::from_str(WEAPON_JSON).unwrap();
}

impl ChangeChecker {
    pub fn check_change (&mut self, current:&Vec<String>) -> bool {
        if current == &self.last_seen {
            return false;
        } else {
            self.last_seen = current.clone();
            return true;
        }
    }

    pub fn new () -> ChangeChecker {
        return ChangeChecker {last_seen:vec!["doesn't exist".to_string()]};
    }
}

const WEAPON_REGEX:&'static str = "(?m)^((?:.+?'s)+)(?:[^.]|Lv.)*?activated";//"(?m)^(.*)'s(?! guildship)[\\s\\S]*?activated";//([^.]|Lv\\.)*activated";//[^.]*activated";
const PLAYER_REGEX:&'static str = "(\\w+)\\sRank\\.\\d+";

const HALF_SECOND:time::Duration = time::Duration::from_millis(500);
const ONE_SECOND:time::Duration = time::Duration::from_millis(1000);
const THREE_SECONDS:time::Duration = time::Duration::from_millis(3000);

const FAKE_NAME_SUFFIXES: &'static [&'static str] = &["Exorcist", "Holy Knight", "Hero", "Knight", "Sorcerer", "Warrior", "Wind God", "Water God", "Flame God", "Sage", "Angel", "Barrier Master"];
//const CORRECTED_WORDS: &'static [&'static str] = &["Zweihänder of Justice", "Kainé's Sword"];

fn main() {
    let args = Cli::from_args();
    //println!("{}", parsed);
    // let grid = scrape();
    // println!("{}", serde_json::to_string(&grid).unwrap())
    //println!("{:?}", res);
    //progress_list();
    let mut all_players = HashMap::<String, Vec<Weapon>>::new();
    let mut seen_players = vec![];
    let mut player = next_player(&seen_players);
    while player.is_some() {
        seen_players.push(player.as_ref().unwrap().clone());
        let grid = scrape();
        all_players.insert(player.unwrap(), grid);
        check_disconnect();
        player = next_player(&seen_players);
        reset_list();
    }
    std::fs::write("all_players.json", serde_json::to_string_pretty(&all_players).unwrap()).unwrap();
}

fn next_player(seen:&Vec<String>) -> Option<String> {
    open_user_filter();
    player_list_reset();
    let mut found_player = None;
    let mut last_list:Vec<(std::string::String, leptess::leptonica::Box)> = vec![];
    loop {
        let data = adb::cap_screen();
        let res = ocr_paragraphs(&data, &leptess::leptonica::Box::new(20,660,945,1140).unwrap());
        let mut players = parse_players(res);
        if players.is_empty() {
            check_disconnect();
            let res = ocr_paragraphs(&data, &leptess::leptonica::Box::new(20,660,945,1140).unwrap());
            players = parse_players(res);
        }
        for player in &players {
            println!("{}", player.0);
            if !seen.contains(&player.0) {
                found_player = Some(player.0.clone());
                tap_player_box(&player.1);
                break;
            }
        }
        if found_player.is_some() || compare_player_lists(&players,&last_list) {
            break;
        }
        last_list = players;
        progress_list();
    }
    player_list_accept();
    return found_player
}

fn compare_player_lists(first:&Vec<(std::string::String, leptess::leptonica::Box)>, second:&Vec<(std::string::String, leptess::leptonica::Box)>) -> bool {
    if first.len() != second.len() {
        return false;
    }
    for i in 0..first.len() {
        if first[i].0 != second[i].0 {
            return false;
        }
    }
    return true;
}

fn scrape() -> Vec<Weapon>{
    let mut weapons = HashMap::<String,u32>::new();
    let mut checker = ChangeChecker::new();
    let mut current_page_set = HashSet::<String>::new();
    let mut exit_loop = false;
    while !exit_loop {
        let current = do_capture();
        let mut parsed = parse_text(&current);
        if parsed.is_empty() {
            check_disconnect();
            parsed = parse_text(&current);
        }
        for weapon in parsed.clone() {
            println!("{}", weapon);
            if weapons.contains_key(&weapon) && !current_page_set.contains(&weapon) {
                let mut finished = true;
                for key in weapons.keys() {
                    if weapons[key] < 2 {
                        println!("Missing {}", key);
                        finished = false;
                    }
                }
                if finished {
                    exit_loop = true;
                }
            }
            if !(current_page_set.contains(&weapon)) {
                current_page_set.insert(weapon.clone());
                if !weapons.contains_key(&weapon) {
                    weapons.insert(weapon.clone(), 1);
                } else {
                    weapons.insert(weapon.clone(), weapons[&weapon] + 1);
                }
            }
        }
        if checker.check_change(&parsed) {
            progress_list();
        } else {
            next_page();
            checker = ChangeChecker::new();
            current_page_set = HashSet::<String>::new();
        }
    }
    let mut vec = Vec::<Weapon>::new();
    for key in weapons.keys() {
        let weapon = WEAPONS[&key.to_lowercase()].clone();
        vec.push(weapon);
    }
    return vec;
}

fn ocr(data:&[u8], ltbox:&leptess::leptonica::Box) -> String{
    let mut lt = leptess::LepTess::new(None, "eng").unwrap();
    lt.set_image_from_mem(&data).unwrap();
    lt.set_source_resolution(70);
    lt.set_rectangle(ltbox);
    return lt.get_utf8_text().unwrap()
}

fn ocr_paragraphs(data:&[u8], ltbox:&leptess::leptonica::Box) -> Vec<(String, leptess::leptonica::Box)>{
    let mut api = tesseract::TessApi::new(None, "eng").unwrap();
    let img = leptess::leptonica::pix_read_mem(data).unwrap();
    api.set_image(&img);
    api.set_source_resolution(70);
    //lt.set_rectangle(ltbox);

    let boxes = api
        .get_component_images(leptess::capi::TessPageIteratorLevel_RIL_TEXTLINE, true)
        .unwrap();


    let mut result:Vec<(String, leptess::leptonica::Box)> = vec![];
    for b in boxes {
        api.set_rectangle(&b);
        let text = api.get_utf8_text().unwrap();
        result.push((text,b));
    }
    return result
}

fn parse_players(blocks:Vec<(String, leptess::leptonica::Box)>) -> Vec<(String, leptess::leptonica::Box)> {
    let re = Regex::new(PLAYER_REGEX).unwrap();
    let mut ret = vec![];
    for line in blocks {
        let captures:Vec<regex::Captures> = re.captures_iter(line.0.as_str()).collect();
        for cap in &captures {
            assert_eq!(&captures.len(), &(1 as usize));
            ret.push((cap[1].to_string(), line.1));
            break;
        }
    }
    return ret;
}

fn player_list_reset () {
    adb::tap(900,320);
    thread::sleep(HALF_SECOND);
}

fn player_list_accept() {
    adb::tap(700,2000);
    thread::sleep(ONE_SECOND);
}

fn disconnect_accept() {
    adb::tap(700,2100);
    thread::sleep(THREE_SECONDS);
}

fn reset_list() {
    press_back();
    adb::tap(200, 1250);
    thread::sleep(THREE_SECONDS);
}

fn press_back() {
    adb::tap(150, 1900);
    thread::sleep(ONE_SECOND);

}

fn check_disconnect() {
    let text = do_capture();
    let text = text.replace("\n", "");
    if text.contains("Connection to server failed.") {
        disconnect_accept();
        check_disconnect();
    }
}

fn tap_player_box(ltbox:&leptess::leptonica::Box) {
    let box_val = ltbox.get_val();
    adb::tap((box_val.x + box_val.w/2) as u32,(box_val.y + box_val.h/2) as u32);
}

fn open_user_filter() {
    adb::tap(900,1900);
    thread::sleep(ONE_SECOND);
}


fn do_capture() -> String{
    let data = adb::cap_screen();
    return ocr(&data, &leptess::leptonica::Box::new(20,660,945,1140).unwrap());
}

fn progress_list() {
    adb::swipe(500,1120,500,680,Some(500));
    thread::sleep(ONE_SECOND);
}

fn next_page() {
    adb::tap(700,1900);
    thread::sleep(THREE_SECONDS);
}

fn parse_text(text:&String) -> Vec<String> {
    let re = Regex::new(WEAPON_REGEX).unwrap();
    let mut weapons:Vec<String> = vec![];
    for cap in re.captures_iter(text.as_str()) {
        let name = String::from(&cap[1]);
        let name = name.strip_suffix("'s").unwrap();
        let name = corrected_name(name.to_string());
        weapons.push(name);
    }
    return weapons;
}

/// remove false possessives IE "Sweet Release Harp's Holy Knight" -> "Sweet Release Harp"
/// also fix bad non-english words, IE the imfamous zweihander
fn corrected_name (name:String) -> String {
    let mut fixed_name = name.clone();
    for suffix in FAKE_NAME_SUFFIXES {
        if name.ends_with(suffix) {
            fixed_name = fixed_name.strip_suffix(format!("'s {}", suffix).as_str()).unwrap().to_string();
        }
    }
    let fixed_name = closest_match_name(fixed_name);
    return fixed_name;
}

fn closest_match_name(weapon:String) -> String {
    if WEAPONS.contains_key(&weapon.to_lowercase()) {
        return weapon;
    } else {
        let mut matches = vec![];
        for weapon_entry in WEAPONS.values() {
            let weapon_name = &weapon_entry.name;
            if edit_distance(&weapon_name, &weapon) < 3 {
                matches.push(weapon_name.clone());
            }
        }
        if matches.len() > 1 {
            panic!("Ambiguous weapon name: {}", weapon);
        }
        if matches.len() == 0 {
            panic!("Unknown weapon name: {}", weapon);
        }
        println!("{} -> {}", &weapon, &matches[0]);
        return matches[0].clone();
    }
}