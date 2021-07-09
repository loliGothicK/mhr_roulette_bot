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

#![allow(clippy::nonstandard_macro_braces)]
use rand::distributions::{Distribution, Uniform};

use strum_macros::EnumIter;
use thiserror::Error;

struct MinMax(i32, i32);

impl std::fmt::Display for MinMax {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut rng = rand::thread_rng();
        let uniform = Uniform::new_inclusive(self.0, self.1);
        write!(f, "{}", uniform.sample(&mut rng))
    }
}

#[derive(Debug, Error, PartialEq, Eq, Hash, EnumIter)]
pub enum Order {
    #[error("アイテムの持ち込み数1個（弾・ビンを除く）")]
    Order1,
    #[error("{}種類の状態異常にする", MinMax(1, 3))]
    Order2,
    #[error("{}回操竜する", MinMax(1, 4))]
    Order3,
}

#[derive(Debug, Error, PartialEq, Eq, Hash, EnumIter)]
pub enum Objective {
    // for Great Sword
    #[error("1回スタンさせる")]
    GreatSword1,
    #[error("真溜め斬りを{}回当てる", MinMax(1, 3))]
    GreatSword2,
    #[error("睡眠真溜め斬りを1回成功させる")]
    GreatSword3,
    // for Long Sword
    #[error("居合抜刀気刃斬りを{}回成功させる", MinMax(1, 3))]
    LongSword1,
    #[error("真溜め斬りを{}回当てる", MinMax(1, 3))]
    LongSword2,
    #[error("兜割りを{}回全ヒットさせる", MinMax(1, 3))]
    LongSword3,
    // for Sword and Shield
    #[error("1回スタンさせる")]
    SwordAndShield1,
    #[error("滅・昇竜拳のカウンターを{}回成功させる", MinMax(2, 5))]
    SwordAndShield2,
    #[error("ジャストラッシュを{}回成功させる", MinMax(5, 10))]
    SwordAndShield3,
    // for Dual Blades
    #[error("朧翔の回避を{}回成功させる", MinMax(2, 5))]
    DualBlades1,
    #[error("鉄蟲斬糸を{}回成功させる", MinMax(5, 10))]
    DualBlades2,
    #[error("空中鬼人化から空中回転乱舞を出してモンスターに当てる")]
    DualBlades3,
    // for Lance
    #[error("スタンを{}回とる", MinMax(2, 5))]
    Lance1,
    #[error("ジャストガードを{}回成功させる", MinMax(2, 5))]
    Lance2,
    #[error("アンカーレイジで黄色をもらう")]
    Lance3,
    // for Gunlance
    #[error("竜撃砲を{}回当てる", MinMax(2, 5))]
    Gunlance1,
    #[error("ガードエッジを{}回成功させる", MinMax(2, 5))]
    Gunlance2,
    #[error("空中フルバーストを1回成功させる")]
    Gunlance3,
    // for Hammer
    #[error("スタンを{}回とる", MinMax(2, 5))]
    Hammer1,
    #[error("水面打ちを{}回成功させる", MinMax(2, 5))]
    Hammer2,
    // #[error("減気ひるみインパクトクレーターを1回成功させる")]
    #[error("睡眠インパクトクレーターを1回成功させる")]
    Hammer3,
    // for Hunting Horn
    #[error("操竜を{}回する", MinMax(2, 5))]
    HuntingHorn1,
    #[error("体力回復の旋律で{}回以上回復する", MinMax(2, 5))]
    HuntingHorn2,
    #[error("震打を{}回当てる", MinMax(2, 5))]
    HuntingHorn3,
    // for Switch Axe
    #[error("金剛連斧で{}回ゴリ押す", MinMax(2, 5))]
    SwitchAxe1,
    #[error("飛翔竜剣を{}回当てる", MinMax(2, 5))]
    SwitchAxe2,
    #[error("零距離属性解放突きを{}回成功させる", MinMax(2, 5))]
    SwitchAxe3,
    // for Charge Blade
    #[error("高出力属性解放斬りを{}回当てる", MinMax(2, 5))]
    ChargeBlade1,
    #[error("カウンターフルチャージを{}回成功させる", MinMax(2, 5))]
    ChargeBlade2,
    #[error("アックスホッパーからの空中高出力属性解放斬りを当てる")]
    ChargeBlade3,
    // for Insect Glaive
    #[error("降竜を{}回以上当てる", MinMax(5, 10))]
    InsectGlaive1,
    #[error("跳躍で{}回攻撃を回避する", MinMax(2, 5))]
    InsectGlaive2,
    #[error("跳躍で回攻撃を回避したあとに降竜を当てる")]
    InsectGlaive3,
    // for Light Bowgun
    #[error("状態異常を{}種類以上いれる", MinMax(1, 2))]
    LightBowgun1,
    #[error("回復弾で味方を{}回以上回復する", MinMax(2, 5))]
    LightBowgun2,
    #[error("起爆榴弾直挿しを{}回成功させる", MinMax(1, 3))]
    LightBowgun3,
    // for Heavy Bowgun
    #[error("狙撃竜弾を{}回使う", MinMax(1, 3))]
    HeavyBowgun1,
    #[error("カウンターショットを{}回成功させる", MinMax(2, 5))]
    HeavyBowgun2,
    #[error("タックルのスーパーアーマーで{}回攻撃を耐える", MinMax(1, 3))]
    HeavyBowgun3,
    // for Bow
    #[error("身躱し矢切りを{}回成功させる", MinMax(1, 3))]
    Bow1,
    #[error("状態異常を1回いれる")]
    Bow2,
    #[error("身躱し矢切り竜の一矢を成功させる")]
    Bow3,
}
