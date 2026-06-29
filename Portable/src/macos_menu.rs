//! macOS のネイティブなグローバルメニューバー(画面最上部)を構築する。
//!
//! eframe(egui)はウィンドウ内に自前でメニューを描画するだけで、macOS の
//! 「アプリ名メニュー / 編集 / ウインドウ / ヘルプ」が出る上部バーには手を出さない。
//! そこを埋めるのがこのファイル。Tauri の内部部品(muda)は使わず、
//! AppKit を `cocoa` + `objc` で直接叩いてネイティブメニューを作る。
//!
//! ## アクションの橋渡し
//!
//! 「環境設定」「ログフォルダを開く」「コピー/カット/ペースト/全選択/取り消す/やり直す」は
//! egui 側の状態やテキスト編集に作用させる必要がある。AppKit のメニュー項目は別スレッド
//! 文脈ではなくメインスレッドのアクションとして発火するので、ここでは「フラグを立てるだけ」に
//! 留め、egui の update ループ側が [`drain_requests`] でフラグを回収して実処理を行う。
//!
//! 標準動作で完結するもの(About パネル / 終了 / 隠す / 最小化 / 拡大縮小)は AppKit 標準の
//! セレクタ(`terminate:` など)へ素直に流す。これらはフラグを経由しない。

#![cfg(target_os = "macos")]
// このモジュールは AppKit への FFI のかたまり。unsafe fn の中でさらに unsafe fn を呼ぶ箇所が
// 多く、個別の unsafe ブロックでは可読性が落ちるため、モジュール単位でまとめて許可する。
#![allow(unsafe_op_in_unsafe_fn)]

use std::sync::{Mutex, Once};

use cocoa::appkit::NSApp;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSString, NSUInteger};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};

// AppKit の修飾キーマスク(NSEventModifierFlags)。bitflags 版の API ブレを避けて生値で持つ。
const NS_COMMAND: NSUInteger = 1 << 20;
const NS_SHIFT: NSUInteger = 1 << 17;
const NS_OPTION: NSUInteger = 1 << 19;

/// ネイティブメニューから egui 側へ渡したい要求。[`drain_requests`] が返す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuRequest {
    /// 環境設定ウィンドウを開く。
    Preferences,
    /// ログ保存フォルダを Finder で開く。
    OpenLogs,
    Copy,
    Cut,
    Paste,
    SelectAll,
    Undo,
    Redo,
}

// 押された順序を保ったまま egui ループへ渡すためのキュー。
// メニュー押下(メインスレッド)で push、egui の update が毎フレーム drain する。
// 個別フラグだと「全選択→コピー」が同一フレームに来たときに順序が崩れるため、
// 順序を保持するキューにしている。
static QUEUE: Mutex<Vec<MenuRequest>> = Mutex::new(Vec::new());

fn push_request(req: MenuRequest) {
    if let Ok(mut q) = QUEUE.lock() {
        q.push(req);
    }
}

/// 溜まった要求を押された順に取り出して返す(キューは空になる)。
/// egui の update ループから毎フレーム呼ぶ想定。
pub fn drain_requests() -> Vec<MenuRequest> {
    match QUEUE.lock() {
        Ok(mut q) => std::mem::take(&mut *q),
        Err(_) => Vec::new(),
    }
}

// --- カスタムメニュー項目のアクション実装(セレクタの中身) ---
// いずれもメインスレッドで呼ばれる。対応する要求をキューへ積むだけ。

extern "C" fn act_preferences(_: &Object, _: Sel, _: id) {
    push_request(MenuRequest::Preferences);
}
extern "C" fn act_open_logs(_: &Object, _: Sel, _: id) {
    push_request(MenuRequest::OpenLogs);
}
extern "C" fn act_copy(_: &Object, _: Sel, _: id) {
    push_request(MenuRequest::Copy);
}
extern "C" fn act_cut(_: &Object, _: Sel, _: id) {
    push_request(MenuRequest::Cut);
}
extern "C" fn act_paste(_: &Object, _: Sel, _: id) {
    push_request(MenuRequest::Paste);
}
extern "C" fn act_select_all(_: &Object, _: Sel, _: id) {
    push_request(MenuRequest::SelectAll);
}
extern "C" fn act_undo(_: &Object, _: Sel, _: id) {
    push_request(MenuRequest::Undo);
}
extern "C" fn act_redo(_: &Object, _: Sel, _: id) {
    push_request(MenuRequest::Redo);
}

/// カスタムアクションの受け皿となる NSObject 派生クラスを一度だけ登録し、
/// その共有インスタンスを返す。メニュー項目の target に差す。
fn shared_target() -> id {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        let superclass = class!(NSObject);
        let mut decl =
            ClassDecl::new("VltMenuTarget", superclass).expect("VltMenuTarget の登録に失敗");
        unsafe {
            decl.add_method(
                sel!(vltPreferences:),
                act_preferences as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(vltOpenLogs:),
                act_open_logs as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(sel!(vltCopy:), act_copy as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(vltCut:), act_cut as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(vltPaste:), act_paste as extern "C" fn(&Object, Sel, id));
            decl.add_method(
                sel!(vltSelectAll:),
                act_select_all as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(sel!(vltUndo:), act_undo as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(vltRedo:), act_redo as extern "C" fn(&Object, Sel, id));
        }
        decl.register();
    });

    // 1 つあれば十分。アプリ存続中ずっと使うので解放せず持たせ続ける(リークだが意図的)。
    let cls = Class::get("VltMenuTarget").expect("VltMenuTarget が見つからない");
    unsafe { msg_send![cls, new] }
}

/// `&str` から `NSString` を作る。メニュー構築時しか使わないので解放はしない(永続メニュー)。
unsafe fn ns_string(s: &str) -> id {
    NSString::alloc(nil).init_str(s)
}

/// メニューの「○○ について / ○○ を終了」等で使う製品名。
///
/// 配布形態(App 版 / Portable 版)で `.app` のバンドル名は変わる(`vlt-syslogd` /
/// `vlt-syslogd-portable`)が、ユーザーから見た製品名はどちらも「vlt-syslogd」。
/// ウィンドウタイトル(main.rs の `with_title`)とも揃えるため、ここは固定値にする。
const PRODUCT_NAME: &str = "vlt-syslogd";

/// メニューへ項目を 1 つ足してその項目を返す。
/// `target` が nil なら responder chain(NSApp / キーウィンドウ)へ流す標準アクション。
unsafe fn add_item(menu: id, title: &str, action: Sel, key: &str, target: id) -> id {
    let item: id = msg_send![class!(NSMenuItem), alloc];
    let item: id =
        msg_send![item, initWithTitle: ns_string(title) action: action keyEquivalent: ns_string(key)];
    if target != nil {
        let _: () = msg_send![item, setTarget: target];
    }
    let _: () = msg_send![menu, addItem: item];
    item
}

/// 区切り線を足す。
unsafe fn add_separator(menu: id) {
    let sep: id = msg_send![class!(NSMenuItem), separatorItem];
    let _: () = msg_send![menu, addItem: sep];
}

/// タイトル付きの空サブメニューを作り、親メニューにぶら下げて、その NSMenu を返す。
unsafe fn add_submenu(main_menu: id, title: &str) -> id {
    let container: id = msg_send![class!(NSMenuItem), alloc];
    let container: id = msg_send![container, init];
    let _: () = msg_send![main_menu, addItem: container];

    let submenu: id = msg_send![class!(NSMenu), alloc];
    let submenu: id = msg_send![submenu, initWithTitle: ns_string(title)];
    let _: () = msg_send![container, setSubmenu: submenu];
    submenu
}

/// ネイティブメニューバーを組み立てて NSApp に設定する。
///
/// eframe(winit)が NSApplication を初期化し終えた後 —— 具体的には
/// `SyslogApp::new`(CreationContext を受け取る時点)で 1 回だけ呼ぶこと。
/// それより前に呼ぶと NSApp がまだ無い。
pub fn install() {
    unsafe {
        // 構築中に AppKit が返す一時オブジェクトを受けるためのプール。
        let _pool = NSAutoreleasePool::new(nil);

        let app = NSApp();
        if app == nil {
            return;
        }

        let target = shared_target();
        let name = PRODUCT_NAME;

        let main_menu: id = msg_send![class!(NSMenu), alloc];
        let main_menu: id = msg_send![main_menu, init];

        // --- アプリ名メニュー(先頭) ---
        // 先頭サブメニューのタイトルは AppKit がアプリ名で上書きするので、ここでは空でよい。
        let app_menu = add_submenu(main_menu, "");
        add_item(
            app_menu,
            &format!("{name} について"),
            sel!(orderFrontStandardAboutPanel:),
            "",
            nil,
        );
        add_separator(app_menu);
        add_item(app_menu, "環境設定…", sel!(vltPreferences:), ",", target);
        add_separator(app_menu);
        add_item(app_menu, &format!("{name} を隠す"), sel!(hide:), "h", nil);
        let hide_others = add_item(
            app_menu,
            "ほかを隠す",
            sel!(hideOtherApplications:),
            "h",
            nil,
        );
        let _: () = msg_send![hide_others, setKeyEquivalentModifierMask: NS_COMMAND | NS_OPTION];
        add_item(app_menu, "すべてを表示", sel!(unhideAllApplications:), "", nil);
        add_separator(app_menu);
        add_item(
            app_menu,
            &format!("{name} を終了"),
            sel!(terminate:),
            "q",
            nil,
        );

        // --- 編集メニュー ---
        let edit_menu = add_submenu(main_menu, "編集");
        add_item(edit_menu, "取り消す", sel!(vltUndo:), "z", target);
        let redo = add_item(edit_menu, "やり直す", sel!(vltRedo:), "z", target);
        let _: () = msg_send![redo, setKeyEquivalentModifierMask: NS_COMMAND | NS_SHIFT];
        add_separator(edit_menu);
        add_item(edit_menu, "カット", sel!(vltCut:), "x", target);
        add_item(edit_menu, "コピー", sel!(vltCopy:), "c", target);
        add_item(edit_menu, "ペースト", sel!(vltPaste:), "v", target);
        add_item(edit_menu, "すべてを選択", sel!(vltSelectAll:), "a", target);

        // --- ウインドウメニュー ---
        let window_menu = add_submenu(main_menu, "ウインドウ");
        add_item(window_menu, "しまう", sel!(performMiniaturize:), "m", nil);
        add_item(window_menu, "拡大/縮小", sel!(performZoom:), "", nil);
        // AppKit に「ウインドウメニュー」として教えると、開いているウインドウ一覧が自動で並ぶ。
        let _: () = msg_send![app, setWindowsMenu: window_menu];

        // --- ヘルプメニュー ---
        let help_menu = add_submenu(main_menu, "ヘルプ");
        add_item(
            help_menu,
            "ログフォルダを開く",
            sel!(vltOpenLogs:),
            "",
            target,
        );

        let _: () = msg_send![app, setMainMenu: main_menu];
    }
}
