// egui 0.27 の Window を実際に描画し、出力 Shape から
//   - タイトルバー背景 Rect の y 範囲
//   - タイトル文字 galley の y 範囲
// を取り出して縦ズレを実測する。数式の前提に依存しない直接計測。
//   cargo run --example font_probe

use eframe::egui;

fn load_cjk_font(fonts: &mut egui::FontDefinitions) {
    for path in &[
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
        "/System/Library/Fonts/ヒラギノ角ゴシック W4.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
    ] {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert("jp".to_owned(), egui::FontData::from_owned(data));
            fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "jp".to_owned());
            return;
        }
    }
    println!("WARNING: no CJK font loaded");
}

fn main() {
    let ctx = egui::Context::default();
    let mut fonts = egui::FontDefinitions::default();
    load_cjk_font(&mut fonts);
    ctx.set_fonts(fonts);
    {
        use egui::{FontFamily, FontId, TextStyle};
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Heading, FontId::new(15.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(12.0, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(12.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace)),
            (TextStyle::Small, FontId::new(9.0, FontFamily::Proportional)),
        ].into();
        style.spacing.window_margin = egui::Margin::symmetric(8.0, 2.0);
        ctx.set_style(style);
    }

    // Window は 1 フレーム目は sizing pass で位置が確定しないので数フレーム回す。
    use egui::epaint::Shape;
    let mut last_bar: Option<egui::Rect> = None;
    let mut last_text: Option<(egui::Rect, f32)> = None;
    for _ in 0..4 {
        let mut input = egui::RawInput::default();
        input.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 600.0)));
        let output = ctx.run(input, |ctx| {
            egui::Window::new(egui::RichText::new("環境設定").size(11.0).strong())
                .collapsible(false)
                .resizable(false)
                .default_pos([100.0, 100.0])
                .show(ctx, |ui| {
                    ui.label("待ち受けポート:");
                    ui.label("ログ保存先:");
                });
        });
        let mut bar: Option<egui::Rect> = None;
        let mut text: Option<(egui::Rect, f32)> = None;
        for cs in &output.shapes {
            match &cs.shape {
                // タイトルバー背景: 横長・低い塗り Rect を最初に拾う
                Shape::Rect(r) if bar.is_none() && r.rect.width() > 60.0 && r.rect.height() < 40.0 => {
                    bar = Some(r.rect);
                }
                // タイトル文字: galley を持つ最初の Text
                Shape::Text(t) if text.is_none() => {
                    // 実グリフの bounding（行高ではなく実インク領域に近い値）
                    let vbr = t.visual_bounding_rect();
                    text = Some((vbr, t.galley.size().y));
                }
                _ => {}
            }
        }
        if bar.is_some() { last_bar = bar; }
        if text.is_some() { last_text = text; }
    }

    match (last_bar, last_text) {
        (Some(bar), Some((text, galley_h))) => {
            println!("タイトルバー背景: y=[{:.2}, {:.2}] 高さ={:.2} 中央={:.2}", bar.min.y, bar.max.y, bar.height(), bar.center().y);
            println!("タイトル文字(描画): y=[{:.2}, {:.2}] 高さ={:.2} 中央={:.2}", text.min.y, text.max.y, text.height(), text.center().y);
            println!("galley 行高 = {:.2}", galley_h);
            println!("バー内 上余白={:.2} / 下余白={:.2}", text.min.y - bar.min.y, bar.max.y - text.max.y);
            let delta = text.center().y - bar.center().y;
            println!(">>> 中央ズレ = {delta:+.2} px  ({})",
                if delta < -0.5 { "文字が上に寄る" } else if delta > 0.5 { "文字が下に寄る" } else { "ほぼ中央" });
        }
        (b, t) => println!("取得失敗 bar={} text={}", b.is_some(), t.is_some()),
    }
}
