// リリースビルドでは、Windowsで起動時にコンソール画面を出さない。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    opensesame_lib::run();
}
