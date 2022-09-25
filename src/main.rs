use anyhow::Result;
use std::ffi::OsStr;
use std::io::prelude::*;
use std::process;
use std::{fs::File, path::PathBuf};

fn main() -> Result<()> {
    let mut config = upkr::Config::default();
    let mut reverse = false;
    let mut unpack = false;
    let mut calculate_margin = false;
    let mut level = 2;
    let mut infile: Option<PathBuf> = None;
    let mut outfile: Option<PathBuf> = None;

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        use lexopt::prelude::*;
        match arg {
            Short('b') | Long("bitstream") => config.use_bitstream = true,
            Short('p') | Long("parity") => config.parity_contexts = parser.value()?.parse()?,
            Short('r') | Long("reverse") => reverse = true,
            Long("invert-is-match-bit") => config.is_match_bit = false,
            Long("invert-new-offset-bit") => config.new_offset_bit = false,
            Long("invert-continue-value-bit") => config.continue_value_bit = false,
            Long("invert-bit-encoding") => config.invert_bit_encoding = true,
            Long("simplified-prob-update") => config.simplified_prob_update = true,
            Long("big-endian-bitstream") => {
                config.use_bitstream = true;
                config.bitstream_is_big_endian = true;
            }
            Long("no-repeated-offsets") => config.no_repeated_offsets = true,

            Long("z80") => {
                config.use_bitstream = true;
                config.bitstream_is_big_endian = true;
                config.invert_bit_encoding = true;
                config.simplified_prob_update = true;
            }
            Long("x86") => {
                config.use_bitstream = true;
                config.continue_value_bit = false;
                config.is_match_bit = false;
            }

            Short('u') | Long("unpack") => unpack = true,
            Long("margin") => calculate_margin = true,
            Short('l') | Long("level") => level = parser.value()?.parse()?,
            Short('h') | Long("help") => print_help(0),
            Value(val) if infile.is_none() => infile = Some(val.try_into()?),
            Value(val) if outfile.is_none() => outfile = Some(val.try_into()?),
            _ => return Err(arg.unexpected().into()),
        }
    }

    let infile = infile.unwrap_or_else(|| print_help(1));
    let outfile = outfile.unwrap_or_else(|| {
        let mut name = infile.clone();
        if unpack {
            if name.extension().filter(|&e| e == "upk").is_some() {
                name.set_extension("");
            } else {
                name.set_extension("bin");
            }
        } else {
            let mut filename = name
                .file_name()
                .unwrap_or_else(|| OsStr::new(""))
                .to_os_string();
            filename.push(".upk");
            name.set_file_name(filename);
        }
        name
    });

    if config.parity_contexts != 1 && config.parity_contexts != 2 && config.parity_contexts != 4 {
        eprintln!("--parity has to be 1, 2, or 4");
        process::exit(1);
    }

    if !unpack && !calculate_margin {
        let mut data = vec![];
        File::open(infile)?.read_to_end(&mut data)?;
        if reverse {
            data.reverse();
        }

        let mut pb = pbr::ProgressBar::new(data.len() as u64);
        pb.set_units(pbr::Units::Bytes);
        let mut packed_data = upkr::pack(
            &data,
            level,
            config,
            Some(&mut |pos| {
                pb.set(pos as u64);
            }),
        );
        pb.finish();

        if reverse {
            packed_data.reverse();
        }

        println!(
            "Compressed {} bytes to {} bytes ({}%)",
            data.len(),
            packed_data.len(),
            packed_data.len() as f32 * 100. / data.len() as f32
        );
        File::create(outfile)?.write_all(&packed_data)?;
    } else {
        let mut data = vec![];
        File::open(infile)?.read_to_end(&mut data)?;
        if reverse {
            data.reverse();
        }
        if unpack {
            let mut unpacked_data = upkr::unpack(&data, &config);
            if reverse {
                unpacked_data.reverse();
            }
            File::create(outfile)?.write_all(&unpacked_data)?;
        }
        if calculate_margin {
            println!("{}", upkr::calculate_margin(&data, &config));
        }
    }

    Ok(())
}

fn print_help(exit_code: i32) -> ! {
    eprintln!("Usage:");
    eprintln!("  upkr [-l level(0-9)] [config options] <infile> [<outfile>]");
    eprintln!("  upkr -u [config options] <infile> [<outfile>]");
    eprintln!("  upkr --margin [config options] <infile>");
    eprintln!();
    eprintln!(" -l, --level N       compression level 0-9");
    eprintln!(" -u, --unpack        unpack infile");
    eprintln!(" --margin            calculate margin for overlapped unpacking of a packed file");
    eprintln!();
    eprintln!("Config presets for specific unpackers:");
    eprintln!(" --z80               --big-endian-bitstream --invert-bit-encoding --simplified-prob-update");
    eprintln!(" --x86               --bitstream --invert-is-match-bit --invert-continue-value-bit");
    eprintln!();
    eprintln!("Config options (need to match when packing/unpacking):");
    eprintln!(" -b, --bitstream     bitstream mode");
    eprintln!(" -p, --parity N      use N (2/4) parity contexts");
    eprintln!(" -r, --reverse       reverse input & output");
    eprintln!();
    eprintln!("Config options to tailor output to specific optimized unpackers:");
    eprintln!(" --invert-is-match-bit");
    eprintln!(" --invert-new-offset-bit");
    eprintln!(" --invert-continue-value-bit");
    eprintln!(" --invert-bit-encoding");
    eprintln!(" --simplified-prob-update");
    eprintln!(" --big-endian-bitstream   (implies --bitstream)");
    eprintln!(" --no-repeated-offsets");
    process::exit(exit_code);
}
