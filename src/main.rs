use std::{
    env, //allows us to access environment stuff
    process, //allows for some better error handling
};

mod lib;
use crate::lib::Config;

fn main() {
    //read command line arguments
    let args: Vec<String> = env::args().collect(); //first argument is the location of the executable relative to where it's being called from
    //println!("{:?}", args);

    let config = Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        lib::help();
        process::exit(1);
    });

    //run the program with the given args, handle errors as needed
    if let Err(e) = lib::run(config) {
        eprintln!("Application Error: {}", e); //use the eprintln! macro to output to standard error
        process::exit(1); //exit the program with an error code
    }
}
