// Monster Hunter Rise version 3.0
use strum::EnumProperty;
use strum_macros::{EnumIter, EnumProperty, EnumString, IntoStaticStr};

#[derive(Debug, PartialEq, Eq, Hash, IntoStaticStr, EnumString, EnumIter, EnumProperty)]
#[strum(serialize_all = "snake_case")]
pub enum Monster {
    #[strum(props(English = "Great Izuchi", Japanese = "オサイズチ"))]
    GreatIzuchi,
    #[strum(props(English = "Great Baggi", Japanese = "ドスバギィ"))]
    GreatBaggi,
    #[strum(props(English = "Kulu-Ya-Ku", Japanese = "クルルヤック"))]
    KuluYaKu,
    #[strum(props(English = "Great Wroggi", Japanese = "ドスフロギィ"))]
    GreatWroggi,
    #[strum(props(English = "Arzuros", Japanese = "アオアシラ"))]
    Arzuros,
    #[strum(props(English = "Lagombi", Japanese = "ラングロトラ"))]
    Lagombi,
    #[strum(props(English = "Aknosom", Japanese = "アケノシルム"))]
    Aknosom,
    #[strum(props(English = "Royal Ludroth", Japanese = "ロアルドロス"))]
    RoyalLudroth,
    #[strum(props(English = "Barroth", Japanese = "ボルボロス"))]
    Barroth,
    #[strum(props(English = "Khezu", Japanese = "フルフル"))]
    Khezu,
    #[strum(props(English = "Teranadon", Japanese = "ヨツミワドウ"))]
    Teranadon,
    #[strum(props(English = "Bishaten", Japanese = "ビシュテンゴ"))]
    Bishaten,
    #[strum(props(English = "Pukei-Pukei", Japanese = "プケプケ"))]
    PukeiPukei,
    #[strum(props(English = "Jyuratodus", Japanese = "ジュラトドス"))]
    Jyuratodus,
    #[strum(props(English = "Basarios", Japanese = "バサルモス"))]
    Basarios,
    #[strum(props(English = "Somnacanth", Japanese = "イソネミクニ"))]
    Somnacanth,
    #[strum(props(English = "Rathian", Japanese = "リオレイア"))]
    Rathian,
    #[strum(props(English = "Barioth", Japanese = "ベリオロス"))]
    Barioth,
    #[strum(props(English = "Tobi-Kadachi", Japanese = "トビカガチ"))]
    TobiKadachi,
    #[strum(props(English = "Magnamolo", Japanese = "マガイマガド"))]
    Magnamolo,
    #[strum(props(English = "Anjanath", Japanese = "アンジャナフ"))]
    Anjanath,
    #[strum(props(English = "Nargacuga", Japanese = "ナルガクルガ"))]
    Nargacuga,
    #[strum(props(English = "Mizutsune", Japanese = "タマミツネ"))]
    Mizutsune,
    #[strum(props(English = "Goss Harag", Japanese = "ゴシャハギ"))]
    GossHarag,
    #[strum(props(English = "Ratharos", Japanese = "リオレウス"))]
    Ratharos,
    #[strum(props(English = "Almudron", Japanese = "オロミドロ"))]
    Almudron,
    #[strum(props(English = "Zinogre", Japanese = "ジンオウガ"))]
    Zinogre,
    #[strum(props(English = "Tigrex", Japanese = "ティガレックス"))]
    Tigrex,
    #[strum(props(English = "Diablos", Japanese = "ディアブロス"))]
    Diablos,
    #[strum(props(English = "Rakna-Kadaki", Japanese = "ヤツカダキ"))]
    RaknaKadaki,
    #[strum(props(English = "Kushala Daora", Japanese = "クシャルダオラ"))]
    KushalaDaora, // since version 2.0
    #[strum(props(English = "Chameleos", Japanese = "オオナズチ"))]
    Chameleos, // since version 2.0
    #[strum(props(English = "Teostra", Japanese = "テオ・テスカトル"))]
    Teostra, // since version 2.0
    #[strum(props(English = "Rajang", Japanese = "ラージャン"))]
    Rajang,
    #[strum(props(English = "Bazelgeuse", Japanese = "バゼルギウス"))]
    Bazelgeuse, // since version 2.0
    // #[strum(serialize="イブシマキヒコ", props(English="Wind Serpent Ibushi", Japanese="イブシマキヒコ"))]
    // WindSerpentIbushi,
    #[strum(props(English = "Thunder Serpent Narwa", Japanese = "ナルハタタヒメ"))]
    ThunderSerpentNarwa, // since version 3.0
    #[strum(props(English = "Narwa The Allmother", Japanese = "百竜ノ淵源ナルハタタヒメ"))]
    NarwaTheAllmother, // since version 3.0
    #[strum(props(
        English = "Crimson Glow Valstrax",
        Japanese = "奇しき赫耀のバルファルク"
    ))]
    CrimsonGlowValstrax, // since version 3.0
    #[strum(props(English = "Apex Arzuros", Japanese = "ヌシ・アオアシラ"))]
    ApexArzuros, // since version 3.0
    #[strum(props(English = "Apex Rathian", Japanese = "ヌシ・リオレイア"))]
    ApexRathian, // since version 3.0
    #[strum(props(English = "Apex Mizutsune", Japanese = "ヌシ・タマミツネ"))]
    ApexMizutsune, // since version 3.0
    #[strum(props(English = "Apex Rathalos", Japanese = "ヌシ・リオレウス"))]
    ApexRathalos, // since version 3.0
    #[strum(props(English = "Apex Diablos", Japanese = "ヌシ・ディアブロス"))]
    ApexDiablos, // since version 3.0
    #[strum(props(English = "Apex Zinogre", Japanese = "ヌシ・ジンオウガ"))]
    ApexZinogre, // since version 3.0
}

impl Monster {
    #[allow(dead_code)]
    fn en(&self) -> &'static str {
        self.get_str("English").unwrap()
    }
    pub fn ja(&self) -> &'static str {
        self.get_str("Japanese").unwrap()
    }
}
