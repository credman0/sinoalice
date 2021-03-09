use crate::parser;
use crate::ratios;

pub fn parse(text:&String) -> sinoalice_core::CSkill {
    let mut tokens = parser::tokenify(text);
    let token = tokens.pop().unwrap();
    match token.to_lowercase().as_str() {
        "deal" => return deal(&mut tokens),
        "reduce" => return reduce(&mut tokens),
        "increase" => return increase(&mut tokens),
        "heal" => return heal(&mut tokens),
        _=>panic!("Unknown effect token: {}", token)
    }
}

fn deal(tokens:&mut Vec<&str>) -> sinoalice_core::CSkill {
    let token = tokens.pop().unwrap();
    let effect_amount = parser::EffectAmount::from_adjective(token);
    let token = tokens.pop().unwrap();
    let damage_type;
    match token.to_lowercase().as_str() {
        "physical" => damage_type = sinoalice_core::DamageType::Physical,
        "magical" => damage_type = sinoalice_core::DamageType::Physical,
        _=>panic!("Unknown damage type token: {}", token)
    }
    parser::pop_and_assert(tokens,"damage");
    parser::pop_and_assert(tokens,"to");
    let targets = parse_target_quantity(tokens);
    let skill = sinoalice_core::CSkill{effect:sinoalice_core::SkillEffect::Damage(-1.0,damage_type), min_targets:targets.0, max_targets:targets.1};
    let token = tokens.pop().unwrap();
    match token.to_lowercase().as_str() {
        "enemies," => {},
        "enemies." => {},
        "enemy." => {},
        "enemy," => {},
        _=>panic!("Unknown enemy type token: {}", token)
    }
    return skill;
}

fn reduce(tokens:&mut Vec<&str>) -> sinoalice_core::CSkill {
    parser::pop_and_assert(tokens,"the");
    let reduction_type = parser::parse_stat_modification_type(tokens);
    parser::pop_and_assert(tokens,"of");
    let targets = parse_target_quantity(tokens);
    // ignore the enemies/allies token here
    tokens.pop().unwrap();
    parser::pop_and_assert(tokens,"by");
    parser::pop_and_assert(tokens,"a");
    let token = tokens.pop().unwrap();
    let amount = parser::EffectAmount::from_adjective(token);
    parser::pop_and_assert(tokens,"amount.");
    let skill = sinoalice_core::CSkill{effect:sinoalice_core::SkillEffect::Debuff(reduction_type), min_targets:targets.0, max_targets:targets.1};
    let skill = ratios::correct_skill_ratio(&skill, amount);
    //println!("{}", serde_json::to_string_pretty(&skill).unwrap());
    return skill;
}

fn parse_target_quantity(tokens:&mut Vec<&str>) -> (u32,u32) {
    let token = tokens.pop().unwrap();
    if token.contains("-") {
        let split:Vec<&str> = token.split("-").collect();
        let first = split[0].parse::<u32>().unwrap();
        let second = split[1].parse::<u32>().unwrap();
        return (first,second);
    } else {
        let result = token.parse::<u32>().unwrap();
        return (result,result);
    }
}

fn increase(tokens:&mut Vec<&str>) -> sinoalice_core::CSkill {
    parser::pop_and_assert(tokens,"the");
    let reduction_type = parser::parse_stat_modification_type(tokens);
    parser::pop_and_assert(tokens,"of");
    let targets = parse_target_quantity(tokens);
    // ignore the enemies/allies token here
    tokens.pop().unwrap();
    parser::pop_and_assert(tokens,"by");
    parser::pop_and_assert(tokens,"a");
    let token = tokens.pop().unwrap();
    let amount = parser::EffectAmount::from_adjective(token);
    let token = tokens.pop().unwrap();
    assert_eq!(token=="amount." || token == "amount,", true);
    let skill = sinoalice_core::CSkill{effect:sinoalice_core::SkillEffect::Buff(reduction_type), min_targets:targets.0, max_targets:targets.1};
    let skill = ratios::correct_skill_ratio(&skill, amount);
    //println!("{}", serde_json::to_string_pretty(&skill).unwrap());
    return skill;
}

fn heal(tokens:&mut Vec<&str>) -> sinoalice_core::CSkill {
    let targets = parse_target_quantity(tokens);
    tokens.pop().unwrap(); // either allies or ally
    parser::pop_and_assert(tokens,"by");
    parser::pop_and_assert(tokens,"a");
    let token = tokens.pop().unwrap();
    let amount = parser::EffectAmount::from_adjective(token);
    parser::pop_and_assert(tokens,"amount");
    parser::pop_and_assert(tokens,"of");
    // TODO parse the rest
    let skill = sinoalice_core::CSkill{effect:sinoalice_core::SkillEffect::Recover(-1.0), min_targets:targets.0, max_targets:targets.1};
    return skill;
}