use fst_native::{FstFilter, FstReader, FstSignalHandle, FstSignalValue};

fn main() {
    let file = std::fs::File::open("./wave.fst").unwrap();
    let mut reader = FstReader::open(std::io::BufReader::new(file)).unwrap();
    let header = reader.get_header();
    println!(
        "fst file start time: {}, fst file end time: {}",
        header.start_time, header.end_time
    );

    reader
        .read_signals(&FstFilter::all(), move |t, handle, value| {
            let actual_as_string = match value {
                FstSignalValue::String(str) => str.to_string(),
                FstSignalValue::Real(value) => format!("{value}"),
            };

            println!(
                "t: {}, h index: {}, v: {}",
                t,
                handle.get_index(),
                actual_as_string
            );
        })
        .unwrap();
}
