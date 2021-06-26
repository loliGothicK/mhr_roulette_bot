use crate::{
    data::{Config, Objective, Quest, Weapon},
    stream::Msg,
};
use anyhow::Context;
use indexmap::map::IndexMap;
use itertools::Itertools;
use once_cell::sync::Lazy;
use sqlite::Connection;
use std::{
    io::Write,
    sync::{Arc, Mutex},
};
use strum::IntoEnumIterator;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub static DB_PATH: Lazy<std::path::PathBuf> = Lazy::new(|| {
    std::path::PathBuf::from(std::env::var("MHR_DB_PATH").expect("env var: MHR_DB_PATH"))
});

pub static CONN: Lazy<Arc<Mutex<Connection>>> = Lazy::new(|| {
    Arc::new(Mutex::new(
        sqlite::open(DB_PATH.as_path()).expect("connection established"),
    ))
});

pub static CONFIG_PATH: Lazy<std::path::PathBuf> = Lazy::new(|| {
    std::path::PathBuf::from(std::env::var("MHR_CONFIG_PATH").expect("env var: MHR_CONFIG_PATH"))
});

pub static CONFIG: Lazy<Arc<Mutex<Config>>> = Lazy::new(|| {
    println!("Reading config...");
    let config: Config = toml::from_str(&std::fs::read_to_string(&*CONFIG_PATH).unwrap()).unwrap();
    Arc::new(Mutex::new(config))
});

pub fn sync_all() -> anyhow::Result<()> {
    let mut conf = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(CONFIG_PATH.as_path())
        .with_context(|| anyhow::anyhow!("Cannot open {:?}", &CONFIG_PATH))?;
    conf.write_all(
        toml::to_string_pretty(&*CONFIG.lock().unwrap())
            .with_context(|| anyhow::anyhow!("Cannot deserialize config."))?
            .as_bytes(),
    )
    .with_context(|| anyhow::anyhow!("Cannot write config to {:?}.", &CONFIG_PATH))?;
    conf.sync_all()
        .with_context(|| anyhow::anyhow!("Data sync failed."))?;
    Ok(())
}

pub struct Tsx<T> {
    sender: Arc<Sender<T>>,
    receiver: Arc<Mutex<Receiver<T>>>,
}

impl<T> Tsx<T> {
    pub fn sender(&self) -> Arc<Sender<T>> {
        Arc::clone(&self.sender)
    }
    pub fn receiver(&self) -> Arc<Mutex<Receiver<T>>> {
        Arc::clone(&self.receiver)
    }
}

pub static SRX: Lazy<Tsx<Msg>> = Lazy::new(|| {
    let (sender, receiver) = channel(32);
    Tsx {
        sender: Arc::new(sender),
        receiver: Arc::new(Mutex::new(receiver)),
    }
});

pub static OBJECTIVES: Lazy<IndexMap<Weapon, Vec<Objective>>> = Lazy::new(|| {
    let objectives = Weapon::iter()
        .zip(&Objective::iter().chunks(3))
        .map(|(k, v)| (k, v.collect::<Vec<_>>()))
        .collect::<IndexMap<_, _>>();
    assert_eq!(14usize, objectives.len());
    objectives
});

pub static QUESTS: Lazy<Vec<Vec<Quest>>> = Lazy::new(|| {
    vec![
        vec![
            // ★0 （探索クエスト）
            ["大社跡の探索ツアー", "大社跡の探索（上位）"],
            ["大社跡の探索ツアー", "大社跡の探索（下位）"],
            ["寒冷群島の探索ツアー", "寒冷群島の探索（上位）"],
            ["寒冷群島の探索ツアー", "寒冷群島の探索（下位）"],
            ["砂原の探索ツアー", "砂原の探索（上位）"],
            ["砂原の探索ツアー", "砂原の探索（下位）"],
            ["水没林の探索ツアー", "水没林の探索（上位）"],
            ["水没林の探索ツアー", "水没林の探索（下位）"],
            ["溶岩洞の探索ツアー", "溶岩洞の探索（上位）"],
            ["溶岩洞の探索ツアー", "溶岩洞の探索（下位）"],
        ],
        vec![
            // ★1 （下位クエスト）
            // Sorry, not implemented yet
        ],
        vec![
            // ★2 （下位クエスト）
            // Sorry, not implemented yet
        ],
        vec![
            // ★3 （下位クエスト）
            // Sorry, not implemented yet
        ],
        vec![
            // ★4 （上位クエスト）
            ["取り巻くつむじ風", "オサイズチ ×1"],
            ["グルメ・モンスターズ", "アオアシラ ×1, クルルヤック ×1"],
            ["寒地にて舟を漕ぐ", "ドスバギィ ×2"],
            ["傘鳥円舞", "アケノシルム ×1"],
            ["大場所・寒冷群島", "ヨツミワドウ ×1"],
            ["可愛いものにも牙はある", "ウルクスス ×1"],
            ["ある夜フルフルを狩る", "フルフル ×1"],
            ["毒の錦を纏う", "ドスフロギィ ×1"],
            ["たまごだんご争奪戦！の巻", "クルルヤック ×2"],
            ["会得せよ！片手剣の型", "アケノシルム ×1"],
            ["理解せよ！狩猟笛の型", "オサイズチ ×1, ヨツミワドウ ×1"],
            ["変幻せよ！剣斧の型", "ウルクスス ×1, フルフル ×1"],
            ["学べ！軽弩の型", "ドスバギィ ×1,ドスフロギィ ×1"],
            ["青くて丸い愛しいあの子", "アオアシラ ×1"],
        ],
        vec![
            // ★5 （上位クエスト）
            ["不穏の沼影", "ジュラトドス ×1"],
            ["女王に魅せられて", "リオレイア ×1"],
            ["岩の上にも三年", "バサルモス ×1"],
            ["それは血となり毒となる", "プケプケ ×1"],
            ["一柿入魂", "ビシュテンゴ ×1"],
            ["砂原の魔球にご注意を", "ラングロトラ ×2"],
            ["泥の中でも立ち上がれ", "ボルボロス ×1"],
            ["水と共に生きるもの", "ロアルドロス ×1"],
            ["寒地を呑み込む影", "フルフル ×1, ヨツミワドウ ×1"],
            ["狙い穿て！重弩の型", "バサルモス ×1, ラングロトラ ×1"],
            ["一体となれ！盾斧の型", "ロアルドロス ×1, ジュラトドス ×1"],
            ["心得よ！ランスの型", "リオレイア ×1"],
            ["体で覚えよ！ハンマーの型", "プケプケ ×1, ボルボロス ×1"],
            ["見極めよ！大剣の型", "ビシュテンゴ ×2"],
        ],
        vec![
            // ★6 （上位クエスト）
            ["妖艶なる舞", "タマミツネ ×1"],
            ["天上に紅蓮咲く", "リオレウス ×1"],
            ["赤き双眸、夜陰を断つ", "ナルガクルガ ×1"],
            ["猛追、蛮顎竜", "アンジャナフ ×1"],
            ["頭上を飛び跳ねる驚異", "トビカガチ ×1"],
            ["琥珀色の牙を研ぐ", "ベリオロス ×1"],
            ["冥途へ誘う歌声", "イソネミクニ ×1"],
            [
                "山河に一閃、響く雷鳴",
                "アンジャナフ ×1, タマミツネ ×1, ジンオウガ ×1",
            ],
            ["鍛えよ！弓の型", "トビカガチ ×2"],
            ["修練せよ！操虫棍の型", "イソネミクニ ×1, アケノシルム ×1"],
            ["磨け！銃槍の型", "ベリオロス ×1"],
            ["乱れ裂け！双剣の型", "アンジャナフ ×1, リオレイア ×1"],
            ["研ぎ澄ませ！太刀の型", "ジンオウガ ×1, タマミツネ ×1"],
        ],
        vec![
            // ★7 （上位クエスト）
            ["雷神", "ナルハタタヒメ ×1"],
            ["火吹き御前", "ヤツカダキ ×1"],
            ["悪鬼羅刹", "ラージャン ×1"],
            ["轟轟たる咆哮", "ティガレックス ×1"],
            ["地底を駆ける角竜", "ディアブロス ×1"],
            ["泥海へ手招く", "オロミドロ ×1"],
            ["鬼火を纏いしモノ", "マガイマガド ×1"],
            ["雪鬼獣がやってくる", "ゴシャハギ ×1"],
            ["方々から迫る脅威", "ヤツカダキ ×1, フルフル ×1"],
            ["乱暴者たちにご注意を", "ラージャン ×1, 像ジンオウガ ×1"],
            ["激突・激烈・激励の乱", "ゴシャハギ ×1, ヨツミワドウ ×1"],
            ["大社跡の大騒動", "オロミドロ ×1, タマミツネ ×1"],
            ["うさ団子貫く四つの角！の巻", "ディアブロス ×2"],
            [
                "火加減注意！紫炎と火球の巻",
                "リオレウス ×1, マガイマガド ×1",
            ],
            [
                "疾風怒濤の大舞台",
                "トビカガチ ×1, ナルガクルガ ×1, ティガレックス ×1",
            ],
        ],
        vec![
            // ★7 HR解放後 （上位クエスト）
            [
                "百竜ノ淵源",
                "イブシマキヒコ（前座）, 百竜ノ淵源ナルハタタヒメ ×1",
            ],
            [
                "奇しき赫耀（彼方より来たる凶星）",
                "奇しき赫耀のバルファルク ×1",
            ],
            [
                "ウツシ教官の挑戦状・其の三",
                "マガイマガド ×1, ナルガクルガ ×1",
            ],
            ["ウツシ教官の挑戦状・其の二", "ゴシャハギ ×1, ラージャン ×1"],
            ["ウツシ教官の挑戦状・其の一", "オロミドロ ×1, ジンオウガ ×1"],
            ["千紫万紅、ヌシ・タマミツネ", "ヌシタマミツネ ×1"],
            ["優美高妙、ヌシ・リオレイア", "ヌシリオレイア ×1"],
            ["牛飲馬食、ヌシ・アオアシラ", "ヌシアオアシラ ×1"],
            ["爆鱗竜、再び飛来す（降り注ぐ爆鱗の矢）", "バゼルギウス ×1"],
            ["猛き炎よ、怒髪を鎮めよ", "ラージャン ×1"],
            [
                "猛き炎と、闊歩する強者ども",
                "リオレイア ×1, ティガレックス ×1, ヤツカダキ ×1",
            ],
            ["炎国の王", "テオテスカトル ×1"],
            ["嵐に舞う黒い影", "クシャルダオラ ×1"],
            ["古の幻影", "オオナズチ ×1"],
            [
                "ウツシ教官の挑戦状・其の四",
                "ティガレックス ×1, アンジャナフ ×1",
            ],
            ["ウツシ教官の挑戦状・其の五", "ヤツカダキ ×1, リオレウス ×1"],
            ["為虎添翼、ヌシ・リオレウス", "ヌシリオレウス ×1"],
            ["痛烈無比、ヌシ・ディアブロス", "ヌシディアブロス ×1"],
            ["電光雷轟、ヌシ・ジンオウガ", "ヌシジンオウガ ×1"],
            ["高難度：災禍を纏うもの", "マガイマガド ×1, バゼルギウス ×1"],
            [
                "高難度：竜獣戯画",
                "ゴシャハギ ×1, ティガレックス ×1, タマミツネ ×1",
            ],
            [
                "高難度：嵐ト炎ヲ司ルモノ",
                "クシャルダオラ ×1, テオ・テスカトル ×1",
            ],
            ["高難度：鬼はいずこ", "オオナズチ ×1, ラージャン ×1"],
            [
                "高難度：凶星、業火の地に降る",
                "バルファルク ×1, ヤツカダキ ×1",
            ],
            ["高難度：猛者たちの酒宴", "バルファルク ×1, ヤツカダキ ×1"],
            [
                "高難度：猛者たちの酒宴",
                "ディアブロス ×1, リオレウス ×1, マガイマガド ×1",
            ],
            [
                "高難度：ヌシの名を戴くもの",
                "ヌシ・タマミツネ ×1, ヌシ・リオレウス ×1, ヌシ・ジンオウガ ×1",
            ],
        ],
    ]
});
