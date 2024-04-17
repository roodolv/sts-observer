// libsts::Saveの各フィールドはここ参照
// https://docs.rs/libsts/latest/libsts/save/struct.Save.html
use libsts::Save;
use serde::de::Error as SerdeError;
use serde_json::{json, Error, Value};
use std::ffi::OsStr;
use std::fs::{self};
use std::path::{Path, PathBuf};

use crate::mode::*;

// 更新日時比較時の条件分岐用
pub enum ModifiedTimeStatus {
    New,
    Old,
    Equal,
}

// 監視対象(autosave)のパラメータを格納する構造体
#[derive(Debug, Clone)]
pub struct Target {
    autosave_dir_path: PathBuf,
    write_txt_dir_path: String,
    full_path: String,
    character_type: String,
    modified_time: u64,
}
#[rustfmt::skip]
impl Target {
    pub fn new() -> Self {
        Self {
            autosave_dir_path: PathBuf::new(),
            write_txt_dir_path: String::new(),
            full_path: String::new(),
            character_type: String::new(),
            modified_time: 0,
        }
    }
    pub fn init_dir_path(&mut self, json_data: &JsonData) {
        self.autosave_dir_path = json_data.get_value_from_key("autosave_dir_path").unwrap();
        self.write_txt_dir_path = json_data.get_value_from_key("write_txt_dir_path").unwrap();
    }
    // getter
    pub fn full_path(&self) -> String { self.full_path.clone() }
    pub fn autosave_dir_path(&self) -> PathBuf { self.autosave_dir_path.clone() }
    pub fn character_type(&self) -> String { self.character_type.clone() }
    pub fn modified_time(&self) -> u64 { self.modified_time }

    pub fn autosave_exists(&self) -> bool {
        // 監視中のautosaveファイルの有無を調べる
        Path::new(&self.full_path).try_exists().unwrap()
    }
    pub fn update_params<P: AsRef<Path>>(&mut self, autosave_path: &P) {
        self.full_path = to_string(autosave_path.as_ref());
        self.character_type = get_file_basename(autosave_path.as_ref());
        self.modified_time = get_file_modified_time(autosave_path).unwrap();
    }
    #[allow(dead_code)]
    pub fn write_autosave(&self, save: &Save) {
        // Get the base64 string representation of our modified savefile
        if let Ok(modified_save) = &save.to_b64_string() {
            let _ = fs::write(&self.full_path, modified_save);
            println!("Save succeeded: autosave");
        }
    }
    #[allow(dead_code)]
    pub fn write_autosave_beta(&self, save: &Save) {
        if let Ok(modified_save) = &save.to_string() {
            let _ = fs::write(&self.full_path, modified_save);
            println!("Save succeeded: autosaveBETA");
        }
    }
    pub fn write_txt_basic_info(&self, save: &Save) {
        if let Ok(txt_body) = compose_txt_basic_info(save) {
            let _ = fs::write(self.write_txt_dir_path.clone() + "sts_basic_info.txt", txt_body);
            println!("Save succeeded: basic-info txt");
        }
    }
    pub fn write_txt_enemies(&self, save: &Save) {
        if let Ok(txt_body) = compose_txt_enemies(save) {
            let _ = fs::write(self.write_txt_dir_path.clone() + "sts_enemies.txt", txt_body);
            println!("Save succeeded: enemies txt");
        }
    }
    pub fn write_txt_empty(&self) {
        if let Ok(txt_body) = compose_txt_empty() {
            let _ = fs::write(self.write_txt_dir_path.clone() + "sts_relic_potion.txt", &txt_body);
            let _ = fs::write(self.write_txt_dir_path.clone() + "sts_card_choice.txt", &txt_body);
            println!("Save succeeded: EMPTY relics&potions txt");
        }
    }
}

const LOCAL_JSON_PATH: &str = "./settings.json";

// JSONのパラメータを格納する構造体
#[derive(Debug, Clone)]
pub struct JsonData {
    path: &'static Path,
    body: Value,
}
#[rustfmt::skip]
impl JsonData {
    pub fn new() -> Self {
        Self {
            path: Path::new(LOCAL_JSON_PATH),
            body: {
                let json_content = fs::read_to_string(Path::new(LOCAL_JSON_PATH)).unwrap();

                serde_json::from_str(&json_content).map_err(|e| Error::custom(e.to_string())).unwrap()
            },
        }
    }
    pub fn get_value_from_key<T>(&self, key: &str) -> Option<T>
        where T: serde::de::DeserializeOwned {
        self.body.get(key).and_then(|value| serde_json::from_value(value.clone()).ok())
    }
    pub fn update_json_body<T>(&mut self, key: &str, new_value: &T)
        where T: serde::Serialize + serde::de::DeserializeOwned + 'static, {
        let current_value = &self.body[key];
        let new_value_type_id = std::any::TypeId::of::<T>();

        // JSON上書き前に型チェック
        match (current_value, new_value_type_id) {
            (Value::Number(_), tid) if tid == std::any::TypeId::of::<u64>() => {
                self.body[key] = json!(new_value);
            },
            (Value::String(_), tid) if tid == std::any::TypeId::of::<String>()
                || tid == std::any::TypeId::of::<&str>()
                || tid == std::any::TypeId::of::<&String>()
                || tid == std::any::TypeId::of::<std::string::String>() => {
                self.body[key] = json!(new_value);
            },
            (Value::Bool(_), tid) if tid == std::any::TypeId::of::<bool>() => {
                self.body[key] = json!(new_value);
            },
            _ => {
                panic!("Incompatible types ('{}') for key: {}", type_of(&new_value), key);
            }
        }
        let _ = fs::write(self.path, self.body.to_string());
    }
    // トレイト境界AsRef<Path>により、Path型もPathBuf型も両方受け取れる
    pub fn compare_modified_time<P: AsRef<Path>>(&self, autosave_path: &P) -> ModifiedTimeStatus {
        let autosave_modified_time = get_file_modified_time(autosave_path).unwrap();
        let json_autosave_modified_time: u64 = self.get_value_from_key("modified_time").unwrap();

        match autosave_modified_time.cmp(&json_autosave_modified_time) {
            std::cmp::Ordering::Less => ModifiedTimeStatus::Old,
            std::cmp::Ordering::Greater => ModifiedTimeStatus::New,
            std::cmp::Ordering::Equal => ModifiedTimeStatus::Equal,
        }
    }
}

#[allow(dead_code)]
fn type_of<T>(_: &T) -> &'static str { std::any::type_name::<T>() }

// Path型やOsStr型向けに共通化されたto_string()関数
fn to_string<T: AsRef<OsStr>>(value: T) -> String {
    let os_str: &OsStr = value.as_ref();
    os_str.to_str().unwrap().to_string()
}
// XXX: これはto_string_lossy()を使うので文字化けの可能性が残る
// fn to_string<T: AsRef<std::ffi::OsStr>>(value: T) -> String { value.as_ref().to_string_lossy().to_string() }
// XXX: こっちだとOsStr型がToStringトレイトを持ってないからエラー
// fn to_string<T: ToString>(value: T) -> String { value.to_string() }

pub fn wait_ms(time_ms: u64) { std::thread::sleep(std::time::Duration::from_millis(time_ms)) }

fn compose_txt_basic_info(save: &Save) -> Result<String, Error> {
    let mut ret: String = String::new();
    ret.push_str(&format!("Player's Name: {}\n", save.name));
    ret.push_str(&format!("Ascension level: {}\n", save.ascension_level.to_string().as_str()));
    ret.push_str(&format!("Gold: {}\n", save.gold.to_string().as_str()));
    ret.push_str(&format!("Health: {}/{}\n", save.current_health.to_string().as_str(), save.max_health.to_string().as_str()));
    ret.push_str(&format!("Play time: {}\n", save.play_time.to_string().as_str()));
    Ok(ret)
}
fn compose_txt_enemies(save: &Save) -> Result<String, Error> {
    let mut ret: String = String::new();
    for (n, monster) in save.monster_list.iter().enumerate() {
        ret.push_str(&format!("Monster{}: {}\n", n + 1, monster));
    }
    for (n, elite) in save.elite_monster_list.iter().enumerate() {
        ret.push_str(&format!("Elite{}: {}\n", n + 1, elite));
    }
    for (n, boss) in save.boss_list.iter().enumerate() {
        ret.push_str(&format!("Boss{}: {}\n", n + 1, boss));
    }
    Ok(ret)
}
fn compose_txt_empty() -> Result<String, Error> {
    let mut ret: String = String::new();
    ret.push_str("---");
    ret.push('\n');
    Ok(ret)
}

// Pathからファイル名(basename)を取得してString型で返す
fn get_file_basename(path: &Path) -> String { to_string(path.file_stem().unwrap()) }

// Path/PathBufからファイルの更新日時を取得する
fn get_file_modified_time<P: AsRef<Path>>(path: &P) -> Result<u64, Error> {
    let target_file = fs::metadata(path).expect("metadata() failed");
    Ok(target_file.modified()
                  .expect("modified_time: modified() failed")
                  .duration_since(std::time::SystemTime::UNIX_EPOCH)
                  .expect("modified_time: duration_since() failed")
                  .as_secs())
}

// 待機(Waiting)モードで実行されるautosaveチェック用関数
pub fn sync_json_with_autosave(mode_selector: &mut ModeSelector, target: &mut Target, json_data: &mut JsonData) {
    let character_list = ["IRONCLAD", "THE SILENT", "DEFECT", "WATCHER"];
    for character in character_list {
        let autosave_path = target.autosave_dir_path().join(&format!("{}.autosave", character));
        if !autosave_path.is_file() {
            continue; // 指定したファイル以外はスキップ
        }

        // JSON内の更新日時と比較
        match json_data.compare_modified_time(&autosave_path) {
            ModifiedTimeStatus::New => {
                println!("The found autosave is NEWer than JSON's one!");
                // 監視対象のフルパス・キャラクタータイプ・更新日時を更新
                target.update_params(&autosave_path);
                // JSONの値を監視対象の値で上書き
                json_data.update_json_body("modified_time", &target.modified_time());
                json_data.update_json_body("character_type", &target.character_type());
                // 監視対象を発見したのでモードセレクト変数を更新
                mode_selector.found_target();
                // 更新差分があるのでファイル書き出しON
                mode_selector.turn_on_do_writing();
            },
            ModifiedTimeStatus::Equal => {
                println!("The found autosave is SAME as JSON's one!");
                // 監視対象のフルパス・キャラクタータイプ・更新日時を更新
                target.update_params(&autosave_path);
                // JSONの値を監視対象の値で上書き
                json_data.update_json_body("character_type", &target.character_type());
                // 監視対象を発見したのでモードセレクト変数を更新
                mode_selector.found_target();
                // 更新差分がないのでファイル書き出しOFF
                mode_selector.turn_off_do_writing();
            },
            ModifiedTimeStatus::Old => {
                println!("Skip! The found autosave is OLDer than JSON's one!");
                /* ファイル書き出しスイッチは敢えて操作しない(元のまま) */
                continue; // JSONの更新日時より古ければスキップ
            },
        }
    }
}
