#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use std::ops;
use serde::{Serialize, Deserialize};

#[derive(PartialEq,Clone,Serialize,Deserialize, Debug)]
pub struct Weapon {
    pub name:String,
    pub c_skill:CSkill,
    pub c_aid_skill:CAidSkill,
    pub weapon_type:WeaponType,
    pub weapon_ele:WeaponEle
}

#[derive(PartialEq,Clone,Serialize,Deserialize, Debug)]
pub struct LeveledWeapon {
    pub weapon:Weapon,
    pub c_skill_lvl:Option<u32>,
    pub c_aid_skill_lvl:Option<u32>
}

#[derive(PartialEq,Clone,Serialize,Deserialize, Debug)]
pub struct SimpleWeapon {
    pub name:String,
    pub c_lvl:Option<u32>,
    pub c_aid_lvl:Option<u32>
}

impl LeveledWeapon {
    pub fn scaled_skill(&self) -> CSkill {
        return CSkill {effect:&self.weapon.c_skill.effect*LeveledWeapon::get_c_skill_level_multiplier(self.c_skill_lvl.unwrap_or(1)), min_targets:self.weapon.c_skill.min_targets, max_targets:self.weapon.c_skill.max_targets}
    }

    pub fn get_c_skill_level_multiplier(level:u32) -> f32 {
        let mut mult = 1.+((level-1) as f32 * 0.04);
        if level >= 15 {
            mult += 0.04;
        }
        if level == 20 {
            mult += 0.05;
        }
        return mult;
    }

    pub fn get_c_aid_skill_level_chance(level:u32) -> f32 {
        let mut chance = 0.04+((level-1) as f32 * 0.005);
        if level >= 15 {
            chance += 0.005;
        }
        if level == 20 {
            chance += 0.005;
        }
        return chance;
    }
}

#[derive(PartialEq,Clone,Serialize,Deserialize, Debug)]
pub struct CSkill {
    pub effect:SkillEffect,
    pub min_targets:u32,
    pub max_targets:u32,
}

impl CSkill {
    pub fn expected_effect(&self) -> SkillEffect {
        return &self.effect*((self.min_targets + self.max_targets) as f32 / 2.)
    }
}

#[derive(PartialEq,Clone,Serialize,Deserialize, Debug)]
pub struct CAidSkill {
    pub trigger:Trigger,
    pub aid_effect:AidEffect
}

#[derive(Eq,PartialEq,Hash,Clone,Serialize,Deserialize, Debug)]
pub enum Trigger {
    Attack,
    Support,
    Recover,
    All
}

#[derive(Eq,PartialEq,Hash,Clone,Serialize,Deserialize, Debug)]
pub enum WeaponType {
    Tome,
    Instr,
    Staff,
    Focus,
    Blade,
    Hammer,
    Ranged,
    Polearm
}

#[derive(Eq,PartialEq,Hash,Clone,Serialize,Deserialize, Debug)]
pub enum WeaponEle {
    Fire,
    Water,
    Wind
}

#[derive(PartialEq,Clone,Serialize,Deserialize, Debug)]
pub enum SkillEffect {
    Buff(StatModifier),
    Debuff(StatModifier),
    Recover(f32),
    Damage(f32, DamageType)
}

impl ops::Mul<f32> for &SkillEffect {
    type Output=SkillEffect;
    fn mul (self, mult:f32) -> SkillEffect {
        match self {
            SkillEffect::Buff(modifier) => {
                return SkillEffect::Buff(modifier*mult)
            },
            SkillEffect::Debuff(modifier) => {
                return SkillEffect::Debuff(modifier*mult)
            },
            SkillEffect::Recover(modifier) => {
                return SkillEffect::Recover(modifier*mult)
            },
            SkillEffect::Damage(amount, damage_type) => {
                return SkillEffect::Damage(amount*mult, damage_type.clone())
            },
        }
    }
}


#[derive(PartialEq,Clone,Serialize,Deserialize, Debug)]
pub struct StatModifier {
    pub patk:f32,
    pub matk:f32,
    pub pdef:f32,
    pub mdef:f32
}

impl ops::Mul<f32> for &StatModifier {
    type Output=StatModifier;
    fn mul (self, mult:f32) -> StatModifier {
        let mut out = self.clone();
        out.patk *= mult;
        out.matk *= mult;
        out.pdef *= mult;
        out.mdef *= mult;
        return out;
    }

}

impl StatModifier {
    pub fn new () -> StatModifier {
        return StatModifier {patk:0., matk:0., pdef:0., mdef:0.}
    }
    pub fn multiply_def(&mut self, multiplier:f32) {
        self.pdef*=multiplier;
        self.mdef*=multiplier;
    }
    pub fn multiply_atk(&mut self, multiplier:f32) {
        self.patk*=multiplier;
        self.matk*=multiplier;
    }
    pub fn stat_count(&self) -> u32 {
        let mut count = 0;
        if self.patk != 0. {
            count+=1;
        }
        if self.matk != 0. {
            count+=1;
        }
        if self.pdef != 0. {
            count+=1;
        }
        if self.mdef != 0. {
            count+=1;
        }
        return count;
    }
}

#[derive(PartialEq,Clone,Serialize,Deserialize, Debug)]
pub enum DamageType {
    Magic,
    Physical
}

#[derive(PartialEq,Clone,Serialize,Deserialize, Debug)]
pub enum AidEffect {
    Amplify(f32),
    Buff(StatModifier),
    Debuff(StatModifier),
    RestoreSp(f32)
}

impl WeaponType {
    pub fn from_str(string:&str) ->WeaponType {
        match string.to_lowercase().as_str() {
            "tome"=>return WeaponType::Tome,
            "instr."=>return WeaponType::Instr,
            "staff"=>return WeaponType::Staff,
            "focus"=>return WeaponType::Focus,
            "blade"=>return WeaponType::Blade,
            "hammer"=>return WeaponType::Hammer,
            "ranged"=>return WeaponType::Ranged,
            "polearm"=>return WeaponType::Polearm,
            _=>panic!("Unrecognized weapon type: {}", string)
        }
    }
}
