use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};

fn main() -> io::Result<()> {
    let folder_path = "char/";
    let summary_path = format!("{}/summary", folder_path);

    let mut summary_file = File::create(summary_path.clone())?;

    let files = fs::read_dir(folder_path)?;

    for file in files {
        let file = file?;
       
        let file_path = file.path();
        let file_name_os = file.file_name();
        let file_name = file_name_os.to_string_lossy();
        if file_name != "summary" {
            writeln!(summary_file, "{}", file.path().display())?;

            let file_lines = BufReader::new(File::open(file_path)?).lines().take(3);
            for line in file_lines {
                let line = line?;
                writeln!(summary_file, "{}", line)?;
            }
        }
    }

    Ok(())
}


