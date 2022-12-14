use std::process::Command;

mod stapel;

use stapel::*;

fn main() {
    // Getting command line arguments
    let args = std::env::args().skip(1).collect::<Vec<String>>();

    // Checking and the arguments
    if args.len() == 1 && args[0] == "help" {
        println!("USAGE:\n\tstapel  sim|com  <path>");
        std::process::exit(0);
    } else if args.len() != 2 {
        println!("Please provide two arguments\nType: 'stapel --help' for help");
        std::process::exit(0);
    } else if args[0] != "sim" && args[0] != "com"  {
        println!("'{}', is not a execution option.\nType: 'stapel --help' for help", args[1]);
    }

    if !args[2].ends_with(".spl") {
        println!("File must end with '.spl', not {}", args[2]);
        std::process::exit(1);
    }

    // Reading and parsing the file
    let input = read_file(args[1].to_string());
    let program = parse(input);
    
    // Running the Stapel code
    if args[0] == "com" {
        // Compiling stapel
        let output = compile_x86_64(program);
        println!("[INFO] Succesfully compiled Stapel to X86_64 assembly");

        // Defining paths for compilation files
        let assembly_path = format!("temp_{}.asm", args[1]);
        let object_path = format!("temp_{}.asm.o", args[1]);
        let executable_path = (&args[1][..(args[1].len()-7)]).to_string();

        // Writing assembly file to fs
        let res = std::fs::write(&assembly_path, output);
        if let Err(_) = res {
            println!("Could not save file at: {}", assembly_path);
            std::process::exit(1);
        }

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
        let res = c.output();

        // Printing linking result
        if let Err(_) = res {
            println!("[ERROR] Failed to compile object file to binary");
        } else {
            println!("[INFO] Compilation succesfull, path to exe: './{}'", executable_path);
        }

        // Removing object file from compilation
        let res = std::fs::remove_file(object_path);
        if let Err(_) = res {
            println!("[ERROR] Failed to remove object file");
        }
    } else {
        run(program);
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
