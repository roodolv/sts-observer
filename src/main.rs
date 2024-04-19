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
        println!("\n<<<<Loop{}>>>>", loop_counter);

        // モード確認
        println!("current_mode: {:?}", mode_selector.current_mode());

        /* -----------------------------------
            待機モードの処理
        ----------------------------------- */
        while let Mode::IsWaiting(ref mode) = mode_selector.current_mode() {
            println!("\n<<Waiting mode: loop{}>>", mode_selector.times_repeated() + 1);

            /* autosaveの更新日時を比較してJSONを更新&モード分岐

              JSONより新しい場合: txt出力後に監視モードへ遷移
              JSONと等しい場合: 待機モードを反復(txt出力しない)
              それ以外: 初回のみ空のtxt出力後に待機モードを反復
            */
            mode_selector.reset_target();
            let character_list = ["IRONCLAD", "THE SILENT", "DEFECT", "WATCHER"];
            for character in character_list {
                let autosave_path = target.autosave_dir_path().join(&format!("{}.autosave", character));
                if !autosave_path.is_file() {
                    continue; // 指定したファイル以外はスキップ
                }
                // JSONの更新日時と比較＆監視対象更新＆モード分岐
                autosave_mode_selector(&mut mode_selector, &mut target, &mut json_data, &autosave_path);
            }

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
            switch_to_fileio(&mut mode_selector, fileio_mode.clone(), mode);
            wait_ms(loop_interval_ms);
        }

        /* -----------------------------------
            監視モードの処理
        ----------------------------------- */
        while let Mode::IsWatching(ref mode) = mode_selector.current_mode() {
            println!("\n<<Watching mode: loop{}>>", mode_selector.times_repeated() + 1);
            // 定期的にループから抜け出し待機(Waiting)モードへ遷移して他のautosaveファイルを確認
            if mode_selector.times_repeated() >= max_watching_repeat {
                println!("Periodic shift to Waiting mode");
                switch_to_waiting(&mut mode_selector, waiting_mode.clone());
                continue;
            }
            // 毎ループ監視対象のautosaveファイルの存在を確認
            if target.autosave_exists() {
                println!("{}'s autosave exists", &target.character_type());
                mode_selector.found_target(); // 一応
            } else {
                // autosaveが削除されていれば再び待機モードへ
                println!("{}'s autosave does not exist", &target.character_type());
                switch_to_waiting(&mut mode_selector, waiting_mode.clone());
                continue;
            }

            // 監視中autosaveの更新日時比較＆監視対象更新＆モード分岐
            let cloned_full_path = target.full_path();
            let target_path = Path::new(&cloned_full_path);
            autosave_mode_selector(&mut mode_selector, &mut target, &mut json_data, target_path);

            // ファイルI/O遷移判定
            switch_to_fileio(&mut mode_selector, fileio_mode.clone(), mode);
            wait_ms(loop_interval_ms);
        }

        /* -----------------------------------
            ファイルI/Oモードの処理
        ----------------------------------- */
        if let Mode::IsFileIO(ref _mode) = mode_selector.current_mode() {
            println!("\n<<FileI/O mode>>");
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
        println!("\nNow on interval...(main loop)");
        wait_ms(loop_interval_ms);
    }
}
