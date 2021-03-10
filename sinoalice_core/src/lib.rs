#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

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
pub struct CSkill {
    pub effect:SkillEffect,
    pub min_targets:u32,
    pub max_targets:u32,
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

#[derive(PartialEq,Clone,Serialize,Deserialize, Debug)]
pub struct StatModifier {
    pub patk:f32,
    pub matk:f32,
    pub pdef:f32,
    pub mdef:f32
}

#[derive(PartialEq,Clone,Serialize,Deserialize, Debug)]
pub enum DamageType {
    Magic,
    Physical
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
