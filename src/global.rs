/*
 * ISC License
 *
 * Copyright (c) 2021 Mitama Lab
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 *
 */

use crate::{
    bot::Msg,
    data::{Config, Objective, Quest, Weapon},
};
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

/// MHR_DB_PATH
pub static DB_PATH: Lazy<std::path::PathBuf> = Lazy::new(|| {
    std::path::PathBuf::from(std::env::var("MHR_DB_PATH").expect("env var: MHR_DB_PATH"))
});

/// SQLite Connection
pub static CONN: Lazy<Arc<Mutex<Connection>>> = Lazy::new(|| {
    Arc::new(Mutex::new(
        sqlite::open(DB_PATH.as_path()).expect("connection established"),
    ))
});

/// MHR_CONFIG_PATH
pub static CONFIG_PATH: Lazy<std::path::PathBuf> = Lazy::new(|| {
    std::path::PathBuf::from(std::env::var("MHR_CONFIG_PATH").expect("env var: MHR_CONFIG_PATH"))
});

/// In-memory Configures
pub static CONFIG: Lazy<Arc<Mutex<Config>>> = Lazy::new(|| {
    let config: Config = toml::from_str(&std::fs::read_to_string(&*CONFIG_PATH).unwrap()).unwrap();
    Arc::new(Mutex::new(config))
});

/// Write all configures to toml file
pub fn sync_all() -> std::result::Result<(), std::io::Error> {
    let mut conf = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(CONFIG_PATH.as_path())?;
    conf.write_all(
        toml::to_string_pretty(&*CONFIG.lock().unwrap())
            .unwrap()
            .as_bytes(),
    )?;
    conf.sync_all()?;
    Ok(())
}

/// Struct that holds sender and receiver
pub struct Tsx<T> {
    sender: Arc<Sender<T>>,
    receiver: Arc<Mutex<Receiver<T>>>,
}

/// Getter for sender and receiver
impl<T> Tsx<T> {
    pub fn sender(&self) -> Arc<Sender<T>> {
        Arc::clone(&self.sender)
    }

    pub fn receiver(&self) -> Arc<Mutex<Receiver<T>>> {
        Arc::clone(&self.receiver)
    }
}

/// Sender/Receiver
pub static CENTRAL: Lazy<Tsx<Msg>> = Lazy::new(|| {
    let (sender, receiver) = channel(8);
    Tsx {
        sender: Arc::new(sender),
        receiver: Arc::new(Mutex::new(receiver)),
    }
});

/// Optional Objectives
pub static OBJECTIVES: Lazy<IndexMap<Weapon, Vec<Objective>>> = Lazy::new(|| {
    let objectives = Weapon::iter()
        .zip(&Objective::iter().chunks(3))
        .map(|(k, v)| (k, v.collect::<Vec<_>>()))
        .collect::<IndexMap<_, _>>();
    assert_eq!(14usize, objectives.len());
    objectives
});

/// Quest List
pub static QUESTS: Lazy<Vec<Vec<Quest>>> = Lazy::new(|| {
    vec![
        vec![
            // ★0 （探索クエスト）
            Quest("大社跡の探索ツアー", "大社跡の探索（上位）"),
            Quest("大社跡の探索ツアー", "大社跡の探索（下位）"),
            Quest("寒冷群島の探索ツアー", "寒冷群島の探索（上位）"),
            Quest("寒冷群島の探索ツアー", "寒冷群島の探索（下位）"),
            Quest("砂原の探索ツアー", "砂原の探索（上位）"),
            Quest("砂原の探索ツアー", "砂原の探索（下位）"),
            Quest("水没林の探索ツアー", "水没林の探索（上位）"),
            Quest("水没林の探索ツアー", "水没林の探索（下位）"),
            Quest("溶岩洞の探索ツアー", "溶岩洞の探索（上位）"),
            Quest("溶岩洞の探索ツアー", "溶岩洞の探索（下位）"),
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
            Quest("取り巻くつむじ風", "オサイズチ ×1"),
            Quest("グルメ・モンスターズ", "アオアシラ ×1, クルルヤック ×1"),
            Quest("寒地にて舟を漕ぐ", "ドスバギィ ×2"),
            Quest("傘鳥円舞", "アケノシルム ×1"),
            Quest("大場所・寒冷群島", "ヨツミワドウ ×1"),
            Quest("可愛いものにも牙はある", "ウルクスス ×1"),
            Quest("ある夜フルフルを狩る", "フルフル ×1"),
            Quest("毒の錦を纏う", "ドスフロギィ ×1"),
            Quest("たまごだんご争奪戦！の巻", "クルルヤック ×2"),
            Quest("会得せよ！片手剣の型", "アケノシルム ×1"),
            Quest("理解せよ！狩猟笛の型", "オサイズチ ×1, ヨツミワドウ ×1"),
            Quest("変幻せよ！剣斧の型", "ウルクスス ×1, フルフル ×1"),
            Quest("学べ！軽弩の型", "ドスバギィ ×1,ドスフロギィ ×1"),
            Quest("青くて丸い愛しいあの子", "アオアシラ ×1"),
        ],
        vec![
            // ★5 （上位クエスト）
            Quest("不穏の沼影", "ジュラトドス ×1"),
            Quest("女王に魅せられて", "リオレイア ×1"),
            Quest("岩の上にも三年", "バサルモス ×1"),
            Quest("それは血となり毒となる", "プケプケ ×1"),
            Quest("一柿入魂", "ビシュテンゴ ×1"),
            Quest("砂原の魔球にご注意を", "ラングロトラ ×2"),
            Quest("泥の中でも立ち上がれ", "ボルボロス ×1"),
            Quest("水と共に生きるもの", "ロアルドロス ×1"),
            Quest("寒地を呑み込む影", "フルフル ×1, ヨツミワドウ ×1"),
            Quest("狙い穿て！重弩の型", "バサルモス ×1, ラングロトラ ×1"),
            Quest("一体となれ！盾斧の型", "ロアルドロス ×1, ジュラトドス ×1"),
            Quest("心得よ！ランスの型", "リオレイア ×1"),
            Quest("体で覚えよ！ハンマーの型", "プケプケ ×1, ボルボロス ×1"),
            Quest("見極めよ！大剣の型", "ビシュテンゴ ×2"),
        ],
        vec![
            // ★6 （上位クエスト）
            Quest("妖艶なる舞", "タマミツネ ×1"),
            Quest("天上に紅蓮咲く", "リオレウス ×1"),
            Quest("赤き双眸、夜陰を断つ", "ナルガクルガ ×1"),
            Quest("猛追、蛮顎竜", "アンジャナフ ×1"),
            Quest("頭上を飛び跳ねる驚異", "トビカガチ ×1"),
            Quest("琥珀色の牙を研ぐ", "ベリオロス ×1"),
            Quest("冥途へ誘う歌声", "イソネミクニ ×1"),
            Quest(
                "山河に一閃、響く雷鳴",
                "アンジャナフ ×1, タマミツネ ×1, ジンオウガ ×1",
            ),
            Quest("鍛えよ！弓の型", "トビカガチ ×2"),
            Quest("修練せよ！操虫棍の型", "イソネミクニ ×1, アケノシルム ×1"),
            Quest("磨け！銃槍の型", "ベリオロス ×1"),
            Quest("乱れ裂け！双剣の型", "アンジャナフ ×1, リオレイア ×1"),
            Quest("研ぎ澄ませ！太刀の型", "ジンオウガ ×1, タマミツネ ×1"),
        ],
        vec![
            // ★7 （上位クエスト）
            Quest("雷神", "ナルハタタヒメ ×1"),
            Quest("火吹き御前", "ヤツカダキ ×1"),
            Quest("悪鬼羅刹", "ラージャン ×1"),
            Quest("轟轟たる咆哮", "ティガレックス ×1"),
            Quest("地底を駆ける角竜", "ディアブロス ×1"),
            Quest("泥海へ手招く", "オロミドロ ×1"),
            Quest("鬼火を纏いしモノ", "マガイマガド ×1"),
            Quest("雪鬼獣がやってくる", "ゴシャハギ ×1"),
            Quest("方々から迫る脅威", "ヤツカダキ ×1, フルフル ×1"),
            Quest("乱暴者たちにご注意を", "ラージャン ×1, 像ジンオウガ ×1"),
            Quest("激突・激烈・激励の乱", "ゴシャハギ ×1, ヨツミワドウ ×1"),
            Quest("大社跡の大騒動", "オロミドロ ×1, タマミツネ ×1"),
            Quest("うさ団子貫く四つの角！の巻", "ディアブロス ×2"),
            Quest(
                "火加減注意！紫炎と火球の巻",
                "リオレウス ×1, マガイマガド ×1",
            ),
            Quest(
                "疾風怒濤の大舞台",
                "トビカガチ ×1, ナルガクルガ ×1, ティガレックス ×1",
            ),
        ],
        vec![
            // ★7 HR解放後 （上位クエスト）
            Quest(
                "百竜ノ淵源",
                "イブシマキヒコ（前座）, 百竜ノ淵源ナルハタタヒメ ×1",
            ),
            Quest(
                "奇しき赫耀（彼方より来たる凶星）",
                "奇しき赫耀のバルファルク ×1",
            ),
            Quest(
                "ウツシ教官の挑戦状・其の三",
                "マガイマガド ×1, ナルガクルガ ×1",
            ),
            Quest("ウツシ教官の挑戦状・其の二", "ゴシャハギ ×1, ラージャン ×1"),
            Quest("ウツシ教官の挑戦状・其の一", "オロミドロ ×1, ジンオウガ ×1"),
            Quest("千紫万紅、ヌシ・タマミツネ", "ヌシタマミツネ ×1"),
            Quest("優美高妙、ヌシ・リオレイア", "ヌシリオレイア ×1"),
            Quest("牛飲馬食、ヌシ・アオアシラ", "ヌシアオアシラ ×1"),
            Quest("爆鱗竜、再び飛来す（降り注ぐ爆鱗の矢）", "バゼルギウス ×1"),
            Quest("猛き炎よ、怒髪を鎮めよ", "ラージャン ×1"),
            Quest(
                "猛き炎と、闊歩する強者ども",
                "リオレイア ×1, ティガレックス ×1, ヤツカダキ ×1",
            ),
            Quest("炎国の王", "テオテスカトル ×1"),
            Quest("嵐に舞う黒い影", "クシャルダオラ ×1"),
            Quest("古の幻影", "オオナズチ ×1"),
            Quest(
                "ウツシ教官の挑戦状・其の四",
                "ティガレックス ×1, アンジャナフ ×1",
            ),
            Quest("ウツシ教官の挑戦状・其の五", "ヤツカダキ ×1, リオレウス ×1"),
            Quest("為虎添翼、ヌシ・リオレウス", "ヌシリオレウス ×1"),
            Quest("痛烈無比、ヌシ・ディアブロス", "ヌシディアブロス ×1"),
            Quest("電光雷轟、ヌシ・ジンオウガ", "ヌシジンオウガ ×1"),
            Quest("高難度：災禍を纏うもの", "マガイマガド ×1, バゼルギウス ×1"),
            Quest(
                "高難度：竜獣戯画",
                "ゴシャハギ ×1, ティガレックス ×1, タマミツネ ×1",
            ),
            Quest(
                "高難度：嵐ト炎ヲ司ルモノ",
                "クシャルダオラ ×1, テオ・テスカトル ×1",
            ),
            Quest("高難度：鬼はいずこ", "オオナズチ ×1, ラージャン ×1"),
            Quest(
                "高難度：凶星、業火の地に降る",
                "バルファルク ×1, ヤツカダキ ×1",
            ),
            Quest("高難度：猛者たちの酒宴", "バルファルク ×1, ヤツカダキ ×1"),
            Quest(
                "高難度：猛者たちの酒宴",
                "ディアブロス ×1, リオレウス ×1, マガイマガド ×1",
            ),
            Quest(
                "高難度：ヌシの名を戴くもの",
                "ヌシ・タマミツネ ×1, ヌシ・リオレウス ×1, ヌシ・ジンオウガ ×1",
            ),
        ],
    ]
});
