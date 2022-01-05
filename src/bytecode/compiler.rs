use super::scanner::Scanner;

pub struct Compiler<'a> {
    scanner: Scanner<'a>
}

impl<'a> Compiler<'a> {

    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(&source)
        }
    }

    pub fn compile(&mut self) {
        self.scanner.scan_token();
    }
}
