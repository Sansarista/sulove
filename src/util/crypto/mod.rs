use std::io::{self, Read, Write};
use std::fs::File;
use std::path::Path;

pub struct ZIP;

impl ZIP {
    pub fn compress(input_path: &Path, output_path: &Path) -> io::Result<()> {
        let file = File::open(input_path)?;
        let output_file = File::create(output_path)?;
        
        let mut zip = zip::ZipWriter::new(output_file);
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);
        
        let file_name = input_path.file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name"))?;
        
        zip.start_file(file_name.to_string_lossy(), options)?;
        
        let mut buffer = Vec::new();
        let mut reader = File::open(input_path)?;
        reader.read_to_end(&mut buffer)?;
        
        zip.write_all(&buffer)?;
        zip.finish()?;
        
        Ok(())
    }
    
    pub fn decompress(zip_path: &Path, output_dir: &Path) -> io::Result<()> {
        let file = File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = output_dir.join(file.name());
            
            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                
                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
        
        Ok(())
    }
}