use std::{fs, path::Path};

use argparse;

fn read_path_size(path: &str) -> u64 {
    let mut size = 0u64;
    fn read(path: &str, size: &mut u64) {
        if Path::new(path).is_dir() {
            fs::read_dir(path).unwrap().for_each(|item| {
                let path = item.unwrap().path();
                let path = path.to_str().unwrap();
                read(path, size);
            });
        } else {
            let meta = Path::new(path).metadata();
            match meta {
                Ok(meta) => *size += meta.len(),
                Err(exn) => println!("get size of {} failed: {:?}", path, exn),
            }
        }
    }
    read(path, &mut size);
    size
}

fn main() {
    let mut path = "".to_string();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.refer(&mut path)
            .add_argument("path", argparse::Store, "the path to analyse.")
            .required();
        ap.parse_args_or_exit();
    }
    let mut file_metalist: Vec<(u64, Option<String>)> = fs::read_dir(Path::new(path.as_str()))
        .unwrap()
        .map(|item| {
            let path = item.unwrap().path();
            let ext = Path::new(path.to_str().unwrap()).file_name();
            let ext = match ext {
                None => None,
                Some(ext) => match ext.to_str() {
                    None => None,
                    Some(ext) => Some(ext.to_string()),
                },
            };
            (read_path_size(path.to_str().unwrap()), ext)
        })
        .collect::<Vec<_>>();
    file_metalist.sort_by(|v1, v2| v2.0.cmp(&v1.0));
    fn str_padding(str: &str, indent: i32) -> String {
        let mut padding = vec![];
        let gap = indent - str.len() as i32;
        if gap > 0 {
            for _ in 0..gap {
                padding.push(" ");
            }
        }
        str.to_string() + padding.join("").as_str()
    }
    println!("{}{}", str_padding("size", 24), str_padding("dir", 24));
    let num = if file_metalist.len() < 10 {
        file_metalist.len()
    } else {
        10
    };
    file_metalist[0..num].iter().for_each(|(size, path)| {
        let mb = f64::from((*size / 1024 / 1024) as u32);
        let mb_str = ((mb * 100.).round() / 100.).to_string() + "mb";
        let path = match path {
            None => "",
            Some(path) => path.as_str(),
        };
        println!(
            "{}{}",
            str_padding(mb_str.as_str(), 24),
            str_padding(path, 24)
        );
    });
}
