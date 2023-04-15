use clap::Parser;
use simple_error::bail;
use simple_error::SimpleError;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::BufRead;
use std::process;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about, verbatim_doc_comment)]
/// samcut is cut for sam: `samtools view in.bam | samcut`. See --help for examples.
///
/// Print the standard 11 columns (rname to qual) with a header row:
///     $ samtools view in.bam | samcut -H
///
/// Print qname, cigar, pos, and tlen only:
///     $ samtools view in.bam | samcut qname cigar pos tlen
///
/// Also print a specific tag:
///     $ samtools view in.bam | samcut qname cigar pos tlen MD
///
/// Get a histogram of (read1, secondary, supplementary) flag values:
///     $ samtools view in.bam | samcut read1 secondary supplementary | sort | uniq -c
pub struct Args {
    /// Print a header row with column names
    #[arg(short = 'H', long)]
    header: bool,

    /// Character to use as delimiter for output
    #[arg(short, long, default_value = "\t")]
    delim: char,

    /// String to fill missing values with
    #[arg(short = 'i', long, default_value = ".")]
    fill: String,

    /// Select only these fields. Example: `samcut n qname tlen read1 MD`. See --help for details.
    /// If not supplied, `std` is used.
    ///
    /// Standard keys:
    ///     key        description
    ///     -----------------------------------------------------------------------
    ///     qname      query template name
    ///     flag       flag
    ///     rname      reference sequence name
    ///     pos        1-based leftmost mapping
    ///     mapq       mapping quality
    ///     cigar      cigar string
    ///     rnext      ref. name of the mate/next read
    ///     pnext      position of the mat/next reas
    ///     tlen       observed template length
    ///     seq        segment sequence
    ///     qual       ascii of phred-scaled base quality+33
    ///
    /// Flag keys:
    ///     key              description
    ///     -----------------------------------------------------------------------
    ///     paired           paired-end (or multiple-segment) sequencing technology
    ///     proper_pair      each segment properly aligned according to the aligner
    ///     unmap            segment unmapped
    ///     munmap           next segment in the template unmapped
    ///     reverse          SEQ is reverse complemented
    ///     mreverse         SEQ of the next segment in the template is reversed
    ///     read1            the first segment in the template
    ///     read2            the last segment in the template
    ///     secondary        secondary alignment
    ///     qcfail           not passing quality controls
    ///     dup              PCR or optical duplicate
    ///     supplementary    supplementary alignment
    ///
    /// Speical keys:
    ///     key        description
    ///     -----------------------------------------------------------------------
    ///     n          One-based index for the input stream
    ///     std        The first 9 columns
    #[arg(verbatim_doc_comment)]
    fields: Vec<String>,
}

/// Replace `search` (item) in `v` (vector) with `replace` (vector)
fn replace_items(mut v: Vec<String>, search: &str, replace: &[&str]) -> Vec<String> {
    let mut i = 0;
    while i < v.len() {
        if v[i] == search {
            v.remove(i);
            for item in replace.iter().rev() {
                v.insert(i, item.to_string());
            }
            i += replace.len();
        } else {
            i += 1;
        }
    }
    v
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        println!("Error: {}", e);
        process::exit(1);
    }
}

fn run(mut args: Args) -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin.lock());
    let lines = reader.lines().filter_map(|line| line.ok());

    if args.fields.is_empty() {
        args.fields = Vec::from([String::from("std")])
    }
    args.fields = replace_items(args.fields, "std", &SAM_FIELDS);

    if args.header {
        println!("{}", args.fields.join(&args.delim.to_string()));
    }
    let mut line_no: usize = 1;
    for line in lines {
        let rec = parse_alignment_line(
            &line,
            &args.fields,
            &args.delim.to_string(),
            &args.fill,
            &line_no,
        )
        .unwrap();
        if rec.is_some() {
            println!("{}", rec.unwrap());
            line_no += 1;
        }
    }
    Ok(())
}

const SAM_FIELDS: [&str; 11] = [
    "qname", "flag", "rname", "pos", "mapq", "cigar", "rnext", "pnext", "tlen", "seq", "qual",
];

const FLAG_FIELDS: [&str; 12] = [
    "paired",
    "proper_pair",
    "unmap",
    "munmap",
    "reverse",
    "mreverse",
    "read1",
    "read2",
    "secondary",
    "qcfail",
    "dup",
    "supplementary",
];

fn sam_flag_as_str(d: &HashMap<&str, bool>) -> String {
    let mut out: Vec<String> = Vec::new();
    for f in FLAG_FIELDS {
        if d[f] {
            out.push(f.to_string());
        }
    }
    out.join(",")
}

fn sam_flag_as_hashmap(i: &i32) -> HashMap<&str, bool> {
    let mut x: i32 = *i;
    let mut out: HashMap<&str, bool> = HashMap::new();
    for f in FLAG_FIELDS {
        out.insert(f, x & 1 != 0);
        x >>= 1;
    }
    out
}

fn parse_alignment_line(
    line: &str,
    print_fields: &Vec<String>,
    delim: &str,
    fill: &str,
    line_no: &usize,
) -> Result<Option<String>, SimpleError> {
    // Skip header lines
    if line.starts_with('@') {
        return Ok(None);
    }

    let fields: Vec<&str> = line.split('\t').collect();
    let mut opt_fields: HashMap<&str, &str> = HashMap::new();

    let tmp = line_no.to_string();
    opt_fields.insert("n", &tmp);

    // Parse the standard 11 fields
    if fields.len() < SAM_FIELDS.len() {
        bail!(
            "Malformed input. Expected at least {} fields; found {} instead on line {}.",
            SAM_FIELDS.len(),
            fields.len(),
            line_no
        );
    }
    for i in 0..SAM_FIELDS.len() {
        opt_fields.insert(SAM_FIELDS[i], fields[i]);
    }

    // Parse the optional fields
    let mut tag_type_value;
    if fields.len() > 11 {
        for f in fields.iter().skip(11) {
            tag_type_value = f.split(':').collect::<Vec<&str>>();
            if tag_type_value.len() != 3 {
                bail!("malformed opt field: {}", f)
            }
            opt_fields.insert(tag_type_value[0], tag_type_value[2]);
        }
    }

    // Parse the flag
    let flag_int = opt_fields["flag"].parse::<i32>().unwrap();
    let flag_map = sam_flag_as_hashmap(&flag_int);
    let flag_str = sam_flag_as_str(&flag_map);
    opt_fields.insert("flags", flag_str.as_str());
    for (k, v) in flag_map.iter() {
        opt_fields.insert(k, if *v { "1" } else { "0" });
    }

    // Build the final output
    let mut out: Vec<String> = Vec::new();
    let mut x: &str;
    for fname in print_fields {
        x = opt_fields.get(&fname.as_str()).unwrap_or(&fill);
        out.push(x.to_string());
    }
    Ok(Some(out.join(delim)))
}

#[cfg(test)]
mod tests {
    use crate::parse_alignment_line;
    use regex::Regex;

    #[test]
    fn it_works() {
        let line = "read1 115 chr1 100 255 2M = 200 25 AA II NM:i:0 ZP:i:65536 ZL:i:25";
        let fields = ["n", "read1", "missing", "ZP", "rname", "flag"];
        let delim = ",";
        let fill = "-";
        let expected = "123,1,-,65536,chr1,115";

        let re = Regex::new(r" +").unwrap();
        let line = re.replace_all(line, "\t").to_string();
        let fields = Vec::from(fields).iter().map(|x| x.to_string()).collect();
        let out = parse_alignment_line(
            &line,
            &fields,
            &String::from(delim),
            &String::from(fill),
            &123,
        );
        assert_eq!(out.unwrap().unwrap(), expected);
    }
}
