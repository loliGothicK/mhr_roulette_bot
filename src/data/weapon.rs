use strum::EnumProperty;
use strum_macros::{EnumIter, EnumProperty, EnumString, IntoStaticStr, ToString};

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    PartialOrd,
    Eq,
    Hash,
    ToString,
    IntoStaticStr,
    EnumString,
    EnumIter,
    EnumProperty,
)]
#[strum(serialize_all = "snake_case")]
pub enum Weapon {
    #[strum(props(English = "Great Sword", Japanese = "大剣"))]
    GreatSword,
    #[strum(props(English = "Long Sword", Japanese = "太刀"))]
    LongSword,
    #[strum(props(English = "Sword and Shield", Japanese = "片手剣"))]
    SwordAndShield,
    #[strum(props(English = "Dual Blades", Japanese = "双剣"))]
    DualBlades,
    #[strum(props(English = "Lance", Japanese = "ランス"))]
    Lance,
    #[strum(props(English = "Gunlance", Japanese = "ガンランス"))]
    Gunlance,
    #[strum(props(English = "Hammer", Japanese = "ハンマー"))]
    Hammer,
    #[strum(props(English = "Hunting Horn", Japanese = "狩猟笛"))]
    HuntingHorn,
    #[strum(props(English = "Switch Axe", Japanese = "スラッシュアックス"))]
    SwitchAxe,
    #[strum(props(English = "Charge Blade", Japanese = "チャージアックス"))]
    ChargeBlade,
    #[strum(props(English = "Insect Glaive", Japanese = "操虫棍"))]
    InsectGlaive,
    #[strum(props(English = "Light Bowgun", Japanese = "ライトボウガン"))]
    LightBowgun,
    #[strum(props(English = "Heavy Bowgun", Japanese = "ヘヴィボウガン"))]
    HeavyBowgun,
    #[strum(props(English = "Bow", Japanese = "弓"))]
    Bow,
    #[strum(props(English = "Restricted: Tackle Only", Japanese = "縛り: タックルのみ"))]
    TackleOnly,
    #[strum(props(
        English = "Restricted: Counter Only",
        Japanese = "縛り: カウンターのみ"
    ))]
    CounterOnly,
    #[strum(props(English = "Restricted: Melee-Attack Only", Japanese = "オサイズチ"))]
    MeleeAttackOnly,
    #[strum(props(English = "Restricted: Skills Only", Japanese = "縛り: 鉄蟲糸技"))]
    SkillsOnly,
    #[strum(props(English = "Restricted: Palamute Only", Japanese = "縛り: ガルク搭乗"))]
    PalamuteOnly,
    #[strum(props(English = "Restricted: Bom Only", Japanese = "縛り: 爆弾のみ"))]
    BomOnly,
    #[strum(props(English = "Restricted: Insect Only", Japanese = "縛り: 虫のみ"))]
    InsectOnly,
}

impl Weapon {
    #[allow(dead_code)]
    pub fn en(&self) -> &'static str {
        self.get_str("English").unwrap()
    }
    #[allow(dead_code)]
    pub fn ja(&self) -> &'static str {
        self.get_str("Japanese").unwrap()
    }
}
