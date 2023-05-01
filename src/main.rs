/*
   指定したディレクトリ内で、指定したハッシュロジックで得たハッシュ値が一致するファイルのうち最新のファイルを残し、他のファイルは削除するプログラム
    ディレクトリの指定と、ハッシュロジックの指定は、JSONで記述したファイルで設定する
    設定ファイルはコマンドライン引数で指定するが、指定されなかった場合はカレントディレクトのconfig.jsonであると仮定し処理を行う
    設定ファイルが存在しない場合は、カレントディレクトリに対して処理を行い、ハッシュロジックはMD5とする
    設定可能なハッシュロジックはMD5、SHA1、SHA256、SHA512とする
    なお、次回以降の処理を高速化するために、ハッシュ値の計算結果をファイルに保存しておき、次回以降はそのファイルを読み込んで処理を行う
    計算結果の保存ファイルはresults.jsonとし、処理対象のディレクトリに保存することとする
 */
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json;

use md5::{Digest as Md5Digest, Md5};
use sha1::{Sha1};
use sha2::{Sha256, Sha512};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    directory: String,
    hash_logic: String,
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let config_file = if args.len() > 1 {
        args[1].clone()
    } else {
        "config.json".to_string()
    };

    let config: Config = if Path::new(&config_file).exists() {
        let file = File::open(&config_file)?;
        serde_json::from_reader(file).expect("Error parsing config file")
    } else {
        Config {
            directory: ".".to_string(),
            hash_logic: "MD5".to_string(),
        }
    };

    let target_directory = Path::new(&config.directory);

    let hash_results_file = target_directory.join("results.json");
    let mut file_hashes: HashMap<String, String> = if hash_results_file.exists() {
        let file = File::open(hash_results_file)?;
        serde_json::from_reader(file).expect("Error parsing results file")
    } else {
        HashMap::new()
    };

    let mut files_to_remove = Vec::new();

    for entry in fs::read_dir(target_directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_hash = compute_hash(&path, &config.hash_logic);

            if let Some(existing_file) = file_hashes.get(&file_hash) {
                let existing_metadata = fs::metadata(existing_file)?;
                let current_metadata = fs::metadata(&path)?;

                if current_metadata.modified()? > existing_metadata.modified()? {
                    files_to_remove.push(existing_file.clone());
                } else {
                    files_to_remove.push(path.to_str().unwrap().to_string());
                }
            } else {
                file_hashes.insert(file_hash, path.to_str().unwrap().to_string());
            }
        }
    }

    for file in files_to_remove {
        fs::remove_file(file)?;
    }

    let results_file = File::create(target_directory.join("results.json"))?;
    serde_json::to_writer(results_file, &file_hashes)?;

    Ok(())
}

fn compute_hash(path: &Path, hash_logic: &str) -> String {
    let mut file = File::open(path).expect("Failed to open file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");

    match hash_logic {
        "MD5" => {
            let mut hasher = Md5::new();
            hasher.update(&buffer);
            format!("{:x}", hasher.finalize())
        }
        "SHA1" => {
            let mut hasher = Sha1::new();
            hasher.update(&buffer);
            format!("{:x}", hasher.finalize())
        }
        "SHA256" => {
            let mut hasher = Sha256::new();
            hasher.update(&buffer);
            format!("{:x}", hasher.finalize())
        }
        "SHA512" => {
            let mut hasher = Sha512::new();
            hasher.update(&buffer);
            format!("{:x}", hasher.finalize())
        }
        _ => panic!("Invalid hash logic specified"),
    }
}