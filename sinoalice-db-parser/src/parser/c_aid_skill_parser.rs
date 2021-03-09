use crate::parser;

pub fn parse(text:&String) -> sinoalice_core::CAidSkill {
    let mut tokens = parser::tokenify(text);
    parser::pop_and_assert(&mut tokens, "Fixed");
    parser::pop_and_assert(&mut tokens, "chance");
    parser::pop_and_assert(&mut tokens, "to");
    let token = tokens.pop().unwrap();
    if token.to_lowercase().as_str() == "increase" {
        return alt_increase(&mut tokens);
    }
    let effect_amount = parser::EffectAmount::from_adverb(token);
    let token = tokens.pop().unwrap();
    match token.to_lowercase().as_str() {
        "increase" => return increase(&mut tokens, effect_amount),
        "reduce" => return reduce(&mut tokens, effect_amount),
        _=>panic!("Unknown effect token: {}", token)
    }
}

fn alt_increase(tokens:&mut Vec<&str>) -> sinoalice_core::CAidSkill {
    let token = tokens.pop().unwrap();
    if token == "the" {
        return increase_the(tokens);
    } else {
        tokens.push(token);
    }
    let mut effects = parser::parse_stat_modification_type(tokens);
    parser::pop_and_assert(tokens, "of");
    parser::pop_and_assert(tokens, "one");
    parser::pop_and_assert(tokens, "member");
    parser::pop_and_assert(tokens, "of");
    parser::pop_and_assert(tokens, "the");
    parser::pop_and_assert(tokens, "vanguard");
    parser::pop_and_assert(tokens, "by");
    parser::pop_and_assert(tokens, "a");
    let token = tokens.pop().unwrap();
    let effect_amount = parser::EffectAmount::from_adjective(token);
    if effect_amount == parser::EffectAmount::Slight {
        effects.multiply_atk(0.24);
        effects.multiply_def(0.40);
    } else {
        // Sometimes they say great or moderate interchangabley
        effects.multiply_atk(0.36);
        effects.multiply_def(0.64);
    }
    parser::pop_and_assert(tokens, "amount");
    let trigger = parse_trigger(tokens);
    let skill = sinoalice_core::CAidSkill{trigger:trigger, aid_effect:sinoalice_core::AidEffect::Buff(effects)};
    return skill;
}

fn increase_the (tokens:&mut Vec<&str>)-> sinoalice_core::CAidSkill {
    let mut effects = parser::parse_stat_modification_type(tokens);
    parser::pop_and_assert(tokens, "of");
    parser::pop_and_assert(tokens, "1");
    parser::pop_and_assert(tokens, "ally");
    parser::pop_and_assert(tokens, "in");
    parser::pop_and_assert(tokens, "the");
    parser::pop_and_assert(tokens, "vanguard");
    parser::pop_and_assert(tokens, "by");
    parser::pop_and_assert(tokens, "a");
    let token = tokens.pop().unwrap();
    let effect_amount = parser::EffectAmount::from_adjective(token);
    if effect_amount == parser::EffectAmount::Great {
        effects.multiply_atk(0.36);
        effects.multiply_def(0.64);
    } else {
        effects.multiply_atk(0.24);
        effects.multiply_def(0.40);
    }
    parser::pop_and_assert(tokens, "amount");
    let trigger = parse_trigger(tokens);
    let skill = sinoalice_core::CAidSkill{trigger:trigger, aid_effect:sinoalice_core::AidEffect::Buff(effects)};
    return skill;
}

fn increase(tokens:&mut Vec<&str>, effect_amount:parser::EffectAmount) -> sinoalice_core::CAidSkill {
    let token = tokens.pop().unwrap();
    match token.to_lowercase().as_str() {
        "support" => return support_boon(tokens,effect_amount),
        "hp" => return recovery_support(tokens,effect_amount),
        "one" => return one_buff(tokens,effect_amount),
        "own" => return own_buff(tokens,effect_amount),
        "damage" => return dauntless_courage(tokens,effect_amount),
        _=>panic!("Unknown effect token: {}", token)
    }
}

fn support_boon(tokens:&mut Vec<&str>, effect_amount:parser::EffectAmount)-> sinoalice_core::CAidSkill {
    parser::pop_and_assert(tokens, "effects");
    assert_eq!(parse_trigger(tokens), sinoalice_core::Trigger::Support);
    let aid_effect:sinoalice_core::AidEffect;
    if effect_amount == parser::EffectAmount::Great {
        aid_effect = sinoalice_core::AidEffect::Amplify(0.2);
    } else if effect_amount == parser::EffectAmount::Moderate {
        aid_effect = sinoalice_core::AidEffect::Amplify(0.15);
    } else {
        aid_effect = sinoalice_core::AidEffect::Amplify(0.1);
    }
    let skill = sinoalice_core::CAidSkill{trigger:sinoalice_core::Trigger::Support, aid_effect:aid_effect};
    return skill;
}

fn recovery_support(tokens:&mut Vec<&str>, effect_amount:parser::EffectAmount)-> sinoalice_core::CAidSkill {
    parser::pop_and_assert(tokens, "recovered");
    assert_eq!(parse_trigger(tokens), sinoalice_core::Trigger::Recover);
    let aid_effect:sinoalice_core::AidEffect;
    if effect_amount == parser::EffectAmount::Great {
        aid_effect = sinoalice_core::AidEffect::Amplify(0.2);
    } else if effect_amount == parser::EffectAmount::Moderate {
        aid_effect = sinoalice_core::AidEffect::Amplify(0.15);
    } else {
        aid_effect = sinoalice_core::AidEffect::Amplify(0.1);
    }
    let skill = sinoalice_core::CAidSkill{trigger:sinoalice_core::Trigger::Recover, aid_effect:aid_effect};
    return skill;
}


fn dauntless_courage(tokens:&mut Vec<&str>, effect_amount:parser::EffectAmount)-> sinoalice_core::CAidSkill {
    assert_eq!(parse_trigger(tokens), sinoalice_core::Trigger::Attack);
    let aid_effect:sinoalice_core::AidEffect;
    if effect_amount == parser::EffectAmount::Great {
        aid_effect = sinoalice_core::AidEffect::Amplify(0.2);
    } else if effect_amount == parser::EffectAmount::Moderate {
        aid_effect = sinoalice_core::AidEffect::Amplify(0.15);
    } else {
        aid_effect = sinoalice_core::AidEffect::Amplify(0.1);
    }
    let skill = sinoalice_core::CAidSkill{trigger:sinoalice_core::Trigger::Attack, aid_effect:aid_effect};
    return skill;
}

fn one_buff (tokens:&mut Vec<&str>, effect_amount:parser::EffectAmount)-> sinoalice_core::CAidSkill {
    parser::pop_and_assert(tokens, "vanguard");
    parser::pop_and_assert(tokens, "member's");
    let mut effects = parser::parse_stat_modification_type(tokens);
    if effect_amount == parser::EffectAmount::Great {
        effects.multiply_atk(0.36);
        effects.multiply_def(0.64);
    } else {
        effects.multiply_atk(0.24);
        effects.multiply_def(0.40);
    }
    let trigger = parse_trigger(tokens);
    let skill = sinoalice_core::CAidSkill{trigger:trigger, aid_effect:sinoalice_core::AidEffect::Buff(effects)};
    return skill;
}

fn own_buff (tokens:&mut Vec<&str>, effect_amount:parser::EffectAmount)-> sinoalice_core::CAidSkill {
    let mut effects = parser::parse_stat_modification_type(tokens);
    if effect_amount == parser::EffectAmount::Great {
        effects.multiply_atk(0.36);
        effects.multiply_def(0.64);
    } else {
        effects.multiply_atk(0.24);
        effects.multiply_def(0.40);
    }
    let trigger = parse_trigger(tokens);
    let skill = sinoalice_core::CAidSkill{trigger:trigger, aid_effect:sinoalice_core::AidEffect::Buff(effects)};
    return skill;
}


fn one_debuff (tokens:&mut Vec<&str>, effect_amount:parser::EffectAmount)-> sinoalice_core::CAidSkill {
    let mut effects = parser::parse_stat_modification_type(tokens);
    if effect_amount == parser::EffectAmount::Moderate {
        effects.multiply_atk(0.36);
        effects.multiply_def(0.64);
    } else {
        effects.multiply_atk(0.24);
        effects.multiply_def(0.40);
    }
    let trigger = parse_trigger(tokens);
    let skill = sinoalice_core::CAidSkill{trigger:trigger, aid_effect:sinoalice_core::AidEffect::Debuff(effects)};
    return skill;
}

fn replenish_magic(tokens:&mut Vec<&str>, effect_amount:parser::EffectAmount)-> sinoalice_core::CAidSkill {
    parser::pop_and_assert(tokens, "consumption");
    assert_eq!(parse_trigger(tokens), sinoalice_core::Trigger::All);
    let aid_effect:sinoalice_core::AidEffect;
    if effect_amount == parser::EffectAmount::Great {
        aid_effect = sinoalice_core::AidEffect::RestoreSp(0.7);
    } else {
        aid_effect = sinoalice_core::AidEffect::RestoreSp(0.8);
    }
    let skill = sinoalice_core::CAidSkill{trigger:sinoalice_core::Trigger::All, aid_effect:aid_effect};
    return skill;
}

fn parse_trigger(tokens:&mut Vec<&str>) -> sinoalice_core::Trigger {
    parser::pop_and_assert(tokens, "while");
    let token = tokens.pop().unwrap();
    match token.to_lowercase().as_str() {
        "executing" => {
            parser::pop_and_assert(tokens, "commands.");
            return sinoalice_core::Trigger::All
        },
        "providing" => {
            parser::pop_and_assert(tokens, "backup.");
            return sinoalice_core::Trigger::Support
        },
        "attacking." => {
            return sinoalice_core::Trigger::Attack
        },
        "healing." => {
            return sinoalice_core::Trigger::Recover
        },
        _=>panic!("Unknown effect token: {}", token)
    }
}

fn reduce(tokens:&mut Vec<&str>, effect_amount:parser::EffectAmount) -> sinoalice_core::CAidSkill {
    let token = tokens.pop().unwrap();
    match token.to_lowercase().as_str() {
        "sp" => return replenish_magic(tokens,effect_amount),
        "enemy's" => return one_debuff(tokens,effect_amount),
        _=>panic!("Unknown effect token: {}", token)
    }
}