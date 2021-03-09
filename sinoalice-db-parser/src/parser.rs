mod c_skill_parser;
mod c_aid_skill_parser;

pub fn parse_c_skill(text:String) -> sinoalice_core::CSkill {
    c_skill_parser::parse(&text)
} 

pub fn parse_c_aid_skill(text:String) -> sinoalice_core::CAidSkill {
    return c_aid_skill_parser::parse(&text);
} 

pub fn parse_ele(text:String) -> sinoalice_core::WeaponEle {
    match text.as_str() {
        "./Weapons - SINoALICE Database_files/attribute_001.png" => return sinoalice_core::WeaponEle::Fire,
        "./Weapons - SINoALICE Database_files/attribute_002.png" => return sinoalice_core::WeaponEle::Water,
        "./Weapons - SINoALICE Database_files/attribute_003.png" => return sinoalice_core::WeaponEle::Wind,
        _=> panic!("Ele not found: {}", text)
    }
}



pub fn parse_stat_modification_type(tokens:&mut Vec<&str>) -> sinoalice_core::StatModifier {
    let mut modifier = sinoalice_core::StatModifier::new();
    let token = tokens.pop().unwrap();
    match token {
        "physical" => {
            let token = tokens.pop().unwrap();
            match token {
                "ATK" => {
                    modifier.patk = 1.;
                },
                "DEF" => {
                    modifier.pdef = 1.;
                },
                _=>panic!("Unknown stat modifier token: {}", token)
            }
        },
        "magical" => {
            let token = tokens.pop().unwrap();
            match token {
                "ATK" => {
                    modifier.matk = 1.;
                },
                "DEF" => {
                    modifier.mdef = 1.;
                },
                _=>panic!("Unknown stat modifier token: {}", token)
            }
        },
        _=>panic!("Unknown stat modifier token: {}", token)
    }
    let token = tokens.pop().unwrap();
    if token == "and" {
        let token = tokens.pop().unwrap();
        match token {
            "physical" => {
                let token = tokens.pop().unwrap();
                match token {
                    "ATK" => {
                        modifier.patk = 1.;
                    },
                    "DEF" => {
                        modifier.pdef = 1.;
                    },
                    _=>panic!("Unknown stat modifier token: {}", token)
                }
            },
            "magical" => {
                let token = tokens.pop().unwrap();
                match token {
                    "ATK" => {
                        modifier.matk = 1.;
                    },
                    "DEF" => {
                        modifier.mdef = 1.;
                    },
                    _=>panic!("Unknown stat modifier token: {}", token)
                }
            },
            _=>panic!("Unknown stat modifier token: {}", token)
        }
    } else {
        tokens.push(token);
    }
    return modifier;
}


pub fn pop_and_assert(vec:&mut Vec<&str>, eq:&str) {
    assert_eq!(vec.pop().unwrap(),eq)
}

#[derive(PartialEq,Clone)]
pub enum EffectAmount {
    Slight,
    Moderate,
    Great,
    Massive
}

impl EffectAmount {
    fn from_adjective(adjective:&str) -> EffectAmount{
        match adjective {
            "small" => return EffectAmount::Slight,
            "slight" => return EffectAmount::Slight,
            "slightly" => return EffectAmount::Slight, // problem in the data
            "moderate" => return EffectAmount::Moderate,
            "great" => return EffectAmount::Great,
            "massive" => return EffectAmount::Massive,
            _=>panic!("Unknown stat modifier adjective: {}", adjective)
        }
    }
    fn from_adverb(adverb:&str) -> EffectAmount {
        match adverb {
            "slightly" => return EffectAmount::Slight,
            "moderately" => return EffectAmount::Moderate,
            "greatly" => return EffectAmount::Great,
            "massively" => return EffectAmount::Massive,
            _=>panic!("Unknown stat modifier adverb: {}", adverb)
        }
    }
}

pub fn tokenify(text:&String) -> Vec<&str> {
    return text.split_whitespace().rev().collect();
}