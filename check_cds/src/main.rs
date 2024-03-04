use clap::{App, Arg};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use std::io::Write;

struct CdsSequence {
    id: String,
    sequence: String,
}

struct ErrorReport {
    missing_start_codon: bool,
    missing_stop_codon: bool,
    illegal_codon_sequence: bool,
    non_multiple_of_three_length: bool,
    premature_stop_codon: bool,
}

type ReportTable = HashMap<String, ErrorReport>;

fn read_cds_file(filename: &str) -> Result<Vec<CdsSequence>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut cds_sequences = Vec::new();
    let mut current_id = String::new();
    let mut current_sequence = String::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('>') {
            if !current_id.is_empty() {
                cds_sequences.push(CdsSequence {
                    id: current_id.clone(),
                    sequence: current_sequence.clone(),
                });
                current_sequence.clear();
            }
            current_id = line.trim_start_matches('>').to_string();
        } else {
            current_sequence.push_str(&line);
        }
    }

    if !current_id.is_empty() {
        cds_sequences.push(CdsSequence {
            id: current_id,
            sequence: current_sequence,
        });
    }

    Ok(cds_sequences)
}

// fn check_for_premature_stop_codon(sequence: &str) -> bool {
//     let codon_length = 3;
//     let sequence_len = sequence.len();
//     let mut codon_iterator = sequence.chars().enumerate().peekable();
//     while let Some((index, _)) = codon_iterator.next() {
//         if let Some(next_index) = index.checked_add(codon_length) {
//             if next_index >= sequence_len { // 不考虑最后三个碱基
//                 break;
//             }
//         } else {
//             // Integer overflow
//             break;
//         }
//         let mut codon = String::new();
//         for _ in 0..codon_length {
//             if let Some((_, base)) = codon_iterator.next() {
//                 codon.push(base);
//             } else {
//                 // Sequence has ended
//                 return false;
//             }
//         }
//         if codon == "TAA" || codon == "TAG" || codon == "TGA" {
//             return true;
//         }
//     }
//     false
// }

fn check_for_errors(sequence: &str) -> ErrorReport {
    let mut error_report = ErrorReport {
        missing_start_codon: false,
        missing_stop_codon: false,
        illegal_codon_sequence: false,
        non_multiple_of_three_length: false,
        premature_stop_codon: false,
    };

    // Convert sequence to uppercase to ensure consistency
    let sequence = sequence.to_uppercase();

    // Check for missing start codon
    if !sequence.starts_with("ATG") {
        error_report.missing_start_codon = true;
    }
    // Check for missing stop codon
    if !sequence.ends_with("TAA") && !sequence.ends_with("TAG") && !sequence.ends_with("TGA") {
        error_report.missing_stop_codon = true;
    }
    // Check for illegal codon sequence
    if sequence.chars().any(|c| !"ATCG".contains(c)) {
        error_report.illegal_codon_sequence = true;
    }
    // Check for non-multiple of three length
    if sequence.len() % 3 != 0 {
        error_report.non_multiple_of_three_length = true;
    }
    // Check for premature stop codon
    let stop_codons = ["TAA", "TAG", "TGA"];
    let codon_length = 3;
    let substrings = sequence
        .chars()
        .collect::<Vec<_>>()
        .chunks(codon_length)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>();

    // Remove the last substring
    let substrings_without_last = if substrings.len() > 1 {
        &substrings[..substrings.len() - 1]
    } else {
        &[]
    };

    // Check for premature stop codon in each substring
    let has_stop_codon = substrings_without_last.iter().any(|substring| {
        stop_codons
            .iter()
            .any(|&stop| substring == stop)
    });

    error_report.premature_stop_codon = has_stop_codon;

    error_report
}

fn generate_report_table(cds_sequences: &[CdsSequence]) -> ReportTable {
    let mut report_table = ReportTable::new();

    for sequence in cds_sequences {
        let error_report = check_for_errors(&sequence.sequence);
        // Only insert into report_table if errors are found
        if error_report.missing_start_codon
            || error_report.missing_stop_codon
            || error_report.illegal_codon_sequence
            || error_report.non_multiple_of_three_length 
            || error_report.premature_stop_codon {
            report_table.insert(sequence.id.clone(), error_report);
        }
    }

    report_table
}

fn write_report_table_to_file(report_table: &ReportTable, filename: &str) -> Result<()> {
    let mut file = File::create(filename)?;
    writeln!(file, "Sequence_ID\tMissing_Start_Codon\tMissing_Stop_Codon\tIllegal_Codon_Sequence\tNon_Multiple_Of_Three_Length]\tPremature_stop_codon")?;
    
    for (id, report) in report_table {
        writeln!(file, "{}\t{}\t{}\t{}\t{}\t{}", id,
            if report.missing_start_codon {1} else {0},
            if report.missing_stop_codon {1} else {0},
            if report.illegal_codon_sequence {1} else {0},
            if report.non_multiple_of_three_length {1} else {0},
            if report.premature_stop_codon {1} else {0}
        )?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let matches = App::new("\nCDS Error Checker")
        .version("1.0")
        .author("\nZhongsi Wang, 20240222")
        .about("\nChecks a CDS file for errors: \n\tType1: missing start codon\n\tType2: missing stop codon\n\tType3: illegal codon sequence\n\tType4: non_multiple of three length\n\tType5: premature stop codon")
        .arg(Arg::new("input")
            .short('i')
            .long("input")
            .value_name("FILE")
            .help("Sets the input file to use")
            .takes_value(true)
            .required(true))
        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .value_name("FILE")
            .help("Sets the output file to use")
            .takes_value(true)
            .required(true))
        .get_matches_safe();

    match matches {
        Ok(matches) => {
            let cds_filename = matches.value_of("input").unwrap();
            let output_filename = matches.value_of("output").unwrap();

            let cds_sequences = read_cds_file(cds_filename)?;
            let report_table = generate_report_table(&cds_sequences);
            write_report_table_to_file(&report_table, output_filename)?;

            println!("Error report generated successfully.");
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }

    Ok(())
}