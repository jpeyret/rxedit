use sha2::{Sha256, Digest};


#[derive(Debug)]
struct LineStatus {
    visible : bool,
    line : String
}

trait GetLine {
    fn line(&self) -> &str;
}

impl GetLine for LineStatus {
    fn line(&self) -> &str {
        &self.line
    }
}

fn main() {

    let mut v_lines : Vec<LineStatus> = Vec::new();
    
    for ix in 0..10 {
        v_lines.push(
            LineStatus{
                line :             format!("  {}",ix),
                visible : true,
            }
        );
    }

    let hash2 = digest_lines(&v_lines);
    let chunk = chunk_it(&v_lines);

    println!("\n\n🐞hash2={}",hash2);
    println!("\n\n🐞chunk=:{}:",chunk);
    print!("\n\n🐞v_lines[0]={:?}",v_lines[0]);

}
