use crate::parser;

pub fn correct_skill_ratio(c_skill:&sinoalice_core::CSkill, amount:parser::EffectAmount) -> sinoalice_core::CSkill {
    let mut skill = c_skill.clone();
    match &mut skill.effect {
        sinoalice_core::SkillEffect::Buff(ref mut stat_modifier) => {
            correct_stat_ratio(stat_modifier, skill.min_targets, skill.max_targets, amount)
        }, 
        sinoalice_core::SkillEffect::Debuff(ref mut stat_modifier) => {
            correct_stat_ratio(stat_modifier, skill.min_targets, skill.max_targets, amount)
        },
        _ => {panic!("Wrong skill type for ratio correcter")}
    }
    return skill;
}

fn correct_stat_ratio(stats:&mut sinoalice_core::StatModifier, min_targets:u32, max_targets:u32, amount:parser::EffectAmount) {
    let count = stats.stat_count();
    if count == 1 {
        correct_one_stat(stats, min_targets, max_targets, amount);
    } else {
        correct_two_stats(stats, min_targets, max_targets, amount);
    }
}

fn correct_one_stat(stats:&mut sinoalice_core::StatModifier, min_targets:u32, max_targets:u32, amount:parser::EffectAmount) {
    match min_targets {
        1 => {
            match max_targets {
                1 => {
                    match amount {
                        parser::EffectAmount::Slight => {panic!("Unknown ratio")},
                        parser::EffectAmount::Moderate => {panic!("Unknown ratio")},
                        parser::EffectAmount::Great => {
                            stats.multiply_atk(0.76);
                            stats.multiply_def(1.36);
                        },
                        parser::EffectAmount::Massive => {
                            stats.multiply_atk(1.06);
                            stats.multiply_def(1.8);
                        },
                    }
                },
                2 => {
                    match amount {
                        parser::EffectAmount::Slight => {panic!("Unknown ratio")},
                        parser::EffectAmount::Moderate => {
                            stats.multiply_atk(-0.6);
                            stats.multiply_def(-0.84);
                            
                        },
                        parser::EffectAmount::Great => {
                            stats.multiply_atk(0.9);
                            stats.multiply_def(1.5);
                        },
                        parser::EffectAmount::Massive => {panic!("Unknown ratio")},
                    }
                },
                _ => panic!("Bad max targets: {}", max_targets)
            }
        },
        2 => {
            match max_targets {
                2 => {
                    match amount {
                        parser::EffectAmount::Slight => {panic!("Unknown ratio")},
                        parser::EffectAmount::Moderate => {
                            stats.multiply_atk(0.6);
                            stats.multiply_def(1.2);
                        },
                        parser::EffectAmount::Great => {panic!("Unknown ratio")},
                        parser::EffectAmount::Massive => {
                            stats.multiply_atk(0.84);
                            stats.multiply_def(1.36);
                        },
                    }
                },
                _ => panic!("Bad max targets: {}", max_targets)
            }
        },
        _ => panic!("Bad min targets: {}", min_targets)
    }
}

fn correct_two_stats(stats:&mut sinoalice_core::StatModifier, min_targets:u32, max_targets:u32, amount:parser::EffectAmount) {
    match min_targets {
        1 => {
            match max_targets {
                1 => {
                    match amount {
                        parser::EffectAmount::Slight => {
                            stats.multiply_atk(0.54);
                            stats.multiply_def(0.72);
                        },
                        parser::EffectAmount::Moderate => {
                            stats.multiply_atk(0.70);
                            stats.multiply_def(1.36);
                        },
                        parser::EffectAmount::Great => {
                            stats.multiply_atk(0.96);
                            stats.multiply_def(1.66);
                        },
                        parser::EffectAmount::Massive => {
                            stats.multiply_atk(1.06);
                            stats.multiply_def(1.8);
                        },
                    }
                },
                2 => {
                    match amount {
                        parser::EffectAmount::Slight => {panic!("Unknown ratio")},
                        parser::EffectAmount::Moderate => {panic!("Unknown ratio")},
                        parser::EffectAmount::Great => {
                            stats.multiply_atk(0.84);
                            stats.multiply_def(1.36);
                        },
                        parser::EffectAmount::Massive => {panic!("Unknown ratio")},
                    }
                },
                _ => panic!("Bad max targets: {}", max_targets)
            }
        },
        2 => {
            match max_targets {
                2 => {
                    match amount {
                        parser::EffectAmount::Slight => {
                            stats.multiply_atk(0.42);
                            stats.multiply_def(0.72);
                        },
                        parser::EffectAmount::Moderate => {
                            stats.multiply_atk(0.66);
                            stats.multiply_def(1.2);
                        },
                        parser::EffectAmount::Great => {panic!("Unknown ratio")},
                        parser::EffectAmount::Massive => {
                            stats.multiply_atk(0.84);
                            stats.multiply_def(1.36);
                        },
                    }
                },
                _ => panic!("Bad max targets: {}", max_targets)
            }
        },
        _ => panic!("Bad min targets: {}", min_targets)
    }
}