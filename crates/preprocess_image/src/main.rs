use std::{
    io::Write,
    path::{Path, PathBuf},
};

use image::{EncodableLayout, ImageReader};
use walkdir::WalkDir;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!(
            "usage: `{} <file_or_dir_in> <file_or_dir_out>`",
            std::env::args().next().unwrap(),
        );
        std::process::exit(1);
    }
    let in_path = PathBuf::from(&args[1]);
    let out_path = PathBuf::from(&args[2]);

    if in_path.is_dir() {
        for entry in WalkDir::new(&in_path) {
            if let Ok(path) = entry.map(|e| e.into_path())
                && path.is_file()
                && path.extension().is_some_and(|ext| ext == "png")
            {
                let in_file = &path;
                let out_file =
                    out_path.join(path.strip_prefix(&in_path).unwrap().with_extension("rgba"));
                println!("Converting {in_file:?} -> {out_file:?} ...");
                convert_file(in_file, &out_file)?;
            }
        }
    } else {
        convert_file(&in_path, &out_path)?;
    }

    Ok(())
}

fn convert_file(in_file: &Path, out_file: &Path) -> std::io::Result<()> {
    let img = ImageReader::open(in_file)?.decode().unwrap();

    if let Some(parent) = out_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut f = std::fs::File::create(out_file)?;
    f.write_all(&img.width().to_le_bytes())?;
    f.write_all(&img.height().to_le_bytes())?;
    f.write_all(img.into_rgba8().as_bytes())?;

    Ok(())
}
