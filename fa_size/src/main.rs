use std::env;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};

fn main() {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_fa_file> <output_file>", args[0]);
        std::process::exit(1);
    }
    let input_file_path = &args[1];
    let output_file_path = &args[2];

    // 打开输入文件
    let file = File::open(input_file_path).expect("Failed to open input file");
    let reader = BufReader::new(file);

    // 打开输出文件，使用写入模式来覆盖已存在的文件
    let mut output_file = File::create(output_file_path).expect("Failed to create output file");

    // 定义变量来跟踪当前序列的名称
    let mut current_sequence_name = String::new();
    let mut current_sequence_length = 0;

    // 逐行读取输入文件内容
    for line in reader.lines() {
        let line = line.expect("Failed to read line");

        // 如果行以 ">" 开头，表示新的序列开始
        if line.starts_with('>') {
            // 处理前一个序列（如果有的话）
            if !current_sequence_name.is_empty() {
                // 输出当前序列的名称和长度到输出文件中，用 tab 分隔
                writeln!(output_file, "{}\t{}", current_sequence_name, current_sequence_length).expect("Failed to write output");
            }
            // 更新当前序列的名称，同时重置当前序列的长度
            current_sequence_name = line[1..].trim().split_whitespace().next().unwrap_or("").to_string();
            current_sequence_length = 0;
        } else {
            // 如果不是以 ">" 开头，则将行的长度添加到当前序列的长度中
            current_sequence_length += line.trim().len();
        }
    }

    // 处理最后一个序列
    if !current_sequence_name.is_empty() {
        // 输出最后一个序列的名称和长度到输出文件中，用 tab 分隔
        writeln!(output_file, "{}\t{}", current_sequence_name, current_sequence_length).expect("Failed to write output");
    }
}
