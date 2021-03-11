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
use sinoalice_core::{Weapon, LeveledWeapon};
use leptess::tesseract;
use edit_distance::edit_distance;
// Our CLI arguments. (help and version are automatically generated)
// Documentation on how to use:
// https://docs.rs/structopt/0.2.10/structopt/index.html#how-to-derivestructopt
#[derive(StructOpt, Debug)]
struct Cli {
    /// If given, parse only the specified players' grids
    player_names:Vec<String>
}

const WEAPON_JSON:&'static str = include_str!("weapons.json");
lazy_static! {
    static ref WEAPONS:HashMap::<String, Weapon> = serde_json::from_str(WEAPON_JSON).unwrap();
}

struct ChangeChecker {
    last_seen:Vec<SkillActivation>
}

impl ChangeChecker {
    pub fn check_change (&mut self, current:&Vec<SkillActivation>) -> bool {
        if current == &self.last_seen {
            return false;
        } else {
            self.last_seen = current.clone();
            return true;
        }
    }

    pub fn new () -> ChangeChecker {
        return ChangeChecker {last_seen:vec![SkillActivation{weapon_name:"doesn't exist".to_string(), lvl:0}]};
    }
}

const WEAPON_REGEX:&'static str = "(?m)^((?:.+?'s)+)[^.]*?Lv.(\\d+)\\s+activated";//"(?m)^(.*)'s(?! guildship)[\\s\\S]*?activated";//([^.]|Lv\\.)*activated";//[^.]*activated";
const SUPPORT_REGEX:&'static str = "(?m)^((?:.+?'s)+)[^.]*?Lv.(\\d+)\\s+also\\s+activated";
const PLAYER_REGEX:&'static str = "(\\w+)\\sRank\\.\\d+";
const PAGE_REGEX:&'static str = "(\\d+)m\\s*/\\s*20m";

const HALF_SECOND:time::Duration = time::Duration::from_millis(500);
const ONE_SECOND:time::Duration = time::Duration::from_millis(1000);
const THREE_SECONDS:time::Duration = time::Duration::from_millis(3000);

const XOR_INVERTER:u8 = u8::MAX;

const FAKE_NAME_SUFFIXES: &'static [&'static str] = &["Exorcist", "Holy Knight", "Hero", "Knight", "Sorcerer", "Warrior", "Wind God", "Water God", "Flame God", "Sage", "Angel", "Barrier Master"];
//const CORRECTED_WORDS: &'static [&'static str] = &["Zweihänder of Justice", "Kainé's Sword"];

fn main() {
    let args = Cli::from_args();
    //println!("{}", parsed);
    // let grid = scrape();
    // println!("{}", serde_json::to_string(&grid).unwrap())
    //println!("{:?}", res);
    //progress_list();
    let mut all_players = HashMap::<String, Vec<LeveledWeapon>>::new();
    if args.player_names.is_empty() {
        let mut seen_players = vec![];
        let mut player = next_player(&seen_players);
        while player.is_some() {
            seen_players.push(player.as_ref().unwrap().clone());
            reset_list();
            let grid = scrape();
            all_players.insert(player.unwrap(), grid);
            check_disconnect();
            player = next_player(&seen_players);
        }
    } else {
        for player in &args.player_names {
            let player = select_player(&player);
            reset_list();
            let grid = scrape();
            all_players.insert(player.unwrap(), grid);
            check_disconnect();
        }
    }
    std::fs::write("all_players.json", serde_json::to_string_pretty(&all_players).unwrap()).unwrap();
}

fn select_player(chosen_player_name:&String) -> Option<String> {
    let data = adb::cap_screen();
    navigate_to_log(&data);
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
            let player_name = player.0.to_lowercase();
            println!("{}", player_name);
            if chosen_player_name == &player_name {
                found_player = Some(player_name);
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

fn next_player(seen:&Vec<String>) -> Option<String> {
    let data = adb::cap_screen();
    navigate_to_log(&data);
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
            let player_name = player.0.to_lowercase();
            println!("{}", player_name);
            if !seen.contains(&player_name) {
                found_player = Some(player_name);
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
    let data = adb::cap_screen();
    if !is_battle_log(&data) {
        return next_player(seen)
    } else {
        return found_player
    }
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

fn scrape() -> Vec<LeveledWeapon>{
    let mut weapons_seen_count = HashMap::<String,u32>::new();
    let mut weapons_levels = HashMap::<String,u32>::new();
    let mut weapons_aid_levels = HashMap::<String,u32>::new();
    let mut checker = ChangeChecker::new();
    let mut current_page_set = HashSet::<String>::new();
    let mut exit_loop = false;
    while !exit_loop {
        let current = do_capture();
        let mut parsed = parse_text(&current);
        if parsed.0.is_empty() {
            check_disconnect();
            parsed = parse_text(&current);
        }
        for skill_activation in parsed.0.clone() {
            let weapon = skill_activation.weapon_name.to_lowercase();
            println!("\t{} skill lvl {}", &weapon, skill_activation.lvl);
            if !weapons_levels.contains_key(&weapon) {
                weapons_levels.insert(weapon.clone(), skill_activation.lvl);
            }
            if !(current_page_set.contains(&weapon)) {
                current_page_set.insert(weapon.clone());
                if !weapons_seen_count.contains_key(&weapon) {
                    weapons_seen_count.insert(weapon.clone(), 1);
                } else {
                    weapons_seen_count.insert(weapon.clone(), weapons_seen_count[&weapon] + 1);
                }
            }
        }
        for aid_skill_activation in parsed.1.clone() {
            let weapon = aid_skill_activation.weapon_name.to_lowercase();
            println!("\t\t{} aid lvl {}", &weapon, aid_skill_activation.lvl);
            if !weapons_aid_levels.contains_key(&weapon) {
                weapons_aid_levels.insert(weapon.clone(), aid_skill_activation.lvl);
            } else {
                assert_eq!(weapons_aid_levels[&weapon], aid_skill_activation.lvl);
            }
        }
        // let mut finished = true;
        // for key in weapons_seen_count.keys() {
        //     if weapons_seen_count[key] < NUM_REQUIRED_ACTIVATIONS{
        //         println!("Missing {} ({}/{})", key, weapons_seen_count[key], NUM_REQUIRED_ACTIVATIONS);
        //         finished = false;
        //     }
        // }
        // for key in weapons_aid_levels.keys() {
        //     if !weapons_levels.contains_key(key) {
        //         println!("Missing {} (0/{})", key, NUM_REQUIRED_ACTIVATIONS);
        //         finished = false;
        //     }
        // }
        // if finished {
        //     exit_loop = true;
        // }
        if checker.check_change(&parsed.0) {
            progress_list();
        } else {
            if get_page_num() == 1 {
                exit_loop = true;
            } else {
                next_page();
                checker = ChangeChecker::new();
                current_page_set = HashSet::<String>::new();
            }
        }
    }
    let mut vec = Vec::<LeveledWeapon>::new();
    for key in weapons_seen_count.keys() {
        let key = key.to_lowercase();
        let weapon = WEAPONS[&key].clone();
        let skill_level = weapons_levels.get(&key);
        let aid_skill_level = weapons_aid_levels.get(&key);
        vec.push(LeveledWeapon{weapon:weapon, c_skill_lvl:skill_level.copied(), c_aid_skill_lvl:aid_skill_level.copied()});
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
    let data = adb::cap_screen();
    navigate_to_log(&data);
    press_back();
    enter_log_from_history();
    let data = adb::cap_screen();
    navigate_to_log(&data);
}

fn enter_log_from_history() {
    adb::tap(200, 1250);
    thread::sleep(THREE_SECONDS);
}

fn press_back() {
    adb::tap(150, 1900);
    thread::sleep(ONE_SECOND);
}

fn get_page_string() -> String {
    let data = adb::cap_screen();
    return ocr(&data, &leptess::leptonica::Box::new(400,1850,250,100).unwrap());
}

fn get_page_num() -> u32 {
    let re = Regex::new(PAGE_REGEX).unwrap();
    let page_string = get_page_string();
    let cap = re.captures(page_string.as_str()).unwrap();
    return (&cap[1]).parse::<u32>().unwrap();
}

fn is_battle_log(data:&[u8]) -> bool {
    return true;
    println!("{}", data.len());
    let data = data_invert(data);
    let text = ocr(&data, &leptess::leptonica::Box::new(30,300,300,100).unwrap());
    let text = text.trim();
    let text = text.to_lowercase();
    println!("{}", text);
    return text == "battle log"
}

fn is_user_list(data:&[u8]) -> bool {
    let text = ocr(data, &leptess::leptonica::Box::new(400,1850,250,100).unwrap());
    let text = text.trim();
    let text = text.to_lowercase();
    return text == "user log"
}

fn is_history_page(data:&[u8]) -> bool {
    let text = ocr(data, &leptess::leptonica::Box::new(400,1850,250,100).unwrap());
    let text = text.trim();
    let text = text.to_lowercase();
    return text == "hs log"
}

fn navigate_to_log(data:&[u8]) {
    if is_battle_log(data) {
        return
    } else if is_user_list(data) {
        player_list_accept();
        return
    } else if is_history_page(data) {
        enter_log_from_history();
    }
}

fn data_invert(data:&[u8]) -> Vec<u8> {
    return data.iter().map(|x| x^XOR_INVERTER).collect();
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
    return ocr(&data, &leptess::leptonica::Box::new(20,650,945,1140).unwrap());
}

fn progress_list() {
    adb::swipe(500,1120,500,680,Some(500));
    thread::sleep(HALF_SECOND);
}

fn next_page() {
    adb::tap(700,1900);
    thread::sleep(THREE_SECONDS);
}

#[derive(PartialEq,Clone, Debug)]
struct SkillActivation {
    weapon_name:String,
    lvl:u32
}

#[derive(PartialEq,Clone, Debug)]
struct AidSkillActivation {
    weapon_name:String,
    lvl:u32
}

fn parse_text(text:&String) -> (Vec<SkillActivation>, Vec<AidSkillActivation>) {
    // I recompile these regexes on every run because I don't want to store them. Sue me.
    let weapon_re = Regex::new(WEAPON_REGEX).unwrap();
    let mut skill_activations:Vec<SkillActivation> = vec![];
    for cap in weapon_re.captures_iter(text.as_str()) {
        let name = String::from(&cap[1]);
        let name = corrected_name(name.to_string());
        if name.is_some() {
            // TODO move list to try to correct
            skill_activations.push(SkillActivation{weapon_name:name.unwrap(), lvl:(&cap[2]).parse::<u32>().unwrap()});
        }
    }
    let aid_re = Regex::new(SUPPORT_REGEX).unwrap();
    let mut aid_skill_activations:Vec<AidSkillActivation> = vec![];
    for cap in aid_re.captures_iter(text.as_str()) {
        let name = String::from(&cap[1]);
        let name = corrected_name(name.to_string());
        if name.is_some() {
            // TODO move list to try to correct
            aid_skill_activations.push(AidSkillActivation{weapon_name:name.unwrap(), lvl:(&cap[2]).parse::<u32>().unwrap()});
        }
    }
    return (skill_activations, aid_skill_activations);
}

/// remove false possessives IE "Sweet Release Harp's Holy Knight" -> "Sweet Release Harp"
/// also fix bad non-english words, IE the imfamous zweihander
fn corrected_name (name:String) -> Option<String> {
    let mut fixed_name = name.clone();
    for suffix in FAKE_NAME_SUFFIXES {
        if fixed_name.ends_with(format!(" {}'s",suffix).as_str()) {
            fixed_name = fixed_name.strip_suffix(format!(" {}'s",suffix).as_str()).unwrap().to_string();
        }
    }
    let fixed_name = closest_match_name(fixed_name);
    return fixed_name;
}

fn closest_match_name(weapon:String) -> Option<String> {
    // sometimes the ocr will not put a possessive in the correct place, so we try to find it as well with the edit distance
    if weapon.ends_with("'s") && WEAPONS.contains_key(&weapon.strip_suffix("'s").unwrap().to_lowercase()) {
        return Some(weapon.strip_suffix("'s").unwrap().to_string());
    } else {
        let mut matches = vec![];
        for weapon_entry in WEAPONS.values() {
            let weapon_name = format!("{}'s", &weapon_entry.name.to_lowercase());
            if edit_distance(&weapon_name, &weapon.to_lowercase()) < 4 {
                matches.push(weapon_name.clone());
            }
        }
        if matches.len() > 1 {
            println!("Ambiguous weapon name: {}", weapon);
            return None;
        }
        if matches.len() == 0 {
            println!("Unknown weapon name: {}", weapon);
            return None;
        }
        println!("{} -> {}", &weapon.to_lowercase(), &matches[0]);
        return Some(matches[0].clone().strip_suffix("'s").unwrap().to_string());
    }
}