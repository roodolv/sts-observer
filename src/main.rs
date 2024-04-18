mod common;
mod mode;

use common::*;
use mode::*;

use libsts::Save;
use std::fs::{self};
use std::path::Path;

fn main() {
    let mut target: Target = Target::new(); // autosave監視用
    let mut json_data: JsonData = JsonData::new(); // JSONデータ関連
    target.init_dir_path(&json_data);

    let mut mode_selector = ModeSelector::new(); // モード管理用マシン
    let waiting_mode = Mode::IsWaiting(Waiting::new());
    let watching_mode = Mode::IsWatching(Watching::new());
    let fileio_mode = Mode::IsFileIO(FileIO::new());

    let max_watching_repeat: u8 = json_data.get_value_from_key("max_watching_repeat").unwrap();
    let mut loop_counter: u8 = 0;
    let loop_interval_ms: u64 = json_data.get_value_from_key("loop_interval_ms").unwrap();

    loop {
        // ループカウンタ処理
        loop_counter += 1;
        println!("<<<<Loop{}>>>>", loop_counter);

        // モード確認
        println!("current_mode: {:?}", mode_selector.current_mode());

        /* -----------------------------------
            待機モードの処理
        ----------------------------------- */
        while let Mode::IsWaiting(ref mode) = mode_selector.current_mode() {
            println!("<<Waiting mode: loop{}>>", mode_selector.times_repeated() + 1);

            /* autosaveの更新日時を比較してJSONを更新&モード分岐

              JSONより新しい場合: txt出力後に監視モードへ遷移
              JSONと等しい場合: 待機モードを反復(txt出力しない)
              JSONより古い場合: 初回のみ空のtxt出力後に待機モードを反復
            */
            sync_json_with_autosave(&mut mode_selector, &mut target, &mut json_data);

            if mode_selector.has_target() {
                println!("\nAutosave file found!");
                println!("1. autosave_path: {}\n2. modified_time: {}\n3. character_type: {}",
                         &target.full_path(),
                         &target.modified_time(),
                         &target.character_type());
            } else {
                println!("\nNo '.autosave' file found");
            }

            // ファイルI/O遷移判定
            switch_fileio_transition(&mut mode_selector, fileio_mode.clone(), mode);
            wait_ms(loop_interval_ms);
        }

        /* -----------------------------------
            監視モードの処理
        ----------------------------------- */
        while let Mode::IsWatching(ref mode) = mode_selector.current_mode() {
            println!("<<Watching mode: loop{}>>", mode_selector.times_repeated() + 1);
            // 定期的にループから抜け出し待機(Waiting)モードへ遷移して他のautosaveファイルを確認
            if mode_selector.times_repeated() >= max_watching_repeat {
                println!("Periodic shift to Waiting mode");
                switch_watching_to_waiting(&mut mode_selector, waiting_mode.clone());
                continue;
            }
            // 毎ループ監視対象のautosaveファイルの存在を確認
            if target.autosave_exists() {
                println!("{}'s autosave exists", &target.character_type());
                mode_selector.found_target(); // 一応
            } else {
                // autosaveが削除されていれば再び待機モードへ
                println!("{}'s autosave does not exist", &target.character_type());
                switch_watching_to_waiting(&mut mode_selector, waiting_mode.clone());
                continue;
            }

            // 監視中autosaveの更新日時比較＆モード分岐
            let cloned_full_path = target.full_path();
            let target_path = Path::new(&cloned_full_path);
            match json_data.compare_modified_time(&target_path) {
                ModifiedTimeStatus::New => {
                    println!("Update JSON since the found autosave is NEWer!\n");
                    // 監視対象のフルパス・更新日時・キャラクタータイプを更新
                    target.update_params(&target_path);
                    // JSONの値を監視対象の値で上書き
                    json_data.update_json_body("modified_time", &target.modified_time());
                    json_data.update_json_body("character_type", &target.character_type());

                    // 更新されていればファイルI/Oモードへの遷移をトリガー
                    mode_selector.turn_on_do_writing(); // 差分がある時のみファイル書き出しをON
                },
                ModifiedTimeStatus::Equal => {
                    // 同じ更新日時の場合は監視モードをループ
                    println!("Continue watching! The found autosave is SAME as JSON's one!");
                    mode_selector.turn_off_do_writing(); // 差分がない場合はファイル書き出しをOFF
                },
                _ => {},
            }

            // ファイルI/O遷移判定
            switch_fileio_transition(&mut mode_selector, fileio_mode.clone(), mode);
            wait_ms(loop_interval_ms);
        }

        /* -----------------------------------
            ファイルI/Oモードの処理
        ----------------------------------- */
        if let Mode::IsFileIO(ref _mode) = mode_selector.current_mode() {
            println!("<<FileI/O mode>>");
            // 監視対象の有無で書き出すファイル内容を場合分け
            if mode_selector.has_target() {
                // 監視対象のファイルを読み込む
                let target_autosave_contents = fs::read_to_string(&target.full_path()).unwrap();
                // txt書き出し(targetありautosaveの更新差分あり)
                let save = Save::new(&target_autosave_contents).expect("An error occurred during the file writing process");
                target.write_txt_basic_info(&save);
                target.write_txt_enemies(&save);
                // target.write_autosave(&save);
                // target.write_autosave_beta(&save);

                // 監視対象があるので監視(Watching)モードへ遷移
                println!("Mode transition: from FileIO to Watching");
                mode_selector.switch_mode(&watching_mode);
                assert_eq!(mode_selector.current_mode(), watching_mode);
            } else {
                // 書き出し(targetなし)
                target.write_txt_empty();

                // 監視対象がないので待機(Waiting)モードへ遷移
                println!("Mode transition: from FileIO to Waiting");
                mode_selector.turn_off_do_writing(); // 次に更新差分を検知するまで書き出しOFF
                mode_selector.switch_mode(&waiting_mode);
                assert_eq!(mode_selector.current_mode(), waiting_mode);
            }
        }

        /* -----------------------------------
            共通のウェイト処理
        ----------------------------------- */
        println!("\nNow on interval...(main loop)\n");
        wait_ms(loop_interval_ms);
    }
}
