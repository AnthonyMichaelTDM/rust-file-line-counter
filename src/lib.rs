use std::{
    fs, //access to files / file system
    //env, //give access to environment stuff
    fmt::Debug,
    error::Error,//allows for some better errors
    path::{Path, PathBuf}, ffi::OsString, //system specific file separator, and path operations
};


//tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_help_flag() {
        let args: Vec<String> = vec![String::from("-h")];
        let expected_config: Config = Config{
            path: String::new(),
            path_is_directory:false,
            extensions_to_filter_for: Vec::new(),
            output_format: FORMAT::Default,
            search_subdirectories_recursively:false,
            show_help:true,
        };
        let actual_config = Config::new(&args).expect("test resulted in error creating config");

        assert_eq!(expected_config, actual_config);
    }
    #[test]
    fn config_no_arguments_given() {
        let args: Vec<String> = vec![];
        let expected_config: Config = Config{
            path: String::new(),
            path_is_directory:false,
            extensions_to_filter_for: Vec::new(),
            output_format: FORMAT::Default,
            search_subdirectories_recursively:false,
            show_help:true,
        };
        let actual_config = Config::new(&args).expect("test resulted in error creating config");

        assert_eq!(expected_config, actual_config);
    }
    #[test]
    fn config_filter_for_extension() {
        let args: Vec<String> = vec!["--f", "exe,rs", "../"]
        .iter().map(|s| s.to_string()).collect(); //this is just because i'm too lazy to manually make all the str's into strings

        let expected_config: Config = Config{
            path: String::from("../"),
            path_is_directory:true,
            extensions_to_filter_for: vec!["rs".to_string(),"exe".to_string()],
            output_format: FORMAT::Default,
            search_subdirectories_recursively:false,
            show_help:false,
        };
        let actual_config = Config::new(&args).expect("test resulted in error creating config");

        assert_eq!(expected_config, actual_config);
    }
    #[test]
    fn search_normal_text() {
        let text = "
Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Nullam vitae suscipit ipsum. 
Etiam pulvinar ullamcorper scelerisque. Aliquam malesuada libero nec ante commodo ornare. 
Nam nec leo diam. 
Suspendisse lectus dolor, tristique tempus massa sit amet, feugiat vehicula elit. Sed eu nisl porta, hendrerit augue at, laoreet ex. 
Nam sollicitudin tempor ligula quis condimentum. 
Donec id pretium sapien, eu pharetra neque. In tempus tortor in congue cursus.";

        assert_eq!(count_lines(text), 7);
    }
    #[test]
    fn search_empty_text() {
        let text = "";

        assert_eq!(count_lines(text), 0);
    }
    #[test]
    fn search_new_line_characters_inserted() {
        let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nNullam vitae suscipit ipsum.\nEtiam pulvinar ullamcorper scelerisque. Aliquam malesuada libero nec ante commodo ornare.\nNam nec leo diam.\nSuspendisse lectus dolor, tristique tempus massa sit amet, feugiat vehicula elit. Sed eu nisl porta, hendrerit augue at, laoreet ex.\nNam sollicitudin tempor ligula quis condimentum.\nDonec id pretium sapien, eu pharetra neque. In tempus tortor in congue cursus.";

        assert_eq!(count_lines(text), 7);
    }
}

//handles output format
#[derive(Debug, PartialEq)]
pub enum FORMAT {
    Default,
    MarkdownList,
    Numeric,
}

//handles parsing of arguments
const VALID_OPTIONS: [&str; 8] = [
    "--filter-for-extensions",
    "--format=MARKDOWN-LIST","--format=NUMERIC",
    "-r", "--recursive",
    "-h", "--help", "help",
];
#[derive(Debug, PartialEq)]
pub struct Config {
    pub path: String,
    pub path_is_directory: bool,
    pub extensions_to_filter_for: Vec<String>,
    pub output_format: FORMAT,
    pub search_subdirectories_recursively: bool,
    pub show_help: bool,
}
impl Config {
    pub fn new(args: &[String]) -> Result<Config, &str> {
        //DATA
        let mut config: Config = Config {
            path: String::new(),
            path_is_directory:false,
            extensions_to_filter_for: Vec::new(),
            output_format: FORMAT::Default,
            search_subdirectories_recursively:false,
            show_help:false,
        };
        let options: Vec<String>;
        let path: String;
        let full_path: &Path;

        // make sure enough arguments are given
        if args.len() <= 1 { //if there are none given
            config.show_help = true;
            return Ok(config);
        }

        //parse arguments for options
        options = (&args[1..]).iter()//iterator of arguments that ignores the first one
        .filter(|a| a.starts_with("-"))//filter out things that don't start with '-'
        .map(|a|a.to_lowercase().clone())//clone the strings
        .filter(|o| VALID_OPTIONS.contains(&o.as_str()))//filter out options that aren't valid
        .collect(); //collect into vector

        //parse arguments for the path
        //last argument
        path = args.last().unwrap().to_string(); 

        //modify config as needed depending on options passed
        options.iter().for_each(|option| {
            match option.trim() {
                /* filter for extensions */
                //if list_after_option() failed, print the error to std_err and set config.extensions_to_filter_for to an empty vector, otherwise set config.extensions_to_filter_for to the vector returned
                "--filter-for-extensions" => config.extensions_to_filter_for = get_list_from_args_after_option(&args, "--filter-for-extensions").unwrap_or_else(|e| {eprintln!("Error finding extensions list: {}", e);Vec::new()}), //if li
                /* output format */
                "--format=MARKDOWN-LIST" => config.output_format = FORMAT::MarkdownList,
                "--format=NUMERIC" => config.output_format = FORMAT::Numeric,
                /* search subdirectories recursively */
                "-r"|"--recursive" => config.search_subdirectories_recursively = true,
                /* help */
                "-h"|"--help" => config.show_help = true,
                _ => {},
            }
        });

        //extract / verify the path
        full_path = Path::new(&path);
        //is path a valid file path
        if full_path.exists() {
            config.path_is_directory = full_path.is_dir();
            config.path = path;
        } else {
            //return an error
            return Err("Invalid path given")
        }
        

        //return
        return Ok(config);
    }
}
//private function that goes through the arguments to find a list after a specified option
fn get_list_from_args_after_option<'a>(args:&[String], option: &'a str) -> Result<Vec<String>,&'a str> {
    //move iterator to the specified option
    let mut arg_iter = args.iter();
    let mut arg = arg_iter.next().unwrap_or(&String::new()).clone();
    while !arg.eq_ignore_ascii_case(option) {arg = arg_iter.next().unwrap_or(&String::new()).clone()} //advance iterator to "--filter-for-extensions"

    //get the list, return an error if it's not found
    match arg_iter.next().ok_or("ran out of arguments, could not find extensions list") {
        Ok(extension_list) => return Ok(
            extension_list.clone().chars().filter(|c| c.is_ascii_alphabetic() || c.eq_ignore_ascii_case(&',')).collect::<String>() //remove invalid characters
            .split(",").filter_map(|s| (!s.is_empty()).then(|| s.to_ascii_lowercase())).collect() //split the string into vector at Commas, then remove empty values and convert to lowercase
        ),
        Err(err) => return Err(err.clone()), //return with the error
    }
}



/**
 * run the program
 */
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    //DATA
    let mut paths_to_process:Vec<PathBuf>; 
    let path = PathBuf::from(&config.path);

    //if the user wants/needs help print instructions and exit
    if config.show_help {
        help();
        return Ok(());
    }
    
    // if path is a file
    if !config.path_is_directory {
        let count = match count_lines_of_file(&config.path) {
            Ok(c) => c,
            Err(e) => return Err(e), 
        };
        println!("{}: {} Lines", config.path, count);
    }
    //if path is a directory
    else  {
        if config.search_subdirectories_recursively { //user wants us to search subdirectories recursively
            paths_to_process = list_files_recursively(&path)
        }
        else { //user does not want us to search subdirectories recursively
            paths_to_process = list_files(&path);
        }

        //if user want to filter for some given extensions, do that here
        if !config.extensions_to_filter_for.is_empty() {
            paths_to_process = paths_to_process.into_iter().filter(|raw_path| {
                if let Some(extension) = raw_path.extension() {
                    if config.extensions_to_filter_for.iter().map(|ext| OsString::from(ext)).any(|ext| ext.eq_ignore_ascii_case(extension)) {
                        return true;
                    }
                }
                return false;
            }).collect();
        }

        //count lines of every file in paths_to_process
        let mut i = 1; //counter variable for the Numberic format
        for path_name in paths_to_process.into_iter().filter_map(|p| p.into_os_string().into_string().ok()) { //convert them all into strings
            let count = match count_lines_of_file(&path_name) {
                Ok(c) => c,
                Err(e) => return Err(e), 
            };

            //format output as specified by config.output_format
            match config.output_format {
                FORMAT::Default => println!("{}: {} Lines", path_name, count),
                FORMAT::MarkdownList => println!("- {}: {} Lines", path_name, count),
                FORMAT::Numeric => println!("{}.) {}: {} Lines", i, path_name, count),
            }

            i+=1;
        };
    }


    //if path is a directory, and the -r argument was passed, run count_lines() on all files in it and subdirectories
    //if path is a directory and the -r argument was not passed, run count_lines() on the files in it
    
    //if path is a file, run count_lines() on it

    //return () if no issue
    return Ok(());
}

/**
 * run count_lines on a given path
 */
pub fn count_lines_of_file<'a>(path: &'a str) -> Result<usize, Box<dyn Error>> {
    //count lines in path
    let file_contents;
    match fs::read_to_string(&path) {
        Ok(s) => file_contents = s,
        Err(_e) => return Err(format!("Could not read contents of {}", path).into()), //create and return an error with that message
    }
    return Ok(count_lines(&file_contents));
}
/**
 * count number of newline characters in a given string
 */
pub fn count_lines<'a>(file_contents: &'a str) -> usize {
    return file_contents.chars().filter(|c| *c == '\n').count();
}

/***
 * print instructions
 */
pub fn help() {
    println!("line-counter.exe");
    println!("count lines of a file or of files in directory\n");

    println!("USAGE:\n\tline-counter.exe [OPTIONS]... [PATH]\n");

    println!("OPTIONS:");
    println!("\t\t--filter-for-extensions <EXTENSIONS>...\t\tComma separated list of extensions, will only count lines of files with these extensions");
    println!("\t-r,\t--recursive\t\t\t\t\tSearch through subdirectories");
    println!("\t-h,\t-help\t\t\t\t\t\tPrints help information");
}

/**
 * returns a vector containing paths to all files in path and subdirectories of path
 */
fn list_files_recursively(path: &Path) -> Vec<PathBuf> {
    let mut vec = Vec::new();
    _list_files_recursively(&mut vec,&path);
    vec
}
fn _list_files_recursively(vec: &mut Vec<PathBuf>, path: &Path) {
    if path.is_dir() {
        let paths = fs::read_dir(&path).unwrap();
        for path_result in paths {
            let full_path = path_result.unwrap().path();
            if full_path.is_dir() {
                _list_files_recursively(vec, &full_path);
            } else {
                vec.push(full_path);
            }
        }
    }
}
/**
 * returns a vector containing paths to all files in path, but not subdirectories of path
 */
fn list_files(path: &Path) -> Vec<PathBuf> {
    let mut vec = Vec::new();
    if path.is_dir() {
        let paths = fs::read_dir(&path).unwrap();
        for path_results in paths {
            let full_path = path_results.unwrap().path();
            if !full_path.is_dir() {
                vec.push(full_path);
            }
        }
    }
    return vec;
}