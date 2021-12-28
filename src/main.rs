use anyhow::{bail, Result};
use std::io::prelude::*;
use std::{fs::File, path::PathBuf};

fn main() -> Result<()> {
    let mut args = pico_args::Arguments::from_env();

    match args.subcommand()?.as_ref().map(|s| s.as_str()) {
        None => print_help(),
        Some("pack") => {
            let level = args.opt_value_from_str(["-l", "--level"])?.unwrap_or(2u8);
            let use_bitstream = args.contains(["-b", "--bitstream"]);

            let infile = args.free_from_os_str::<PathBuf, bool>(|s| Ok(s.into()))?;
            let outfile = args.free_from_os_str::<PathBuf, bool>(|s| Ok(s.into()))?;

            let mut data = vec![];
            File::open(infile)?.read_to_end(&mut data)?;
            
            let mut pb = pbr::ProgressBar::new(data.len() as u64);
            pb.set_units(pbr::Units::Bytes);
            let packed_data = upkr::pack(
                &data,
                level,
                use_bitstream,
                Some(&mut |pos| {
                    pb.set(pos as u64);
                }),
            );
            pb.finish();

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

            let infile = args.free_from_os_str::<PathBuf, bool>(|s| Ok(s.into()))?;
            let outfile = args.free_from_os_str::<PathBuf, bool>(|s| Ok(s.into()))?;

            let mut data = vec![];
            File::open(infile)?.read_to_end(&mut data)?;
            let packed_data = upkr::unpack(&data, use_bitstream);
            File::create(outfile)?.write_all(&packed_data)?;
        }
        Some(other) => {
            bail!("Unknown subcommand '{}'", other);
        }
    }

    Ok(())
}

fn print_help() {
    eprintln!("Usage:");
    eprintln!("  upkr pack [-l level(0-9)] <infile> <outfile>");
    eprintln!("  upkr unpack <infile> <outfile>");
    std::process::exit(1);
}
