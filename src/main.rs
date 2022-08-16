use std::{
    borrow::BorrowMut,
    cell::{Cell, RefCell},
    rc::Rc,
    time,
};

use argparse;
use async_std::{
    fs::{self, DirEntry},
    path::Path,
    task,
};
use futures::{Stream, StreamExt};

async fn read_path_size(path: String) -> u64 {
    let mut size = 0u64;
    #[async_recursion::async_recursion]
    async fn read(path: &str, size: &mut u64) {
        if Path::new(path).is_dir().await {
            match fs::read_dir(path).await {
                Err(exn) => println!("read dir {} failed: {:?}", path, exn),
                Ok(mut result) => {
                    while let Some(result) = result.next().await {
                        let path = result.unwrap().path();
                        let path = path.to_str().unwrap();
                        read(path, size).await;
                    }
                }
            }
        } else {
            let meta = Path::new(path).metadata().await;
            match meta {
                Ok(meta) => *size += meta.len(),
                Err(exn) => println!("get size of {} failed: {:?}", path, exn),
            }
        }
    }
    read(path.as_str(), &mut size).await;
    size
}

struct ReaddirResult {
    input: DirEntry,
    output: Cell<Option<(u64, String)>>,
}

struct VecStream<T: Clone> {
    content: Vec<T>,
    pos: Cell<usize>,
}

impl<T: Clone> Stream for VecStream<T> {
    type Item = T;
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        if self.pos.get() < self.content.len() {
            let item = self.content[self.pos.get()].clone();
            self.pos.set(self.pos.get() + 1);
            task::Poll::Ready(Some(item))
        } else {
            task::Poll::Ready(None)
        }
    }
}

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

async fn scan(path: String) -> Vec<(u64, String)> {
    let readdir_result = fs::read_dir(Path::new(path.as_str()))
        .await
        .unwrap()
        .filter_map(|item| async {
            match item {
                Err(exn) => panic!("{:?}", exn),
                Ok(item) => Some(Rc::new(RefCell::new(ReaddirResult {
                    input: item,
                    output: Cell::new(None),
                }))),
            }
        })
        .collect::<Vec<_>>()
        .await;
    let cpu_num = num_cpus::get();
    let readdir_stream = VecStream {
        content: readdir_result.clone(),
        pos: Cell::new(0),
    };
    readdir_stream
        .for_each_concurrent(cpu_num, |item| {
            let item = item.clone();
            async move {
                match item.try_borrow() {
                    Err(exn) => panic!("{:?}", exn),
                    Ok(mut item) => {
                        let path = item.input.path();
                        let ext = Path::new(path.to_str().unwrap())
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string();
                        let path = path.to_str().unwrap().to_string();
                        let count = read_path_size(path.clone()).await;
                        // println!("count: {}", count);
                        item.borrow_mut().output.set(Some((count, ext)));
                    }
                }
            }
        })
        .await;
    let mut readdir_result = readdir_result
        .iter()
        .filter_map(|item| {
            let item = &RefCell::borrow(item).output;
            let value = item.take();
            let result = value.clone();
            item.set(value);
            result
        })
        .collect::<Vec<_>>();
    readdir_result.sort_by(|a, b| {
        let (count1, _) = a;
        let (count2, _) = b;
        // sort desend
        count2.cmp(count1)
    });
    readdir_result
}

async fn start() {
    let mut path = "".to_string();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.refer(&mut path)
            .add_argument("path", argparse::Store, "the path to analyse.")
            .required();
        ap.parse_args_or_exit();
    }
    let start_time = time::SystemTime::now();
    let readdir_result = scan(path).await;
    let end_time = time::SystemTime::now();
    let time_use = end_time.duration_since(start_time).unwrap();
    if readdir_result.len() > 0 {
        println!("{}{}", str_padding("size", 24), str_padding("dir", 24));
    } else {
        println!("nothing in the path.")
    }
    readdir_result.iter().take(10).for_each({
        move |item| {
            let (count, ext) = item;
            let mb = f64::from((*count / 1024 / 1024) as u32);
            let mb_str = ((mb * 100.).round() / 100.).to_string() + "mb";
            println!(
                "{}{}",
                str_padding(mb_str.as_str(), 24),
                str_padding(ext, 24)
            );
        }
    });
    println!("scan time use: {}s", time_use.as_secs());
}

fn main() {
    task::block_on(start())
}
