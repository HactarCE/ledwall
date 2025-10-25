use image::ImageReader;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!(
            "usage: `{} <filename_in> <filename_out>`",
            std::env::args().next().unwrap(),
        );
        std::process::exit(1);
    }
    let in_file = &args[1];
    let out_file = &args[2];

    let img = ImageReader::open(in_file).unwrap().decode().unwrap();
    let rgba8 = img.into_rgba8();
    let flat_samples = rgba8.as_flat_samples();
    let mut rgba_data_out = vec![];
    let padding_len = flat_samples.min_length().unwrap() / 2;
    rgba_data_out.extend(std::iter::repeat_n(0, padding_len));
    rgba_data_out.extend_from_slice(flat_samples.as_slice());
    rgba_data_out.extend(std::iter::repeat_n(0, padding_len));
    std::fs::write(out_file, rgba_data_out).unwrap();
}
