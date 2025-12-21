pub mod compiler;
pub mod operators;
pub mod tokens;
pub mod lexer;
pub mod parser;
pub mod program;

use compiler::*;
use parser::{Parser};
use tokens::Span;
use lexer::Lexer;

use std::process::Command;

fn main() {
    // Getting command line arguments
    let args = std::env::args().skip(1).collect::<Vec<String>>();

    // Checking and the arguments
    if args.len() == 1 && args[0] == "help" {
        println!("USAGE:\n\tstapel build  <path>");
        std::process::exit(0);
    } else if args.len() != 2 {
        println!("Please provide two arguments\nType: 'stapel --help' for help");
        std::process::exit(0);
    } else if  args[0] != "build"  {
        println!("'{}', is not a execution option.\nType: 'stapel --help' for help", args[1]);
    }

    if !args[1].ends_with(".spl") {
        println!("File must end with '.spl', not {}", args[2]);
        std::process::exit(1);
    }

    // Reading and parsing the file
    let input = read_file(args[1].to_string());    
    let mut l = Lexer::new(input, args[1].to_string());
    l.tokenize();
    
    let mut p = Parser::new(l.tokens);
    p.parse();

    let mut compiler = Compiler::new(p.program);
    compiler.compile_x86_64();

    // Defining paths for compilation files
    let assembly_path = format!("temp_{}.asm", args[1]);
    let object_path = format!("temp_{}.asm.o", args[1]);
    let executable_path = (&args[1][..(args[1].len()-4)]).to_string();

    // Writing assembly file to fs
    let res = std::fs::write(&assembly_path, compiler.code);
    if let Err(_) = res {
        println!("Could not save file at: {}", assembly_path);
        std::process::exit(1);
    }
    println!("[INFO] Sucessfully compiled Stapel to Assembly (NASM X86 GNU/Linux)");

    // Compiling ASM to object with nasm
    let mut c = Command::new("sh");
    c.arg("-c").arg(format!("nasm -f elf64 -o {} {}", object_path, assembly_path));
    let output = c.output();
    
    // Printing result from nasm
    if let Err(_) = output {
        println!("[ERROR] Failed to compile NASM to *.o");
        std::process::exit(1);
    }
    println!("[INFO] Sucessfully compiled NASM to object");

    // Linking object
    let mut c = Command::new("sh");
    c.arg("-c").arg(format!("ld -o {} {}", executable_path, object_path));
    let _ = c.output();

    // Removing object file from compilation
    let res = std::fs::remove_file(object_path);
    if let Err(_) = res {
        println!("[ERROR] Failed to remove object file");
    }

    // Printing linking result
    if let Err(_) = res {
        println!("[ERROR] Failed to compile object file to binary");
    } else {
        println!("[INFO] Compilation succesfull, path to executable: './{}'", executable_path);
    }
}

fn read_file(path: String) -> String {
    let file = std::fs::read_to_string(&path);
    let Ok(input) = file else {
        println!("Could not read file at location: '{}'", path);
        std::process::exit(0);
    };

    input
}

pub fn throw_exception_span(span: &Span, message: String) {
    println!("Syntax Error {} [{}:{}] ==>\n\t{}", span.file, span.row, span.column, message);
    std::process::exit(1);
}

pub fn throw_exception(message: String) {
    println!("Compilation Error ==>\n\t{}", message);
    std::process::exit(1);
}
