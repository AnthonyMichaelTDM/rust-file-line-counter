use std::{
    fs, //access to files / file system
    fmt::Debug,
    error::Error,//allows for some better errors
    path::{Path, PathBuf}, ffi::OsString, //system specific file separator, and path operations
};

//handles output format
#[derive(Debug, PartialEq)]
pub enum FORMAT {
    Default,
    Bullet,
    Markdown,
    Numeric,
}

//handles parsing of arguments
const VALID_OPTIONS: [&str; 10] = [
    "-f", "--filter",
    "--format=DEFAULT","--format=BULLET", "--format=MARKDOWN","--format=NUMERIC",
    "-r", "--recursive",
    "-h", "--help",
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
    pub fn new(args: &[String]) -> Result<Config, Box<dyn Error>> {
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
        options = (&args[0..]).iter()//iterator of arguments that ignores the first one
        .filter(|a| a.starts_with("-"))//filter out things that don't start with '-'
        .map(|a|a.to_string())//clone the strings
        .collect(); //collect into vector

        if !options.iter().all(|o| VALID_OPTIONS.contains(&o.as_str())) {
            //if there are invalid arguments, throw an error
            return Err("One or more invalid arguments.".into());
        }

        //parse arguments for the path
        //last argument
        path = args.last().unwrap().to_string(); 

        //modify config as needed depending on options passed
        let mut extensions_to_filter_for_or_error = Ok(Vec::new());
        options.iter().for_each(|option| {
            match option.as_str() {
                /* filter for extensions */
                //if list_after_option() failed, print the error to std_err and set config.extensions_to_filter_for to an empty vector, otherwise set config.extensions_to_filter_for to the vector returned
                "-f" => extensions_to_filter_for_or_error = get_list_from_args_after_option(&args, "-f"),
                "--filter" => extensions_to_filter_for_or_error = get_list_from_args_after_option(&args, "--filter"),
                /* output format */
                "--format=DEFAULT" => config.output_format = FORMAT::Default,
                "--format=BULLET" => config.output_format = FORMAT::Bullet,
                "--format=MARKDOWN" => config.output_format = FORMAT::Markdown,
                "--format=NUMERIC" => config.output_format = FORMAT::Numeric,
                /* search subdirectories recursively */
                "-r"|"--recursive" => config.search_subdirectories_recursively = true,
                /* help */
                "-h"|"--help" => config.show_help = true,
                _ => {},
            }
        });
        match extensions_to_filter_for_or_error {
            Err(e) => return Err( format!("Error finding extensions list: {}", e).into()),
            Ok(vec) => if !vec.is_empty() {config.extensions_to_filter_for = vec;},
        }

        //if help, exit early
        if config.show_help {
            return Ok(config);
        }


        //extract / verify the path
        full_path = Path::new(&path);
        //is path a valid file path
        if full_path.exists() {
            config.path_is_directory = full_path.is_dir();
            config.path = path;
        } else {
            //return an error
            return Err("Invalid path given".into())
        }
        

        //return
        return Ok(config);
    }
}
//private function that goes through the arguments to find a list after a specified option
fn get_list_from_args_after_option<'a>(args:&[String], option: &'a str) -> Result<Vec<String>,&'a str> {
    //move iterator to the specified option
    let next_arg;
    
    //get position of option in args
    let pos;
    match args.iter().position(|arg| arg.eq(&option)).ok_or("Could not find option in args.") {
        Ok(index) => pos = index,
        Err(e) => return Err(e),
    }

    //if there is not an argument between it and the last argument
    if args.len() <= 2 || pos > args.len() - 2 {
        return Err("Not enough arguments, Or no list found.");
    }

    //get the list
    next_arg = &args[pos+1];
    //throw an error if the "list" is actually an option
    if VALID_OPTIONS.contains(&next_arg.as_str()) {
        return Err("No list found.");
    }


    //create and return the list
    return Ok(
        next_arg.clone().chars().filter(|c| c.is_ascii_alphabetic() || c.eq(&',')).collect::<String>() //remove invalid characters
        .split(",").filter_map(|s| (!s.is_empty()).then(|| s.to_ascii_lowercase())).collect() //split the string into vector at Commas, then remove empty values and convert to lowercase
    )
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
        } else {
            //just filter out things that return None from .extension
            paths_to_process = paths_to_process.into_iter().filter(|raw_path| raw_path.extension().is_some()).collect();
        }

        //count lines of every file in paths_to_process
        let mut i = 1; //counter variable for the Numberic format
        for path_name in paths_to_process.into_iter().filter_map(|p| p.into_os_string().into_string().ok()) { //convert them all into strings
            let count = match count_lines_of_file(&path_name) {
                Ok(c) => c,
                Err(e) => {eprintln!("!\t{}",e); continue;}, //just print errors to std. error, no use ending the program early 
            };

            //format output as specified by config.output_format
            match config.output_format {
                FORMAT::Default => print!("\t"),
                FORMAT::Bullet => print!("*\t"),
                FORMAT::Markdown => print!("-\t"),
                FORMAT::Numeric => print!("{}.)\t", i),
            }
            println!("{}: {} Lines", path_name, count);

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
    println!("                              line-counter.exe");
    println!("                              By Anthony Rubick\n");
    println!("count lines of a file or of all files in directory\n");

    println!("USAGE:\n\tline-counter.exe [OPTIONS]... [PATH]\n");

    println!("OPTIONS:");
    println!("\t-f\t--filter <EXTENSIONS>...\t\tComma separated list of extensions, will only count lines of files with these extensions");
    println!("\t\t--format=[FORMAT]\t\t\tFormat the output in a list, valid formats are: DEFAULT, BULLET, MARKDOWN, and NUMERIC");
    println!("\t-r,\t--recursive\t\t\t\tSearch through subdirectories");
    println!("\t-h,\t-help\t\t\t\t\tPrints help information\n");

    println!("PATH:\n\tPath to search\n\n")
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















//tests
#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;



    #[test]
    fn config_null_test() {
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
    fn config_all_features_short_flags() {
        let args: Vec<String> = vec!["-r","--format=NUMERIC","-f","exe,rs","../"].iter().map(|s| s.to_string()).collect();
        let expected_config: Config = Config{
            path: String::from("../"),
            path_is_directory:true,
            extensions_to_filter_for: vec!["exe".to_string(),"rs".to_string()],
            output_format: FORMAT::Numeric,
            search_subdirectories_recursively:true,
            show_help:false,
        };
        let actual_config = Config::new(&args).expect("test resulted in error creating config");

        assert_eq!(expected_config, actual_config);
    }
    #[test]
    fn config_all_features_long_flags() {
        let args: Vec<String> = vec!["--recursive","--filter","exe,rs", "--format=MARKDOWN","../"].iter().map(|s| s.to_string()).collect();
        let expected_config: Config = Config{
            path: String::from("../"),
            path_is_directory:true,
            extensions_to_filter_for: vec!["exe".to_string(),"rs".to_string()],
            output_format: FORMAT::Markdown,
            search_subdirectories_recursively:true,
            show_help:false,
        };
        let actual_config = Config::new(&args).expect("test resulted in error creating config");

        assert_eq!(expected_config, actual_config);
    }
    #[test]
    fn config_help_short_flag() {
        let args: Vec<String> = vec!["-h"].iter().map(|s| s.to_string()).collect();
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
    fn config_help_long_flag() {
        let args: Vec<String> = vec!["--help"].iter().map(|s| s.to_string()).collect(); //this is just because i'm too lazy to manually make all the str's into strings


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
    fn config_filter_for_extension_short_flag() {
        let args: Vec<String> = vec!["-f", "exe,rs", "../"].iter().map(|s| s.to_string()).collect(); //this is just because i'm too lazy to manually make all the str's into strings

        let expected_config: Config = Config{
            path: String::from("../"),
            path_is_directory:true,
            extensions_to_filter_for: vec!["exe".to_string(),"rs".to_string()],
            output_format: FORMAT::Default,
            search_subdirectories_recursively:false,
            show_help:false,
        };
        let actual_config = Config::new(&args).expect("test resulted in error creating config");

        assert_eq!(expected_config, actual_config);
    }
    #[test]
    #[should_panic]
    fn config_filter_for_extension_empty() {
        let args: Vec<String> = vec!["-f", "../"].iter().map(|s| s.to_string()).collect(); //this is just because i'm too lazy to manually make all the str's into strings
        //should panic here bc of invalid arguments
        let _actual_config = Config::new(&args).expect("test resulted in error creating config");
    }
    #[test]
    #[should_panic]
    fn config_filter_for_extension_other_option_instead_of_flag() {
        let args: Vec<String> = vec!["-f", "-r", "../"].iter().map(|s| s.to_string()).collect(); //this is just because i'm too lazy to manually make all the str's into strings
        //should panic here bc of invalid arguments
        let _actual_config = Config::new(&args).expect("test resulted in error creating config");
    }
    #[test]
    fn config_filter_for_extension_long_flag() {
        let args: Vec<String> = vec!["--filter", "exe,rs", "../"].iter().map(|s| s.to_string()).collect(); //this is just because i'm too lazy to manually make all the str's into strings

        let expected_config: Config = Config{
            path: String::from("../"),
            path_is_directory:true,
            extensions_to_filter_for: vec!["exe".to_string(),"rs".to_string()],
            output_format: FORMAT::Default,
            search_subdirectories_recursively:false,
            show_help:false,
        };
        let actual_config = Config::new(&args).expect("test resulted in error creating config");

        assert_eq!(expected_config, actual_config);
    }
    #[test]
    fn config_recursion_short_flag() {
        let args: Vec<String> = vec!["-r", "../"].iter().map(|s| s.to_string()).collect(); //this is just because i'm too lazy to manually make all the str's into strings

        let expected_config: Config = Config{
            path: String::from("../"),
            path_is_directory:true,
            extensions_to_filter_for: Vec::new(),
            output_format: FORMAT::Default,
            search_subdirectories_recursively:true,
            show_help:false,
        };
        let actual_config = Config::new(&args).expect("test resulted in error creating config");

        assert_eq!(expected_config, actual_config);
    }
    #[test]
    fn config_recursion_long_flag() {
        let args: Vec<String> = vec!["--recursive", "../"].iter().map(|s| s.to_string()).collect(); //this is just because i'm too lazy to manually make all the str's into strings

        let expected_config: Config = Config{
            path: String::from("../"),
            path_is_directory:true,
            extensions_to_filter_for: Vec::new(),
            output_format: FORMAT::Default,
            search_subdirectories_recursively:true,
            show_help:false,
        };
        let actual_config = Config::new(&args).expect("test resulted in error creating config");

        assert_eq!(expected_config, actual_config);
    }
    #[test]
    fn config_format_bullet() {
        let args: Vec<String> = vec!["--format=BULLET", "../"].iter().map(|s| s.to_string()).collect(); //this is just because i'm too lazy to manually make all the str's into strings

        let expected_config: Config = Config{
            path: String::from("../"),
            path_is_directory:true,
            extensions_to_filter_for: Vec::new(),
            output_format: FORMAT::Bullet,
            search_subdirectories_recursively:false,
            show_help:false,
        };
        let actual_config = Config::new(&args).expect("test resulted in error creating config");

        assert_eq!(expected_config, actual_config);
    }
    #[test]
    fn config_format_markdown() {
        let args: Vec<String> = vec!["--format=MARKDOWN", "../"].iter().map(|s| s.to_string()).collect(); //this is just because i'm too lazy to manually make all the str's into strings

        let expected_config: Config = Config{
            path: String::from("../"),
            path_is_directory:true,
            extensions_to_filter_for: Vec::new(),
            output_format: FORMAT::Markdown,
            search_subdirectories_recursively:false,
            show_help:false,
        };
        let actual_config = Config::new(&args).expect("test resulted in error creating config");

        assert_eq!(expected_config, actual_config);
    }
    #[test]
    fn config_format_numeric() {
        let args: Vec<String> = vec!["--format=NUMERIC", "../"].iter().map(|s| s.to_string()).collect(); //this is just because i'm too lazy to manually make all the str's into strings

        let expected_config: Config = Config{
            path: String::from("../"),
            path_is_directory:true,
            extensions_to_filter_for: Vec::new(),
            output_format: FORMAT::Numeric,
            search_subdirectories_recursively:false,
            show_help:false,
        };
        let actual_config = Config::new(&args).expect("test resulted in error creating config");

        assert_eq!(expected_config, actual_config);
    }
    #[test]
    #[should_panic]
    fn config_mixed_casing() {
        let args: Vec<String> = vec!["-F","eXe,rs", "--forMAt=numMERIC","-r","../"].iter().map(|s| s.to_string()).collect();
        //this should panic because of the improper capitalization
        let _actual_config = Config::new(&args).expect("test resulted in error creating config");
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

        assert_eq!(count_lines(text), 6);
    }
}