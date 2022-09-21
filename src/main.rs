use anyhow::{bail, Result};
use std::io::prelude::*;
use std::process;
use std::{fs::File, path::PathBuf};

fn main() -> Result<()> {
    let mut args = pico_args::Arguments::from_env();

    match args.subcommand()?.as_ref().map(|s| s.as_str()) {
        None => print_help(),
        Some("pack") => {
            let level = args.opt_value_from_str(["-l", "--level"])?.unwrap_or(2u8);
            let use_bitstream = args.contains(["-b", "--bitstream"]);
            let parity_contexts = args
                .opt_value_from_str(["-p", "--parity"])?
                .unwrap_or(1usize);
            let reverse = args.contains(["-r", "--reverse"]);

            if parity_contexts != 1 && parity_contexts != 2 && parity_contexts != 4 {
                eprintln!("--parity has to be 1, 2 or 4");
                process::exit(1);
            }

            let infile = args.free_from_os_str::<PathBuf, bool>(|s| Ok(s.into()))?;
            let outfile = args.free_from_os_str::<PathBuf, bool>(|s| Ok(s.into()))?;

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
                use_bitstream,
                parity_contexts,
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
        }
        Some("unpack") => {
            let use_bitstream = args.contains(["-b", "--bitstream"]);
            let parity_contexts = args
                .opt_value_from_str(["-p", "--parity"])?
                .unwrap_or(1usize);
            let reverse = args.contains(["-r", "--reverse"]);

            if parity_contexts != 1 && parity_contexts != 2 && parity_contexts != 4 {
                eprintln!("--parity has to be 1, 2 or 4");
                process::exit(1);
            }

            let infile = args.free_from_os_str::<PathBuf, bool>(|s| Ok(s.into()))?;
            let outfile = args.free_from_os_str::<PathBuf, bool>(|s| Ok(s.into()))?;

            let mut data = vec![];
            File::open(infile)?.read_to_end(&mut data)?;
            if reverse {
                data.reverse();
            }
            let mut unpacked_data = upkr::unpack(&data, use_bitstream, parity_contexts);
            if reverse {
                unpacked_data.reverse();
            }
            File::create(outfile)?.write_all(&unpacked_data)?;
        }
        Some(other) => {
            bail!("Unknown subcommand '{}'", other);
        }
    }

    Ok(())
}

fn print_help() {
    eprintln!("Usage:");
    eprintln!("  upkr pack [-b] [-l level(0-9)] [-p N] <infile> <outfile>");
    eprintln!("  upkr unpack [-b] [-p N] <infile> <outfile>");
    eprintln!();
    eprintln!(" -b, --bitstream     bitstream mode");
    eprintln!(" -l, --level N       compression level 0-9");
    eprintln!(" -p, --parity N      use N (2/4) parity contexts");
    eprintln!(" -r, --reverse       reverse input & output");
    process::exit(1);
}
