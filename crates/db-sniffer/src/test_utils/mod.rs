pub mod mysql;

pub fn collapse_xml(xml: &str) -> String {
    xml.lines()
       .map(|line| line.trim())
       .filter(|line| !line.is_empty())
       .collect::<Vec<&str>>()
       .join("")
       .replace("\n", "")
}

pub fn compare_xml(a: &str, b: &str) -> bool {
    if collapse_xml(a) == collapse_xml(b) {  return true; }
    
    println!("XMLs are different:");
    println!("Left: {}", a);
    println!("Right: {}", b);
    
    false
}